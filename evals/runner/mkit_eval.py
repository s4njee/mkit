#!/usr/bin/env python3
"""Run GPUI Book evaluations (plan E3.1 and E3.2).

Subcommands:

  list      Show the benchmark apps and their milestones.
  check     Validate the benchmark manifest, specs, tests, and book coverage (fast, offline).
  prepare   Create one isolated agent workspace without running anything.
  run       Prepare workspaces, invoke an agent command, grade, and record attempt JSON.
  grade     Grade an existing candidate crate (for example the hidden reference solution).
  report    Summarize attempt records into summary.json and summary.md.

The agent is any command. A command template names it (see evals/agents/*.json), so the runner
never talks to a model provider itself. `run --dry-run` prepares everything and writes the exact
command it would run, without running it. See evals/README.md for the record format.
"""

from __future__ import annotations

import argparse
import datetime as _dt
import hashlib
import json
import os
import re
import shlex
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCHEMA_VERSION = 1
MAINTAINER_MARKER = "<!-- maintainer-only -->"
ROMAN = {"I": 1, "II": 2, "III": 3, "IV": 4, "V": 5, "VI": 6, "VII": 7, "VIII": 8, "IX": 9,
         "X": 10, "XI": 11, "XII": 12, "XIII": 13, "XIV": 14, "XV": 15}
PLACEHOLDERS = ("workspace", "prompt_file", "mcp_config", "transcript", "model", "benchmark",
                "attempt_dir", "book_path", "inspector", "inspector_toml")


class EvalError(Exception):
    """A user-facing configuration or setup error."""


# ---------------------------------------------------------------------------
# Paths and manifest


def evals_dir(root: Path = ROOT) -> Path:
    return root / "evals"


def load_manifest(root: Path = ROOT) -> dict:
    path = evals_dir(root) / "benchmarks" / "manifest.json"
    return json.loads(path.read_text(encoding="utf-8"))


def benchmark_dir(benchmark: dict, root: Path = ROOT) -> Path:
    return evals_dir(root) / "benchmarks" / benchmark["id"]


def select_benchmarks(manifest: dict, selector: str) -> list[dict]:
    """Select benchmarks by `all`, numbers, ranges, or ids: `1-6`, `1,3`, `01-counter`."""
    benchmarks = manifest["benchmarks"]
    if selector in ("", "all"):
        return list(benchmarks)
    chosen: list[dict] = []
    for token in (part.strip() for part in selector.split(",")):
        if not token:
            continue
        matched = [b for b in benchmarks if b["id"] == token]
        if not matched:
            range_match = re.fullmatch(r"(\d+)(?:-(\d+))?", token)
            if not range_match:
                raise EvalError(f"unknown benchmark selector `{token}`")
            low = int(range_match.group(1))
            high = int(range_match.group(2) or low)
            matched = [b for b in benchmarks if low <= b["number"] <= high]
            if not matched:
                raise EvalError(f"no benchmark numbered in `{token}`")
        for benchmark in matched:
            if benchmark not in chosen:
                chosen.append(benchmark)
    return chosen


def agent_task_text(spec_path: Path) -> str:
    """The agent-visible part of a spec: everything before the maintainer marker."""
    text = spec_path.read_text(encoding="utf-8")
    return text.split(MAINTAINER_MARKER, 1)[0].rstrip() + "\n"


def parse_part_selector(selector: str | None) -> set[int] | None:
    """`1-4` or `1,2,5` to a set of part numbers; `None`/`all` means every part."""
    if selector in (None, "", "all"):
        return None
    parts: set[int] = set()
    for token in selector.split(","):
        token = token.strip()
        match = re.fullmatch(r"(\d+)(?:-(\d+))?", token)
        if not match:
            raise EvalError(f"invalid part selector `{token}`")
        low, high = int(match.group(1)), int(match.group(2) or match.group(1))
        parts.update(range(low, high + 1))
    return parts


# ---------------------------------------------------------------------------
# Book export


def parse_summary(summary: Path) -> list[tuple[int, str]]:
    """Return (part number, chapter path) in reading order.

    Chapters before the first part heading are part 0. Appendix pages are part 0 as well,
    so they are always exported. Draft links without a path are skipped.
    """
    chapters: list[tuple[int, str]] = []
    part = 0
    for line in summary.read_text(encoding="utf-8").splitlines():
        heading = re.match(r"^#\s+Part\s+([IVXLC]+)\b", line)
        if heading:
            part = ROMAN.get(heading.group(1), part)
            continue
        link = re.search(r"\[[^\]]*\]\(([^)]+\.md)\)", line)
        if link:
            path = link.group(1)
            chapters.append((0 if path.startswith("appendices/") else part, path))
    return chapters


_INCLUDE = re.compile(r"\{\{#include\s+([^}\s]+?)(?::([A-Za-z_][\w-]*))?\s*\}\}")
_ANCHOR = re.compile(r"^\s*//\s*ANCHOR(?:_END)?:\s*[A-Za-z_][\w-]*\s*$")


def extract_anchor(source: str, anchor: str | None) -> str:
    lines = source.splitlines()
    if anchor is None:
        return "\n".join(line for line in lines if not _ANCHOR.match(line))
    out: list[str] = []
    inside = False
    for line in lines:
        stripped = line.strip()
        if re.fullmatch(rf"//\s*ANCHOR:\s*{re.escape(anchor)}", stripped):
            inside = True
            continue
        if re.fullmatch(rf"//\s*ANCHOR_END:\s*{re.escape(anchor)}", stripped):
            inside = False
            continue
        if inside and not _ANCHOR.match(line):
            out.append(line)
    return "\n".join(out)


def resolve_includes(text: str, page: Path) -> str:
    def replace(match: re.Match) -> str:
        target = (page.parent / match.group(1)).resolve()
        if not target.is_file():
            return f"<!-- missing include: {match.group(1)} -->"
        body = extract_anchor(target.read_text(encoding="utf-8"), match.group(2))
        if target.suffix == ".md":
            body = resolve_includes(body, target)
        return body

    return _INCLUDE.sub(replace, text)


