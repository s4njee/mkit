#!/usr/bin/env python3
"""Run supported generated combobox keyboard cases through GPUI."""

import json
import os
import subprocess
import sys


case = json.load(sys.stdin)
case_id = case.get("id")
if case.get("kind") != "keyboard_cases":
    print(json.dumps({
        "status": "unsupported",
        "reason": "the draft has no real platform accessibility or screenshot adapter",
    }))
    raise SystemExit(0)

if case_id not in {f"keyboard-{index:02}" for index in (1, 2, 3, 4, 5, 6, 7, 8, 9)}:
    print(json.dumps({"status": "unsupported", "reason": "no generated keyboard case adapter"}))
    raise SystemExit(0)

result = subprocess.run(
    ["cargo", "test", "--quiet", "-p", "mkit", "--test", "combobox_conformance"],
    input=json.dumps(case),
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
    report = json.loads(result.stdout.strip())
except json.JSONDecodeError as error:
    sys.stderr.write(f"adapter executable returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)
print(json.dumps(report))
