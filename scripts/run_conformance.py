#!/usr/bin/env python3
"""Execute a generated conformance manifest through a component adapter.

The adapter is a command that reads one JSON case from stdin and writes one
JSON result to stdout. It must return {"passed": true, "actual": ...}; an
unsupported capability is a failure/pending gate, never a pass. See the
component-spec tooling docs for the adapter contract.
"""

from __future__ import annotations

import argparse
import json
import platform
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any


CASE_KINDS = {
    "keyboard": "keyboard_cases",
    "accessibility": "accessibility_cases",
    "screenshot": "screenshot_cases",
}


def run_case(command: list[str], case: dict[str, Any]) -> tuple[bool, str]:
    try:
        result = subprocess.run(command, input=json.dumps(case), text=True, capture_output=True, check=False)
    except OSError as error:
        return False, f"could not start adapter: {error}"
    if result.returncode:
        return False, f"adapter exited {result.returncode}: {result.stderr.strip()}"
    try:
        report = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        return False, f"adapter did not return JSON: {error}"
    if not isinstance(report, dict):
        return False, "adapter result must be a JSON object"
    if report.get("status") == "unsupported":
        return False, f"unsupported_pending: {report.get('reason', 'adapter capability unavailable')}"
    if report.get("passed") is not True:
        return False, f"adapter reported failure; actual={report.get('actual', report)}"
    verified, reason = verify_actual(case, report.get("actual"))
    if not verified:
        return False, reason
    return True, "passed"


def verify_actual(case: dict[str, Any], actual: Any) -> tuple[bool, str]:
    """Compare structured adapter evidence with the generated expectations."""
    if not isinstance(actual, dict):
        return False, "adapter actual must be a JSON object with assertion evidence"
    kind = case.get("kind")
    if kind == "keyboard_cases":
        expected = case.get("assert")
        if not isinstance(expected, dict) or not expected:
            return False, "generated keyboard case has no assertions"
        for field, value in expected.items():
            if actual.get(field) != value:
                return False, f"keyboard {field} mismatch: expected {value!r}, got {actual.get(field)!r}"
        return True, "matched keyboard assertions"
    if kind == "accessibility_cases":
        expected = case.get("expected_semantics")
        if not isinstance(expected, dict):
            return False, "generated accessibility case has no expected semantics"
        if actual.get("role") != expected.get("role"):
            return False, f"accessibility role mismatch: expected {expected.get('role')!r}, got {actual.get('role')!r}"
        actual_properties = actual.get("properties")
        if not isinstance(actual_properties, dict):
            return False, "accessibility actual.properties must be a JSON object"
        for prop in expected.get("properties", []):
            name, value = prop.get("name"), prop.get("value")
            if name not in actual_properties or actual_properties[name] != value:
                return False, f"accessibility property {name!r} mismatch: expected {value!r}, got {actual_properties.get(name)!r}"
        # A captured tree snapshot is optional evidence today, but once an
        # adapter reports one it must name the component's per-state baseline
        # and match it; a changed tree is a failure until a reviewed update.
        snapshot = actual.get("snapshot_baseline")
        if snapshot is not None:
            component, state = case.get("component"), case.get("state")
            expected_path = f"registry/{component}/tests/baselines/a11y/{state}.txt"
            if snapshot != expected_path:
                return False, f"accessibility snapshot baseline mismatch: expected {expected_path!r}, got {snapshot!r}"
            if actual.get("snapshot_matched") is not True:
                reason = actual.get("snapshot_error", "captured tree differs from the baseline")
                return False, f"accessibility snapshot {snapshot} not matched: {reason}"
            return True, "matched accessibility assertions and tree snapshot"
        return True, "matched accessibility assertions"
    if kind == "screenshot_cases":
        baseline = case.get("baseline")
        if actual.get("baseline") != baseline:
            return False, f"screenshot baseline mismatch: expected {baseline!r}, got {actual.get('baseline')!r}"
        if actual.get("matched") is not True:
            return False, "screenshot evidence must include matched=true"
        return True, "matched screenshot baseline"
    return False, f"unknown conformance case kind {kind!r}"


def select_cases(
    manifest: dict[str, Any], kinds: set[str] | None = None,
    case_ids: set[str] | None = None,
) -> list[dict[str, Any]]:
    selected = []
    for section in ("keyboard_cases", "accessibility_cases", "screenshot_cases"):
        if kinds is not None and section not in kinds:
            continue
        for case in manifest.get(section, []):
            if case_ids is None or case.get("id") in case_ids:
                selected.append({"component": manifest["component"], "kind": section, **case})
    return selected


def run_manifest(manifest: dict[str, Any], adapter: list[str],
                 kinds: set[str] | None = None,
                 case_ids: set[str] | None = None) -> tuple[int, int, int, int]:
    passed = failed = pending = 0
    cases = select_cases(manifest, kinds, case_ids)
    for case in cases:
        if case["kind"] == "screenshot_cases" and platform.system() != "Darwin":
            ok, message = False, "unsupported_pending: screenshot comparison requires macOS"
        else:
            ok, message = run_case(adapter, case)
        if ok:
            passed += 1
        else:
            failed += 1
            pending += message.startswith("unsupported_pending:")
        print(f"{'PASS' if ok else 'PENDING' if message.startswith('unsupported_pending:') else 'FAIL'} "
              f"{case['id']}: {message}")
    return passed, failed, pending, len(cases)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--adapter", required=True, help="component adapter command (quoted command line)")
    parser.add_argument("--kind", action="append", choices=CASE_KINDS,
                        help="run only this kind of case; repeat to select multiple kinds")
    parser.add_argument("--case", action="append", dest="case_ids",
                        help="run only this case ID; repeat to select multiple cases")
    args = parser.parse_args(argv)
    try:
        manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        print(f"cannot read manifest: {error}", file=sys.stderr)
        return 2
    if not isinstance(manifest, dict) or manifest.get("manifest_version") != 1:
        print("unsupported or malformed manifest", file=sys.stderr)
        return 2
    kinds = {CASE_KINDS[kind] for kind in args.kind} if args.kind else None
    case_ids = set(args.case_ids) if args.case_ids else None
    passed, failed, pending, selected = run_manifest(
        manifest, shlex.split(args.adapter), kinds, case_ids
    )
    if selected == 0:
        print("no conformance cases matched the requested filters", file=sys.stderr)
        return 2
    custom_tests = manifest.get("adapter_contract", {}).get("custom_tests")
    if custom_tests and not (kinds or case_ids):
        try:
            custom = subprocess.run(shlex.split(custom_tests), text=True, check=False)
            if custom.returncode:
                failed += 1
                print(f"FAIL custom tests: exited {custom.returncode}")
            else:
                passed += 1
                print("PASS custom tests")
        except OSError as error:
            failed += 1
            print(f"FAIL custom tests: {error}")
    print(f"conformance: {passed} passed, {failed} failed, {pending} pending")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
