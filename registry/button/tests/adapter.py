#!/usr/bin/env python3
"""Run generated button keyboard cases through a real GPUI test window."""

import json
import os
import subprocess
import sys


case = json.load(sys.stdin)
if case.get("kind") == "accessibility_cases":
    # Captures the AccessKit tree GPUI delivers to an active adapter and compares
    # it with tests/baselines/a11y/<state>.txt (see docs/E5_A11Y_CAPTURE.md).
    result = subprocess.run(
        ["cargo", "test", "--quiet", "-p", "mkit", "--test", "a11y_conformance"],
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
    print(result.stdout.strip().splitlines()[-1])
    raise SystemExit(0)
if case.get("kind") != "keyboard_cases":
    print(json.dumps({
        "status": "unsupported",
        "reason": "per-state screenshots are not available",
    }))
    raise SystemExit(0)

if case.get("id") not in {"keyboard-01", "keyboard-02"}:
    print(json.dumps({"status": "unsupported", "reason": "unmapped keyboard case"}))
    raise SystemExit(0)

result = subprocess.run(
    ["cargo", "test", "--quiet", "-p", "mkit", "--test", "button_conformance"],
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
    sys.stderr.write(f"button conformance executable returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)
