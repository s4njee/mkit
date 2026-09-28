"""Offline tests for the evaluation runner. Run: python3 -m unittest discover -s evals/runner"""

import io
import json
import shutil
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import mkit_eval  # noqa: E402

ROOT = mkit_eval.ROOT
PY = sys.executable


def claude_transcript(input_tokens=100, output_tokens=20, cache_read=1000, cost=0.5):
    events = [
        {"type": "system", "subtype": "init"},
        {"type": "assistant", "message": {"content": []}},
        {"type": "result", "subtype": "success", "num_turns": 3, "total_cost_usd": cost,
         "usage": {"input_tokens": input_tokens, "output_tokens": output_tokens,
                   "cache_read_input_tokens": cache_read, "cache_creation_input_tokens": 5}},
    ]
    return "\n".join(json.dumps(e) for e in events) + "\n"


class TempDirTestCase(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="mkit-eval-test-"))

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)


class ManifestTests(unittest.TestCase):
    def test_repository_benchmarks_are_consistent(self):
        self.assertEqual(mkit_eval.check_benchmarks(), [])

    def test_select_benchmarks_by_range_number_and_id(self):
        manifest = mkit_eval.load_manifest()
        ids = [b["id"] for b in mkit_eval.select_benchmarks(manifest, "1-3")]
        self.assertEqual(ids, ["01-counter", "02-todo-list", "03-settings-screen"])
        ids = [b["id"] for b in mkit_eval.select_benchmarks(manifest, "10,01-counter,1")]
        self.assertEqual(ids, ["10-multi-window-notes", "01-counter"])
        self.assertEqual(len(mkit_eval.select_benchmarks(manifest, "all")), 10)
        with self.assertRaises(mkit_eval.EvalError):
            mkit_eval.select_benchmarks(manifest, "11")
        with self.assertRaises(mkit_eval.EvalError):
            mkit_eval.select_benchmarks(manifest, "counter")

    def test_m1_benchmarks_are_the_first_six(self):
        manifest = mkit_eval.load_manifest()
        m1 = [b["number"] for b in manifest["benchmarks"] if b["milestone"] == "M1"]
        self.assertEqual(m1, [1, 2, 3, 4, 5, 6])

    def test_agent_task_text_drops_maintainer_notes(self):
        spec = ROOT / "evals" / "benchmarks" / "01-counter" / "spec.md"
        task = mkit_eval.agent_task_text(spec)
        self.assertIn("## Acceptance criteria", task)
        self.assertNotIn("Book coverage", task)
        self.assertNotIn(mkit_eval.MAINTAINER_MARKER, task)

    def test_check_reports_missing_test_function(self):
        tmp = Path(tempfile.mkdtemp())
        try:
            for name in ("evals", "book/src"):
                (tmp / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copytree(ROOT / "evals", tmp / "evals",
                            ignore=shutil.ignore_patterns("__pycache__"))
            (tmp / "book" / "src").mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / "book" / "src" / "SUMMARY.md", tmp / "book" / "src" / "SUMMARY.md")
            acceptance = tmp / "evals" / "benchmarks" / "01-counter" / "acceptance.rs"
            acceptance.write_text(acceptance.read_text().replace(
                "fn ac1_initial_count_is_zero", "fn renamed_test"))
            errors = mkit_eval.check_benchmarks(tmp)
            self.assertTrue(any("ac1_initial_count_is_zero" in e for e in errors), errors)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)


class BookExportTests(TempDirTestCase):
    def test_summary_assigns_parts(self):
        parts = dict((path, part) for part, path in
                     mkit_eval.parse_summary(ROOT / "book" / "src" / "SUMMARY.md"))
        self.assertEqual(parts["install.md"], 1)
        self.assertEqual(parts["state/entities.md"], 2)
        self.assertEqual(parts["interaction/keyboard-list.md"], 4)
        self.assertEqual(parts["text-input/single-line-field.md"], 5)
        self.assertEqual(parts["appendices/glossary.md"], 0)

    def test_part_selector(self):
        self.assertEqual(mkit_eval.parse_part_selector("1-4"), {1, 2, 3, 4})
        self.assertEqual(mkit_eval.parse_part_selector("1,5-6"), {1, 5, 6})
        self.assertIsNone(mkit_eval.parse_part_selector("all"))
        with self.assertRaises(mkit_eval.EvalError):
            mkit_eval.parse_part_selector("I-IV")

    def test_extract_anchor_keeps_only_the_named_region(self):
        source = "a\n// ANCHOR: one\nb\n// ANCHOR: two\nc\n// ANCHOR_END: two\n// ANCHOR_END: one\nd"
        self.assertEqual(mkit_eval.extract_anchor(source, "one"), "b\nc")
        self.assertEqual(mkit_eval.extract_anchor(source, "two"), "c")
        self.assertEqual(mkit_eval.extract_anchor(source, None), "a\nb\nc\nd")

    def test_text_export_resolves_includes_and_filters_parts(self):
        book = mkit_eval.export_book_text(self.tmp / "book", {1, 2, 3, 4})
        text = book.read_text()
        self.assertNotIn("{{#include", text)
        self.assertIn("<!-- chapter: examples/counter.md -->", text)
        self.assertIn("impl Render for Counter", text)  # resolved from examples/counter
        self.assertIn("<!-- chapter: interaction/keyboard-list.md -->", text)
        self.assertNotIn("<!-- chapter: text-input/", text)
        self.assertIn("<!-- chapter: appendices/glossary.md -->", text)
        self.assertIn("Full text: book.md", (self.tmp / "book" / "llms.txt").read_text())