def export_book_text(dest: Path, parts: set[int] | None = None, root: Path = ROOT) -> Path:
    """Write an llms-style single-file export with includes resolved."""
    src = root / "book" / "src"
    dest.mkdir(parents=True, exist_ok=True)
    sections: list[str] = []
    index: list[str] = ["# The GPUI Book (evaluation export)", ""]
    for part, relative in parse_summary(src / "SUMMARY.md"):
        if parts is not None and part != 0 and part not in parts:
            continue
        page = src / relative
        if not page.is_file():
            continue
        body = resolve_includes(page.read_text(encoding="utf-8"), page)
        sections.append(f"<!-- chapter: {relative} -->\n\n{body.rstrip()}\n")
        title = next((line.lstrip("# ").strip() for line in body.splitlines()
                      if line.startswith("#")), relative)
        index.append(f"- {title} ({relative})")
    book = dest / "book.md"
    book.write_text("\n\n".join(sections))
    (dest / "llms.txt").write_text("\n".join(index) + "\n\nFull text: book.md\n")
    return book


def export_book_html(dest: Path, root: Path = ROOT, build: bool = False) -> Path:
    built = root / "book" / "book"
    if build or not (built / "index.html").is_file():
        result = subprocess.run(["mdbook", "build", str(root / "book")], capture_output=True,
                                text=True)
        if result.returncode != 0:
            raise EvalError(f"mdbook build failed:\n{result.stderr[-2000:]}")
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(built, dest)
    return dest / "index.html"


def book_digest(path: Path) -> str:
    digest = hashlib.sha256()
    files = [path] if path.is_file() else sorted(p for p in path.rglob("*") if p.is_file())
    for file in files:
        digest.update(file.read_bytes())
    return "sha256:" + digest.hexdigest()


# ---------------------------------------------------------------------------
# Workspace preparation


def inspector_binary(cargo_target_dir: Path) -> Path:
    suffix = ".exe" if os.name == "nt" else ""
    return cargo_target_dir / "debug" / f"mkit-inspector{suffix}"


def mcp_config(inspector: Path) -> dict:
    return {"mcpServers": {"mkit-inspector": {"command": str(inspector), "args": []}}}


def prepare_workspace(benchmark: dict, attempt_dir: Path, *, book_format: str = "text",
                      parts: set[int] | None = None, cargo_target_dir: Path | None = None,
                      root: Path = ROOT) -> dict:
    """Create `attempt_dir/workspace` with only the book, task, contract, skeleton, and fixture.

    Nothing from `evals/hidden`, `evals/grading`, or the acceptance tests is copied.
    """
    cargo_target_dir = cargo_target_dir or root / "target"
    workspace = attempt_dir / "workspace"
    if workspace.exists():
        raise EvalError(f"workspace already exists: {workspace}")
    workspace.mkdir(parents=True)
    evals = evals_dir(root)
    # Fresh mtimes: agents share a cargo target dir, and cargo trusts mtimes, so an older
    # skeleton copy could otherwise reuse a previous attempt's `bench-app` artefacts.
    shutil.copy(evals / "skeleton" / "Cargo.toml", workspace / "Cargo.toml")
    shutil.copytree(evals / "skeleton" / "app", workspace / "app", copy_function=shutil.copy)
    lock = root / "Cargo.lock"
    if lock.is_file():
        shutil.copy2(lock, workspace / "Cargo.lock")
    fixture = benchmark_dir(benchmark, root) / "fixture"
    if fixture.is_dir():
        shutil.copytree(fixture, workspace / "fixture")
    else:
        (workspace / "fixture").mkdir()
    (workspace / "TASK.md").write_text(agent_task_text(benchmark_dir(benchmark, root) / "spec.md"))
    shutil.copy2(evals / "benchmarks" / "CONTRACT.md", workspace / "CONTRACT.md")
    if book_format == "text":
        book_path = export_book_text(workspace / "book", parts, root)
    elif book_format == "html":
        if parts is not None:
            raise EvalError("--parts is only supported with --book-format text")
        book_path = export_book_html(workspace / "book", root)
    else:
        raise EvalError(f"unknown book format `{book_format}`")
    config_path = attempt_dir / "mcp.json"
    config_path.write_text(json.dumps(mcp_config(inspector_binary(cargo_target_dir)), indent=2)
                           + "\n")
    prompt_path = attempt_dir / "prompt.md"
    prompt_path.write_text(render_prompt(benchmark, book_path.relative_to(workspace), root))
    return {
        "workspace": workspace,
        "book_path": book_path,
        "mcp_config": config_path,
        "prompt_file": prompt_path,
        "book_digest": book_digest(book_path if book_path.suffix == ".md" else book_path.parent),
    }


def render_prompt(benchmark: dict, book_relative: Path, root: Path = ROOT) -> str:
    template = (evals_dir(root) / "runner" / "prompt.md").read_text(encoding="utf-8")
    return template.format(number=benchmark["number"], title=benchmark["title"],
                           book=book_relative.as_posix())


def forbidden_hashes(root: Path = ROOT) -> dict[str, str]:
    """sha256 -> repository path for files an agent must never see."""
    evals = evals_dir(root)
    candidates = [p for p in (evals / "hidden").rglob("*") if p.is_file()]
    candidates += [p for p in (evals / "grading").rglob("*") if p.is_file()]
    for name in ("acceptance.rs", "screenshots.rs"):
        candidates += list((evals / "benchmarks").glob(f"*/{name}"))
    skeleton = {hashlib.sha256(p.read_bytes()).hexdigest()
                for p in (evals / "skeleton").rglob("*") if p.is_file()}
    hashes = {}
    for path in candidates:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        # Files shared with the skeleton (Cargo.toml, main.rs) are public by design.
        if digest not in skeleton:
            hashes[digest] = path.relative_to(root).as_posix()
    return hashes


