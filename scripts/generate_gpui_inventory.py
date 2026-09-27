#!/usr/bin/env python3
"""Generate the pinned GPUI public-name inventory from rustdoc JSON.

`--check` compares the checked-in snapshots, union JSON, and Markdown without
running nightly rustdoc. `--regenerate` refreshes the macOS snapshot and union.
Refresh other snapshots with `--update-linux`, `--update-windows`, `--update-wasm`,
and `--update-all-features` on a host with the needed targets and dependencies.
All modes validate the exact workspace GPUI version pin.
"""

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "book/gpui-api-inventory.json"
OUT_MD = ROOT / "book/inventory.md"
MAC_SNAPSHOT = ROOT / "book/gpui-api-inventory-macos.json"
LINUX_SNAPSHOT = ROOT / "book/gpui-api-inventory-linux.json"
LINUX_TARGET = "aarch64-unknown-linux-gnu"
WASM_SNAPSHOT = ROOT / "book/gpui-api-inventory-wasm.json"
WASM_TARGET = "wasm32-unknown-unknown"
WINDOWS_SNAPSHOT = ROOT / "book/gpui-api-inventory-windows.json"
WINDOWS_TARGET = "x86_64-pc-windows-msvc"
ALL_FEATURES_SNAPSHOT = ROOT / "book/gpui-api-inventory-all-features.json"
LINUX_FEATURE_AUDIT_SNAPSHOT = ROOT / "book/gpui-api-inventory-linux-feature-audit.json"
WASM_TEST_SUPPORT_SNAPSHOT = ROOT / "book/gpui-api-inventory-wasm-test-support.json"
LINUX_ALL_FEATURES_SNAPSHOT = ROOT / "book/gpui-api-inventory-linux-all-features.json"
VERSION = "0.3.5"
PACKAGES = {"gpui-pre": "gpui", "gpui-pre-shared-string": "gpui_shared_string", "gpui-pre-refineable": "refineable"}
KINDS = {"module", "struct", "enum", "trait", "union", "type_alias", "macro", "proc_macro", "proc_attribute", "proc_derive"}
KIND_ALIASES = {"type_alias": "type", "proc_macro": "macro", "proc_attribute": "macro", "proc_derive": "macro"}
CHAPTERS = {
    "E2.4": "Getting started",
    "E2.5": "App, context, and entities",
    "E2.6": "Elements, styling, and text",
    "E2.7": "Input, actions, and key bindings",
    "E2.8": "Text input and IME",
    "E2.9": "Composition and advanced elements",
    "E2.10": "Async work and queues",
    "E2.11": "Windows, platforms, and accessibility",
    "E2.12": "Testing and profiling",
    "E2.15": "Types, preludes, and interop",
    "E2.16": "Internals",
}


def registry_package(name):
    candidates = list((Path.home() / ".cargo/registry/src").glob(f"*/{name}-{VERSION}"))
    if len(candidates) != 1:
        raise RuntimeError(f"expected one registry source for {name} {VERSION}; found {len(candidates)}")
    return candidates[0]


def validate_workspace_pin():
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())
    dependency = workspace["workspace"]["dependencies"]["gpui_pre"]
    if dependency.get("package") != "gpui-pre" or dependency.get("version") != f"={VERSION}":
        raise RuntimeError(f"workspace gpui_pre pin differs from gpui-pre ={VERSION}; update the generator and snapshots for this upgrade")


def run_rustdoc(package, target=None):
    command = ["cargo", "+nightly", "rustdoc", "-p", package, "--lib"]
    if target:
        command += ["--target", target]
    command += ["--", "-Z", "unstable-options", "--output-format", "json", "--document-hidden-items"]
    subprocess.run(command, cwd=ROOT, check=True)


def run_windows_rustdoc():
    manifest = registry_package("gpui-pre") / "Cargo.toml"
    subprocess.run(["cargo", "+nightly", "rustdoc", "--manifest-path", str(manifest), "--lib",
                    "--target", WINDOWS_TARGET, "--no-default-features",
                    "--features", "font-kit,test-support,wayland,x11", "--", "-Z", "unstable-options",
                    "--output-format", "json", "--document-hidden-items"], cwd=ROOT, check=True)


def run_wasm_rustdoc():
    # test-support pulls in wait-timeout 0.2.1, which has no wasm implementation.
    # This production profile still captures target_family=wasm public exports.
    manifest = registry_package("gpui-pre") / "Cargo.toml"
    subprocess.run(["cargo", "+nightly", "rustdoc", "--manifest-path", str(manifest), "--lib",
                    "--target", WASM_TARGET, "--no-default-features", "--", "-Z",
                    "unstable-options", "--output-format", "json", "--document-hidden-items"], cwd=ROOT, check=True)


def run_all_features_rustdoc():
    manifest = registry_package("gpui-pre") / "Cargo.toml"
    subprocess.run(["cargo", "+nightly", "rustdoc", "--manifest-path", str(manifest), "--lib",
                    "--all-features", "--", "-Z", "unstable-options", "--output-format", "json",
                    "--document-hidden-items"], cwd=ROOT, check=True)


def run_linux_feature_audit_rustdoc():
    # This profile enables every Linux-buildable optional feature except the
    # x11/screen-capture/test-support combination. That combination cannot be
    # cross-built on the macOS inventory host without Linux pkg-config sysroots;
    # test-support also pulls wait-timeout, which has no wasm implementation.
    manifest = registry_package("gpui-pre") / "Cargo.toml"
    subprocess.run(["cargo", "+nightly", "rustdoc", "--manifest-path", str(manifest), "--lib",
                    "--target", LINUX_TARGET, "--no-default-features", "--features",
                    "bench,inspector,leak-detection,profiler,stacker,wayland", "--", "-Z",
                    "unstable-options", "--output-format", "json", "--document-hidden-items"],
                   cwd=ROOT, check=True)


