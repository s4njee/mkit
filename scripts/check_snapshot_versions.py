#!/usr/bin/env python3
"""Check crates.io for newer versions of the pinned GPUI snapshot and GPUI Kit crates (E11.1).

Pins are read from every Cargo.toml in the repository (renamed dependencies such as
`gpui_pre = { package = "gpui-pre", ... }` resolve to their package name). The latest
non-yanked version on crates.io is compared against the oldest pin of each crate.
Prereleases are ignored unless `--include-prerelease` is passed.

Exit status is 0 whether or not updates exist; the result is reported through
`--format`, `--body-file`, and `$GITHUB_OUTPUT` (`updates`, `title`).
"""

import argparse
import json
import os
import re
import sys
import tomllib
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WATCHED_CRATES = ("gpui-pre", "gpui-pre-platform", "gpui-kit")
API_URL = "https://crates.io/api/v1/crates/{name}"
CRATE_URL = "https://crates.io/crates/{name}"
USER_AGENT = "mkit-snapshot-watch/1 (GPUI pin checker; scripts/check_snapshot_versions.py)"
ISSUE_MARKER = "<!-- mkit-snapshot-watch -->"
DEPENDENCY_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")
SEMVER_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$")


def parse_version(text: str) -> tuple:
    """Return a sort key for a semver string; prereleases sort before their release."""
    match = SEMVER_RE.match(text.strip())
    if not match:
        raise ValueError(f"not a semver version: {text!r}")
    major, minor, patch, pre = match.groups()
    if pre is None:
        pre_key = (1,)
    else:
        pre_key = (0, *((0, int(p), "") if p.isdigit() else (1, 0, p) for p in pre.split(".")))
    return (int(major), int(minor), int(patch), pre_key)


def is_prerelease(text: str) -> bool:
    return parse_version(text)[3] != (1,)


def requirement_version(requirement: str) -> str:
    """Extract the version from a requirement such as `=0.3.5`, `^0.6`, or `0.6.4`."""
    version = requirement.strip().lstrip("=^~<> ")
    parts = version.split("-", 1)
    core = parts[0].split(".")
    core += ["0"] * (3 - len(core))
    version = ".".join(core[:3]) + (f"-{parts[1]}" if len(parts) > 1 else "")
    parse_version(version)
    return version


def _dependency_tables(manifest: dict):
    for name in DEPENDENCY_TABLES:
        yield name, manifest.get(name, {})
    yield "workspace.dependencies", manifest.get("workspace", {}).get("dependencies", {})
    for target, table in manifest.get("target", {}).items():
        for name in DEPENDENCY_TABLES:
            yield f"target.{target}.{name}", table.get(name, {})


def parse_pins(text: str, manifest_path: str, watched=WATCHED_CRATES) -> list[dict]:
    """Return the watched-crate pins declared in one Cargo.toml text."""
    pins = []
    for table_name, table in _dependency_tables(tomllib.loads(text)):
        for key, spec in table.items():
            if isinstance(spec, str):
                package, requirement = key, spec
            elif isinstance(spec, dict) and "version" in spec:
                package, requirement = spec.get("package", key), spec["version"]
            else:
                continue  # `workspace = true`, path, or git dependency
            if package in watched:
                pins.append({
                    "crate": package,
                    "requirement": requirement,
                    "version": requirement_version(requirement),
                    "manifest": manifest_path,
                    "table": table_name,
                })
    return pins


def find_manifests(root: Path) -> list[Path]:
    return sorted(
        path for path in root.rglob("Cargo.toml")
        if not any(part == "target" or part.startswith(".") for part in path.relative_to(root).parts)
    )


def collect_pins(root: Path, watched=WATCHED_CRATES) -> list[dict]:
    pins = []
    for path in find_manifests(root):
        pins.extend(parse_pins(path.read_text(), path.relative_to(root).as_posix(), watched))
    return pins


def latest_version(api_response: dict, include_prerelease: bool = False) -> str | None:
    """Return the highest non-yanked version from a crates.io `/api/v1/crates/<name>` response."""
    candidates = [
        v["num"] for v in api_response.get("versions") or []
        if not v.get("yanked") and (include_prerelease or not is_prerelease(v["num"]))
    ]
    if not candidates:
        crate = api_response.get("crate", {})
        fallback = crate.get("max_version") if include_prerelease else crate.get("max_stable_version")
        return fallback or None
    return max(candidates, key=parse_version)


