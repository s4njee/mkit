#!/usr/bin/env python3
"""Run generated SegmentedControl keyboard cases through a real GPUI test window."""
import json
import os
import subprocess
import sys

case = json.load(sys.stdin)
if case.get("kind") != "keyboard_cases":
    print(json.dumps({"status": "unsupported", "reason": "active-platform accessibility and per-state screenshots are not available"}))
    raise SystemExit(0)
result = subprocess.run(
    ["cargo", "run", "--quiet", "-p", "mkit-registry-segmented_control", "--example", "segmented_control_conformance"],
    input=json.dumps(case), text=True, capture_output=True, env=os.environ.copy(), check=False,
)
if result.returncode:
    sys.stderr.write(result.stderr + result.stdout)
    raise SystemExit(result.returncode)
try:
    output = json.loads(result.stdout.strip())
except json.JSONDecodeError as error:
    sys.stderr.write(f"SegmentedControl adapter returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)
for field, expected in case.get("assert", {}).items():
    actual_field = {"focus": "focused_value"}.get(field, field)
    if output["actual"].get(actual_field) != expected:
        raise SystemExit(f"{case['id']}: {actual_field} expected {expected!r}, got {output['actual'].get(actual_field)!r}")
print(json.dumps(output))