def isolated_profile_rustdoc(target, output_path):
    """Extract metadata in a disposable copy with narrowly-scoped dependency shims.

    GPUI itself and its manifest are copied verbatim. The shims only let rustdoc
    reach metadata generation where the host lacks a target sysroot or a
    test-only dependency has no implementation for the requested target.
    These profiles are not executable runtime builds.
    """
    temp = Path(tempfile.mkdtemp(prefix="gpui-inventory-isolated-"))
    try:
        gpui = registry_package("gpui-pre")
        copy_gpui = temp / "gpui-pre"
        shutil.copytree(gpui, copy_gpui, ignore=shutil.ignore_patterns("target"))
        patched = {}
        for name, version in (("wait-timeout", "0.2.1"), ("rusty-fork", "0.3.1"), ("x11", "2.21.0")):
            src = next((Path.home() / ".cargo/registry/src").glob(f"*/{name}-{version}"))
            dest = temp / f"{name}-{version}"
            shutil.copytree(src, dest)
            patched[name] = dest
        if target == WASM_TARGET:
            lib = patched["wait-timeout"] / "src/lib.rs"
            text = lib.read_text()
            anchor = "#[cfg(windows)]\n#[path = \"windows.rs\"]\nmod imp;"
            if anchor not in text:
                raise RuntimeError("wait-timeout 0.2.1 wasm shim anchor changed")
            shim = anchor + "\n#[cfg(target_family = \"wasm\")]\nmod wasm_compile_only {\n    use std::{io, process::{Child, ExitStatus}, time::Duration};\n    pub fn wait_timeout(_: &mut Child, _: Duration) -> io::Result<Option<ExitStatus>> { panic!(\"compile-only inventory shim\") }\n}\n#[cfg(target_family = \"wasm\")]\nuse wasm_compile_only as imp;"
            lib.write_text(text.replace(anchor, shim))
            child = patched["rusty-fork"] / "src/child_wrapper.rs"
            text = child.read_text()
            anchor = "        use std::os::unix::process::ExitStatusExt;"
            if anchor not in text:
                raise RuntimeError("rusty-fork 0.3.1 wasm shim anchor changed")
            text = text.replace(anchor, "        #[cfg(not(target_family = \"wasm\"))]\n        use std::os::unix::process::ExitStatusExt;\n        #[cfg(target_family = \"wasm\")]\n        trait ExitStatusExt { fn signal(&self) -> Option<i32>; }\n        #[cfg(target_family = \"wasm\")]\n        impl ExitStatusExt for std::process::ExitStatus { fn signal(&self) -> Option<i32> { None } }")
            child.write_text(text)
            features = ["--no-default-features", "--features", "test-support"]
        elif target == LINUX_TARGET:
            # x11's build.rs requires pkg-config/native headers even though
            # rustdoc metadata extraction does not link or execute that backend.
            (patched["x11"] / "build.rs").write_text('fn main() { println!("cargo:rustc-check-cfg=cfg(gles)"); }\n')
            # Current nightly denies this old dependency cast via warnings.
            unix = patched["wait-timeout"] / "src/unix.rs"
            text = unix.read_text()
            anchor = "sigchld_handler as usize"
            if anchor not in text:
                raise RuntimeError("wait-timeout 0.2.1 lint shim anchor changed")
            unix.write_text(text.replace(anchor, "sigchld_handler as *const () as usize"))
            features = ["--all-features"]
        else:
            raise RuntimeError(f"unsupported isolated profile target {target}")
        manifest = copy_gpui / "Cargo.toml"
        command = ["cargo", "+nightly", "rustdoc", "--manifest-path", str(manifest), "--lib", "--target", target, *features]
        for crate, path in patched.items():
            command += ["--config", f"patch.crates-io.{crate}.path=\"{path}\""]
        command += ["--", "-Z", "unstable-options", "--output-format", "json", "--document-hidden-items"]
        subprocess.run(command, cwd=ROOT, check=True)
        raw = copy_gpui / "target" / target / "doc/gpui.json"
        shutil.copy2(raw, output_path)
    finally:
        shutil.rmtree(temp, ignore_errors=True)


def load_docs(build):
    docs = {}
    for package, crate in PACKAGES.items():
        if build:
            run_rustdoc(package)
        path = ROOT / "target/doc" / f"{crate}.json"
        if not path.is_file():
            raise RuntimeError(f"missing {path}; run without --from-json to build rustdoc JSON")
        docs[crate] = json.loads(path.read_text())
    return docs


def source_location(item, package, revision, fallback=None):
    span = item.get("span") if item else None
    if span:
        filename = Path(span["filename"])
        root = registry_package(package)
        try:
            rel = filename.relative_to(root).as_posix() if filename.is_absolute() else filename.as_posix()
            line = span["begin"][0]
            upstream_crate = tomllib.loads((root / "Cargo.toml").read_text())["package"]["metadata"]["gpui-pre"]["zed-crate"]
            return rel, line, f"https://github.com/zed-industries/zed/blob/{revision}/crates/{upstream_crate}/{rel}#L{line}"
        except ValueError:
            pass
    return fallback or (None, None, None)


def generate(docs):
    manifest = tomllib.loads((registry_package("gpui-pre") / "Cargo.toml").read_text())
    revision = manifest["package"]["metadata"]["gpui-pre"]["zed-rev"]
    if manifest["package"]["version"] != VERSION:
        raise RuntimeError("registry version differs from inventory pin")
    primary = docs["gpui"]
    if primary["crate_version"] != VERSION:
        raise RuntimeError("rustdoc JSON differs from inventory pin")
    items = {}
    gaps = []

    def record(path, kind, obj, package, fallback=None, provenance=None, hidden=False):
        if kind not in KINDS:
            return
        kind = KIND_ALIASES.get(kind, kind)
        rel, line, url = source_location(obj, package, revision, fallback)
        if not url:
            gaps.append(f"No source span: {path}")
        entry = {"path": path, "kind": kind, "source_file": rel, "source_line": line,
                 "source_url": url, "origin_crate": package, "provenance": provenance or "definition",
                 "doc_hidden": hidden or any("doc(hidden)" in str(a) for a in (obj or {}).get("attrs", []))}
        key = (path, kind)
        # Prefer a definition's source span to a reexport's source span.
        if key not in items or (items[key]["source_url"] is None and url):
            items[key] = entry

    def visit(doc, item_id, public_path, package, seen, fallback=None, provenance=None, override=None, hidden=False):
        if item_id in seen:
            gaps.append(f"Reexport cycle at {public_path}")
            return
        index = doc["index"]
        obj = index.get(str(item_id))
        if obj is None:
            ext = doc["paths"].get(str(item_id))
            if ext:
                record(public_path + "::" + override if override else public_path, ext["kind"], None, package, fallback, "external reexport", hidden)
            else:
                gaps.append(f"Unresolved rustdoc ID {item_id}: {public_path}")
            return
        inner = obj["inner"]
        kind = next(iter(inner))
        hidden = hidden or any("doc(hidden)" in str(a) for a in obj.get("attrs", []))
        if kind == "use":
            use = inner["use"]
            alias = use["name"]
            use_fallback = source_location(obj, package, revision, fallback)
            target = use.get("id")
            if target is None:
                gaps.append(f"Unresolved use: {public_path} ({use['source']})")
                return
            if use["is_glob"]:
                target_obj = index.get(str(target))
                if target_obj and "module" in target_obj["inner"]:
                    for child_id in target_obj["inner"]["module"]["items"]:
                        child = index.get(str(child_id))
                        if child and child["visibility"] == "public":
                            visit(doc, child_id, public_path, package, seen | {item_id}, use_fallback, "glob reexport", hidden=hidden)
                elif use["source"] in ("gpui_shared_string", "refineable"):
                    foreign = docs[use["source"]]
                    for child_id in foreign["index"][str(foreign["root"])]["inner"]["module"]["items"]:
                        child = foreign["index"].get(str(child_id))
                        if child and child["visibility"] == "public":
                            visit(foreign, child_id, public_path, "gpui-pre-shared-string" if use["source"] == "gpui_shared_string" else "gpui-pre-refineable", seen | {item_id}, use_fallback, "glob reexport", hidden=hidden)
                else:
                    gaps.append(f"Unresolved glob: {public_path} ({use['source']})")
            elif alias != "_":
                visit(doc, target, public_path, package, seen | {item_id}, use_fallback, "named reexport", alias, hidden)
            return
        name = obj.get("name")
        if kind == "module":
            path = public_path + ("::" + (override or name) if name or override else "")
            if path != "gpui":
                record(path, kind, obj, package, fallback, provenance, hidden)
            for child_id in inner["module"]["items"]:
                child = index.get(str(child_id))
                if child and child["visibility"] == "public":
                    visit(doc, child_id, path, package, seen | {item_id}, fallback, hidden=hidden)
            return
        if name:
            record(public_path + "::" + (override or name), kind, obj, package, fallback, provenance, hidden)

    root = primary["index"][str(primary["root"])]
    for child_id in root["inner"]["module"]["items"]:
        child = primary["index"].get(str(child_id))
        if child and child["visibility"] == "public":
            visit(primary, child_id, "gpui", "gpui-pre", set())

    # Rustdoc's reexported foreign modules have no bodies in the primary JSON.
    # Their module names are inventoried, but their dependencies' own API is outside GPUI's maintained surface.
    result = sorted(items.values(), key=lambda x: (x["path"].casefold(), x["kind"]))
    counts = dict(sorted(Counter(x["kind"] for x in result).items()))
    root_lines = (registry_package("gpui-pre") / "src/gpui.rs").read_text().splitlines()
    conditional_exports = [
        ("pub mod queue;", "test, Linux, Windows, WebAssembly, test-support, or bench-support"),
        ("pub mod test;", "test or test-support"),
        ("pub use proptest;", "test or test-support"),
        ("pub use debug_overlay::*;", "profiler feature"),
        ("pub use queue::{", "Linux, Windows, or WebAssembly"),
        ("pub use test::*;", "test or test-support"),
        ("pub use pollster::block_on;", "non-WebAssembly"),
        ("pub mod _accessibility;", "cfg(doc)"),
        ("pub mod _ownership_and_data_flow;", "cfg(doc)"),
    ]
    cfg_audit = []
    for declaration, condition in conditional_exports:
        matches = [i + 1 for i, line in enumerate(root_lines) if line.strip().startswith(declaration)]
        if len(matches) != 1:
            raise RuntimeError(f"expected one conditional root declaration: {declaration}")
        cfg_audit.append({"declaration": declaration, "condition": condition,
                          "source_url": f"https://github.com/zed-industries/zed/blob/{revision}/crates/gpui/src/gpui.rs#L{matches[0]}"})
    # Candidate files for a later multi-target/feature comparison. This is a
    # file-level audit, not a claim that every declaration in them is exported.
    conditional_source_files = []
    for source in sorted((registry_package("gpui-pre") / "src").rglob("*.rs")):
        body = source.read_text()
        count = body.count("target_os =") + body.count("target_family =") + body.count("feature =")
        if count and "pub " in body:
            conditional_source_files.append({"source_file": source.relative_to(registry_package("gpui-pre")).as_posix(), "conditional_occurrences": count})
    return {"schema_version": 1, "package": "gpui-pre", "version": VERSION, "zed_revision": revision,
            "rustdoc_format_version": primary["format_version"], "target": primary["target"]["triple"],
            "rustdoc_command": "cargo +nightly rustdoc -p gpui-pre --lib -- -Z unstable-options --output-format json --document-hidden-items",
            "scope": "Workspace-resolved features on the generating target; rustdoc also enables cfg(doc). Foreign crate module reexports are listed as modules, without recursively inventorying dependency APIs.",
            "coverage_limitations": ["Other target triples (Linux, Windows, WebAssembly) were not extracted; target-specific public names in reexported modules may be absent.", "Only crate-root conditional declarations were identified precisely. The file-level conditional source audit lists candidates for a future multi-configuration comparison and does not establish reachability.", "Effective feature and cfg availability is not recorded per item; this workspace resolves test-support."],
            "conditional_root_export_audit": cfg_audit,
            "conditional_source_file_audit": conditional_source_files,
            "count": len(result), "counts_by_kind": counts, "extraction_gaps": sorted(set(gaps)), "items": result}


