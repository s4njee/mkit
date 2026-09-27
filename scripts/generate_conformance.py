#!/usr/bin/env python3
"""Generate a component conformance manifest from its YAML front matter.

The manifest is deliberately an adapter contract rather than a fake test result:
the spec describes behavior in prose, while a component fixture must provide
state transitions, assertions, and rendering. Harness consumers can use the
generated cases uniformly without treating unsupported platform capabilities
as passing checks.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys
from typing import Any

try:
    import yaml
except ImportError as error:  # pragma: no cover - environment-specific diagnostic
    raise SystemExit("PyYAML is required; install it with `python3 -m pip install PyYAML`") from error


THEMES = ("light", "dark", "high-contrast")
SCALES = (1, 2)


class SpecError(ValueError):
    """Invalid or unsupported component-spec front matter."""


def extract_front_matter(source: str) -> dict[str, Any]:
    lines = source.splitlines()
    if not lines or lines[0].strip() != "---":
        raise SpecError("component spec must begin with a `---` YAML front block")
    try:
        end = next(index for index in range(1, len(lines)) if lines[index].strip() == "---")
    except StopIteration as error:
        raise SpecError("component spec front block has no closing `---`") from error
    try:
        data = yaml.safe_load("\n".join(lines[1:end]))
    except yaml.YAMLError as error:
        raise SpecError(f"invalid YAML front block: {error}") from error
    if not isinstance(data, dict):
        raise SpecError("YAML front block must be a mapping")
    return data


def _required_text(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise SpecError(f"{label} must be a non-empty string")
    return value.strip()


def _validate_spec(spec: dict[str, Any]) -> tuple[str, list[dict[str, Any]], list[dict[str, Any]], dict[str, Any]]:
    if spec.get("spec_version") != 1:
        raise SpecError("spec_version must be 1")
    component = _required_text(spec.get("component"), "component")
    states = spec.get("states")
    keys = spec.get("keys")
    accessibility = spec.get("accessibility")
    if not isinstance(states, list) or not states:
        raise SpecError("states must be a non-empty list")
    if not isinstance(keys, list):
        raise SpecError("keys must be a list (use [] when there are no keyboard behaviors)")
    if not isinstance(accessibility, dict):
        raise SpecError("accessibility must be a mapping")
    state_ids: set[str] = set()
    for index, state in enumerate(states):
        if not isinstance(state, dict):
            raise SpecError(f"states[{index}] must be a mapping")
        state_id = _required_text(state.get("id"), f"states[{index}].id")
        _required_text(state.get("description"), f"states[{index}].description")
        _required_text(state.get("fixture"), f"states[{index}].fixture")
        if "accessibility_role" in state:
            _required_text(state["accessibility_role"], f"states[{index}].accessibility_role")
        if "screenshot_status" in state:
            _required_text(state["screenshot_status"], f"states[{index}].screenshot_status")
        if "screenshot_limitation" in state:
            _required_text(state["screenshot_limitation"], f"states[{index}].screenshot_limitation")
            if state.get("screenshot_status") != "unsupported_headless_capture":
                raise SpecError(
                    f"states[{index}].screenshot_limitation requires screenshot_status: unsupported_headless_capture"
                )
        if state_id in state_ids:
            raise SpecError(f"duplicate state id {state_id!r}")
        state_ids.add(state_id)
    for index, key in enumerate(keys):
        if not isinstance(key, dict):
            raise SpecError(f"keys[{index}] must be a mapping")
        _required_text(key.get("key"), f"keys[{index}].key")
        _required_text(key.get("when"), f"keys[{index}].when")
        _required_text(key.get("action"), f"keys[{index}].action")
        initial_state = _required_text(key.get("initial_state"), f"keys[{index}].initial_state")
        if initial_state not in state_ids:
            raise SpecError(f"keys[{index}].initial_state references unknown state {initial_state!r}")
        expect = key.get("expect")
        if not isinstance(expect, dict) or not expect:
            raise SpecError(f"keys[{index}].expect must contain state, event, and/or focus_target")
        unknown_expectations = set(expect) - {"state", "event", "focus_target"}
        if unknown_expectations:
            raise SpecError(f"keys[{index}].expect has unsupported fields: {', '.join(sorted(unknown_expectations))}")
        for name, expected in expect.items():
            _required_text(expected, f"keys[{index}].expect.{name}")
        if "state" in expect and expect["state"] not in state_ids:
            raise SpecError(f"keys[{index}].expect.state references unknown state {expect['state']!r}")
        modifiers = key.get("modifiers", [])
        if not isinstance(modifiers, list) or any(not isinstance(item, str) for item in modifiers):
            raise SpecError(f"keys[{index}].modifiers must be a list of strings")
    _required_text(accessibility.get("role"), "accessibility.role")
    properties = accessibility.get("properties", [])
    if not isinstance(properties, list):
        raise SpecError("accessibility.properties must be a list")
    for index, prop in enumerate(properties):
        if not isinstance(prop, dict):
            raise SpecError(f"accessibility.properties[{index}] must be a mapping")
        _required_text(prop.get("name"), f"accessibility.properties[{index}].name")
        if "value" not in prop:
            raise SpecError(f"accessibility.properties[{index}].value is required")
        if "when" in prop:
            when = _required_text(prop["when"], f"accessibility.properties[{index}].when")
            if when not in state_ids:
                raise SpecError(
                    f"accessibility.properties[{index}].when references unknown state {when!r}"
                )
        if "except_when" in prop:
            excluded = prop["except_when"]
            if not isinstance(excluded, list) or any(state not in state_ids for state in excluded):
                raise SpecError(f"accessibility.properties[{index}].except_when must list known states")
    for state_id in state_ids:
        names: set[str] = set()
        for prop in properties:
            if (prop.get("when") is not None and prop["when"] != state_id) or state_id in prop.get("except_when", []):
                continue
            if prop["name"] in names:
                raise SpecError(
                    f"accessibility property {prop['name']!r} is declared more than once for state {state_id!r}"
                )
            names.add(prop["name"])
    return component, states, keys, accessibility


def build_manifest(spec: dict[str, Any], custom_tests: str | None = None) -> dict[str, Any]:
    component, states, keys, accessibility = _validate_spec(spec)
    keyboard_cases = []
    for index, item in enumerate(keys, 1):
        modifiers = item.get("modifiers", [])
        keyboard_cases.append({
            "id": f"keyboard-{index:02d}",
            "dispatch": {"key": item["key"], "modifiers": modifiers},
            "precondition": item["when"],
            "expected_behavior": item["action"],
            "initial_state": item["initial_state"],
            "load_fixture": next(state["fixture"] for state in states if state["id"] == item["initial_state"]),
            "assert": item["expect"],
        })

    accessibility_cases = []
    for state in states:
        applicable = [
            {"name": item["name"], "value": item["value"]}
            for item in accessibility.get("properties", [])
            if (item.get("when") is None or item.get("when") == state["id"]) and state["id"] not in item.get("except_when", [])
        ]
        accessibility_cases.append({
            "id": f"a11y-{state['id']}",
            "state": state["id"],
            "load_fixture": state["fixture"],
            "expected_semantics": {"role": state.get("accessibility_role", accessibility["role"]), "properties": applicable},
            "capture": "mkit_harness::AccessibilitySnapshot::capture",
            "requires_active_platform_a11y": True,
            "headless_inactive_result": "unsupported_pending",
        })

    screenshots = []
    for state in states:
        for theme in THEMES:
            for scale in SCALES:
                case = {
                    "id": f"screen-{state['id']}-{theme}-{scale}x",
                    "state": state["id"], "theme": theme, "scale": scale,
                    "load_fixture": state["fixture"],
                    "platform": "macos",
                    "status": state.get("screenshot_status", "pending_until_supported_platform_run"),
                    "baseline": f"{component}/{state['id']}/{theme}-{scale}x.png",
                }
                if "screenshot_limitation" in state:
                    case["limitation"] = state["screenshot_limitation"]
                screenshots.append(case)
    return {
        "manifest_version": 1,
        "component": component,
        "source_spec_version": spec["spec_version"],
        "adapter_contract": {
            "fixture": "Provide each declared state fixture; load the initial_state fixture before dispatch.",
            "keyboard": "Dispatch the key with modifiers, then assert every field in the case's assert mapping.",
            "accessibility": "Capture the actual platform accessibility tree and compare declared semantics.",
            "screenshots": "Render each state/theme/scale case and compare its named baseline.",
            "custom_tests": custom_tests,
        },
        "keyboard_cases": keyboard_cases,
        "accessibility_cases": accessibility_cases,
        "screenshot_cases": screenshots,
    }


def serialize_manifest(manifest: dict[str, Any]) -> str:
    return json.dumps(manifest, indent=2, ensure_ascii=False) + "\n"


def generate(spec_path: Path, output_path: Path, custom_tests: str | None = None,
             check: bool = False) -> dict[str, Any]:
    manifest = build_manifest(extract_front_matter(spec_path.read_text(encoding="utf-8")), custom_tests)
    serialized = serialize_manifest(manifest)
    if check:
        try:
            current = output_path.read_text(encoding="utf-8")
        except OSError as error:
            raise SpecError(f"generated manifest is missing or unreadable: {output_path}: {error}") from error
        if current != serialized:
            raise SpecError(f"generated manifest is stale: {output_path}; rerun without --check")
        return manifest
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(serialized, encoding="utf-8")
    return manifest


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("spec", type=Path, help="Markdown component spec containing YAML front matter")
    parser.add_argument("--output", "-o", type=Path, required=True, help="generated JSON manifest path")
    parser.add_argument("--custom-tests", help="path or command for component-specific tests to run alongside")
    parser.add_argument("--check", action="store_true", help="fail unless output exactly matches the generated manifest")
    args = parser.parse_args(argv)
    try:
        manifest = generate(args.spec, args.output, args.custom_tests, args.check)
    except (OSError, SpecError) as error:
        print(f"conformance generation failed: {error}", file=sys.stderr)
        return 2
    verb = "checked" if args.check else "wrote"
    print(f"{verb} {args.output}: {len(manifest['keyboard_cases'])} keyboard, "
          f"{len(manifest['accessibility_cases'])} accessibility, "
          f"{len(manifest['screenshot_cases'])} screenshot cases")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
