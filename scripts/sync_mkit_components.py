#!/usr/bin/env python3
"""Copy ownable registry sources into the publishable `mkit` crate."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[1]


def components() -> list[tuple[str, str, Path]]:
    catalog = json.loads((ROOT / "registry/registry.json").read_text())
    result = []
    for entry in catalog["components"]:
        sources = entry["source_files"]
        if not sources:
            continue
        if len(sources) != 1 or not sources[0].endswith("/src/lib.rs"):
            raise ValueError(f"{entry['name']}: mkit mirror requires one src/lib.rs source file")
        result.append((entry["name"], entry["name"].replace("-", "_"), ROOT / sources[0]))
    return result


def check_exports(entries: list[tuple[str, str, Path]]) -> list[str]:
    """Keep the crate's feature flags and public modules aligned with the catalog."""
    expected = {name: module for name, module, _ in entries}
    catalog = json.loads((ROOT / "registry/registry.json").read_text())
    component_dependencies = {
        entry["name"]: entry["component_dependencies"] for entry in catalog["components"]
    }
    manifest = tomllib.loads((ROOT / "crates/mkit/Cargo.toml").read_text())
    features = manifest["features"]
    errors = []
    if features.get("mkit-mirror") != []:
        errors.append("mkit internal feature `mkit-mirror` must be empty")
    for name in expected:
        required = component_dependencies[name]
        if required:
            required = [*required, "mkit-mirror"]
        if set(features.get(name, [])) != set(required) or len(features.get(name, [])) != len(required):
            errors.append(f"mkit feature `{name}` must enable exactly its registry component dependencies")
    if set(features["default"]) != set(expected) or len(features["default"]) != len(expected):
        errors.append("mkit default features must contain every registry source exactly once")

    source = (ROOT / "crates/mkit/src/lib.rs").read_text()
    pattern = re.compile(
        r'#\[cfg\(feature = "([^"]+)"\)\]\s*'
        r'#\[path = "generated/([^"/]+)\.rs"\]\s*'
        r'pub mod ([A-Za-z_][A-Za-z_0-9]*);'
    )
    actual = {}
    for match in pattern.finditer(source):
        feature, path_module, public_module = match.groups()
        if feature in actual:
            errors.append(f"mkit exports feature `{feature}` more than once")
        actual[feature] = (path_module, public_module)
    for name, module in expected.items():
        if actual.get(name) != (module, module):
            errors.append(f"mkit feature `{name}` must export generated/{module}.rs as `{module}`")
    for name in actual.keys() - expected.keys():
        errors.append(f"mkit exports unknown registry feature `{name}`")
    return errors


def sync(check: bool) -> tuple[list[str], int]:
    errors = []
    entries = components()
    errors.extend(check_exports(entries))
    for registry_name, module_name, source in entries:
        destination = ROOT / "crates" / "mkit" / "src" / "generated" / f"{module_name}.rs"
        contents = source.read_bytes()
        if check:
            if not destination.is_file() or destination.read_bytes() != contents:
                errors.append(f"{destination} is missing or stale; run scripts/sync_mkit_components.py")
        else:
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(contents)
    return errors, len(entries)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if generated copies are stale")
    args = parser.parse_args()
    try:
        errors, count = sync(args.check)
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
        print(f"component sync failed: {error}", file=sys.stderr)
        return 1
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(f"{'checked' if args.check else 'synced'} {count} mkit component module")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
