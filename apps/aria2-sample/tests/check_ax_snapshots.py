#!/usr/bin/env python3
"""Check the captured native macOS AX trees for the consumer's release flows."""
from pathlib import Path
import json

ROOT = Path(__file__).with_name("ax-snapshots")


def nodes(scene):
    root = json.loads((ROOT / f"{scene}.json").read_text())
    def visit(node):
        yield node
        for child in node.get("children", []):
            yield from visit(child)
    return list(visit(root))


def expect(scene, role, title, subrole=None):
    found = any(
        node.get("role") == role
        and node.get("title") == title
        and (subrole is None or node.get("subrole") == subrole)
        for node in nodes(scene)
    )
    assert found, f"{scene}: missing {role} {title!r} {subrole or ''}"


expect("main", "AXTable", "Downloads")
assert len([node for node in nodes("main") if node["role"] == "AXRow"]) == 8
expect("main", "AXTextField", "Filter downloads")
expect("add", "AXWindow", "Add download", "AXDialog")
assert not any(node["role"] == "AXTable" for node in nodes("add")), "modal background leaked into AX tree"
for role, title in [
    ("AXTextField", "Download URL"),
    ("AXTextField", "Save to"),
    ("AXPopUpButton", "Queue"),
    ("AXSlider", "Connections"),
    ("AXTextField", "Split count"),
    ("AXTextField", "Minimum split size"),
    ("AXButton", "Start download"),
]:
    expect("add", role, title)
expect("settings", "AXTextField", "RPC secret token", "AXSecureTextField")
assert next(node for node in nodes("settings") if node.get("title") == "RPC secret token")["value"] == ""
print("PASS: native AX snapshots for main list, Add URL, and Settings")