def merge_targets(mac, linux):
    if linux["target"] != LINUX_TARGET or linux["version"] != mac["version"] or linux["zed_revision"] != mac["zed_revision"]:
        raise RuntimeError("Linux snapshot target/version/revision does not match the macOS extraction")
    by_name = {}
    for data in (mac, linux):
        for original in data["items"]:
            key = (original["path"], original["kind"])
            if key not in by_name:
                by_name[key] = {**original, "available_targets": []}
            by_name[key]["available_targets"].append(data["target"])
            by_name[key]["doc_hidden"] |= original["doc_hidden"]
    merged = {**mac}
    merged["items"] = sorted(by_name.values(), key=lambda x: (x["path"].casefold(), x["kind"]))
    merged["count"] = len(merged["items"])
    merged["counts_by_kind"] = dict(sorted(Counter(x["kind"] for x in merged["items"]).items()))
    merged["target"] = f"{mac['target']}, {linux['target']}"
    merged["per_target_counts"] = {mac["target"]: mac["count"], linux["target"]: linux["count"]}
    merged["extraction_gaps"] = sorted(set(mac["extraction_gaps"] + linux["extraction_gaps"]))
    merged["scope"] = "Union of macOS and Linux rustdoc JSON with workspace-resolved features; rustdoc enables cfg(doc). Foreign crate module reexports are listed without recursively inventorying dependency APIs."
    merged["coverage_limitations"] = ["Windows and WebAssembly targets were not extracted; target-specific public names in reexported modules may be absent.", "Only crate-root conditional declarations were identified precisely. The file-level conditional source audit lists candidates for a future multi-configuration comparison and does not establish reachability.", "Effective feature and cfg availability is not recorded beyond target membership; this workspace resolves test-support."]
    return merged


def add_windows(base, windows):
    if windows["target"] != WINDOWS_TARGET or windows["version"] != base["version"] or windows["zed_revision"] != base["zed_revision"]:
        raise RuntimeError("Windows snapshot target/version/revision does not match the inventory")
    by_name = {(x["path"], x["kind"]): x for x in base["items"]}
    for original in windows["items"]:
        key = (original["path"], original["kind"])
        if key not in by_name:
            by_name[key] = {**original, "available_targets": []}
        by_name[key]["available_targets"].append(WINDOWS_TARGET)
        by_name[key]["doc_hidden"] |= original["doc_hidden"]
    result = {**base}
    result["items"] = sorted(by_name.values(), key=lambda x: (x["path"].casefold(), x["kind"]))
    result["count"] = len(result["items"])
    result["counts_by_kind"] = dict(sorted(Counter(x["kind"] for x in result["items"]).items()))
    result["target"] = base["target"] + ", " + WINDOWS_TARGET
    result["per_target_counts"] = {**base["per_target_counts"], WINDOWS_TARGET: windows["count"]}
    result["extraction_gaps"] = sorted(set(base["extraction_gaps"] + windows["extraction_gaps"]))
    result["scope"] = "Union of macOS, Linux, and Windows rustdoc JSON. Windows has windows-manifest disabled; other targets use workspace-resolved features. Rustdoc enables cfg(doc). Foreign crate modules are listed at the reexport boundary."
    result["feature_profiles"] = {"macOS/Linux": "workspace-resolved features including test-support", "Windows": "font-kit,test-support,wayland,x11; default windows-manifest disabled because llvm-rc is unavailable on this host"}
    result["coverage_limitations"] = ["WebAssembly rustdoc failed in wait-timeout 0.2.1 (E0433: unresolved imp::wait_timeout at src/lib.rs:68); wasm-specific public names may be absent.", "Windows extraction disabled windows-manifest after the default build failed in gpui-pre build.rs with NotAttempted(\"llvm-rc\"). This feature embeds a manifest and is not expected to change Rust API, but the variant differs from default.", "Only crate-root conditional declarations were identified precisely. The file-level conditional source audit lists candidates for multi-configuration comparison and does not establish reachability.", "Effective feature and cfg availability is recorded only as observed target membership."]
    result["attempted_target_blockers"] = {"wasm32-unknown-unknown": "wait-timeout 0.2.1 E0433: unresolved imp::wait_timeout at src/lib.rs:68", "x86_64-pc-windows-msvc default features": "gpui-pre build.rs: embed_resource NotAttempted(\"llvm-rc\"); documented no-default-features variant succeeded"}
    return result


