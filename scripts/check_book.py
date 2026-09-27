#!/usr/bin/env python3
"""Check book includes, local links, screenshot provenance, and spelling.

Run `python scripts/check_book.py` for structural checks and the generated
concept-index consistency check. Add `--spell` after
installing codespell to check prose as well. Rust includes are compiled with
`cargo check --all-targets` unless `--skip-cargo` is passed (CI already builds
the workspace before running this checker).
"""

import argparse
import hashlib
from html.parser import HTMLParser
import json
import re
import struct
import subprocess
import sys
import tomllib
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
BOOK = ROOT / "book/src"
EXAMPLES = ROOT / "examples"
FENCE = re.compile(r"^ {0,3}(`{3,}|~{3,})(.*)$")
INCLUDE = re.compile(r"^\s*\{\{#include\s+([^}:\s]+):([A-Za-z_][\w-]*)\s*\}\}\s*$")
ANCHOR = re.compile(r"^\s*//\s*ANCHOR(?:_END)?:\s*([A-Za-z_][\w-]*)\s*$")
LINK = re.compile(r"(!?)\[[^\]\n]*\]\((<[^>]+>|[^)\s]+)(?:\s+\"[^\"]*\")?\)")
REFERENCE = re.compile(r"^ {0,3}\[[^\]\n]+\]:\s*(<[^>]+>|\S+)", re.MULTILINE)
HTML_ASSET = re.compile(r"<(a|img)\b[^>]*\b(?:href|src)=[\"']([^\"']+)[\"']", re.IGNORECASE)
STRING = re.compile(r'"([^"\n]+)"')
NON_RUST_FENCES = {"sh", "bash", "console", "text", "toml", "json", "yaml", "yml", "html", "css", "mermaid", "diff"}

# The book and the component site are dark by default, so the E7 component
# images are the shadcn dark 2x conformance baselines, not the 1x ones.
# Regenerate them with scripts/sync_e7_book_images.py.
E7_BOOK_THEME = "dark"


def within(path, directory):
    return path == directory or directory in path.parents


def markdown_files(book=BOOK):
    return sorted(book.rglob("*.md"))


def anchor_region(source, name):
    start = end = None
    stack = []
    for number, line in enumerate(source.read_text().splitlines(), 1):
        marker = ANCHOR.match(line)
        if not marker:
            continue
        marker_name = marker.group(1)
        if "ANCHOR_END:" in line:
            if not stack or stack[-1][0] != marker_name:
                raise ValueError(f"{source}:{number}: unmatched ANCHOR_END: {marker_name}")
            opened_name, opened_line = stack.pop()
            if opened_name == name:
                if start is not None:
                    raise ValueError(f"{source}: duplicate anchor {name}")
                start, end = opened_line, number
        else:
            stack.append((marker_name, number))
    if stack:
        raise ValueError(f"{source}: unclosed anchor {stack[-1][0]}")
    if start is None or end - start <= 1:
        raise ValueError(f"{source}: missing or empty anchor {name}")


def example_crate(source, root=ROOT):
    examples = root / "examples"
    if source.suffix != ".rs" or not within(source, examples):
        raise ValueError(f"include is not Rust source in examples/: {source}")
    crate = next((parent for parent in source.parents if parent.parent == examples), None)
    if crate is None or not (crate / "Cargo.toml").is_file():
        raise ValueError(f"include has no examples/ crate manifest: {source}")
    manifest = tomllib.loads((crate / "Cargo.toml").read_text())
    package = manifest.get("package", {}).get("name")
    workspace = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]
    if not package or crate.relative_to(root).as_posix() not in workspace["members"]:
        raise ValueError(f"include crate is not a workspace member: {source}")
    return crate


