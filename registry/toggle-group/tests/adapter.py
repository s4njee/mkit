#!/usr/bin/env python3
"""Run generated toggle-group keyboard cases through a GPUI window."""

import json
import os
import subprocess
import sys


case = json.load(sys.stdin)
if case.get("kind") != "keyboard_cases":
    print(json.dumps({
        "status": "unsupported",
        "reason": "active-platform accessibility and per-state screenshots are not available",
    }))
    raise SystemExit(0)

if case.get("id") not in {f"keyboard-{index:02}" for index in range(1, 7)}:
    print(json.dumps({"status": "unsupported", "reason": "unmapped keyboard case"}))
    raise SystemExit(0)

result = subprocess.run(
    ["cargo", "run", "--quiet", "-p", "mkit", "--example", "toggle_group_conformance"],
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
    print(json.dumps(json.loads(result.stdout.strip())))
except json.JSONDecodeError as error:
    sys.stderr.write(f"toggle group example returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)
