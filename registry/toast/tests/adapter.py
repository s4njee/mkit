#!/usr/bin/env python3
"""Run generated toast keyboard cases through a real GPUI window."""

import json
import os
import subprocess
import sys

case = json.load(sys.stdin)
if case.get("kind") != "keyboard_cases":
    print(json.dumps({"status": "unsupported", "reason": "active-platform accessibility and per-state screenshots are not available"}))
    raise SystemExit(0)

result = subprocess.run(
    ["cargo", "run", "--quiet", "-p", "mkit", "--example", "toast_conformance"],
    input=json.dumps(case), text=True, capture_output=True, env=os.environ.copy(), check=False,
)
if result.returncode:
    sys.stderr.write(result.stderr)
    sys.stderr.write(result.stdout)
    raise SystemExit(result.returncode)
try:
    output = json.loads(result.stdout.strip())
except json.JSONDecodeError as error:
    sys.stderr.write(f"toast adapter returned invalid JSON: {error}\n{result.stdout}")
    raise SystemExit(1)

actual = output["actual"]
for field, expected in case.get("assert", {}).items():
    if actual.get(field) != expected:
        raise SystemExit(f"{case['id']}: {field} expected {expected!r}, got {actual.get(field)!r}")
if len(actual["events"]) > 1:
    raise SystemExit(f"{case['id']}: duplicate OpenChanged events")
print(json.dumps(output))
