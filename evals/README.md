# Evaluations

Book evaluations measure whether *The GPUI Book* teaches (plan goal G3, epics E3.1 and E3.2): a
fresh coding agent gets only the book and the inspector MCP server, builds a benchmark app, and the
harness decides whether it works. Book v0.1 (milestone M1) needs at least 90% success on apps 1–6;
1.0 needs at least 90% on all 10.

What exists now:

- **E3.1 benchmark apps:** 10 specs with harness acceptance tests and screenshot checks.
- **E3.2 scaffolding:** a runner that builds isolated agent workspaces, invokes any agent through a
  command template, grades the result, records each attempt as JSON, and summarizes runs. No paid
  evaluation has been run yet.
- A hidden reference solution for app 1 that passes its acceptance tests end to end.

Not done yet: real evaluation runs and published results (E3.2), failure triage (E3.3), reference
solutions for apps 2–10 (E10.3), and the book-health page (E12.1).

## Layout

| Path | Contents | Visible to agents |
|---|---|---|
| `benchmarks/manifest.json` | Benchmark order, milestone, required and optional chapters, criteria mapped to test functions | No |
| `benchmarks/CONTRACT.md` | The library items every candidate crate must export | Yes (copied) |
| `benchmarks/NN-name/spec.md` | Agent task above `<!-- maintainer-only -->`; book coverage and harness gaps below | Task part only (as `TASK.md`) |
| `benchmarks/NN-name/acceptance.rs` | Keyboard, pointer, state, and accessibility tests (`#[gpui_pre::test]`) | No |
| `benchmarks/NN-name/screenshots.rs` | Headless screenshot checks (`harness = false`, macOS) | No |
| `benchmarks/NN-name/fixture/` | Test data the app reads | Yes (copied) |
| `grading/support.rs` | Shared test helpers used by every benchmark | No |
| `skeleton/` | Starting workspace: `app/` crate with the contract stubbed out | Yes (copied) |
| `hidden/reference/` | Reference solutions (currently app 1) — see `hidden/README.md` | Never |
| `agents/*.json` | Agent command templates | No |
| `runner/mkit_eval.py` | Runner, grader, and report generator; `runner/prompt.md` is the agent prompt | No |

## Benchmarks

| # | App | Gate | Book parts needed | Criteria |
|---|---|---|---|---|
| 1 | Counter | M1 | I, II, IV | 7 |
| 2 | Todo list with keyboard shortcuts | M1 | I–IV | 9 |
| 3 | Settings screen | M1 | II–IV | 9 |
| 4 | File browser | M1 | II–IV (VII optional) | 9 |
| 5 | Markdown previewer | M1 | II–IV | 7 |
| 6 | Split-pane editor | M1 | I, III, IV (V optional) | 8 |
| 7 | Image viewer with zoom | 1.0 | III, IV, VI | 8 |
| 8 | Searchable table | 1.0 | IV, V, VI | 8 |
| 9 | Canvas drawing app | 1.0 | IV, VI | 7 |
| 10 | Multi-window notes app | 1.0 | II, IV, V, VII, VIII | 8 |

The list is the one in plan E3.1. `mkit_eval.py check` enforces that every required chapter of an
M1 benchmark is in Parts I–IV (optional chapters may come from anywhere), that every criterion is
described in the agent task, and that each criterion's test function exists.

## How grading works

A candidate is any crate that follows `benchmarks/CONTRACT.md`: `init(&mut App)`,
`root(&Path) -> Root`, `Root::snapshot(&App) -> serde_json::Value`, and `debug_selector` names for
click targets. Tests observe the app only through that contract, so candidates are free to design
everything else.

`grade` copies the candidate into a fresh grading workspace with the benchmark's tests, then runs:

1. **AC0:** `cargo build --bins` on the candidate.
2. `cargo test --test acceptance`: GPUI `VisualTestContext` tests that press keys, type, click and
   drag `debug_selector` targets, and assert on `snapshot()`. Scripts use the `mkit-harness`
   input-script format where they fit (`click @increment`).