class WorkspaceTests(TempDirTestCase):
    def prepare(self, benchmark_id="01-counter"):
        manifest = mkit_eval.load_manifest()
        benchmark = mkit_eval.select_benchmarks(manifest, benchmark_id)[0]
        return mkit_eval.prepare_workspace(benchmark, self.tmp / "attempt", parts={1, 2, 3, 4},
                                           cargo_target_dir=self.tmp / "target")

    def test_workspace_contains_only_agent_material(self):
        paths = self.prepare("04-file-browser")
        workspace = paths["workspace"]
        for name in ("TASK.md", "CONTRACT.md", "Cargo.toml", "app/Cargo.toml", "app/src/lib.rs",
                     "app/src/main.rs", "book/book.md", "fixture/notes.txt"):
            self.assertTrue((workspace / name).is_file(), name)
        names = {p.name for p in workspace.rglob("*")}
        self.assertFalse({"acceptance.rs", "screenshots.rs", "support.rs", "manifest.json"} & names)
        self.assertNotIn("Book coverage", (workspace / "TASK.md").read_text())
        self.assertEqual(mkit_eval.audit_workspace(workspace), [])
        config = json.loads(paths["mcp_config"].read_text())
        self.assertTrue(config["mcpServers"]["mkit-inspector"]["command"].endswith("mkit-inspector"))
        prompt = paths["prompt_file"].read_text()
        self.assertIn("benchmark 4, File browser", prompt)
        self.assertIn("book/book.md", prompt)
        self.assertFalse(paths["prompt_file"].is_relative_to(workspace))

    def test_audit_detects_hidden_reference_copy(self):
        workspace = self.prepare()["workspace"]
        reference = ROOT / "evals" / "hidden" / "reference" / "01-counter" / "src" / "lib.rs"
        shutil.copy2(reference, workspace / "app" / "src" / "lib.rs")
        findings = mkit_eval.audit_workspace(workspace)
        self.assertTrue(any("hidden evaluation file" in f for f in findings), findings)

    def test_audit_detects_outside_path_dependency_and_git(self):
        workspace = self.prepare()["workspace"]
        manifest = workspace / "app" / "Cargo.toml"
        manifest.write_text(manifest.read_text() + '\n[dependencies.x]\npath = "../../../x"\n')
        (workspace / ".git").mkdir()
        findings = mkit_eval.audit_workspace(workspace)
        self.assertTrue(any("path dependency" in f for f in findings), findings)
        self.assertTrue(any("repository metadata" in f for f in findings), findings)

    def test_grading_crates_get_unique_package_names_and_fresh_mtimes(self):
        manifest = mkit_eval.load_manifest()
        benchmark = mkit_eval.select_benchmarks(manifest, "01-counter")[0]
        candidate = ROOT / "evals" / "skeleton" / "app"
        first = mkit_eval.generate_grading_crate(benchmark, candidate, self.tmp / "g1")
        second = mkit_eval.generate_grading_crate(benchmark, candidate, self.tmp / "g2")
        self.assertNotEqual(first, second)
        app_manifest = (self.tmp / "g1" / "app" / "Cargo.toml").read_text()
        self.assertIn(f'name = "{first[0]}"', app_manifest)
        self.assertIn('name = "bench_app"', app_manifest, "the [lib] name is unchanged")
        acceptance = (self.tmp / "g1" / "acceptance" / "Cargo.toml").read_text()
        self.assertIn(f'name = "{first[1]}"', acceptance)
        self.assertIn(f'package = "{first[0]}"', acceptance)
        copied = self.tmp / "g1" / "app" / "src" / "lib.rs"
        self.assertGreater(copied.stat().st_mtime, (candidate / "src" / "lib.rs").stat().st_mtime)
        for name in ("acceptance.rs", "screenshots.rs", "support/mod.rs"):
            self.assertTrue((self.tmp / "g1" / "acceptance" / "tests" / name).is_file(), name)

    def test_run_dir_inside_repository_is_rejected(self):
        with self.assertRaises(mkit_eval.EvalError):
            mkit_eval.ensure_outside_repo(ROOT / "target" / "evals")
        mkit_eval.ensure_outside_repo(self.tmp)


