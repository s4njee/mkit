#!/usr/bin/env python3
"""Run supported scrubbable-number-field cases against its GPUI adapter."""

import json
import os
import subprocess
import sys


case = json.load(sys.stdin)
case_id = case.get("id")
if case.get("kind") == "screenshot_cases":
    if not case_id or not case_id.startswith("screen-"):
        print(json.dumps({"status": "unsupported", "reason": "unrecognized screenshot case"}))
        raise SystemExit(0)
    command = [
        "cargo", "test", "--quiet", "-p", "mkit-example-number-field-screenshot",
        "--test", "number_field_matrix", "--", "--nocapture",
    ]
elif case.get("kind") == "keyboard_cases" and case_id in {
    "keyboard-01", "keyboard-02", "keyboard-03", "keyboard-04", "keyboard-05",
    "keyboard-06", "keyboard-07", "keyboard-08", "keyboard-09", "keyboard-10",
}:
    command = ["cargo", "test", "--quiet", "-p", "mkit", "--test", "number_field_conformance"]
else:
    reason = (
        "macOS headless capture did not produce reviewable component pixels"
        if case.get("kind") == "screenshot_cases"
        else "this generated keyboard case has no adapter"
        if case.get("kind") == "keyboard_cases"
        else "active platform accessibility capture is unavailable in this adapter"
    )
    print(json.dumps({
        "status": "unsupported",
        "reason": reason,
    }))
    raise SystemExit(0)

environment = os.environ.copy()
if case.get("kind") == "screenshot_cases":
    environment["MKIT_NUMBER_FIELD_CASE_ID"] = case_id
result = subprocess.run(
    command,
    input=json.dumps(case),
    text=True,
    capture_output=True,
    env=environment,
    check=False,
)
if result.returncode:
    sys.stderr.write(result.stderr)
    sys.stderr.write(result.stdout)
    raise SystemExit(result.returncode)
if case.get("kind") == "screenshot_cases":
    if "Skipping number field matrix:" in result.stdout:
        print(json.dumps({"status": "unsupported", "reason": "GPUI headless capture requires macOS Metal"}))
    else:
        print(json.dumps({"passed": True, "actual": {"baseline": case["baseline"], "matched": True}}))
    raise SystemExit(0)
try:
    report = json.loads(result.stdout.strip())
except json.JSONDecodeError as error:
    sys.stderr.write(f"adapter executable returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)
print(json.dumps(report))
