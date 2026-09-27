# Own component source from the registry

The component registry in `registry/` is a catalog checked into this
repository. A `source_ready` entry can supply files that your application
owns and can edit. The source CLI is implemented and checked with fixtures.
The catalog currently has 33 entries: 32 E7 everyday component drafts and the
E8 scrubbable number field pilot. Every entry is
`implementation_in_progress`, so none can be installed yet.

## The add command

Run the command from the target Cargo project's root:

```sh
cargo mkit add <component> [--path <repo-root>] [--dest <project-relative-dir>]
```

The CLI looks for `registry/registry.json` in the current directory and its
ancestors. Pass `--path <repo-root>` if the registry is elsewhere. The default
destination is `src/ui/`; `--dest` is relative to the current target project.
When a component becomes `source_ready`, `add` copies its source,
resolves its registry component dependencies, and adds its required crates to
the project's `Cargo.toml`. It records the installed component's
origin and version in `mkit.toml`, along with paths and hashes for exact copies
of the upstream source files in `.mkit/base/`. A repeated `add` for a component
already recorded in `mkit.toml` is rejected; it does not overwrite your copy.

`add` does not add Rust module declarations to your application. For a
single-file registry source named `src/lib.rs`, the default destination is
`src/ui/<module_name>.rs`. Hyphens become underscores, so
`scrubbable-number-field` would become `src/ui/scrubbable_number_field.rs` once
it is source-ready. Declare `mod ui;` in your crate root, then declare
`pub mod <module_name>;` in `src/ui.rs` (or use equivalent inline modules) so
Rust compiles the copied file. Multi-file sources are copied under
`src/ui/<module_name>/...`.

`add` rejects every current catalog entry because none is `source_ready`.
The specs describe intended contracts; component implementation and
conformance work are still in progress. The source mirror in the `mkit` crate
does not change registry status.

## After installation

Use `cargo mkit diff [component] [--path <repo-root>]` to inspect an installed
component. With no component name, it reports all entries in `mkit.toml`. The
command compares your files and the current registry files separately against
the exact recorded base. It shows the recorded and current versions, changed
content, added upstream files, and files deleted locally or upstream. It is
read-only; it does not merge or rewrite your source.

Use `cargo mkit doctor [--path <repo-root>]` to check the target project's
dependency declarations. It checks package identity and versions for
`gpui-pre` and `mkit-core` against requirements it can confirm for installed
components. It also reports whether GPUI Kit matches the repository's known
coexistence pair, `gpui-kit` 0.6.4 with `gpui-pre` 0.3.5. If the registry
version cannot confirm a recorded component's requirements, doctor warns that
compatibility is unknown. Warnings make the command exit nonzero.

Use `cargo mkit update [component] [--path <repo-root>] [--check]` when
upstream has changed. `--check` reads files without changing them and
exits nonzero when an update is available. In normal mode, the command makes a
line-based three-way merge of the recorded base, your copy, and current
upstream. Nonoverlapping edits merge automatically. Overlapping edits produce
standard conflict markers in the installed source. The command advances the
recorded base and component version to current upstream even if it writes
conflicts, then exits nonzero. Resolve those markers manually before building.
The current catalog has no `source_ready` component, so this workflow has been
verified with fixtures, not with a released component. Maintainer review of
the component's public API and contract is also pending before release.

For components being updated, normal update checks every crate requirement in
the current registry. It adds missing dependencies to `Cargo.toml` in the same
transaction as the source and lockfile. If a project or inherited workspace
dependency has an incompatible package or version, update stops before changing
any files. Resolve the declaration in the project or workspace manifest, then
rerun update. `--check` reports available updates without editing files.

For exploration through a normal crate dependency, `mkit` mirrors all 33 draft
sources. Each registry component has its own feature flag; all are enabled by
default. The sync check verifies that the feature flags and public module
exports match the catalog. These drafts still require contract and public API
review before release.
