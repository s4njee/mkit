#!/usr/bin/env python3
"""Run the generated Select keyboard cases in a real GPUI test window."""

import json
import os
import subprocess
import sys


case = json.load(sys.stdin)
if case.get("kind") != "keyboard_cases":
    print(json.dumps({
        "status": "unsupported",
        "reason": "native accessibility and screenshot capture are not available in the headless adapter",
    }))
    raise SystemExit(0)

if case.get("id") not in {f"keyboard-{index:02}" for index in range(1, 6)}:
    print(json.dumps({"status": "unsupported", "reason": "unmapped Select keyboard case"}))
    raise SystemExit(0)

result = subprocess.run(
    ["cargo", "test", "--quiet", "-p", "mkit", "--test", "select_conformance"],
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
    sys.stderr.write(f"Select conformance executable returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)