def rust_includes(page, root=ROOT):
    crates = set()
    lines = page.read_text().splitlines()
    opener = None
    body = []
    for number, line in enumerate([*lines, ""], 1):
        fence = FENCE.match(line)
        if opener is None:
            if fence:
                token, info = fence.groups()
                language = info.strip().lower().split(",", 1)[0].split(" ", 1)[0]
                if language not in {"rust", "rs", *NON_RUST_FENCES}:
                    raise ValueError(f"{page}:{number}: code fence needs an explicit supported language; Rust must use anchored includes")
                opener = (token[0], len(token), number, language in {"rust", "rs"})
                body = []
            continue
        if fence and fence.group(1)[0] == opener[0] and len(fence.group(1)) >= opener[1] and not fence.group(2).strip():
            if opener[3]:
                if not body or not any(part.strip() for _, part in body):
                    raise ValueError(f"{page}:{opener[2]}: empty Rust code block")
                for line_number, content in body:
                    match = INCLUDE.match(content)
                    if not match:
                        raise ValueError(f"{page}:{line_number}: Rust blocks must contain only anchored {{{{#include}}}} directives")
                    relative, anchor = match.groups()
                    source = (page.parent / relative).resolve()
                    if not source.is_file():
                        raise ValueError(f"{page}:{line_number}: include does not exist: {relative}")
                    crates.add(example_crate(source, root))
                    anchor_region(source, anchor)
            opener = None
            body = []
        else:
            body.append((number, line))
    if opener is not None:
        raise ValueError(f"{page}:{opener[2]}: unclosed code fence")
    return crates


def heading_ids(page):
    used = {}
    result = set()
    for line in page.read_text().splitlines():
        match = re.match(r"^#{1,6}\s+(.+?)\s*#*\s*$", line)
        if not match:
            continue
        slug = re.sub(r"[^\w\- ]", "", match.group(1).lower()).strip().replace(" ", "-")
        count = used.get(slug, 0)
        result.add(f"{slug}-{count}" if count else slug)
        used[slug] = count + 1
    result.update(re.findall(r'<a\s+(?:id|name)="([^"]+)"', page.read_text()))
    return result


def png_dimensions(path):
    with path.open("rb") as stream:
        header = stream.read(24)
    if len(header) < 24 or header[:8] != b"\x89PNG\r\n\x1a\n" or header[12:16] != b"IHDR":
        raise ValueError(f"not a PNG baseline: {path}")
    width, height = struct.unpack(">II", header[16:24])
    if width == 0 or height == 0:
        raise ValueError(f"empty PNG baseline: {path}")


def screenshot_provenance(image, root=ROOT):
    if image.suffix.lower() != ".png" or not within(image, root / "book/src/images"):
        raise ValueError(f"book screenshot must be a PNG under book/src/images/: {image}")
    png_dimensions(image)
    # E7 book images are exact copies of manifest-declared macOS baselines.
    e7_images = root / "book/src/images/e7"
    if image.parent == e7_images:
        component = image.stem
        component_root = root / "registry" / component
        manifest_path = component_root / "tests/conformance.json"
        if manifest_path.is_file():
            manifest = json.loads(manifest_path.read_text())
            if manifest.get("component") == component:
                digest = hashlib.sha256(image.read_bytes()).digest()
                baselines = component_root / "tests/baselines"
                for case in manifest.get("screenshot_cases", []):
                    if case.get("theme") != E7_BOOK_THEME or case.get("scale") != 2:
                        continue
                    relative = Path(case["baseline"])
                    candidates = [baselines / relative]
                    if relative.parts and relative.parts[0] == component:
                        candidates.append(baselines.joinpath(*relative.parts[1:]))
                    for candidate in candidates:
                        baseline = candidate.resolve()
                        if not within(baseline, baselines.resolve()) or not baseline.is_file():
                            continue
                        if hashlib.sha256(baseline.read_bytes()).digest() == digest:
                            return
        raise ValueError(f"no matching E7 registry {E7_BOOK_THEME}-2x screenshot baseline: {image}")
    # E8 component pages may reuse exact manifest-declared registry captures from
    # dedicated macOS state matrices. Keep the byte-for-byte provenance check.
    if image.name.startswith("e8-"):
        digest = hashlib.sha256(image.read_bytes()).digest()
        for manifest_path in sorted((root / "registry").glob("*/tests/conformance.json")):
            manifest = json.loads(manifest_path.read_text())
            baselines = manifest_path.parent / "baselines"
            for case in manifest.get("screenshot_cases", []):
                if case.get("platform") != "macos":
                    continue
                relative = Path(case["baseline"])
                candidates = [baselines / relative]
                if relative.parts and relative.parts[0] == manifest_path.parents[1].name:
                    candidates.append(baselines.joinpath(*relative.parts[1:]))
                for candidate in candidates:
                    baseline = candidate.resolve()
                    if not within(baseline, baselines.resolve()) or not baseline.is_file():
                        continue
                    if hashlib.sha256(baseline.read_bytes()).digest() == digest:
                        return
    for test in sorted((root / "examples").glob("*/tests/*.rs")):
        crate = test.parents[1]
        body = test.read_text()
        if not all(token in body for token in ("mkit_harness", "screenshot(", "UPDATE_SNAPSHOTS", ".save(")):
            continue
        for literal in STRING.findall(body):
            if literal.endswith(".png") and (crate / literal).resolve() == image:
                return
    raise ValueError(f"no examples/ harness screenshot test generates baseline: {image}")