def add_all_features(base, all_features):
    if all_features["target"] != "aarch64-apple-darwin" or all_features["version"] != base["version"] or all_features["zed_revision"] != base["zed_revision"]:
        raise RuntimeError("all-features snapshot target/version/revision does not match the inventory")
    by_name = {(x["path"], x["kind"]): x for x in base["items"]}
    profiles = {"aarch64-apple-darwin": "macOS workspace", LINUX_TARGET: "Linux workspace", WINDOWS_TARGET: "Windows variant", WASM_TARGET: "WebAssembly no default features"}
    for item in by_name.values():
        item["observed_profiles"] = [profiles[target] for target in item["available_targets"]]
    for original in all_features["items"]:
        key = (original["path"], original["kind"])
        if key not in by_name:
            by_name[key] = {**original, "available_targets": ["aarch64-apple-darwin"], "observed_profiles": []}
        by_name[key]["observed_profiles"].append("macOS all features")
        by_name[key]["doc_hidden"] |= original["doc_hidden"]
    result = {**base}
    result["items"] = sorted(by_name.values(), key=lambda x: (x["path"].casefold(), x["kind"]))
    result["count"] = len(result["items"])
    result["counts_by_kind"] = dict(sorted(Counter(x["kind"] for x in result["items"]).items()))
    result["per_profile_counts"] = {**base.get("per_profile_counts", base["per_target_counts"]), "macOS all features": all_features["count"]}
    result["extraction_gaps"] = sorted(set(base["extraction_gaps"] + all_features["extraction_gaps"]))
    result["scope"] += " macOS all-features extraction is also included."
    result["feature_profiles"] = {**base["feature_profiles"], "macOS all features": "all gpui-pre features enabled using the published package manifest"}
    result["coverage_limitations"] = [x for x in base["coverage_limitations"] if not x.startswith("Effective feature and cfg availability")]
    result["coverage_limitations"].append("Feature availability is recorded as observed profiles, not as an exhaustive Boolean feature formula for each name.")
    return result


def add_wasm(base, wasm):
    if wasm["target"] != WASM_TARGET or wasm["version"] != base["version"] or wasm["zed_revision"] != base["zed_revision"]:
        raise RuntimeError("WebAssembly snapshot target/version/revision does not match the inventory")
    by_name = {(x["path"], x["kind"]): x for x in base["items"]}
    for original in wasm["items"]:
        key = (original["path"], original["kind"])
        if key not in by_name:
            by_name[key] = {**original, "available_targets": []}
        by_name[key]["available_targets"].append(WASM_TARGET)
        by_name[key]["doc_hidden"] |= original["doc_hidden"]
    result = {**base}
    result["items"] = sorted(by_name.values(), key=lambda x: (x["path"].casefold(), x["kind"]))
    result["count"] = len(result["items"])
    result["counts_by_kind"] = dict(sorted(Counter(x["kind"] for x in result["items"]).items()))
    result["target"] = base["target"] + ", " + WASM_TARGET
    result["per_target_counts"] = {**base["per_target_counts"], WASM_TARGET: wasm["count"]}
    result["extraction_gaps"] = sorted(set(base["extraction_gaps"] + wasm["extraction_gaps"]))
    result["scope"] += " WebAssembly target rustdoc JSON with no default features is also included; test-support is omitted because its wait-timeout dependency does not compile for wasm."
    result["coverage_limitations"] = [x for x in result["coverage_limitations"] if not x.startswith("WebAssembly rustdoc failed") and not x.startswith("Other target triples")]
    result["coverage_limitations"].append("WebAssembly was extracted with no default features; test-support and other profile-only declarations on that target are not observed.")
    result["feature_profiles"] = {**result.get("feature_profiles", {}), WASM_TARGET: "no default features; test-support omitted because wait-timeout 0.2.1 does not compile for wasm"}
    return result


def add_source_audit(base):
    package_root = registry_package("gpui-pre")
    observed_names = {x["path"].split("::")[-1] for x in base["items"]}
    declaration = re.compile(r"^\s*pub\s+(?:struct|enum|trait|type|mod|union|macro)\s+([A-Za-z_][A-Za-z_0-9]*)")
    candidate_dispositions = {
        ("scap_screen_capture", "src/platform.rs"): "root-reachable; observed in Linux all-features rustdoc metadata",
        ("TaffyLayoutEngine", "src/taffy.rs"): "not root-reachable; taffy is a private module and gpui.rs imports this type privately",
    }
    candidates = []
    for file in sorted((package_root / "src").rglob("*.rs")):
        relative = file.relative_to(package_root).as_posix()
        lines = file.read_text().splitlines()
        for index, line in enumerate(lines):
            match = declaration.match(line)
            if not match or match.group(1) in observed_names:
                continue
            nearby = lines[max(0, index - 12):index]
            if not any("cfg(" in previous or "cfg_attr(" in previous or "target_" in previous or "feature =" in previous for previous in nearby):
                continue
            key = (match.group(1), relative)
            if key not in candidate_dispositions:
                raise RuntimeError(f"new cfg-adjacent public declaration needs reachability review: {key} at line {index + 1}")
            candidates.append({"name": match.group(1), "source_file": relative, "source_line": index + 1,
                               "disposition": candidate_dispositions[key]})
    expected_candidates = {key for key in candidate_dispositions if key[0] not in observed_names}
    if {(x["name"], x["source_file"]) for x in candidates} != expected_candidates:
        raise RuntimeError("cfg-adjacent public declaration audit changed; review reachability")
    source = registry_package("gpui-pre") / "src/platform.rs"
    lines = source.read_text().splitlines()
    matches = [i + 1 for i, line in enumerate(lines) if line.strip() == "pub mod scap_screen_capture;"]
    if len(matches) != 1 or "pub use platform::*;" not in (registry_package("gpui-pre") / "src/gpui.rs").read_text():
        raise RuntimeError("screen capture source audit no longer matches the pinned crate")
    if any(x["path"] == "gpui::scap_screen_capture" and x["kind"] == "module" for x in base["items"]):
        result = {**base, "cfg_public_declaration_candidates": candidates}
        result["source_audited_unverified"] = []
        result["coverage_limitations"] = [x for x in base["coverage_limitations"] if "gpui::scap_screen_capture" not in x]
        return result
    revision = base["zed_revision"]
    item = {"path": "gpui::scap_screen_capture", "kind": "module", "source_file": "src/platform.rs",
            "source_line": matches[0], "source_url": f"https://github.com/zed-industries/zed/blob/{revision}/crates/gpui/src/platform.rs#L{matches[0]}",
            "origin_crate": "gpui-pre", "provenance": "source audit (rustdoc build blocked)", "doc_hidden": False,
            "available_targets": [], "observed_profiles": [],
            "availability_condition": "screen-capture feature on Windows, Linux, or FreeBSD"}
    result = {**base}
    result["items"] = sorted([*base["items"], item], key=lambda x: (x["path"].casefold(), x["kind"]))
    result["count"] = len(result["items"])
    result["counts_by_kind"] = dict(sorted(Counter(x["kind"] for x in result["items"]).items()))
    result["source_audited_unverified"] = [item["path"]]
    result["cfg_public_declaration_candidates"] = candidates
    result["coverage_limitations"] = [*base["coverage_limitations"], "gpui::scap_screen_capture is included from a verified public source declaration but is not rustdoc-observed."]
    return result


