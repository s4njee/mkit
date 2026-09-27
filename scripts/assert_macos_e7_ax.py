#!/usr/bin/env python3
"""Wait for and assert representative native accessibility semantics in E7 gallery.

Usage: python3 scripts/assert_macos_e7_ax.py <pid> [--timeout 20]

Requires macOS Accessibility permission for the invoking process. The successful
capture is written as JSON to stdout so callers can preserve the authoritative
platform tree alongside the assertion result.
"""
import argparse
import json
import pathlib
import subprocess
import sys
import time


CAPTURE = pathlib.Path(__file__).with_name("capture_macos_ax.swift")

# These assertions intentionally cover representative main-gallery controls,
# not every component/state in the generated E5 conformance matrix.
EXPECTED = [
    ("AXCheckBox", "Email updates", "1", True),
    ("AXCheckBox", "Mixed", "1", True),
    ("AXRadioGroup", "Plan", None, True),
    ("AXSlider", "Volume", "42", True),
    ("AXProgressIndicator", "Partial progress", "64", True),
    ("AXTextField", "Workspace", "Design system", True),
    ("AXTextArea", "Release notes", "Managed by your administrator.", False),
    ("AXPopUpButton", "Team", "Design", True),
    ("AXPopUpButton", "Team (disabled)", "Engineering", False),
    ("AXPopUpButton", "Teams", "Design, Product", True),
    ("AXPopUpButton", "Teams (disabled)", "Engineering", False),
    ("AXComboBox", "City (disabled)", "San Francisco", False),
]


def walk(node):
    yield node
    for child in node.get("children", []):
        yield from walk(child)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pid", type=int)
    parser.add_argument("--timeout", type=float, default=20.0)
    args = parser.parse_args()
    if args.pid <= 0 or args.timeout <= 0:
        parser.error("pid and timeout must be positive")

    deadline = time.monotonic() + args.timeout
    last_error = "native tree has not populated"
    while time.monotonic() < deadline:
        result = subprocess.run(
            ["swift", str(CAPTURE), str(args.pid)],
            capture_output=True,
            text=True,
        )
        if result.returncode == 0:
            try:
                tree = json.loads(result.stdout)
            except json.JSONDecodeError as error:
                last_error = f"capture returned invalid JSON: {error}"
            else:
                nodes = list(walk(tree))
                missing = []
                for role, title, value, enabled in EXPECTED:
                    found = any(
                        node.get("role") == role
                        and node.get("title") == title
                        and node.get("value") == value
                        and node.get("enabled") is enabled
                        for node in nodes
                    )
                    if not found:
                        missing.append(f"{role} title={title!r} value={value!r} enabled={enabled}")
                if not missing:
                    print(json.dumps(tree, indent=2, sort_keys=True))
                    print(
                        f"PASS: {len(EXPECTED)} representative E7 native accessibility assertions",
                        file=sys.stderr,
                    )
                    return 0
                last_error = "missing expected semantics: " + "; ".join(missing)
        else:
            last_error = result.stderr.strip() or f"capture exited {result.returncode}"
            # Permission and process errors cannot improve by waiting.
            if "Accessibility permission" in last_error or "usage:" in last_error:
                break
        time.sleep(0.5)

    print(f"FAIL: E7 accessibility assertion timed out: {last_error}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