def check_links(page, root=ROOT):
    body = page.read_text()
    targets = [(bool(mark), raw) for mark, raw in LINK.findall(body)]
    targets.extend((False, raw) for raw in REFERENCE.findall(body))
    targets.extend((tag.lower() == "img", raw) for tag, raw in HTML_ASSET.findall(body))
    for image_marker, raw in targets:
        target = raw[1:-1] if raw.startswith("<") and raw.endswith(">") else raw
        parsed = urlsplit(target)
        if parsed.scheme in ("http", "https", "mailto"):
            if parsed.scheme in ("http", "https") and not parsed.netloc:
                raise ValueError(f"{page}: malformed external link {target}")
            continue
        if parsed.scheme or parsed.netloc:
            raise ValueError(f"{page}: unsupported link scheme {target}")
        destination = (page.parent / unquote(parsed.path)).resolve() if parsed.path else page
        if not destination.exists():
            raise ValueError(f"{page}: broken local link {target}")
        if parsed.fragment and destination.suffix == ".md" and unquote(parsed.fragment) not in heading_ids(destination):
            raise ValueError(f"{page}: missing heading #{parsed.fragment} in {destination}")
        if image_marker:
            screenshot_provenance(destination, root)


def check_book(root=ROOT, compile_examples=True, spell=False):
    pages = markdown_files(root / "book/src")
    if not pages:
        raise ValueError("book/src has no Markdown pages")
    crates = set()
    for page in pages:
        crates.update(rust_includes(page, root))
        check_links(page, root)
    if compile_examples:
        for crate in sorted(crates):
            subprocess.run(["cargo", "check", "--manifest-path", str(crate / "Cargo.toml"), "--all-targets", "--locked"], cwd=root, check=True)
    if spell:
        subprocess.run(["codespell", "--quiet-level", "2", str(root / "book/src")], cwd=root, check=True)
    return len(pages), len(crates)


def check_built_banner(root=ROOT):
    manifest = tomllib.loads((root / "Cargo.toml").read_text())
    dependency = manifest["workspace"]["dependencies"]["gpui_pre"]
    if dependency.get("package") != "gpui-pre" or not dependency.get("version", "").startswith("="):
        raise ValueError("workspace GPUI dependency must pin gpui-pre exactly")
    version = dependency["version"][1:]
    config = tomllib.loads((root / "book/book.toml").read_text())
    if "version-banner" not in config.get("preprocessor", {}):
        raise ValueError("mdBook version-banner preprocessor is not configured")
    for page in markdown_files(root / "book/src"):
        if page.name == "SUMMARY.md":
            continue
        relative = page.relative_to(root / "book/src")
        output_name = "index.html" if relative.name == "README.md" else relative.with_suffix(".html").name
        html = root / "book/book" / relative.parent / output_name
        if not html.is_file():
            raise ValueError(f"mdBook did not render chapter: {page}")
        body = html.read_text()
        if body.count('class="gpui-version-banner"') != 1 or f"<code>gpui-pre {version}</code>" not in body:
            raise ValueError(f"built chapter has missing or incorrect GPUI pin banner: {html}")