def add_linux_feature_audit(base, audit):
    if audit["target"] != LINUX_TARGET or audit["version"] != base["version"] or audit["zed_revision"] != base["zed_revision"]:
        raise RuntimeError("Linux feature-audit snapshot target/version/revision does not match the inventory")
    profile = "Linux broad optional-feature audit"
    by_name = {(item["path"], item["kind"]): item for item in base["items"]}
    for original in audit["items"]:
        key = (original["path"], original["kind"])
        if key not in by_name:
            by_name[key] = {**original, "available_targets": [LINUX_TARGET], "observed_profiles": [profile]}
        else:
            item = by_name[key]
            item.setdefault("available_targets", [])
            if LINUX_TARGET not in item["available_targets"]:
                item["available_targets"].append(LINUX_TARGET)
            item.setdefault("observed_profiles", []).append(profile)
            item["doc_hidden"] |= original["doc_hidden"]
    result = {**base}
    result["items"] = sorted(by_name.values(), key=lambda x: (x["path"].casefold(), x["kind"]))
    result["count"] = len(result["items"])
    result["counts_by_kind"] = dict(sorted(Counter(x["kind"] for x in result["items"]).items()))
    result["per_profile_counts"] = {**result.get("per_profile_counts", {}), profile: audit["count"]}
    result["linux_feature_profile_audit"] = {
        "target": audit["target"],
        "profile": audit["feature_profile"],
        "count": audit["count"],
        "new_names_added": sum((item["path"], item["kind"]) not in {(x["path"], x["kind"]) for x in base["items"]} for item in audit["items"]),
        "excludes": ["x11", "screen-capture", "test-support", "windows-manifest"],
        "snapshot": LINUX_FEATURE_AUDIT_SNAPSHOT.name,
    }
    result["scope"] += " A Linux broad optional-feature rustdoc profile is also compared and merged."
    result["coverage_limitations"] = [
        *result["coverage_limitations"],
        "The additional Linux feature audit omits x11, screen-capture, test-support, and windows-manifest; it is not a substitute for Linux all-features rustdoc.",
    ]
    return result


def add_linux_all_features(base, audit):
    if audit["target"] != LINUX_TARGET or audit["version"] != base["version"] or audit["zed_revision"] != base["zed_revision"]:
        raise RuntimeError("isolated Linux all-features snapshot does not match the inventory")
    profile = "Linux all features (isolated rustdoc)"
    by_name = {(x["path"], x["kind"]): x for x in base["items"]}
    for original in audit["items"]:
        key = (original["path"], original["kind"])
        item = by_name.get(key)
        if item is None:
            item = {**original, "available_targets": [LINUX_TARGET], "observed_profiles": [profile]}
            by_name[key] = item
        else:
            item.setdefault("available_targets", [])
            if LINUX_TARGET not in item["available_targets"]:
                item["available_targets"].append(LINUX_TARGET)
            item.setdefault("observed_profiles", []).append(profile)
            item["doc_hidden"] |= original["doc_hidden"]
        if key == ("gpui::scap_screen_capture", "module"):
            item["availability_condition"] = "screen-capture feature on Windows, Linux, or FreeBSD"
    result = {**base}
    result["items"] = sorted(by_name.values(), key=lambda x: (x["path"].casefold(), x["kind"]))
    result["count"] = len(result["items"])
    result["counts_by_kind"] = dict(sorted(Counter(x["kind"] for x in result["items"]).items()))
    result["per_profile_counts"] = {**result.get("per_profile_counts", {}), profile: audit["count"]}
    result["linux_all_features_profile"] = {"snapshot": LINUX_ALL_FEATURES_SNAPSHOT.name, "count": audit["count"], "new_names_added": sum((x["path"], x["kind"]) not in {(y["path"], y["kind"]) for y in base["items"]} for x in audit["items"]), "dependency_shims": audit["dependency_shims"]}
    result["feature_profiles"] = {**result.get("feature_profiles", {}), profile: audit["feature_profile"]}
    result["coverage_limitations"] = [x for x in result["coverage_limitations"] if "Linux all-features extraction failed" not in x]
    result["scope"] += " Linux all-features rustdoc metadata is included from an isolated copy using compile-only dependency shims; this is not a runnable native Linux build."
    return result


def add_wasm_test_support(base, audit):
    if audit["target"] != WASM_TARGET or audit["version"] != base["version"] or audit["zed_revision"] != base["zed_revision"]:
        raise RuntimeError("isolated wasm test-support snapshot does not match the inventory")
    profile = "WebAssembly test-support (isolated rustdoc)"
    by_name = {(x["path"], x["kind"]): x for x in base["items"]}
    for original in audit["items"]:
        key = (original["path"], original["kind"])
        item = by_name.get(key)
        if item is None:
            item = {**original, "available_targets": [WASM_TARGET], "observed_profiles": [profile]}
            by_name[key] = item
        else:
            item.setdefault("available_targets", [])
            if WASM_TARGET not in item["available_targets"]:
                item["available_targets"].append(WASM_TARGET)
            item.setdefault("observed_profiles", []).append(profile)
            item["doc_hidden"] |= original["doc_hidden"]
    result = {**base}
    result["items"] = sorted(by_name.values(), key=lambda x: (x["path"].casefold(), x["kind"]))
    result["count"] = len(result["items"])
    result["counts_by_kind"] = dict(sorted(Counter(x["kind"] for x in result["items"]).items()))
    result.setdefault("per_profile_counts", {})[profile] = audit["count"]
    result["feature_profiles"] = {**result.get("feature_profiles", {}), profile: audit["feature_profile"]}
    result["wasm_test_support_profile"] = {"snapshot": WASM_TEST_SUPPORT_SNAPSHOT.name, "count": audit["count"], "new_names_added": sum((x["path"], x["kind"]) not in {(y["path"], y["kind"]) for y in base["items"]} for x in audit["items"]), "dependency_shims": audit["dependency_shims"]}
    result["coverage_limitations"] = [x for x in result["coverage_limitations"] if "test-support and other profile-only declarations" not in x]
    result["coverage_limitations"].append("WebAssembly test-support names were extracted with compile-only wait-timeout and rusty-fork shims; this does not establish runnable wasm test support.")
    return result