class CommandTests(unittest.TestCase):
    def test_render_command_quotes_values(self):
        command = mkit_eval.render_command("agent --model {model} < {prompt_file}",
                                           {"model": "m 1", "prompt_file": "/a b/p.md"})
        self.assertEqual(command, "agent --model 'm 1' < '/a b/p.md'")

    def test_render_command_rejects_unknown_and_missing_values(self):
        with self.assertRaises(mkit_eval.EvalError):
            mkit_eval.render_command("agent {secret}", {})
        with self.assertRaises(mkit_eval.EvalError):
            mkit_eval.render_command("agent --model {model}", {"model": None})

    def test_bundled_agent_configs_render(self):
        values = {name: f"/v/{name}" for name in mkit_eval.PLACEHOLDERS}
        for path in sorted((ROOT / "evals" / "agents").glob("*.json")):
            agent = mkit_eval.load_agent(path, None)
            self.assertIn(agent["token_parser"], mkit_eval.TOKEN_PARSERS, path)
            command = mkit_eval.render_command(agent["command"], values)
            self.assertIn("/v/prompt_file", command, path)


class ParserTests(TempDirTestCase):
    def test_claude_tokens(self):
        path = self.tmp / "t.jsonl"
        path.write_text("not json\n" + claude_transcript())
        tokens = mkit_eval.parse_tokens_claude(path)
        self.assertEqual(tokens["input"], 100)
        self.assertEqual(tokens["total"], 100 + 20 + 1000 + 5)
        self.assertEqual(tokens["cost_usd"], 0.5)
        self.assertEqual(tokens["turns"], 3)
        path.write_text("")
        self.assertIsNone(mkit_eval.parse_tokens_claude(path))

    def test_codex_tokens(self):
        path = self.tmp / "t.jsonl"
        events = [{"type": "turn.completed",
                   "usage": {"input_tokens": 10, "cached_input_tokens": 4, "output_tokens": 2}}] * 2
        path.write_text("\n".join(json.dumps(e) for e in events))
        self.assertEqual(mkit_eval.parse_tokens_codex(path),
                         {"input": 20, "output": 4, "cache_read": 8, "total": 24})

    def test_libtest_and_screenshot_output(self):
        output = ("running 3 tests\ntest ac1_a ... ok\ntest ac2_b ... FAILED\ntest ac3_c ... ignored\n"
                  "\nfailures:\n\n---- ac2_b stdout ----\nassertion failed: count\n\nfailures:\n")
        self.assertEqual(mkit_eval.parse_libtest(output),
                         {"ac1_a": "passed", "ac2_b": "failed", "ac3_c": "skipped"})
        self.assertIn("assertion failed", mkit_eval.failure_detail(output, "ac2_b"))
        shots = 'noise\nMKIT_EVAL_RESULT {"test": "ac5_x", "status": "skipped", "detail": "no gpu"}\n'
        self.assertEqual(mkit_eval.parse_shot_results(shots)["ac5_x"]["status"], "skipped")

    def test_criteria_summary_and_judgement(self):
        criteria = [{"id": "AC0", "status": "passed"}, {"id": "AC1", "status": "passed"},
                    {"id": "AC2", "status": "skipped"}]
        summary = mkit_eval.summarize_criteria(criteria)
        self.assertEqual(summary["status"], "passed_with_skips")
        record = {"harness": summary, "isolation": {"after": []}}
        self.assertEqual(mkit_eval.judge(record), (True, None))
        criteria[1]["status"] = "failed"
        record["harness"] = mkit_eval.summarize_criteria(criteria)
        self.assertEqual(mkit_eval.judge(record), (False, "AC1"))
        record["harness"] = mkit_eval.summarize_criteria(
            [{"id": "AC0", "status": "failed"}, {"id": "AC1", "status": "not_run"}])
        self.assertEqual(mkit_eval.judge(record), (False, "build"))
        record["isolation"]["after"] = ["leak"]
        self.assertEqual(mkit_eval.judge(record), (False, "isolation"))
        self.assertEqual(mkit_eval.judge({"agent_timed_out": True}), (False, "agent_timeout"))