class BuiltPageLinks(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.links = []
        self.ids = set()

    def handle_starttag(self, tag, attrs):
        attributes = dict(attrs)
        if attributes.get("id"):
            self.ids.add(attributes["id"])
        if tag == "a" and attributes.get("href"):
            self.links.append(("link", attributes["href"]))
        elif tag == "img" and attributes.get("src"):
            self.links.append(("image", attributes["src"]))

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)


def check_built_links(root=ROOT):
    """Verify reader-facing local links resolve inside the rendered mdBook site."""
    output_root = root / "book/book"
    if not output_root.is_dir():
        raise ValueError(f"missing built mdBook output: {output_root}; run mdbook build book")
    parsed_pages = {}
    for page in sorted(output_root.rglob("*.html")):
        parser = BuiltPageLinks()
        parser.feed(page.read_text())
        parsed_pages[page] = parser

    for page, parser in parsed_pages.items():
        for kind, target in parser.links:
            parsed = urlsplit(target)
            if parsed.scheme in ("http", "https", "mailto", "tel", "data"):
                if parsed.scheme in ("http", "https") and not parsed.netloc:
                    raise ValueError(f"{page}: malformed external {kind} {target}")
                continue
            if parsed.scheme or parsed.netloc:
                raise ValueError(f"{page}: unsupported built {kind} scheme {target}")
            path = unquote(parsed.path)
            if path.startswith("/"):
                destination = (output_root / path.lstrip("/")).resolve()
            elif path:
                destination = (page.parent / path).resolve()
            else:
                destination = page
            if not within(destination, output_root):
                raise ValueError(f"{page}: built local {kind} escapes mdBook output: {target}")
            if destination.is_dir() or target.endswith("/"):
                destination = destination / "index.html"
            if not destination.is_file():
                raise ValueError(f"{page}: broken built local {kind} {target} (resolved to {destination})")
            fragment = unquote(parsed.fragment)
            if fragment and destination.suffix.lower() == ".html":
                target_page = parsed_pages.get(destination)
                if target_page is None:
                    target_parser = BuiltPageLinks()
                    target_parser.feed(destination.read_text())
                    target_page = target_parser
                    parsed_pages[destination] = target_parser
                if fragment not in target_page.ids:
                    raise ValueError(f"{page}: missing built heading #{fragment} in {destination}")


def check_concept_index(root=ROOT):
    generator = root / "scripts/generate_concept_index.py"
    if not generator.is_file():
        raise ValueError(f"missing concept-index generator: {generator}")
    subprocess.run([sys.executable, str(generator), "--check"], cwd=root, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-cargo", action="store_true", help="use after a successful workspace build")
    parser.add_argument("--spell", action="store_true", help="also run codespell (must be installed)")
    parser.add_argument("--built-html", action="store_true", help="verify local links and the GPUI pin banner in rendered HTML after mdbook build")
    args = parser.parse_args()
    try:
        check_concept_index()
        pages, crates = check_book(compile_examples=not args.skip_cargo, spell=args.spell)
        if args.built_html:
            check_built_links()
            check_built_banner()
    except (ValueError, subprocess.CalledProcessError, FileNotFoundError) as error:
        print(f"book check failed: {error}", file=sys.stderr)
        return 1
    print(f"book check passed: {pages} pages, {crates} example crates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
