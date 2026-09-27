#!/usr/bin/env python3
"""Keep the E5.4 review checklist aligned with checks that CI actually runs.

This verifies the review contract and CI wiring. It does not certify that an
individual component is complete; that requires evidence and maintainer review.
"""

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TEMPLATE = ROOT / ".github/pull_request_template.md"
WORKFLOW = ROOT / ".github/workflows/ci.yml"

REQUIRED_TEMPLATE_TEXT = (
    "Maintainer approved the spec, API, keyboard map, and accessibility contract.",
    "Generated keyboard cases, per-state accessibility snapshots",
    "state × light/dark/high-contrast × 1×/2× screenshot matrix",
    "three compiling",
    "`cargo mkit add`, the `mkit` crate, and GPUI Kit coexistence build.",
)
REQUIRED_CI_COMMANDS = (
    "cargo fmt --all -- --check",
    "cargo build --workspace --locked",
    "python scripts/check_component_specs.py",
    "python scripts/check_registry.py",
    "python scripts/run_conformance.py",
    "registry/combobox/tests/conformance.json",
    "registry/button/tests/conformance.json",
    "registry/icon-button/tests/conformance.json",
    "registry/toggle-button/tests/conformance.json",
    "registry/toggle-group/tests/conformance.json",
    "registry/checkbox/tests/conformance.json",
    "registry/radio-group/tests/conformance.json",
    "registry/switch/tests/conformance.json",
    "registry/slider/tests/conformance.json",
    "registry/multi-select/tests/conformance.json",
    "registry/select/tests/conformance.json",
    "registry/virtual-list/tests/conformance.json",
    "registry/tree/tests/conformance.json",
    "registry/data-table/tests/conformance.json",
    "registry/sidebar/tests/conformance.json",
    "registry/breadcrumbs/tests/conformance.json",
    "registry/tabs/tests/conformance.json",
    "registry/segmented-control/tests/conformance.json",
    "registry/scroll-area/tests/conformance.json",
    "registry/split-pane/tests/conformance.json",
    "registry/dropdown-menu/tests/conformance.json",
    "registry/context-menu/tests/conformance.json",
    "registry/text-field/tests/conformance.json",
    "registry/text-area/tests/conformance.json",
    "registry/toast/tests/conformance.json",
    "registry/dialog/tests/conformance.json",
    "registry/sheet/tests/conformance.json",
    "registry/scrubbable-number-field/tests/conformance.json",
    "--kind screenshot --case screen-idle-light-1x",
    "cargo test --workspace --locked",
    "cargo clippy --workspace --all-targets --locked -- -D warnings",
    "mdbook build book",
    "cargo build --manifest-path apps/coexistence/Cargo.toml --locked",
)


def validate(template: str, workflow: str) -> list[str]:
    errors = [f"PR template is missing required E5.4 text: {text}" for text in REQUIRED_TEMPLATE_TEXT if text not in template]
    errors.extend(f"CI is missing required E5.4 gate: {command}" for command in REQUIRED_CI_COMMANDS if command not in workflow)
    if "python scripts/check_e5_dod.py" not in workflow:
        errors.append("CI must run scripts/check_e5_dod.py")
    return errors


def main() -> int:
    try:
        errors = validate(TEMPLATE.read_text(), WORKFLOW.read_text())
    except OSError as error:
        print(f"E5.4 check failed: {error}", file=sys.stderr)
        return 1
    if errors:
        print("E5.4 check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("E5.4 checklist and available CI gates are wired")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
