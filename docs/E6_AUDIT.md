# E6 progress audit — 2026-09-25

## Current state

- **E6.1 registry format: complete.** The checked-in
  [catalog](../registry/registry.json) and [schema](../registry/registry.schema.json)
  record release and component versions, lifecycle status, source files, specs,
  conformance manifests, required crates, and registry component dependencies.
  `scripts/check_registry.py` validates the catalog against the workspace and
  referenced files. The repository serves this catalog directly; there is no
  registry server or published component feed. The CLI rejects duplicate
  component names and component dependencies when loading the catalog; during
  dependency resolution it rejects cycles and duplicate source paths.
- **E6.2 `cargo mkit add`: implemented, fixture verified.** Its command is
  `cargo mkit add <component> [--path <repo-root>]
  [--dest <project-relative-dir>]`, with `src/ui/` as the destination default.
  Run it from the target Cargo project. Without `--path`, the CLI searches the
  current directory and its ancestors for `registry/registry.json`; `--path`
  selects another registry repository root. `--dest` is relative to the target
  project. It copies source, resolves component dependencies, adds required
  crates to `Cargo.toml`, and writes `mkit.toml` with component origin, version,
  and paths and hashes for exact upstream source snapshots in `.mkit/base/`.
  Those snapshots form the base for later comparison and merging. Installation
  rejects entries that are not `source_ready`, and a repeated `add` for an
  already recorded component does not overwrite the project's copy. The CLI
  copies source but does not add Rust module declarations to the target crate;
  the app must wire them in.
- **E6.3 `cargo mkit diff`: implemented.**
  `cargo mkit diff [component] [--path <repo-root>]` reads `mkit.toml` and the
  exact base snapshots. It shows local and current upstream changes separately,
  recorded and current versions, added upstream files, and local or upstream
  deletions. With no component argument, it covers all locked components. It
  does not change project files.
- **E6.4 `cargo mkit update`: implemented in the CLI, fixture verified.**
  `cargo mkit update [component] [--path <repo-root>] [--check]` uses the
  recorded base, local copy, and current upstream source. It merges
  nonoverlapping line edits automatically and writes standard conflict markers
  for overlapping edits. Normal mode advances the recorded base and version to
  current upstream even when a conflict needs manual resolution. It then exits
  nonzero if conflicts were written, so resolve the markers in the installed
  source before building. `--check` reads only and exits nonzero when updates
  are available. CLI fixtures cover this behavior; the current product catalog
  has no `source_ready` component for an end-to-end release update. For
  components being updated, normal mode checks every current registry crate
  requirement and adds missing dependencies to `Cargo.toml` in the same
  transaction as the source and lockfile. An incompatible existing project or
  inherited workspace package or version stops the update before any file
  changes; the developer must resolve the manifest declaration and rerun. The
  CLI does not silently rewrite application or workspace version choices.
- **E6.5 `cargo mkit doctor`: implemented.**
  `cargo mkit doctor [--path <repo-root>]` checks `gpui-pre` and `mkit-core`
  package identity and declared versions against requirements confirmed from
  the recorded component versions and current registry. It reports the known
  GPUI Kit coexistence pair (`gpui-kit` 0.6.4 and `gpui-pre` 0.3.5). Unknown
  compatibility and other warnings cause a nonzero exit.
- **E6.6 `mkit` crate: implemented for the current catalog.** The crate mirrors
  all 33 registry sources through `scripts/sync_mkit_components.py`: 32 E7
  everyday component drafts and the E8 scrubbable number field pilot. Each
  has an independent feature flag, and all are enabled by default. The sync
  check now also verifies that these features and the public generated-module
  exports cover the catalog exactly once. The existing combobox and
  scrubbable number field conformance targets are feature-gated. Mirroring a
  draft does not make its API approved or its source installable.

E6.1–E6.6 are implemented for the current catalog, with CLI workflows checked
against fixtures. All 33 catalog entries are currently
`implementation_in_progress`; none can be installed by `cargo mkit add`.
An end-to-end release workflow with a real component remains blocked until a
component reaches `source_ready` and receives maintainer review. See the
[book page](../book/src/components/source-registry.md) for the developer-facing
command syntax and current availability.

## Checks and review

`python3 scripts/check_book.py --skip-cargo` passed for this refresh: 96 book
pages and 14 example crates. It checks book structure and links without
compiling example crates. Build, test, lint, rendered-book, and
coexistence results should be recorded separately when rerun against this
expanded catalog; earlier E6 results covered only the two pilots.

The CLI syntax, `mkit.toml` format, component naming, registry component
contracts, and book accuracy need maintainer review under `AGENTS.md`. The
pilot specs, keyboard and accessibility contracts, and eventual visual
baselines also need that review. The current drafts have not received it.