def destination(item):
    path, source = item["path"], item["source_file"] or ""
    if path.startswith("gpui::private") or item["doc_hidden"]:
        return "Excluded: hidden implementation support"
    if path in ("gpui::_accessibility", "gpui::_ownership_and_data_flow"):
        return "Excluded: documentation-only conceptual module"
    if path in ("gpui::accesskit", "gpui::http_client", "gpui::proptest"):
        return "Excluded: foreign crate module reexport"
    if path == "gpui::ctor":
        return "Excluded: foreign macro reexport"
    exact_overrides = {
        ("gpui::Application", "struct"): "E2.4",
        ("gpui::ApplicationHandle", "struct"): "E2.4",
        ("gpui::App", "struct"): "E2.4",
        ("gpui::Window", "struct"): "E2.4",
        ("gpui::WindowOptions", "struct"): "E2.4",
        ("gpui::AnyDrag", "struct"): "E2.7",
        ("gpui::ExternalDragPayloadSource", "type"): "E2.7",
        ("gpui::KeystrokeEvent", "struct"): "E2.7",
        ("gpui::QuitMode", "enum"): "E2.11",
        ("gpui::SystemWindowTabController", "struct"): "E2.11",
        ("gpui::Element", "trait"): "E2.9",
        ("gpui::Drawable", "struct"): "E2.9",
        ("gpui::profiler", "module"): "E2.11",
        ("gpui::BenchAppContext", "struct"): "E2.12",
        ("gpui::BenchWindowContext", "struct"): "E2.12",
        ("gpui::BenchReport", "struct"): "E2.12",
        ("gpui::inspector_reflection", "module"): "E2.16",
        ("gpui::styled_reflection", "module"): "E2.16",
    }
    for name in ("ElementInputHandler", "EntityInputHandler", "InputHandler", "PlatformInputHandler", "TextInputAction", "TextInputConfiguration", "TextInputStateChange", "UTF16Selection", "PendingInputStatus", "PendingInputTimeoutStatus"):
        exact_overrides[(f"gpui::{name}", item["kind"])] = "E2.8"
    if (path, item["kind"]) in exact_overrides:
        return exact_overrides[(path, item["kind"])]
    root_name = path.removeprefix("gpui::").split("::")[0]
    leaf = path.split("::")[-1]
    name_overrides = {
        "E2.5": {"Render", "RenderOnce", "Observation"},
        "E2.6": {"Image", "ImageFormat", "ImageFormatIter"},
        "E2.7": {"register_action", "Focusable", "FocusHandle", "FocusId", "FocusOutEvent", "WeakFocusHandle", "AsKeystroke", "Keystroke", "InvalidKeystrokeError", "KeybindingKeystroke", "Modifiers", "DispatchEventResult", "DispatchPhase", "ElementId", "OsAction", "InteractiveElement", "StatefulInteractiveElement", "InteractiveElementState", "Interactivity", "HoverListenerMode", "DragMoveEvent", "ElementClickedState", "ElementHoverState"},
        "E2.9": {"PaintQuad", "ContentMask", "TileId"},
        "E2.10": {"FutureExt", "Timeout"},
        "E2.11": {"GpuSpecs"},
        "E2.12": {"bench", "property_test"},
        "E2.16": {"ArenaClearNeeded", "AtlasKey", "AtlasTextureId", "AtlasTextureKind", "AtlasTile", "PlatformAtlas", "Tiling", "ThreadedDispatcher", "NoopTextSystem"},
    }
    if leaf.startswith(("Clipboard", "Menu", "OwnedMenu", "Hitbox")) or leaf in {"OsMenu"}:
        return "E2.7"
    for chapter, names in name_overrides.items():
        if leaf in names:
            return chapter
    root_overrides = {
        "AppContext": "E2.5", "VisualContext": "E2.5", "BorrowAppContext": "E2.5",
        "Reservation": "E2.5", "EventEmitter": "E2.5", "Render": "E2.5",
        "IntoElement": "E2.6", "GpuSpecs": "E2.11",
        "bench_group": "E2.12", "bench_main": "E2.12", "test": "E2.12",
        "AccessibleAction": "E2.11", "Role": "E2.11", "Toggled": "E2.11", "Orientation": "E2.11",
        "queue": "E2.10", "profiler": "E2.12", "colors": "E2.6",
        "prelude": "E2.15", "Result": "E2.15", "SharedString": "E2.15",
    }
    if source == "src/gpui.rs" and root_name in root_overrides:
        return root_overrides[root_name]
    if source.startswith(("src/_accessibility", "src/_ownership_and_data_flow")):
        return "Excluded: documentation-only conceptual module"
    if source.startswith(("src/app/async_context", "src/executor", "src/queue")):
        return "E2.10"
    if source.startswith("src/profiler"):
        return "E2.11"
    if source.startswith(("src/app/test", "src/app/headless", "src/app/visual_test", "src/test", "src/platform/test", "src/platform/visual_test")):
        return "E2.12"
    if source.startswith(("src/action", "src/key_dispatch", "src/keymap", "src/interactive", "src/input", "src/gestures")):
        return "E2.7"
    if source.startswith(("src/app", "src/view", "src/global", "src/subscription")):
        return "E2.5"
    if source.startswith(("src/elements/anchored", "src/elements/animation", "src/elements/canvas", "src/elements/deferred", "src/elements/list", "src/elements/surface", "src/elements/uniform_list", "src/path_builder", "src/scene", "src/spring", "src/svg_renderer")):
        return "E2.9"
    if source.startswith(("src/element", "src/elements", "src/asset_cache", "src/assets", "src/color", "src/geometry", "src/style", "src/styled", "src/taffy", "src/text_system")):
        return "E2.6"
    if source.startswith(("src/platform", "src/window", "src/inspector", "src/debug_overlay")):
        return "E2.11"
    if source.startswith(("src/arena", "src/bounds_tree")):
        return "E2.16"
    if source.startswith(("src/prelude", "src/util", "src/shared_uri", "src/gpui", "src/refineable")) or source == "gpui_shared_string.rs":
        return "E2.15"
    raise RuntimeError(f"unclassified API: {path} ({source})")


