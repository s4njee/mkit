#!/usr/bin/env python3
"""Check every registry component spec and its generated conformance cases."""

from __future__ import annotations

from pathlib import Path
import sys

from generate_conformance import SpecError, generate


ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "registry"
REQUIRED_SECTIONS = (
    "Purpose", "Anatomy", "States", "Props and events", "Keyboard map",
    "Pointer behaviour", "Accessibility role and properties",
    "Theme tokens used", "WAI-ARIA pattern reference", "Platform notes",
    "Open questions",
)


def check_registry(registry: Path = REGISTRY) -> list[str]:
    errors: list[str] = []
    for spec in sorted(registry.glob("*/spec.md")):
        source = spec.read_text(encoding="utf-8")
        for section in REQUIRED_SECTIONS:
            if f"\n## {section}\n" not in source:
                errors.append(f"{spec}: missing required section {section!r}")
        manifest = spec.parent / "tests" / "conformance.json"
        try:
            generated = generate(spec, manifest, check=True)
        except (OSError, SpecError) as error:
            errors.append(f"{spec}: {error}")
            continue
        if generated["component"] != spec.parent.name:
            errors.append(
                f"{spec}: component name {generated['component']!r} must match directory {spec.parent.name!r}"
            )
    return errors


def main() -> int:
    errors = check_registry()
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    count = sum(1 for _ in REGISTRY.glob("*/spec.md"))
    print(f"checked {count} registry component specs and conformance manifests")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