3. `cargo test --test screenshots`: renders the root at 800 × 600, scale 1, with
   `mkit_harness::HeadlessSession`, saves PNGs to `grading/artifacts/screenshots/` for review, and
   checks that frames are not blank and change when state changes (or, for app 3, get darker).

Each criterion ends as `passed`, `failed`, `skipped`, or `not_run` (the build failed). An attempt
succeeds when nothing failed; skipped criteria are allowed but counted in the report.

Known harness limits, recorded as skips rather than passes:

- **Screenshots** run on macOS only; GPUI 0.3.5 has no headless renderer elsewhere.
- **Accessibility** criteria are skipped while headless GPUI windows report accessibility inactive
  (`mkit_harness::AccessibilitySnapshot`). They assert automatically once the harness can read the
  tree.
- The headless screenshot session cannot resolve `debug_selector` targets (GPUI exposes debug bounds
  only on `VisualTestContext`), so screenshot scripts use keys or fixed window coordinates.
- **The inspector MCP server cannot launch a candidate app.** It launches the book's own examples
  (`hello`, `counter`, `state_entities`, `reactivity_views`, `gallery`). Agents can use it to study
  the book's examples, not to check their own app. Extending `launch` to a workspace crate is the
  main open E1.5 item for evaluations.

Grading builds share a cargo target dir with the repository to reuse the GPUI build. Each grading
run gets unique package names (cargo would otherwise reuse another run's test binaries) and removes
its own artefacts afterwards; pass `--keep-build` to keep them. Attempts run one at a time;
do not run two evaluations against the same target dir at once, because every agent
workspace builds a package named `bench-app`.

## Running

All commands run from the repository root. Runs must live outside the repository so agents cannot
reach hidden material; the default is `$MKIT_EVAL_RUNS` or `<system temp>/mkit-evals/<run id>`.

```sh
python3 evals/runner/mkit_eval.py list
python3 evals/runner/mkit_eval.py check

# Prove the grader with the hidden reference (under a minute once GPUI is built).
python3 evals/runner/mkit_eval.py grade 01-counter \
  --candidate evals/hidden/reference/01-counter --out /tmp/mkit-grade-01

# Dry run: prepares workspaces, prompts, MCP configs, and the exact agent command; runs nothing.
python3 evals/runner/mkit_eval.py run --dry-run --benchmarks 1-6 --parts 1-4 \
  --agent evals/agents/claude-code.json --model MODEL_ID

# Real M1 run (paid): 3 attempts per app with Parts I–IV as context.
python3 evals/runner/mkit_eval.py run --benchmarks 1-6 --attempts 3 --parts 1-4 \
  --agent evals/agents/claude-code.json --model MODEL_ID --run-dir ~/mkit-evals/m1-round1

python3 evals/runner/mkit_eval.py report ~/mkit-evals/m1-round1 --out-dir ~/mkit-evals/m1-round1/report
```

`prepare BENCHMARK --out DIR` creates a single workspace for a manual trial.

### Context given to the agent

The workspace contains the skeleton crate, `TASK.md`, `CONTRACT.md`, `fixture/`, and the book:

- `--book-format text` (default): `book/book.md`, one llms-style file with every `{{#include}}`
  resolved, plus a `book/llms.txt` chapter index. `--parts 1-4` limits it to Parts I–IV (plus the
  introduction and appendices). Parts I–IV are about 110k tokens, half of which is the API inventory
  and concept index appendices; the whole book is about 230k.
- `--book-format html`: the built mdBook site (`mdbook build book`), all parts.

The MCP config (`mcp.json`, outside the workspace) starts `target/debug/mkit-inspector`, which
`run` builds first unless `--no-build-inspector` is given. Cargo runs offline
(`CARGO_NET_OFFLINE=true`) against a copy of the repository's `Cargo.lock`.

### Agent command templates

An agent config is JSON with `name`, `command`, `token_parser` (`claude-stream-json`,
`codex-jsonl`, or `none`), optional `default_model`, and `env`. `--agent-command` takes a template
directly. The command runs through the shell with the workspace as its working directory; stdout
is saved as the transcript and stderr as `agent-stderr.log`. Placeholders are shell-quoted:

| Placeholder | Value |
|---|---|
| `{workspace}` | The agent workspace |
| `{prompt_file}` | `prompt.md` (outside the workspace) |
| `{mcp_config}` | MCP server config JSON for the inspector |
| `{inspector}`, `{inspector_toml}` | Inspector binary path, raw or as a TOML string |
| `{transcript}` | Transcript path |
| `{model}` | `--model`, or the config's `default_model` |
| `{benchmark}`, `{attempt_dir}`, `{book_path}` | Benchmark id, attempt directory, book entry file |

`agents/claude-code.json` and `agents/codex.json` are starting points; their flags have not been
exercised in a paid run. The runner is not a sandbox: the template must keep the agent inside the
workspace and away from the web and the cargo registry (which holds GPUI source). The isolation
audit catches copied hidden files, `.git` directories, vendored GPUI source, and path dependencies
that leave the workspace, and fails the attempt if it finds any. User-level agent configuration
(for example a global `CLAUDE.md`) also reaches the agent; run evaluations under a clean profile.

## Attempt record (`attempt.json`, schema version 1)

| Field | Meaning |
|---|---|
| `schema_version`, `run_id`, `attempt_id` | Format version, run, and `benchmark#n` |
| `benchmark` | `id`, `number`, `title`, `milestone` |
| `agent` | `name`, `model`, `command_template`, rendered `command` |
| `context` | `book_format`, `book_parts`, `book_path`, `book_digest` (sha256), `book_commit`, `mcp_servers`, `mcp_config` |
| `workspace`, `transcript_path`, `stderr_path` | Where the attempt's files are |
| `started_at`, `finished_at` | UTC timestamps |
| `wall_time_seconds`, `agent_exit_code`, `agent_timed_out` | Agent process result |
| `tokens` | `input`, `output`, `cache_read`, `cache_creation`, `total`, and when available `cost_usd`, `turns`; `null` without a parser |
| `isolation` | Audit findings `before` and `after` the agent ran |
| `harness` | Grading result: `status` (`passed`, `passed_with_skips`, `failed`), per-criterion `criteria` with `status` and `detail`, counts, `first_failure`, `grading_dir`, exit codes; `null` for dry runs and `--skip-grade` |
| `success` | `true`, `false`, or `null` when not graded |
| `point_of_failure` | `agent_timeout`, `isolation`, `build`, or the first failing criterion id |
| `status` | `completed` or `dry_run` |
| `triage` | `classification` (E3.3: missing concept, unclear explanation, wrong example, GPUI bug or gap) and `notes`, filled in by a maintainer |

`report` reads any number of run directories or attempt files and writes `summary.json` and
`summary.md`: per-benchmark attempts, success rate, median wall time, mean tokens, cost, points of
failure, and skipped checks, plus the M1 and 1.0 gates. A gate is met when every benchmark in it has
attempts and the pooled success rate is at least the threshold (0.9 by default); the lowest
per-benchmark rate is reported alongside. Dry runs and ungraded attempts are not scored.

## Cost and tokens

Plan §9 budgets 80–150M tokens for evaluations and triage rounds. An attempt re-reads the book and
its own code many times, so most tokens are cached reads; expect roughly 1–5M tokens per attempt
(an estimate until measured). One M1 round of 6 apps × 3 attempts is therefore about 20–90M tokens,
so start with one attempt per app and a single model, measure, then scale. Use `--timeout` to cap
runaway attempts (default one hour).

## Checks

```sh
python3 evals/runner/mkit_eval.py check
python3 -m unittest discover -s evals/runner -p 'test_*.py'
```

Both are offline and take a few seconds; CI runs them. Grading needs a GPUI build and is not run in
CI; the reference grade above is the end-to-end check.
