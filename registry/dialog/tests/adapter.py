#!/usr/bin/env python3
"""Run dialog generated keyboard cases in a real GPUI window."""

import json
import os
import subprocess
import sys

case = json.load(sys.stdin)
if case.get("kind") != "keyboard_cases":
    print(json.dumps({"status": "unsupported", "reason": "active-platform accessibility and per-state screenshots are not available"}))
    raise SystemExit(0)
result = subprocess.run(
    ["cargo", "run", "--quiet", "-p", "mkit", "--example", "surface_conformance"],
    input=json.dumps(case), text=True, capture_output=True, env=os.environ.copy(), check=False,
)
if result.returncode:
    sys.stderr.write(result.stderr + result.stdout)
    raise SystemExit(result.returncode)
output = json.loads(result.stdout.strip())
actual = output["actual"]
for field, expected in case.get("assert", {}).items():
    if actual.get(field) != expected:
        raise SystemExit(f"{case['id']}: {field} expected {expected!r}, got {actual.get(field)!r}")
if case["dispatch"]["key"] == "Tab" and (actual["state"] != "open" or actual["events"]):
    raise SystemExit(f"{case['id']}: Tab dismissed the dialog")
print(json.dumps(output))