def audit_workspace(workspace: Path, root: Path = ROOT) -> list[str]:
    """Report leaked hidden files, repository metadata, vendored GPUI source, or outside paths."""
    findings: list[str] = []
    hidden = forbidden_hashes(root)
    for path in sorted(workspace.rglob("*")):
        relative = path.relative_to(workspace).as_posix()
        if relative.startswith("target/") or "/target/" in relative:
            continue
        if path.is_dir():
            if path.name == ".git":
                findings.append(f"repository metadata present: {relative}")
            elif re.fullmatch(r"gpui(-pre)?-\d+\.\d+\.\d+", path.name):
                findings.append(f"vendored GPUI source present: {relative}")
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if digest in hidden:
            findings.append(f"hidden evaluation file copied: {relative} == {hidden[digest]}")
    manifest = workspace / "app" / "Cargo.toml"
    if manifest.is_file():
        for match in re.finditer(r'path\s*=\s*"([^"]+)"', manifest.read_text(encoding="utf-8")):
            target = (manifest.parent / match.group(1)).resolve()
            if workspace.resolve() not in target.parents and target != workspace.resolve():
                findings.append(f"app/Cargo.toml path dependency leaves the workspace: "
                                f"{match.group(1)}")
    return findings


# ---------------------------------------------------------------------------
# Agent invocation


def load_agent(path: Path | None, command: str | None) -> dict:
    if command:
        return {"name": "custom", "command": command, "token_parser": "none", "env": {}}
    if path is None:
        raise EvalError("provide --agent CONFIG.json or --agent-command TEMPLATE")
    config = json.loads(Path(path).read_text(encoding="utf-8"))
    for key in ("name", "command"):
        if key not in config:
            raise EvalError(f"agent config {path} is missing `{key}`")
    config.setdefault("token_parser", "none")
    config.setdefault("env", {})
    return config


def render_command(template: str, values: dict) -> str:
    """Substitute `{placeholder}` values, shell-quoted. Unknown names are an error."""
    def replace(match: re.Match) -> str:
        name = match.group(1)
        if name not in PLACEHOLDERS:
            raise EvalError(f"unknown placeholder `{{{name}}}` in agent command")
        value = values.get(name)
        if value in (None, ""):
            raise EvalError(f"agent command needs `{{{name}}}` but no value was given")
        return shlex.quote(str(value))

    return re.sub(r"\{([a-z_]+)\}", replace, template)


def run_agent(command: str, cwd: Path, transcript: Path, stderr_log: Path, timeout: float,
              env: dict[str, str]) -> dict:
    started = time.monotonic()
    timed_out = False
    with transcript.open("wb") as out, stderr_log.open("wb") as err:
        process = subprocess.Popen(command, shell=True, cwd=cwd, stdout=out, stderr=err,
                                   stdin=subprocess.DEVNULL, env={**os.environ, **env},
                                   start_new_session=True)
        try:
            exit_code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            _terminate(process)
            exit_code = process.wait()
    return {"exit_code": exit_code, "timed_out": timed_out,
            "wall_time_seconds": round(time.monotonic() - started, 3)}


def _terminate(process: subprocess.Popen) -> None:
    try:
        if os.name != "nt":
            os.killpg(process.pid, 15)
        else:
            process.terminate()
        process.wait(timeout=10)
    except Exception:  # noqa: BLE001 - best effort shutdown
        process.kill()


def _json_lines(path: Path):
    if not path.is_file():
        return
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            yield json.loads(line)
        except json.JSONDecodeError:
            continue


def parse_tokens_claude(path: Path) -> dict | None:
    """Claude Code `--output-format stream-json` (or `json`): the final `result` event."""
    result = None
    for event in _json_lines(path):
        if event.get("type") == "result":
            result = event
    if result is None or not isinstance(result.get("usage"), dict):
        return None
    usage = result["usage"]
    tokens = {
        "input": usage.get("input_tokens", 0),
        "output": usage.get("output_tokens", 0),
        "cache_read": usage.get("cache_read_input_tokens", 0),
        "cache_creation": usage.get("cache_creation_input_tokens", 0),
    }
    tokens["total"] = sum(tokens.values())
    if "total_cost_usd" in result:
        tokens["cost_usd"] = result["total_cost_usd"]
    if "num_turns" in result:
        tokens["turns"] = result["num_turns"]
    return tokens


def parse_tokens_codex(path: Path) -> dict | None:
    """`codex exec --json`: sum `usage` over `turn.completed` events."""
    totals = {"input": 0, "output": 0, "cache_read": 0}
    seen = False
    for event in _json_lines(path):
        usage = event.get("usage")
        if event.get("type") == "turn.completed" and isinstance(usage, dict):
            seen = True
            totals["input"] += usage.get("input_tokens", 0)
            totals["output"] += usage.get("output_tokens", 0)
            totals["cache_read"] += usage.get("cached_input_tokens", 0)
    if not seen:
        return None
    totals["total"] = totals["input"] + totals["output"]
    return totals


TOKEN_PARSERS = {
    "none": lambda path: None,
    "claude-stream-json": parse_tokens_claude,
    "codex-jsonl": parse_tokens_codex,
}


# ---------------------------------------------------------------------------
# Grading


def grading_manifest(root: Path, app_package: str, acceptance_package: str) -> str:
    harness = (root / "crates" / "mkit-harness").as_posix()
    return f"""[package]
name = "{acceptance_package}"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
bench_app = {{ package = "{app_package}", path = "../app" }}
gpui_pre = {{ package = "gpui-pre", version = "=0.3.5", features = ["test-support"] }}
# `#[gpui_pre::test]` expands to paths under the gpui-kit facade when gpui-kit is a dependency.
gpui-kit = "=0.6.4"
mkit-harness = {{ path = "{harness}" }}
serde_json = "1"
image = "=0.25.10"

[[test]]
name = "acceptance"
path = "tests/acceptance.rs"

[[test]]
name = "screenshots"
path = "tests/screenshots.rs"
harness = false
"""


_PACKAGE_NAME = re.compile(r'(\[package\][^\[]*?\bname\s*=\s*")([^"]+)(")', re.S)


def grading_packages(grading_dir: Path) -> tuple[str, str]:
    """Unique package names for one grading run.

    Grading shares a cargo target dir to reuse the GPUI build. Cargo derives artifact hashes for
    path packages from the package name, version, and workspace-relative path, so two grading
    workspaces with the same layout would otherwise share (and wrongly reuse) test binaries.
    """
    tag = hashlib.sha256(f"{grading_dir.resolve()}:{time.time_ns()}".encode()).hexdigest()[:10]
    return f"bench-app-{tag}", f"bench-acceptance-{tag}"


