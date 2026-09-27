#!/usr/bin/env python3
"""Run one E5 overlay case through its real GPUI integration fixture."""

import json
import os
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[3]
case = json.load(sys.stdin)
kind = case["kind"]
if kind == "accessibility_cases":
    print(json.dumps({"status": "unsupported", "reason": "GPUI headless accessibility tree is inactive"}))
    raise SystemExit(0)
kind = {"keyboard_cases": "keyboard", "screenshot_cases": "screenshot"}.get(kind)
if not kind:
    print(json.dumps({"status": "unsupported", "reason": f"unsupported case kind {case['kind']}"}))
    raise SystemExit(0)
if kind == "screenshot" and (
    case["state"], case["theme"], case["scale"]
) != ("nested_open", "light", 1):
    print(json.dumps({"status": "unsupported", "reason": "this smoke fixture has only the nested-open light 1x baseline"}))
    raise SystemExit(0)

env = os.environ.copy()
env["MKIT_E5_CASE_KIND"] = kind
env["MKIT_E5_CASE_ID"] = case["id"]
if kind == "screenshot":
    env["MKIT_E5_ADAPTER"] = "1"
    target = "e4_overlay_screenshot"
else:
    dispatch = case["dispatch"]
    env["MKIT_E5_DISPATCH_KEY"] = dispatch["key"]
    env["MKIT_E5_DISPATCH_MODIFIERS"] = ",".join(dispatch["modifiers"])
    target = "e5_conformance"
result = subprocess.run(
    ["cargo", "test", "-p", "mkit-example-interaction", "--test", target]
    + ([] if kind == "screenshot" else ["--", "--exact", "e5_conformance_adapter", "--nocapture"]),
    cwd=root,
    env=env,
    text=True,
    capture_output=True,
    check=False,
)
marker = "MKIT_E5_RESULT="
for line in result.stdout.splitlines():
    if marker in line:
        print(line.split(marker, 1)[1])
        raise SystemExit(result.returncode)
sys.stderr.write(result.stdout)
sys.stderr.write(result.stderr)
raise SystemExit(result.returncode or 1)