def find_updates(pins: list[dict], api_responses: dict[str, dict], include_prerelease: bool = False) -> list[dict]:
    """Compare pins against crates.io responses; return one entry per crate with a newer release."""
    by_crate: dict[str, list[dict]] = {}
    for pin in pins:
        by_crate.setdefault(pin["crate"], []).append(pin)
    updates = []
    for crate in sorted(by_crate):
        latest = latest_version(api_responses.get(crate, {}), include_prerelease)
        crate_pins = by_crate[crate]
        pinned = sorted({p["version"] for p in crate_pins}, key=parse_version)
        if latest is None or parse_version(latest) <= parse_version(pinned[0]):
            continue
        updates.append({
            "crate": crate,
            "pinned": pinned,
            "latest": latest,
            "url": CRATE_URL.format(name=crate),
            "manifests": sorted({p["manifest"] for p in crate_pins}),
        })
    return updates


def render_issue(updates: list[dict]) -> tuple[str, str]:
    """Return a deterministic issue title and markdown body for the given updates."""
    title = "Snapshot watch: " + ", ".join(f"{u['crate']} {u['latest']}" for u in updates) + " available"
    lines = [
        ISSUE_MARKER,
        "Newer versions of pinned GPUI crates are available on crates.io (plan item E11.1).",
        "",
        "| Crate | Pinned | Latest | crates.io |",
        "| --- | --- | --- | --- |",
    ]
    for u in updates:
        pinned = ", ".join(f"`{v}`" for v in u["pinned"])
        lines.append(f"| `{u['crate']}` | {pinned} | `{u['latest']}` | [{u['crate']}]({u['url']}) |")
    lines += ["", "<details><summary>Manifests with pins</summary>", ""]
    for u in updates:
        lines.append(f"- `{u['crate']}`: " + ", ".join(f"`{m}`" for m in u["manifests"]))
    lines += [
        "",
        "</details>",
        "",
        "### Follow-up (see AGENTS.md and plan item E11.2)",
        "",
        "- [ ] Bump the pins in every manifest listed above (keep `gpui-pre` and `gpui-pre-platform` in lockstep)",
        "- [ ] Check that the GPUI Kit release targets the same GPUI snapshot before bumping either",
        "- [ ] Run the workspace build, tests, clippy, and book build",
        "- [ ] Regenerate the GPUI API inventory (`python scripts/generate_gpui_inventory.py`) and review the diff",
        "- [ ] Run the coexistence sample (`apps/coexistence`)",
        "- [ ] Run harness keyboard scripts, accessibility snapshots, and screenshots; inspect diffs before updating baselines",
        "- [ ] Request human review for visual baseline changes and book updates",
        "",
        "_This issue is opened and updated by `.github/workflows/snapshot-watch.yml`._",
    ]
    return title, "\n".join(lines) + "\n"


def fetch_crate(name: str, timeout: float = 30.0) -> dict:
    request = urllib.request.Request(
        API_URL.format(name=name),
        headers={"User-Agent": USER_AGENT, "Accept": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return json.load(response)


def write_github_output(path: str, updates: list[dict], title: str) -> None:
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(f"updates={'true' if updates else 'false'}\n")
        handle.write(f"title={title}\n")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", type=Path, default=ROOT, help="repository root to scan for Cargo.toml files")
    parser.add_argument("--format", choices=("json", "markdown"), default="json")
    parser.add_argument("--include-prerelease", action="store_true", help="treat prerelease versions as updates")
    parser.add_argument("--body-file", type=Path, help="write the issue body here when updates exist")
    parser.add_argument("--github-output", default=os.environ.get("GITHUB_OUTPUT"), help="defaults to $GITHUB_OUTPUT")
    args = parser.parse_args(argv)

    pins = collect_pins(args.root)
    missing = sorted(set(WATCHED_CRATES) - {p["crate"] for p in pins})
    if missing:
        print(f"note: no pins found for {', '.join(missing)}", file=sys.stderr)
    responses = {crate: fetch_crate(crate) for crate in sorted({p["crate"] for p in pins})}
    updates = find_updates(pins, responses, args.include_prerelease)
    title, body = render_issue(updates) if updates else ("", "")

    if args.format == "json":
        latest = {c: latest_version(r, args.include_prerelease) for c, r in responses.items()}
        print(json.dumps({"updates": updates, "latest": latest}, indent=2))
    else:
        print(f"# {title}\n\n{body}" if updates else "All watched GPUI crates are up to date.")
    if updates and args.body_file:
        args.body_file.write_text(body)
    if args.github_output:
        write_github_output(args.github_output, updates, title)
    return 0


if __name__ == "__main__":
    sys.exit(main())