class RunTests(TempDirTestCase):
    def run_cli(self, *args):
        out = io.StringIO()
        with redirect_stdout(out):
            code = mkit_eval.main(list(args))
        return code, out.getvalue()

    def test_dry_run_prepares_workspaces_without_running_the_agent(self):
        marker = self.tmp / "agent-ran"
        code, output = self.run_cli(
            "run", "--dry-run", "--benchmarks", "1-2", "--parts", "1-4", "--model", "test-model",
            "--agent-command", f"touch {marker} && agent --model {{model}} < {{prompt_file}}",
            "--run-dir", str(self.tmp / "run"), "--cargo-target-dir", str(self.tmp / "target"))
        self.assertEqual(code, 0, output)
        self.assertFalse(marker.exists(), "dry run must not execute the agent command")
        for benchmark in ("01-counter", "02-todo-list"):
            attempt = self.tmp / "run" / benchmark / "attempt-1"
            record = json.loads((attempt / "attempt.json").read_text())
            self.assertEqual(record["status"], "dry_run")
            self.assertEqual(record["schema_version"], mkit_eval.SCHEMA_VERSION)
            self.assertIsNone(record["success"])
            self.assertEqual(record["agent"]["model"], "test-model")
            self.assertIn(str(attempt / "prompt.md"), record["agent"]["command"])
            self.assertEqual(record["context"]["book_parts"], [1, 2, 3, 4])
            self.assertEqual(record["isolation"]["before"], [])
            self.assertTrue((attempt / "workspace" / "book" / "book.md").is_file())
            self.assertIn("agent --model test-model", (attempt / "command.txt").read_text())
        summary = json.loads((self.tmp / "run" / "summary.json").read_text())
        self.assertEqual(summary["dry_run_attempts"], 2)
        self.assertEqual(summary["graded_attempts"], 0)

    def test_missing_model_fails_before_preparing(self):
        code, _ = self.run_cli(
            "run", "--dry-run", "--benchmarks", "1", "--agent-command", "agent {model}",
            "--run-dir", str(self.tmp / "run"))
        self.assertEqual(code, 2)
        self.assertFalse((self.tmp / "run").exists())

    def test_agent_run_records_transcript_tokens_and_wall_time(self):
        script = self.tmp / "fake_agent.py"
        script.write_text(
            "import json, pathlib, sys\n"
            "pathlib.Path('app/src/extra.rs').write_text('// agent output\\n')\n"
            f"sys.stdout.write({claude_transcript()!r})\n"
            "sys.stderr.write('agent log\\n')\n")
        agent = self.tmp / "agent.json"
        agent.write_text(json.dumps({"name": "fake", "command": f"{PY} {script}",
                                     "token_parser": "claude-stream-json"}))
        code, output = self.run_cli(
            "run", "--skip-grade", "--no-build-inspector", "--benchmarks", "1", "--agent", str(agent), "--parts", "1",
            "--run-dir", str(self.tmp / "run"), "--cargo-target-dir", str(self.tmp / "target"),
            "--timeout", "60")
        self.assertEqual(code, 0, output)
        attempt = self.tmp / "run" / "01-counter" / "attempt-1"
        record = json.loads((attempt / "attempt.json").read_text())
        self.assertEqual(record["status"], "completed")
        self.assertEqual(record["agent_exit_code"], 0)
        self.assertFalse(record["agent_timed_out"])
        self.assertGreater(record["wall_time_seconds"], 0)
        self.assertEqual(record["tokens"]["total"], 1125)
        self.assertEqual(record["isolation"]["after"], [])
        self.assertIsNone(record["harness"])
        self.assertIsNone(record["success"])
        self.assertTrue((attempt / "workspace" / "app" / "src" / "extra.rs").is_file())
        self.assertIn('"type": "result"', Path(record["transcript_path"]).read_text())
        self.assertIn("agent log", (attempt / "agent-stderr.log").read_text())

    def test_agent_timeout_is_a_failure(self):
        code, _ = self.run_cli(
            "run", "--skip-grade", "--no-build-inspector", "--benchmarks", "1", "--parts", "1", "--timeout", "1",
            "--agent-command", f"{PY} -c 'import time; time.sleep(30)'",
            "--run-dir", str(self.tmp / "run"), "--cargo-target-dir", str(self.tmp / "target"))
        self.assertEqual(code, 0)
        record = json.loads(
            (self.tmp / "run" / "01-counter" / "attempt-1" / "attempt.json").read_text())
        self.assertTrue(record["agent_timed_out"])
        self.assertEqual((record["success"], record["point_of_failure"]), (False, "agent_timeout"))
        self.assertLess(record["wall_time_seconds"], 20)

    def test_leaked_reference_fails_the_attempt(self):
        reference = ROOT / "evals" / "hidden" / "reference" / "01-counter" / "src" / "lib.rs"
        code, _ = self.run_cli(
            "run", "--skip-grade", "--no-build-inspector", "--benchmarks", "1", "--parts", "1",
            "--agent-command", f"cp {reference} app/src/lib.rs",
            "--run-dir", str(self.tmp / "run"), "--cargo-target-dir", str(self.tmp / "target"))
        self.assertEqual(code, 0)
        record = json.loads(
            (self.tmp / "run" / "01-counter" / "attempt-1" / "attempt.json").read_text())
        self.assertEqual((record["success"], record["point_of_failure"]), (False, "isolation"))


