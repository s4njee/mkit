#!/usr/bin/env python3
"""Run generated DataTable keyboard cases in a real GPUI test window."""

import json
import os
import subprocess
import sys


case = json.load(sys.stdin)
if case.get("kind") != "keyboard_cases":
    print(json.dumps({
        "status": "unsupported",
        "reason": "active-platform accessibility and screenshot capture are unavailable in this adapter",
    }))
    raise SystemExit(0)

if case.get("id") not in {f"keyboard-{i:02}" for i in range(1, 10)}:
    print(json.dumps({"status": "unsupported", "reason": "unmapped DataTable keyboard case"}))
    raise SystemExit(0)

key = case["dispatch"]["key"]
assert case["dispatch"]["modifiers"] == []
state = case["initial_state"]
if state == "active-header":
    assert key == "Enter"
    pre_dispatch = []
elif state == "active-cell":
    pre_dispatch = {
        "ArrowDown": ["ArrowDown"],
        "ArrowUp": ["ArrowDown", "ArrowDown"],
        "ArrowRight": ["ArrowDown"],
        "ArrowLeft": ["ArrowDown", "ArrowRight"],
        "Home": ["ArrowDown", "ArrowRight"],
        "End": ["ArrowDown"],
        "Enter": ["ArrowDown"],
        "Space": ["ArrowDown"],
    }[key]
else:
    raise SystemExit(f"unsupported DataTable fixture {state!r}")

request = {**case, "pre_dispatch": pre_dispatch}
result = subprocess.run(
    ["cargo", "run", "--quiet", "-p", "mkit-registry-data-table", "--example", "data_table_conformance"],
    input=json.dumps(request),
    text=True,
    capture_output=True,
    env=os.environ.copy(),
    check=False,
)
if result.returncode:
    sys.stderr.write(result.stderr)
    sys.stderr.write(result.stdout)
    raise SystemExit(result.returncode)
try:
    output = json.loads(result.stdout.strip())
except json.JSONDecodeError as error:
    sys.stderr.write(f"DataTable conformance returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)

actual = output["actual"]
expected_event = case["assert"]["event"]
if expected_event not in actual["events"]:
    raise SystemExit(f"{case['id']}: missing {expected_event}: {actual!r}")
if key in {"ArrowDown", "ArrowUp"} and "ActiveChanged" not in actual["events"]:
    raise SystemExit(f"{case['id']}: row navigation omitted ActiveChanged: {actual!r}")
expected_cell = {
    "ArrowDown": "row-a:name",
    "ArrowUp": "row-b:name",
    "ArrowRight": "row-b:kind",
    "ArrowLeft": "row-b:name",
    "Home": "row-b:name",
    "End": "row-b:kind",
}.get(key)
if expected_cell is not None and actual["active_cell"] != expected_cell:
    raise SystemExit(f"{case['id']}: expected active cell {expected_cell}, got {actual!r}")
if key in {"Enter", "Space"} and state == "active-cell" and "row-b" not in actual["selection"]:
    raise SystemExit(f"{case['id']}: {key} did not select active row: {actual!r}")
if key == "Enter" and state == "active-header":
    if actual["sort"] != {"column": "name", "direction": "Ascending"}:
        raise SystemExit(f"{case['id']}: Enter did not sort active header: {actual!r}")

actual["event"] = expected_event
print(json.dumps({"passed": True, "actual": actual}))
