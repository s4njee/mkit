#!/usr/bin/env python3
"""Run generated multi-select keys against a real GPUI TestApp window."""

import json
import os
import subprocess
import sys

case = json.load(sys.stdin)
if case.get("kind") != "keyboard_cases":
    print(json.dumps({"status": "unsupported", "reason": "active-platform accessibility and per-state screenshots are not available"}))
    raise SystemExit(0)

result = subprocess.run(
    ["cargo", "run", "--quiet", "-p", "mkit", "--example", "multi_select_conformance"],
    input=json.dumps(case), text=True, capture_output=True, env=os.environ.copy(), check=False,
)
if result.returncode:
    sys.stderr.write(result.stderr)
    sys.stderr.write(result.stdout)
    raise SystemExit(result.returncode)
try:
    output = json.loads(result.stdout.strip())
except json.JSONDecodeError as error:
    sys.stderr.write(f"multi-select adapter returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)

actual = output["actual"]
for field, expected in case.get("assert", {}).items():
    if actual.get(field) != expected:
        raise SystemExit(f"{case['id']}: {field} expected {expected!r}, got {actual.get(field)!r}")

key = case["dispatch"]["key"]
if key == "ArrowDown" and case["initial_state"] == "closed" and actual["active_option"] != "beta":
    raise SystemExit(f"{case['id']}: ArrowDown did not skip the disabled row")
if key == "Space" and case["initial_state"] == "open" and actual["values"] != []:
    raise SystemExit(f"{case['id']}: Space did not remove the active selected value")
if key == "Escape" and actual["values"] != actual["values_before"]:
    raise SystemExit(f"{case['id']}: Escape changed committed values")
if key == "Home" and actual["active_option"] != "alpha":
    raise SystemExit(f"{case['id']}: Home did not select the first enabled row")
if key == "End" and actual["active_option"] != "item-11":
    raise SystemExit(f"{case['id']}: End did not select the last enabled row")
if key == "Tab":
    expected_target = "previous_tab_stop" if "Shift" in case["dispatch"].get("modifiers", []) else "next_tab_stop"
    if actual["focus_target"] != expected_target:
        raise SystemExit(f"{case['id']}: Tab traversal expected {expected_target!r}, got {actual['focus_target']!r}")
    if actual["state"] != "closed":
        raise SystemExit(f"{case['id']}: Tab left the popup open")
    if actual["values"] != actual["values_before"]:
        raise SystemExit(f"{case['id']}: Tab traversal changed committed values")
if case["initial_state"] == "disabled" and actual["values"] != actual["values_before"]:
    raise SystemExit(f"{case['id']}: disabled input changed values")
print(json.dumps(output))