def record(benchmark, number, success, milestone="M1", tokens=1000, failure=None, skipped=0):
    return {
        "schema_version": mkit_eval.SCHEMA_VERSION, "run_id": "r1", "status": "completed",
        "benchmark": {"id": benchmark, "number": number, "title": benchmark,
                      "milestone": milestone},
        "success": success, "point_of_failure": failure, "wall_time_seconds": 60.0,
        "tokens": {"total": tokens, "cost_usd": 0.25}, "harness": {"skipped": skipped},
    }


class ReportTests(TempDirTestCase):
    def setUp(self):
        super().setUp()
        self.manifest = mkit_eval.load_manifest()
        self.m1 = [b for b in self.manifest["benchmarks"] if b["milestone"] == "M1"]

    def test_m1_gate_met_when_every_m1_app_passes(self):
        records = [record(b["id"], b["number"], True) for b in self.m1]
        summary = mkit_eval.summarize(records, self.manifest)
        self.assertTrue(summary["gates"]["M1"]["met"])
        self.assertFalse(summary["gates"]["1.0"]["met"], "apps 7-10 were not attempted")
        self.assertEqual(summary["total_tokens"], 6000)

    def test_m1_gate_needs_every_app_and_the_threshold(self):
        records = [record(b["id"], b["number"], True) for b in self.m1[:5]]
        self.assertFalse(mkit_eval.summarize(records, self.manifest)["gates"]["M1"]["met"])
        records = [record(b["id"], b["number"], True) for b in self.m1]
        records.append(record(self.m1[5]["id"], 6, False, failure="AC3"))
        summary = mkit_eval.summarize(records, self.manifest)
        gate = summary["gates"]["M1"]
        self.assertAlmostEqual(gate["success_rate"], 6 / 7, places=3)
        self.assertFalse(gate["met"])
        self.assertEqual(gate["min_benchmark_rate"], 0.5)
        six = next(b for b in summary["benchmarks"] if b["number"] == 6)
        self.assertEqual(six["points_of_failure"], {"AC3": 1})

    def test_dry_runs_are_not_scored_and_report_renders(self):
        records = [record("01-counter", 1, True, skipped=1),
                   {**record("02-todo-list", 2, None), "status": "dry_run"}]
        summary = mkit_eval.summarize(records, self.manifest)
        self.assertEqual(summary["graded_attempts"], 1)
        self.assertEqual(summary["dry_run_attempts"], 1)
        markdown = mkit_eval.render_report(summary)
        self.assertIn("| 1 | Counter | M1 | 1 | 1/1 (100%) | 60s | 1000 | $0.25 | – | 1 |",
                      markdown)
        self.assertIn("| M1 (≥ 90%) |", markdown)

    def test_report_command_reads_attempt_files(self):
        run = self.tmp / "run" / "01-counter" / "attempt-1"
        run.mkdir(parents=True)
        mkit_eval.write_json(run / "attempt.json", record("01-counter", 1, False, failure="build"))
        out = io.StringIO()
        with redirect_stdout(out):
            code = mkit_eval.main(["report", str(self.tmp / "run"), "--out-dir",
                                   str(self.tmp / "report")])
        self.assertEqual(code, 0)
        summary = json.loads((self.tmp / "report" / "summary.json").read_text())
        self.assertEqual(summary["benchmarks"][0]["points_of_failure"], {"build": 1})
        self.assertTrue((self.tmp / "report" / "summary.md").is_file())

    def test_unknown_schema_version_is_rejected(self):
        path = self.tmp / "attempt.json"
        path.write_text(json.dumps({"schema_version": 99}))
        with self.assertRaises(mkit_eval.EvalError):
            mkit_eval.load_records([path])


if __name__ == "__main__":
    unittest.main()