def markdown(data):
    def observed(item):
        if not item.get("available_targets"):
            return "source audit only"
        names = {"aarch64-apple-darwin": "macOS", "aarch64-unknown-linux-gnu": "Linux", "x86_64-pc-windows-msvc": "Windows", "wasm32-unknown-unknown": "WebAssembly"}
        result = []
        for target in item.get("available_targets", [data["target"]]):
            label = names.get(target, target)
            if target == "aarch64-apple-darwin" and "macOS workspace" not in item.get("observed_profiles", ["macOS workspace"]):
                label += " (all features)"
            result.append(label)
        return ", ".join(result)

    groups = {chapter: [] for chapter in CHAPTERS}
    excluded = {}
    for item in data["items"]:
        dest = destination(item)
        (excluded.setdefault(dest, []) if dest.startswith("Excluded:") else groups[dest]).append(item)
    lines = ["# GPUI public API inventory", "",
             f"Pinned package: `gpui-pre` {data['version']} at Zed revision [`{data['zed_revision']}`](https://github.com/zed-industries/zed/tree/{data['zed_revision']}).",
             "", f"This snapshot records **{data['count']}** public type, trait, macro, and module names from rustdoc JSON extracts for `{data['target']}`. It includes a Linux all-features extract (632 names) and a WebAssembly test-support extract (566 names) produced in disposable copies with compile-only dependency shims. Those shims permit rustdoc metadata generation only; they do not prove that either runtime profile executes. Rustdoc also enables `cfg(doc)`. Chapter destinations are planned coverage, not a claim that chapters already exist.",
             "", "Observed target labels: **macOS** = `aarch64-apple-darwin`; **Linux** = `aarch64-unknown-linux-gnu`; **Windows** = `x86_64-pc-windows-msvc`; **WebAssembly** = `wasm32-unknown-unknown`. **macOS (all features)** means the name appeared only with every `gpui-pre` feature enabled. An absent label means that name was not observed in that extract; it does not establish unavailability on the platform.",
             "", "The inventory excludes functions, constants, associated items, fields, and variants. Foreign crate modules are listed at the reexport boundary; their dependencies' own APIs are outside this inventory. The Windows extract uses `font-kit,test-support,wayland,x11` with default `windows-manifest` disabled because the `llvm-rc` tool is unavailable. Linux all-features uses a temporary x11 build-script pkg-config bypass and an equivalent wait-timeout cast edit for current-nightly lint compatibility; rustdoc does not link or run the backend. WebAssembly test-support uses a temporary panic stub for the missing `wait-timeout` implementation and a wasm `ExitStatus` shim in `rusty-fork`; neither shim is executed. The Linux broad optional-feature audit remains a separate profile and omits `x11`, `screen-capture`, `test-support`, and `windows-manifest`.",
             "", "This is a cross-profile name union, not proof of exhaustive API coverage. FreeBSD and other target triples are not extracted. Windows `windows-manifest` is unobserved, and source cfg branches that are enabled in all-features profiles or share a name with an observed declaration can still expose distinct paths. The file-level cfg scan is only a review aid; it cannot establish all target/feature reachability. The first E2.1 coverage checklist item therefore remains open pending a complete source-reachability audit or additional target/profile evidence.",
             "", "Refresh the macOS extract and union with `python3 scripts/generate_gpui_inventory.py --regenerate`; refresh standard target extracts with `--update-linux`, `--update-windows`, `--update-wasm`, `--update-all-features`, and `--update-linux-feature-audit`. Reproduce the two isolated metadata profiles with `--update-isolated-audits`. Generate this page with `python3 scripts/generate_gpui_inventory.py --markdown > book/inventory.md`; verify snapshots, union JSON, and page with `python3 scripts/generate_gpui_inventory.py --check`. The check also fails when the workspace GPUI pin changes.", ""]
    for chapter, title in CHAPTERS.items():
        rows = groups[chapter]
        lines.extend([f"## {chapter}: {title} ({len(rows)})", "", "| API | Kind | Observed target | Source |", "| --- | --- | --- | --- |"])
        for x in rows:
            lines.append(f"| `{x['path']}` | {x['kind']} | {observed(x)} | [source]({x['source_url']}) |")
        lines.append("")
    for reason, rows in sorted(excluded.items()):
        lines.extend([f"## {reason} ({len(rows)})", "", "| API | Kind | Observed target | Source |", "| --- | --- | --- | --- |"])
        for x in rows:
            lines.append(f"| `{x['path']}` | {x['kind']} | {observed(x)} | [source]({x['source_url']}) |")
        lines.append("")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if generated inventory differs from checked-in data")
    parser.add_argument("--from-json", action="store_true", help="consume existing target/doc JSON without running rustdoc")
    parser.add_argument("--markdown", action="store_true", help="print classified Markdown instead of writing the JSON file")
    parser.add_argument("--update-linux", action="store_true", help="regenerate the checked-in Linux extraction snapshot")
    parser.add_argument("--update-windows", action="store_true", help="regenerate the checked-in Windows extraction snapshot")
    parser.add_argument("--update-wasm", action="store_true", help="regenerate the checked-in WebAssembly extraction snapshot")
    parser.add_argument("--update-all-features", action="store_true", help="regenerate the checked-in macOS all-features snapshot")
    parser.add_argument("--update-linux-feature-audit", action="store_true", help="regenerate Linux optional-feature audit snapshot (excludes x11, screen-capture, test-support, windows-manifest)")
    parser.add_argument("--update-isolated-audits", action="store_true", help="extract wasm test-support and Linux all-features metadata in disposable copies with documented compile-only dependency shims")
    parser.add_argument("--regenerate", action="store_true", help="rebuild macOS rustdoc and regenerate the inventory from checked-in target snapshots")
    args = parser.parse_args()
    validate_workspace_pin()
    refresh_mac = args.regenerate or not (args.check or args.markdown or args.update_linux or args.update_windows or args.update_wasm or args.update_all_features or args.update_linux_feature_audit or args.update_isolated_audits)
    if refresh_mac:
        mac = generate(load_docs(not args.from_json))
        MAC_SNAPSHOT.write_text(json.dumps(mac, indent=2, ensure_ascii=False) + "\n")
    else:
        if not MAC_SNAPSHOT.exists():
            raise RuntimeError(f"missing {MAC_SNAPSHOT}; run --regenerate first")
        mac = json.loads(MAC_SNAPSHOT.read_text())
        if mac["version"] != VERSION:
            raise RuntimeError("macOS snapshot differs from workspace pin")
    data = mac
    if args.update_linux:
        if not args.from_json:
            run_rustdoc("gpui-pre", LINUX_TARGET)
        linux_path = ROOT / "target" / LINUX_TARGET / "doc/gpui.json"
        linux_docs = load_docs(False)
        linux_docs["gpui"] = json.loads(linux_path.read_text())
        linux = generate(linux_docs)
        linux["rustdoc_command"] = f"cargo +nightly rustdoc -p gpui-pre --lib --target {LINUX_TARGET} -- -Z unstable-options --output-format json --document-hidden-items"
        LINUX_SNAPSHOT.write_text(json.dumps(linux, indent=2, ensure_ascii=False) + "\n")
    elif LINUX_SNAPSHOT.exists():
        linux = json.loads(LINUX_SNAPSHOT.read_text())
    else:
        linux = None
    if linux is not None:
        data = merge_targets(data, linux)
    if args.update_windows:
        if not args.from_json:
            run_windows_rustdoc()
        windows_path = registry_package("gpui-pre") / "target" / WINDOWS_TARGET / "doc/gpui.json"
        windows_docs = load_docs(False)
        windows_docs["gpui"] = json.loads(windows_path.read_text())
        windows = generate(windows_docs)
        windows["feature_profile"] = "font-kit,test-support,wayland,x11 (no default features; windows-manifest disabled)"
        windows["rustdoc_command"] = f"cargo +nightly rustdoc --manifest-path <registry gpui-pre-0.3.5/Cargo.toml> --lib --target {WINDOWS_TARGET} --no-default-features --features font-kit,test-support,wayland,x11 -- -Z unstable-options --output-format json --document-hidden-items"
        windows["default_build_blocker"] = "gpui-pre build.rs: embed_resource returned NotAttempted(\"llvm-rc\")"
        WINDOWS_SNAPSHOT.write_text(json.dumps(windows, indent=2, ensure_ascii=False) + "\n")
    elif WINDOWS_SNAPSHOT.exists():
        windows = json.loads(WINDOWS_SNAPSHOT.read_text())
    else:
        windows = None
    if windows is not None:
        data = add_windows(data, windows)
    if args.update_wasm:
        if not args.from_json:
            run_wasm_rustdoc()
        wasm_path = registry_package("gpui-pre") / "target" / WASM_TARGET / "doc/gpui.json"
        wasm_docs = load_docs(False)
        wasm_docs["gpui"] = json.loads(wasm_path.read_text())
        wasm = generate(wasm_docs)
        wasm["feature_profile"] = "no default features (test-support omitted because wait-timeout 0.2.1 does not compile on wasm)"
        wasm["rustdoc_command"] = f"cargo +nightly rustdoc --manifest-path <registry gpui-pre-0.3.5/Cargo.toml> --lib --target {WASM_TARGET} --no-default-features -- -Z unstable-options --output-format json --document-hidden-items"
        wasm["attempted_target_blocker"] = "test-support profile: wait-timeout 0.2.1 has no wasm implementation (E0433 at src/lib.rs:68)"
        WASM_SNAPSHOT.write_text(json.dumps(wasm, indent=2, ensure_ascii=False) + "\n")
    elif WASM_SNAPSHOT.exists():
        wasm = json.loads(WASM_SNAPSHOT.read_text())
    else:
        wasm = None
    if wasm is not None:
        data = add_wasm(data, wasm)
    if args.update_isolated_audits:
        for target, snapshot, key in ((WASM_TARGET, WASM_TEST_SUPPORT_SNAPSHOT, "wasm-test-support"),
                                      (LINUX_TARGET, LINUX_ALL_FEATURES_SNAPSHOT, "linux-all-features")):
            with tempfile.NamedTemporaryFile(prefix="gpui-inventory-", suffix=".json") as raw_file:
                isolated_profile_rustdoc(target, Path(raw_file.name))
                docs = {crate: json.loads((ROOT / "target/doc" / f"{crate}.json").read_text()) for crate in PACKAGES.values()}
                docs["gpui"] = json.loads(Path(raw_file.name).read_text())
                profile = generate(docs)
            if key == "wasm-test-support":
                profile["feature_profile"] = "no default features; test-support enabled; temporary compile-only wait-timeout and rusty-fork shims"
                profile["rustdoc_command"] = f"cargo +nightly rustdoc --manifest-path <isolated gpui-pre {VERSION}/Cargo.toml> --lib --target {WASM_TARGET} --no-default-features --features test-support -- -Z unstable-options --output-format json --document-hidden-items"
                profile["dependency_shims"] = ["wait-timeout 0.2.1: add wasm-only panic stub for imp::wait_timeout so metadata can be emitted; never executed", "rusty-fork 0.3.1: provide wasm-only ExitStatusExt::signal stub returning None so metadata can be emitted; never executed"]
                WASM_TEST_SUPPORT_SNAPSHOT.write_text(json.dumps(profile, indent=2, ensure_ascii=False) + "\n")
            else:
                profile["feature_profile"] = "all gpui-pre features enabled; temporary x11 build-script pkg-config bypass and wait-timeout current-nightly lint compatibility edit"
                profile["rustdoc_command"] = f"cargo +nightly rustdoc --manifest-path <isolated gpui-pre {VERSION}/Cargo.toml> --lib --target {LINUX_TARGET} --all-features -- -Z unstable-options --output-format json --document-hidden-items"
                profile["dependency_shims"] = ["x11 2.21.0: temporary build.rs bypasses pkg-config/native header probe; rustdoc metadata only, no linking", "wait-timeout 0.2.1: temporary equivalent function-pointer cast avoids current-nightly deny(warnings); no behavior change"]
                LINUX_ALL_FEATURES_SNAPSHOT.write_text(json.dumps(profile, indent=2, ensure_ascii=False) + "\n")
    if args.update_all_features:
        if not args.from_json:
            run_all_features_rustdoc()
        all_path = registry_package("gpui-pre") / "target/doc/gpui.json"
        all_docs = load_docs(False)
        all_docs["gpui"] = json.loads(all_path.read_text())
        all_features = generate(all_docs)
        all_features["feature_profile"] = "all gpui-pre features enabled using the published package manifest"
        all_features["rustdoc_command"] = "cargo +nightly rustdoc --manifest-path <registry gpui-pre-0.3.5/Cargo.toml> --lib --all-features -- -Z unstable-options --output-format json --document-hidden-items"
        ALL_FEATURES_SNAPSHOT.write_text(json.dumps(all_features, indent=2, ensure_ascii=False) + "\n")
    elif ALL_FEATURES_SNAPSHOT.exists():
        all_features = json.loads(ALL_FEATURES_SNAPSHOT.read_text())
    else:
        all_features = None
    if all_features is not None:
        data = add_all_features(data, all_features)
    if WASM_TEST_SUPPORT_SNAPSHOT.exists():
        data = add_wasm_test_support(data, json.loads(WASM_TEST_SUPPORT_SNAPSHOT.read_text()))
    if LINUX_ALL_FEATURES_SNAPSHOT.exists():
        data = add_linux_all_features(data, json.loads(LINUX_ALL_FEATURES_SNAPSHOT.read_text()))
    if linux is not None and windows is not None and all_features is not None and LINUX_ALL_FEATURES_SNAPSHOT.exists():
        data = add_source_audit(data)
    if args.update_linux_feature_audit:
        if not args.from_json:
            run_linux_feature_audit_rustdoc()
        linux_audit_path = ROOT / "target" / LINUX_TARGET / "doc/gpui.json"
        linux_audit_docs = load_docs(False)
        linux_audit_docs["gpui"] = json.loads(linux_audit_path.read_text())
        linux_audit = generate(linux_audit_docs)
        linux_audit["feature_profile"] = "no default features; bench,inspector,leak-detection,profiler,stacker,wayland (x11,screen-capture,test-support,windows-manifest omitted)"
        linux_audit["rustdoc_command"] = f"cargo +nightly rustdoc --manifest-path <registry gpui-pre-0.3.5/Cargo.toml> --lib --target {LINUX_TARGET} --no-default-features --features bench,inspector,leak-detection,profiler,stacker,wayland -- -Z unstable-options --output-format json --document-hidden-items"
        LINUX_FEATURE_AUDIT_SNAPSHOT.write_text(json.dumps(linux_audit, indent=2, ensure_ascii=False) + "\n")
    elif LINUX_FEATURE_AUDIT_SNAPSHOT.exists():
        linux_audit = json.loads(LINUX_FEATURE_AUDIT_SNAPSHOT.read_text())
    else:
        linux_audit = None
    if linux_audit is not None:
        if not data.get("cfg_public_declaration_candidates"):
            raise RuntimeError("Linux feature-audit merge requires the macOS/Linux/Windows/all-features source audit")
        data = add_linux_feature_audit(data, linux_audit)
    rendered = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
    if args.markdown:
        sys.stdout.write(markdown(data))
    elif args.check:
        bad = False
        if not OUT.exists() or OUT.read_text() != rendered:
            print(f"stale inventory: {OUT}", file=sys.stderr)
            bad = True
        if not OUT_MD.exists() or OUT_MD.read_text() != markdown(data):
            print(f"stale inventory page: {OUT_MD}", file=sys.stderr)
            bad = True
        if linux_audit is None:
            print(f"missing Linux feature-audit snapshot: {LINUX_FEATURE_AUDIT_SNAPSHOT}; run --update-linux-feature-audit", file=sys.stderr)
            bad = True
        for snapshot in (WASM_TEST_SUPPORT_SNAPSHOT, LINUX_ALL_FEATURES_SNAPSHOT):
            if not snapshot.exists():
                print(f"missing isolated profile snapshot: {snapshot}; run --update-isolated-audits", file=sys.stderr)
                bad = True
        if bad:
            return 1
    else:
        OUT.write_text(rendered)
    if not args.markdown:
        print(f"{data['count']} names; {data['counts_by_kind']}; {len(data['extraction_gaps'])} unresolved rustdoc IDs; {len(data.get('source_audited_unverified', []))} source-audited")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
