# Hidden evaluation material

Files here must never reach an evaluation agent (plan E10.3 and §10.1: reference solutions are
hidden from evaluation agents).

- `reference/01-counter/` is a minimal reference solution for benchmark 01. It exists to prove the
  acceptance tests and grading pipeline work end to end:
  `python3 evals/runner/mkit_eval.py grade 01-counter --candidate evals/hidden/reference/01-counter --out <dir outside the repo>`.
  Reference solutions for benchmarks 02–10 belong to E10.3 (book companion apps).

How the runner keeps this directory away from agents:

1. `prepare_workspace` copies only the skeleton, `TASK.md` (the agent part of the spec),
   `CONTRACT.md`, the fixture, and the book export into a workspace. Nothing under
   `evals/hidden/`, `evals/grading/`, or a benchmark's `acceptance.rs` / `screenshots.rs` is copied.
2. Runs must live outside the repository (`ensure_outside_repo`), so an agent working in its
   workspace cannot walk up into this directory.
3. `audit_workspace` hashes every workspace file before and after the agent runs and fails the
   attempt (`point_of_failure: "isolation"`) if any file matches a file in this directory or in the
   grading sources.

The runner cannot stop an agent that is allowed to read arbitrary paths. The agent command template
must restrict file access to the workspace (see `evals/agents/`).
