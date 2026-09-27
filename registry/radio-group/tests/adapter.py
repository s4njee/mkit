#!/usr/bin/env python3
"""Execute generated radio-group keyboard cases in a real GPUI test window."""

import json
import os
import subprocess
import sys

case = json.load(sys.stdin)
if case.get("kind") != "keyboard_cases":
    print(json.dumps({"status": "unsupported", "reason": "active-platform accessibility and per-state screenshots are not available"}))
    raise SystemExit(0)

result = subprocess.run(
    ["cargo", "run", "--quiet", "-p", "mkit", "--example", "radio_group_conformance"],
    input=json.dumps(case), text=True, capture_output=True, env=os.environ.copy(), check=False,
)
if result.returncode:
    sys.stderr.write(result.stderr)
    sys.stderr.write(result.stdout)
    raise SystemExit(result.returncode)
try:
    output = json.loads(result.stdout.strip())
except json.JSONDecodeError as error:
    sys.stderr.write(f"radio-group adapter returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)

actual = output["actual"]
for field, expected in case.get("assert", {}).items():
    actual_field = "event" if field == "event" else field
    if actual.get(actual_field) != expected:
        raise SystemExit(f"{case['id']}: {actual_field} expected {expected!r}, got {actual.get(actual_field)!r}")
event = case.get("assert", {}).get("event", "")
if event.startswith("ChangeRequested("):
    expected_focused_option = event.removeprefix("ChangeRequested(").removesuffix(")")
    if actual.get("focused_option") != expected_focused_option:
        raise SystemExit(
            f"{case['id']}: focused_option expected {expected_focused_option!r}, "
            f"got {actual.get('focused_option')!r}"
        )
print(json.dumps(output))
