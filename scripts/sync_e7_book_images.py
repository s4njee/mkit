#!/usr/bin/env python3
"""Sync the E7 component screenshots used by the book and the site.

The book and the component site are dark by default, so the component images in
`book/src/images/e7/` must be the shadcn **dark** 2x baselines rather than the
1x ones. Each image is most recently a 1x capture of one state; this
script finds that state in the component's conformance manifest and replaces the
image with the matching dark 2x baseline.

Run `python3 scripts/sync_e7_book_images.py` after refreshing the registry
baselines. `scripts/check_book.py` verifies the dark 2x provenance.

This is idempotent: an image already equal to the dark 2x baseline is left alone.
"""

import hashlib
import json
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BOOK_E7 = ROOT / "book/src/images/e7"
REGISTRY = ROOT / "registry"
LIGHT_THEME = "light"
DARK_THEME = "dark"


def baseline_paths(component_root, case):
    baselines = component_root / "tests/baselines"
    relative = Path(case["baseline"])
    candidates = [baselines / relative]
    if relative.parts and relative.parts[0] == component_root.name:
        candidates.append(baselines.joinpath(*relative.parts[1:]))
    return [candidate.resolve() for candidate in candidates]


def find_case(manifest, component_root, theme, scale, state=None):
    for case in manifest.get("screenshot_cases", []):
        if case.get("theme") != theme or case.get("scale") != scale:
            continue
        if state is not None and case.get("state") != state:
            continue
        for candidate in baseline_paths(component_root, case):
            if candidate.is_file():
                return case, candidate
    return None, None


def digest(path):
    return hashlib.sha256(path.read_bytes()).digest()


def sync(book_e7=BOOK_E7, registry=REGISTRY):
    if not book_e7.is_dir():
        raise SystemExit(f"missing book component images: {book_e7}")
    changed, current, skipped = [], [], []
    for image in sorted(book_e7.glob("*.png")):
        component = image.stem
        component_root = registry / component
        manifest_path = component_root / "tests/conformance.json"
        if not manifest_path.is_file():
            skipped.append((component, "no conformance manifest"))
            continue
        manifest = json.loads(manifest_path.read_text())
        image_digest = digest(image)

        # Already the dark 2x baseline? Then nothing to do.
        dark_case, dark_path = find_case(manifest, component_root, DARK_THEME, 2)
        if dark_path is not None and digest(dark_path) == image_digest:
            current.append(component)
            continue

        # Otherwise identify which 1x state the image currently shows by
        # matching it against the dark 1x baselines first, then light 1x.
        matched_state = None
        for theme in (DARK_THEME, LIGHT_THEME):
            for case in manifest.get("screenshot_cases", []):
                if case.get("theme") != theme or case.get("scale") != 1:
                    continue
                for candidate in baseline_paths(component_root, case):
                    if candidate.is_file() and digest(candidate) == image_digest:
                        matched_state = case.get("state")
                        break
                if matched_state is not None:
                    break
            if matched_state is not None:
                break
        if matched_state is None:
            skipped.append((component, "image does not match a 1x baseline"))
            continue

        dark_case, dark_path = find_case(manifest, component_root, DARK_THEME, 2, matched_state)
        if dark_path is None:
            skipped.append((component, f"no dark 2x baseline for state {matched_state!r}"))
            continue
        shutil.copyfile(dark_path, image)
        changed.append(component)

    return changed, current, skipped


def main():
    changed, current, skipped = sync()
    for component in changed:
        print(f"updated {component}.png to the shadcn dark 2x baseline")
    print(
        f"E7 book images: {len(changed)} updated, {len(current)} already dark, "
        f"{len(skipped)} skipped"
    )
    for component, reason in skipped:
        print(f"  skipped {component}: {reason}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
