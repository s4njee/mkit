#!/usr/bin/env python3
"""Run generated Tree keyboard cases in a real GPUI test window."""

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

if case.get("id") not in {"keyboard-01", "keyboard-02"}:
    print(json.dumps({"status": "unsupported", "reason": "unmapped Tree keyboard case"}))
    raise SystemExit(0)

result = subprocess.run(
    ["cargo", "test", "--quiet", "-p", "mkit", "--test", "tree_conformance"],
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
    sys.stderr.write(f"Tree conformance returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)
