# Component registry

`registry.json` is the checked-in catalog for component packages and drafts. The
registry is served directly from this repository under `registry/`; there is no
separate registry service or published package feed yet. Its `release_version`
and each component's `version` follow `[workspace.package].version` in the root
`Cargo.toml`. `registry_format_version` versions the catalog format itself.

Each component entry declares its spec and conformance manifest, source file
list, lifecycle status, version, required crate dependencies, and a separate
`component_dependencies` list for registry-to-registry dependencies. The two
current pilot drafts have no component dependencies. Dependencies are
versioned against the workspace declarations: `mkit-core` and `gpui-pre`
(currently `0.1.0` and `0.3.5`). The checked-in [JSON Schema](registry.schema.json)
documents the format. Run `python3 scripts/check_registry.py` to check registry
structure, workspace versions, required dependencies, and referenced files.
Focused validator checks are in `scripts/test_check_registry.py` and can be run
with `python3 -m unittest discover -s scripts -p 'test_check_registry.py'`.

`draft_spec_only` entries have no source. `implementation_in_progress` entries
may have source but are not installable. Only `source_ready` entries are intended
for installation and full conformance gating. The combobox and scrubbable
number field have partial implementations in progress; neither is source-ready
or installable.

`registry/` remains the source of truth. Both draft sources are mirrored into the
publishable `mkit` crate by `python3 scripts/sync_mkit_components.py`; CI runs
its `--check` mode so the compiled copy cannot drift. This mirror does not make
the component installable through `cargo mkit add`.

## Source installation contract

The command is `cargo mkit add <component> [--path <repo-root>]
[--dest <project-relative-dir>]`. Run it in the target Cargo project. Without
`--path`, the CLI searches the current directory and its ancestors for
`registry/registry.json`; pass `--path <repo-root>` when the registry is
elsewhere. `--dest` selects where the current project owns the copied source
(default `src/ui/`).
The CLI resolves `component_dependencies`, adds required crate dependencies
to the project's `Cargo.toml`, and writes `mkit.toml` with the component's origin,
version, and paths and hashes for exact upstream source snapshots in
`.mkit/base/`. Those snapshots are used by `diff` and three-way `update`.
An already recorded component is rejected on a repeated
`add`, so the command does not overwrite the project's copy.

`add` copies files but does not wire Rust modules into the app. For a
single-file component whose registry source is `src/lib.rs`, the default
installed path is `src/ui/<module_name>.rs`. The module name replaces hyphens
with underscores: `scrubbable-number-field` becomes
`src/ui/scrubbable_number_field.rs`. The app must declare `mod ui;` at its crate
root and `pub mod <module_name>;` in `src/ui.rs`, or use equivalent inline module
declarations. A multi-file component is copied under
`src/ui/<module_name>/...`.

Installation rejects any entry whose status is not `source_ready`. Both
current pilots are `implementation_in_progress`, so neither is a valid `add`
target. See the [book's source registry page](../book/src/components/source-registry.md)
for the user-facing status and command syntax.

`cargo mkit diff [component] [--path <repo-root>]` is read-only. It compares
installed files and current registry files separately against the recorded
base, reporting local and upstream edits and recorded versus current component
versions. It detects added upstream files and deletions of tracked local or
upstream files. Without a component argument, it reports all locked
components. `cargo mkit doctor [--path <repo-root>]` checks declared
package identity and compatible versions for `gpui-pre` and `mkit-core` when
registry requirements can be confirmed. It reports whether GPUI Kit matches
the known coexistence pair, `gpui-kit` 0.6.4 with `gpui-pre` 0.3.5. Unknown
compatibility produces a warning; warnings make doctor exit nonzero.

`cargo mkit update [component] [--path <repo-root>] [--check]` is implemented
and fixture verified. Its `--check` mode is read-only and exits nonzero when
updates are available. Normal mode merges nonoverlapping line edits and writes
standard conflict markers for overlaps. It advances the lock's base snapshots
and version to current upstream even if conflicts occur, then exits nonzero so
the developer can resolve markers manually. The current catalog has no
`source_ready` component for an end-to-end release update.

The E6.2–E6.5 CLI workflows are implemented and verified with fixtures,
including a transitive `add` followed by Cargo check, read-only `diff`, clean
and conflicting `update`, `update --check`, and `doctor`. The `mkit` crate has
per-component features for the current catalog. A real component release
workflow still awaits a `source_ready` entry and maintainer review of its
public API and contract.

For spec authoring and generated keyboard, accessibility, and screenshot cases,
see the [component spec template](../docs/component-spec-template.md),
[conformance generation](../docs/conformance-generation.md), and
`scripts/generate_conformance.py`. CI separately checks spec validity and
conformance manifest freshness with `scripts/check_component_specs.py`.