def generate_grading_crate(benchmark: dict, candidate: Path, grading_dir: Path,
                           root: Path = ROOT) -> tuple[str, str]:
    """Copy the candidate and this benchmark's tests into a fresh grading workspace.

    Returns the (candidate, acceptance) package names used in this workspace.
    """
    if not (candidate / "Cargo.toml").is_file():
        raise EvalError(f"candidate crate has no Cargo.toml: {candidate}")
    if grading_dir.exists():
        shutil.rmtree(grading_dir)
    grading_dir.mkdir(parents=True)
    # copy, not copy2: fresh mtimes so cargo never trusts an older fingerprint.
    shutil.copytree(candidate, grading_dir / "app", copy_function=shutil.copy,
                    ignore=shutil.ignore_patterns("target", ".git"))
    app_package, acceptance_package = grading_packages(grading_dir)
    app_manifest = grading_dir / "app" / "Cargo.toml"
    text, count = _PACKAGE_NAME.subn(lambda m: m.group(1) + app_package + m.group(3),
                                     app_manifest.read_text(encoding="utf-8"), count=1)
    if count != 1:
        raise EvalError(f"candidate Cargo.toml has no [package] name: {candidate}")
    app_manifest.write_text(text)
    tests = grading_dir / "acceptance" / "tests"
    (tests / "support").mkdir(parents=True)
    source = benchmark_dir(benchmark, root)
    shutil.copy(source / "acceptance.rs", tests / "acceptance.rs")
    shutil.copy(source / "screenshots.rs", tests / "screenshots.rs")
    shutil.copy(evals_dir(root) / "grading" / "support.rs", tests / "support" / "mod.rs")
    (grading_dir / "acceptance" / "Cargo.toml").write_text(
        grading_manifest(root, app_package, acceptance_package))
    (grading_dir / "Cargo.toml").write_text(
        '[workspace]\nresolver = "2"\nmembers = ["app", "acceptance"]\n')
    if (root / "Cargo.lock").is_file():
        shutil.copy2(root / "Cargo.lock", grading_dir / "Cargo.lock")
    fixture = source / "fixture"
    if fixture.is_dir():
        shutil.copytree(fixture, grading_dir / "fixture")
    else:
        (grading_dir / "fixture").mkdir()
    return app_package, acceptance_package


_LIBTEST = re.compile(r"^test (\S+) \.\.\. (ok|FAILED|ignored)\b", re.M)


def parse_libtest(output: str) -> dict[str, str]:
    mapping = {"ok": "passed", "FAILED": "failed", "ignored": "skipped"}
    return {name: mapping[status] for name, status in _LIBTEST.findall(output)}


def parse_shot_results(output: str) -> dict[str, dict]:
    results = {}
    for line in output.splitlines():
        if line.startswith("MKIT_EVAL_RESULT "):
            try:
                record = json.loads(line[len("MKIT_EVAL_RESULT "):])
            except json.JSONDecodeError:
                continue
            results[record.get("test", "")] = record
    return results


def failure_detail(output: str, test: str) -> str:
    match = re.search(rf"---- {re.escape(test)} stdout ----\n(.*?)(?:\n---- |\nfailures:\n)",
                      output, re.S)
    return match.group(1).strip()[-1500:] if match else ""


def _cargo(args: list[str], cwd: Path, env: dict, log: Path, timeout: float) -> tuple[int, str]:
    try:
        result = subprocess.run(["cargo", *args], cwd=cwd, env={**os.environ, **env},
                                capture_output=True, text=True, timeout=timeout)
        output = result.stdout + "\n" + result.stderr
        code = result.returncode
    except subprocess.TimeoutExpired as error:
        output = f"timed out after {timeout}s\n{error.stdout or ''}{error.stderr or ''}"
        code = -1
    log.write_text(output)
    return code, output


def grade(benchmark: dict, candidate: Path, grading_dir: Path, *, cargo_target_dir: Path,
          offline: bool = True, timeout: float = 3600, keep_build: bool = False,
          root: Path = ROOT) -> dict:
    """Build the candidate, run the benchmark's acceptance and screenshot tests, map criteria.

    Unless `keep_build` is set, this run's packages are removed from the shared target dir
    afterwards (each grading run leaves roughly 200 MB of debug test binaries otherwise).
    """
    app_package, acceptance_package = generate_grading_crate(benchmark, candidate, grading_dir,
                                                             root)
    artifacts = grading_dir / "artifacts"
    artifacts.mkdir()
    env = {"CARGO_TARGET_DIR": str(cargo_target_dir), "MKIT_EVAL_ARTIFACTS": str(artifacts),
           "MKIT_EVAL_FIXTURE": str(grading_dir / "fixture"), "CARGO_TERM_COLOR": "never",
           # One-off builds gain nothing from incremental caches, which are large.
           "CARGO_INCREMENTAL": "0"}
    flags = ["--offline"] if offline else []
    manifest = ["--manifest-path", str(grading_dir / "Cargo.toml")]
    started = time.monotonic()
    build_code, _ = _cargo(["build", *manifest, *flags, "-p", app_package, "--bins"],
                           grading_dir, env, grading_dir / "build.log", timeout)
    results: dict[str, dict] = {}
    commands = []
    if build_code == 0:
        test_code, test_out = _cargo(
            ["test", *manifest, *flags, "-p", acceptance_package, "--test", "acceptance"],
            grading_dir, env, grading_dir / "acceptance.log", timeout)
        compiled = "running " in test_out
        for name, status in parse_libtest(test_out).items():
            results[name] = {"status": status, "detail": failure_detail(test_out, name)}
        shot_code, shot_out = _cargo(
            ["test", *manifest, *flags, "-p", acceptance_package, "--test", "screenshots"],
            grading_dir, env, grading_dir / "screenshots.log", timeout)
        for name, record in parse_shot_results(shot_out).items():
            results[name] = {"status": record.get("status", "failed"),
                             "detail": record.get("detail", "")}
        commands = [test_code, shot_code]
        if not compiled:
            for criterion in benchmark["criteria"]:
                if criterion.get("target") == "acceptance":
                    results.setdefault(criterion["test"], {
                        "status": "failed",
                        "detail": "acceptance tests did not compile against the candidate; "
                                  "check the contract (see acceptance.log)"})
    skipped_dir = artifacts / "skipped"
    if skipped_dir.is_dir():
        for marker in skipped_dir.glob("*.txt"):
            entry = results.get(marker.stem)
            if entry and entry["status"] == "passed":
                entry["status"] = "skipped"
                entry["detail"] = marker.read_text(encoding="utf-8")
    criteria = []
    for criterion in benchmark["criteria"]:
        if criterion["target"] == "build":
            status = "passed" if build_code == 0 else "failed"
            detail = "" if build_code == 0 else "cargo build --bins failed (see build.log)"
        elif build_code != 0:
            status, detail = "not_run", "build failed"
        else:
            entry = results.get(criterion["test"])
            status, detail = ((entry["status"], entry["detail"]) if entry else
                              ("failed", "test did not report a result"))
        criteria.append({"id": criterion["id"], "test": criterion.get("test"),
                         "kind": criterion.get("kind"), "status": status, "detail": detail})
    if not keep_build:
        _cargo(["clean", *manifest, *flags, "-p", app_package, "-p", acceptance_package],
               grading_dir, env, grading_dir / "clean.log", timeout)
    return summarize_criteria(criteria, {
        "grading_dir": str(grading_dir),
        "wall_time_seconds": round(time.monotonic() - started, 3),
        "build_exit_code": build_code,
        "test_exit_codes": commands,
    })


