#!/usr/bin/env python3
"""Run generated text-area keyboard cases in the mkit GPUI test window."""

import json
import os
import subprocess
import sys


case = json.load(sys.stdin)
if case.get("kind") != "keyboard_cases":
    print(json.dumps({
        "status": "unsupported",
        "reason": "active-platform accessibility and screenshot capture are not available",
    }))
    raise SystemExit(0)

if case.get("id") not in {
    "keyboard-01", "keyboard-02", "keyboard-03", "keyboard-04",
    "keyboard-05", "keyboard-06", "keyboard-07", "keyboard-08",
}:
    print(json.dumps({"status": "unsupported", "reason": "unmapped text-area keyboard case"}))
    raise SystemExit(0)

result = subprocess.run(
    ["cargo", "test", "--quiet", "-p", "mkit", "--test", "text_area_conformance"],
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
    sys.stderr.write(f"text-area conformance executable returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)