def summarize_criteria(criteria: list[dict], extra: dict | None = None) -> dict:
    counts = {status: sum(1 for c in criteria if c["status"] == status)
              for status in ("passed", "failed", "skipped", "not_run")}
    first_failure = next((c["id"] for c in criteria if c["status"] in ("failed", "not_run")),
                         None)
    if counts["failed"] or counts["not_run"]:
        status = "failed"
    elif counts["skipped"]:
        status = "passed_with_skips"
    else:
        status = "passed"
    return {**(extra or {}), "status": status, "criteria": criteria, **counts,
            "first_failure": first_failure}


# ---------------------------------------------------------------------------
# Attempts and runs


def utc_now() -> str:
    return _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def git_commit(root: Path = ROOT) -> str | None:
    try:
        return subprocess.run(["git", "rev-parse", "HEAD"], cwd=root, capture_output=True,
                              text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def run_attempt(benchmark: dict, attempt_dir: Path, *, agent: dict, model: str | None,
                run_id: str, attempt: int, dry_run: bool, skip_grade: bool,
                book_format: str, parts: set[int] | None, cargo_target_dir: Path,
                timeout: float, offline: bool = True, keep_build: bool = False,
                root: Path = ROOT) -> dict:
    attempt_dir.mkdir(parents=True, exist_ok=True)
    paths = prepare_workspace(benchmark, attempt_dir, book_format=book_format, parts=parts,
                              cargo_target_dir=cargo_target_dir, root=root)
    transcript = attempt_dir / "transcript.jsonl"
    values = {
        "workspace": paths["workspace"], "prompt_file": paths["prompt_file"],
        "mcp_config": paths["mcp_config"], "transcript": transcript,
        "model": model or agent.get("default_model"), "benchmark": benchmark["id"],
        "attempt_dir": attempt_dir, "book_path": paths["book_path"],
        "inspector": inspector_binary(cargo_target_dir),
        # A TOML basic string, for agents configured with `-c key=value` overrides.
        "inspector_toml": json.dumps(str(inspector_binary(cargo_target_dir))),
    }
    command = render_command(agent["command"], values)
    (attempt_dir / "command.txt").write_text(command + "\n")
    record = {
        "schema_version": SCHEMA_VERSION,
        "run_id": run_id,
        "attempt_id": f"{benchmark['id']}#{attempt}",
        "benchmark": {key: benchmark[key] for key in ("id", "number", "title", "milestone")},
        "agent": {"name": agent["name"], "model": values["model"],
                  "command_template": agent["command"], "command": command},
        "context": {
            "book_format": book_format,
            "book_parts": sorted(parts) if parts else "all",
            "book_path": str(paths["book_path"]),
            "book_digest": paths["book_digest"],
            "book_commit": git_commit(root),
            "mcp_servers": ["mkit-inspector"],
            "mcp_config": str(paths["mcp_config"]),
        },
        "workspace": str(paths["workspace"]),
        "transcript_path": str(transcript),
        "stderr_path": str(attempt_dir / "agent-stderr.log"),
        "started_at": utc_now(),
        "finished_at": None,
        "wall_time_seconds": None,
        "agent_exit_code": None,
        "agent_timed_out": False,
        "tokens": None,
        "isolation": {"before": audit_workspace(paths["workspace"], root), "after": None},
        "harness": None,
        "success": None,
        "point_of_failure": None,
        "status": "dry_run" if dry_run else "completed",
        "triage": {"classification": None, "notes": ""},
    }
    if dry_run:
        record["finished_at"] = utc_now()
        write_json(attempt_dir / "attempt.json", record)
        return record

    env = {"CARGO_TARGET_DIR": str(cargo_target_dir), **agent.get("env", {})}
    if offline:
        env["CARGO_NET_OFFLINE"] = "true"
    outcome = run_agent(command, paths["workspace"], transcript,
                        attempt_dir / "agent-stderr.log", timeout, env)
    record.update({"agent_exit_code": outcome["exit_code"],
                   "agent_timed_out": outcome["timed_out"],
                   "wall_time_seconds": outcome["wall_time_seconds"]})
    parser = TOKEN_PARSERS.get(agent.get("token_parser", "none"))
    if parser is None:
        raise EvalError(f"unknown token parser `{agent.get('token_parser')}`")
    record["tokens"] = parser(transcript)
    record["isolation"]["after"] = audit_workspace(paths["workspace"], root)
    if not skip_grade:
        record["harness"] = grade(benchmark, paths["workspace"] / "app", attempt_dir / "grading",
                                  cargo_target_dir=cargo_target_dir, offline=offline,
                                  timeout=timeout, keep_build=keep_build, root=root)
    record["success"], record["point_of_failure"] = judge(record)
    record["finished_at"] = utc_now()
    write_json(attempt_dir / "attempt.json", record)
    return record


def judge(record: dict) -> tuple[bool | None, str | None]:
    """Success needs a graded pass and a clean isolation audit. Timeouts always fail."""
    if record.get("agent_timed_out"):
        return False, "agent_timeout"
    if (record.get("isolation") or {}).get("after"):
        return False, "isolation"
    harness = record.get("harness")
    if harness is None:
        return None, None
    if harness["status"] in ("passed", "passed_with_skips"):
        return True, None
    first = harness.get("first_failure")
    return False, "build" if first == "AC0" else first


def write_json(path: Path, value) -> None:
    path.write_text(json.dumps(value, indent=2, default=str) + "\n")


def new_run_id() -> str:
    return _dt.datetime.now(_dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def default_run_root() -> Path:
    return Path(os.environ.get("MKIT_EVAL_RUNS", Path(tempfile.gettempdir()) / "mkit-evals"))


def ensure_outside_repo(path: Path, root: Path = ROOT) -> None:
    resolved = path.resolve()
    if resolved == root.resolve() or root.resolve() in resolved.parents:
        raise EvalError(
            f"run directory {path} is inside the repository; agents could read hidden reference "
            "solutions and GPUI sources from there. Choose a directory outside the repository.")


def build_inspector(cargo_target_dir: Path, root: Path = ROOT) -> None:
    result = subprocess.run(["cargo", "build", "-p", "mkit-inspector", "--locked"], cwd=root,
                            env={**os.environ, "CARGO_TARGET_DIR": str(cargo_target_dir)},
                            capture_output=True, text=True)
    if result.returncode != 0:
        raise EvalError(f"building mkit-inspector failed:\n{result.stderr[-2000:]}")


# ---------------------------------------------------------------------------
# Reports


def load_records(paths: list[Path]) -> list[dict]:
    records = []
    for path in paths:
        files = [path] if path.is_file() else sorted(path.rglob("attempt.json"))
        for file in files:
            record = json.loads(file.read_text(encoding="utf-8"))
            if record.get("schema_version") != SCHEMA_VERSION:
                raise EvalError(f"{file}: unsupported schema_version "
                                f"{record.get('schema_version')}")
            records.append(record)
    return records


def summarize(records: list[dict], manifest: dict, threshold: float = 0.9) -> dict:
    graded = [r for r in records if r.get("status") == "completed" and r.get("success") is not None]
    benchmarks = []
    for benchmark in manifest["benchmarks"]:
        attempts = [r for r in graded if r["benchmark"]["id"] == benchmark["id"]]
        successes = sum(1 for r in attempts if r["success"])
        failures: dict[str, int] = {}
        for record in attempts:
            if not record["success"]:
                key = record.get("point_of_failure") or "unknown"
                failures[key] = failures.get(key, 0) + 1
        tokens = [r["tokens"]["total"] for r in attempts if r.get("tokens")]
        costs = [r["tokens"]["cost_usd"] for r in attempts
                 if r.get("tokens") and "cost_usd" in r["tokens"]]
        walls = [r["wall_time_seconds"] for r in attempts if r.get("wall_time_seconds")]
        skipped = sum((r.get("harness") or {}).get("skipped", 0) for r in attempts)
        benchmarks.append({
            "id": benchmark["id"], "number": benchmark["number"], "title": benchmark["title"],
            "milestone": benchmark["milestone"], "attempts": len(attempts),
            "successes": successes,
            "success_rate": round(successes / len(attempts), 4) if attempts else None,
            "points_of_failure": dict(sorted(failures.items())),
            "median_wall_time_seconds": statistics.median(walls) if walls else None,
            "mean_tokens": round(statistics.mean(tokens)) if tokens else None,
            "total_tokens": sum(tokens) if tokens else None,
            "total_cost_usd": round(sum(costs), 4) if costs else None,
            "skipped_criteria": skipped,
        })
    gates = {}
    for name, members in (("M1", [b for b in benchmarks if b["milestone"] == "M1"]),
                          ("1.0", benchmarks)):
        attempted = [b for b in members if b["attempts"]]
        total = sum(b["attempts"] for b in attempted)
        wins = sum(b["successes"] for b in attempted)
        rate = wins / total if total else None
        gates[name] = {
            "benchmarks": [b["id"] for b in members],
            "all_attempted": len(attempted) == len(members),
            "attempts": total,
            "success_rate": round(rate, 4) if rate is not None else None,
            "min_benchmark_rate": min((b["success_rate"] for b in attempted), default=None),
            "threshold": threshold,
            "met": bool(rate is not None and rate >= threshold and len(attempted) == len(members)),
        }
    all_tokens = [r["tokens"]["total"] for r in graded if r.get("tokens")]
    return {
        "schema_version": SCHEMA_VERSION,
        "generated_at": utc_now(),
        "runs": sorted({r["run_id"] for r in records}),
        "graded_attempts": len(graded),
        "dry_run_attempts": sum(1 for r in records if r.get("status") == "dry_run"),
        "ungraded_attempts": sum(1 for r in records
                                 if r.get("status") == "completed" and r.get("success") is None),
        "total_tokens": sum(all_tokens) if all_tokens else None,
        "benchmarks": benchmarks,
        "gates": gates,
    }


def render_report(summary: dict) -> str:
    def fmt(value, suffix=""):
        return "–" if value is None else f"{value}{suffix}"

    lines = ["# Book evaluation report", "",
             f"Generated {summary['generated_at']} from runs: {', '.join(summary['runs']) or '–'}.",
             f"Graded attempts: {summary['graded_attempts']}; dry runs: "
             f"{summary['dry_run_attempts']}; ungraded: {summary['ungraded_attempts']}; "
             f"total tokens: {fmt(summary['total_tokens'])}.", "", "## Gates", "",
             "| Gate | Attempts | Success rate | Lowest app rate | All apps attempted | Met |",
             "|---|---|---|---|---|---|"]
    for name, gate in summary["gates"].items():
        rate = None if gate["success_rate"] is None else f"{gate['success_rate']:.0%}"
        low = None if gate["min_benchmark_rate"] is None else f"{gate['min_benchmark_rate']:.0%}"
        lines.append(f"| {name} (≥ {gate['threshold']:.0%}) | {gate['attempts']} | {fmt(rate)} | "
                     f"{fmt(low)} | {'yes' if gate['all_attempted'] else 'no'} | "
                     f"{'yes' if gate['met'] else 'no'} |")
    lines += ["", "## Benchmarks", "",
              "| # | App | Milestone | Attempts | Success | Median wall time | Mean tokens | "
              "Cost | Points of failure | Skipped checks |",
              "|---|---|---|---|---|---|---|---|---|---|"]
    for b in summary["benchmarks"]:
        rate = None if b["success_rate"] is None else f"{b['successes']}/{b['attempts']} " \
                                                       f"({b['success_rate']:.0%})"
        failures = ", ".join(f"{k} ×{v}" for k, v in b["points_of_failure"].items()) or "–"
        wall = None if b["median_wall_time_seconds"] is None else \
            f"{b['median_wall_time_seconds']:.0f}s"
        cost = None if b["total_cost_usd"] is None else f"${b['total_cost_usd']:.2f}"
        lines.append(f"| {b['number']} | {b['title']} | {b['milestone']} | {b['attempts']} | "
                     f"{fmt(rate)} | {fmt(wall)} | {fmt(b['mean_tokens'])} | {fmt(cost)} | "
                     f"{failures} | {b['skipped_criteria']} |")
    lines += ["", "Skipped checks are criteria the host could not verify (for example screenshots "
              "off macOS or accessibility while headless windows report it inactive). They do not "
              "count as failures, so review them before publishing a result.", ""]
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Manifest validation (the CI check)


def check_benchmarks(root: Path = ROOT) -> list[str]:
    errors: list[str] = []
    manifest = load_manifest(root)
    summary = parse_summary(root / "book" / "src" / "SUMMARY.md")
    chapter_parts = {path: part for part, path in summary}
    numbers = [b["number"] for b in manifest["benchmarks"]]
    if numbers != list(range(1, len(numbers) + 1)):
        errors.append(f"benchmark numbers must be 1..N in order, got {numbers}")
    if len(numbers) != 10:
        errors.append(f"plan E3.1 requires 10 benchmark apps, manifest has {len(numbers)}")
    m1_parts = set(manifest.get("m1_parts", [1, 2, 3, 4]))
    for benchmark in manifest["benchmarks"]:
        label = benchmark["id"]
        directory = benchmark_dir(benchmark, root)
        spec = directory / "spec.md"
        for name in ("spec.md", "acceptance.rs", "screenshots.rs"):
            if not (directory / name).is_file():
                errors.append(f"{label}: missing {name}")
        if not spec.is_file():
            continue
        spec_text = spec.read_text(encoding="utf-8")
        if MAINTAINER_MARKER not in spec_text:
            errors.append(f"{label}: spec.md lacks the `{MAINTAINER_MARKER}` marker")
        task = agent_task_text(spec)
        for heading in ("## Goal", "## Required features", "## Snapshot",
                        "## Acceptance criteria"):
            if heading not in task:
                errors.append(f"{label}: agent-visible spec lacks `{heading}`")
        sources = {target: (directory / f"{target}.rs").read_text(encoding="utf-8")
                   for target in ("acceptance", "screenshots")
                   if (directory / f"{target}.rs").is_file()}
        ids = []
        for criterion in benchmark["criteria"]:
            ids.append(criterion["id"])
            if f"**{criterion['id']}**" not in task:
                errors.append(f"{label}: {criterion['id']} is not described in the agent task")
            target = criterion.get("target")
            if target == "build":
                continue
            if target not in sources:
                errors.append(f"{label}: {criterion['id']} has unknown target `{target}`")
                continue
            if not re.search(rf"\bfn {re.escape(criterion['test'])}\s*\(", sources[target]):
                errors.append(f"{label}: {criterion['id']} test `{criterion['test']}` is not "
                              f"defined in {target}.rs")
        if len(set(ids)) != len(ids):
            errors.append(f"{label}: duplicate criterion ids")
        for key in ("chapters", "optional_chapters"):
            for chapter in benchmark.get(key, []):
                if chapter not in chapter_parts:
                    errors.append(f"{label}: {key} entry `{chapter}` is not in SUMMARY.md")
        if benchmark["milestone"] == "M1":
            for chapter in benchmark.get("chapters", []):
                part = chapter_parts.get(chapter)
                if part is not None and part not in m1_parts and part != 0:
                    errors.append(f"{label}: M1 benchmark requires `{chapter}` outside Parts "
                                  f"{sorted(m1_parts)}")
        if benchmark.get("fixture") and not (directory / "fixture").is_dir():
            errors.append(f"{label}: manifest says fixture but {directory / 'fixture'} is missing")
    hidden = evals_dir(root) / "hidden"
    if not (hidden / "README.md").is_file():
        errors.append("evals/hidden/README.md must explain why the directory is excluded")
    return errors


# ---------------------------------------------------------------------------
# CLI


def _common_workspace_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--book-format", choices=("text", "html"), default="text",
                        help="text: single-file llms-style export; html: built mdBook site")
    parser.add_argument("--parts", default=None,
                        help="book parts to export with --book-format text, e.g. 1-4 (default all)")
    parser.add_argument("--cargo-target-dir", type=Path, default=ROOT / "target",
                        help="shared cargo target dir for agent builds and grading")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("list", help="list benchmarks")
    sub.add_parser("check", help="validate manifest, specs, tests, and chapter coverage")

    prepare = sub.add_parser("prepare", help="create one agent workspace")
    prepare.add_argument("benchmark")
    prepare.add_argument("--out", type=Path, required=True)
    _common_workspace_args(prepare)

    run = sub.add_parser("run", help="run evaluation attempts")
    run.add_argument("--benchmarks", default="1-6", help="e.g. 1-6, all, 01-counter,3")
    run.add_argument("--agent", type=Path, help="agent config JSON (see evals/agents)")
    run.add_argument("--agent-command", help="command template instead of --agent")
    run.add_argument("--model", help="value for {model}; defaults to the config's default_model")
    run.add_argument("--attempts", type=int, default=1, help="attempts per benchmark")
    run.add_argument("--run-dir", type=Path, help="default: $MKIT_EVAL_RUNS or "
                                                   "<tmp>/mkit-evals, plus the run id")
    run.add_argument("--timeout", type=float, default=3600, help="seconds per agent attempt")
    run.add_argument("--dry-run", action="store_true",
                     help="prepare workspaces and commands without running the agent or grader")
    run.add_argument("--skip-grade", action="store_true", help="run the agent but do not grade")
    run.add_argument("--online", action="store_true", help="allow cargo network access")
    run.add_argument("--no-build-inspector", action="store_true",
                     help="do not build mkit-inspector first (the MCP config still points at it)")
    run.add_argument("--keep-build", action="store_true",
                     help="keep grading artefacts in the shared cargo target dir")
    _common_workspace_args(run)

    grade_cmd = sub.add_parser("grade", help="grade an existing candidate crate")
    grade_cmd.add_argument("benchmark")
    grade_cmd.add_argument("--candidate", type=Path, required=True)
    grade_cmd.add_argument("--out", type=Path, required=True, help="grading directory")
    grade_cmd.add_argument("--cargo-target-dir", type=Path, default=ROOT / "target")
    grade_cmd.add_argument("--online", action="store_true")
    grade_cmd.add_argument("--timeout", type=float, default=3600)
    grade_cmd.add_argument("--keep-build", action="store_true")

    report = sub.add_parser("report", help="summarize attempt records")
    report.add_argument("paths", nargs="+", type=Path, help="run dirs or attempt.json files")
    report.add_argument("--out-dir", type=Path, help="write summary.json and summary.md here")
    report.add_argument("--threshold", type=float, default=0.9)

    args = parser.parse_args(argv)
    try:
        manifest = load_manifest()
        if args.command == "list":
            for b in manifest["benchmarks"]:
                print(f"{b['number']:>2}  {b['id']:<24} {b['milestone']:<4} {b['title']}")
            return 0
        if args.command == "check":
            errors = check_benchmarks()
            for error in errors:
                print(f"error: {error}", file=sys.stderr)
            if not errors:
                print(f"{len(manifest['benchmarks'])} benchmarks OK")
            return 1 if errors else 0
        if args.command == "prepare":
            benchmark = select_benchmarks(manifest, args.benchmark)[0]
            ensure_outside_repo(args.out)
            paths = prepare_workspace(benchmark, args.out, book_format=args.book_format,
                                      parts=parse_part_selector(args.parts),
                                      cargo_target_dir=args.cargo_target_dir)
            print(json.dumps({k: str(v) for k, v in paths.items()}, indent=2))
            return 0
        if args.command == "run":
            return command_run(args, manifest)
        if args.command == "grade":
            benchmark = select_benchmarks(manifest, args.benchmark)[0]
            result = grade(benchmark, args.candidate.resolve(), args.out.resolve(),
                           cargo_target_dir=args.cargo_target_dir.resolve(),
                           offline=not args.online, timeout=args.timeout,
                           keep_build=args.keep_build)
            write_json(args.out / "grade.json", result)
            for criterion in result["criteria"]:
                print(f"{criterion['id']:<5} {criterion['status']:<8} {criterion['test'] or ''}")
            print(f"status: {result['status']}")
            return 0 if result["status"] in ("passed", "passed_with_skips") else 1
        if args.command == "report":
            summary = summarize(load_records(args.paths), manifest, args.threshold)
            markdown = render_report(summary)
            if args.out_dir:
                args.out_dir.mkdir(parents=True, exist_ok=True)
                write_json(args.out_dir / "summary.json", summary)
                (args.out_dir / "summary.md").write_text(markdown)
            print(markdown)
            return 0
    except EvalError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    return 1


def command_run(args: argparse.Namespace, manifest: dict) -> int:
    agent = load_agent(args.agent, args.agent_command)
    if "{model}" in agent["command"] and not (args.model or agent.get("default_model")):
        raise EvalError("the agent command uses {model}; pass --model")
    benchmarks = select_benchmarks(manifest, args.benchmarks)
    run_id = new_run_id()
    run_dir = (args.run_dir or default_run_root() / run_id).resolve()
    ensure_outside_repo(run_dir)
    run_dir.mkdir(parents=True, exist_ok=True)
    parts = parse_part_selector(args.parts)
    cargo_target_dir = args.cargo_target_dir.resolve()
    if not args.dry_run and not args.no_build_inspector:
        build_inspector(cargo_target_dir)
    write_json(run_dir / "run.json", {
        "schema_version": SCHEMA_VERSION, "run_id": run_id, "started_at": utc_now(),
        "agent": agent["name"], "model": args.model or agent.get("default_model"),
        "benchmarks": [b["id"] for b in benchmarks], "attempts": args.attempts,
        "dry_run": args.dry_run, "book_format": args.book_format,
        "book_parts": sorted(parts) if parts else "all", "book_commit": git_commit(),
    })
    records = []
    for benchmark in benchmarks:
        for attempt in range(1, args.attempts + 1):
            attempt_dir = run_dir / benchmark["id"] / f"attempt-{attempt}"
            record = run_attempt(
                benchmark, attempt_dir, agent=agent, model=args.model, run_id=run_id,
                attempt=attempt, dry_run=args.dry_run, skip_grade=args.skip_grade,
                book_format=args.book_format, parts=parts, cargo_target_dir=cargo_target_dir,
                timeout=args.timeout, offline=not args.online, keep_build=args.keep_build)
            records.append(record)
            outcome = record["status"] if record["success"] is None else \
                ("success" if record["success"] else f"failed at {record['point_of_failure']}")
            print(f"{benchmark['id']} attempt {attempt}: {outcome}")
            if args.dry_run:
                print(f"  command: {record['agent']['command']}")
    summary = summarize(records, manifest)
    write_json(run_dir / "summary.json", summary)
    (run_dir / "summary.md").write_text(render_report(summary))
    print(f"run directory: {run_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
