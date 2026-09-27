# Aria2 Manager consumer sample

This standalone GPUI app recreates the [Aria2 Download Manager handoff](../../xdl/design_handoff_aria2_download_manager/README.md) with mock data. It is a consumer of the public `mkit` crate through a path dependency and a separate Cargo workspace. It does not connect to aria2, start downloads, change files, or install a browser extension.

On macOS, run it from the repository root:

```sh
cargo run --manifest-path apps/aria2-sample/Cargo.toml --locked
```

The sample uses mkit `Button`, `TextField`, `Dialog`, `Progress`, `DataTable`, `Select`, and `Slider`, plus the core `Theme` global. IBM Plex font files are bundled under `assets/fonts/` with their OFL license. The download grid is an mkit DataTable with visual cell renderers and plain-text accessibility equivalents. The scheduler cells and captured-link rows use GPUI layout primitives.

## Try the flows

- Select a download, then use **Resume**, **Pause**, **Remove**, or **Enter** for detail. **Up/Down** moves selection; **Escape** returns to downloads. **Cmd/Ctrl+N** opens Add URL.
- Filter by filename or host. Add a URL to create a queued row in local sample state.
- Switch the detail between connection lanes, block map, and ribbon. Click scheduler cells to cycle their local state.
- In Settings, edit the mkit text fields and apply values locally, or switch between light, dark, and paper themes. In Capture, select links and see the button count update.

Add URL uses mkit controls for the save location, queue, connection count, and advanced split options. Their values are applied to a local mock download; split counts outside 1–32 are rejected. The speed limiter and RPC restart are still mock controls. The RPC token uses `TextField::secure(true)` and is never sent to a server. The sample uses an explicit “Sample data” status pill so it does not imply an RPC connection.

## Verification

```sh
cargo build --manifest-path apps/aria2-sample/Cargo.toml --locked
cargo test --manifest-path apps/aria2-sample/Cargo.toml --locked
cargo clippy --manifest-path apps/aria2-sample/Cargo.toml --all-targets --locked -- -D warnings
```

The screenshot harness compares 36 baselines: six scenes, three themes, and 1×/2× scale. On macOS Metal it can write candidates before baseline updates with `ARIA2_CAPTURE_DIR=/tmp/mkit-aria2-candidates cargo test --manifest-path apps/aria2-sample/Cargo.toml --test screenshots --locked`. The interaction tests exercise pointer selection, keyboard navigation, category filtering, and capture selection.

Verification on 2026-09-26 after the table and form migration: the standalone build, 5 sample unit tests, 3 interaction tests, all 36 screenshot comparisons, formatting, and sample Clippy with `-D warnings` passed. Focused registry suites passed 5 DataTable, 7 Dialog, and 9 TextField tests; the DataTable generated keyboard adapter passed 9/9 cases. The secure TextField screenshot matrix passed 36/36; its six new baselines were inspected. The real same-window coexistence test passed. `mdbook build book` and `python3 scripts/check_book.py --built-html` passed (146 pages, 15 example crates), as did component source sync, spec, and registry checks. The running native app yielded AX snapshots for the main list, Add URL, and Settings, checked by `tests/check_ax_snapshots.py`. A fresh external source-vend project passed `cargo check --offline` and `cargo-mkit doctor` using a temporary release-candidate registry entry. Cargo reports a future-compatibility warning in transitive `block 0.1.6`.

GPUI 0.3.5 does not activate accessibility in headless test windows. The interaction test asserts `AccessibilityError::Inactive`. A native macOS capture of the running sample is saved under `tests/ax-snapshots/` for the main list, Add URL dialog, and Settings. `python3 apps/aria2-sample/tests/check_ax_snapshots.py` verifies the table, dialog, form roles, secure token subrole, and that the modal background is absent from the AX tree. These are native accessibility snapshots, but a human screen-reader walkthrough remains required.

## Release findings

The realistic consumer surfaced four release gates:

1. **Collection composition:** the sample now uses mkit DataTable for row selection, keyboard navigation, sorting hooks, and visual cells. `DataRow.cells` supplies accessible equivalents. The new renderer and activation APIs need maintainer approval.
2. **Forms and modal focus:** Add URL now uses mkit Select, Slider, and TextField controls; the token field is masked and uses the native secure text-field subrole. The dialog now traverses its rendered tab stops and the host hides background AX content while it is open. Maintainer review of these contracts is pending.
3. **Accessibility evidence:** native macOS trees were captured for three sample scenes and checked by script. Headless AccessKit capture remains inactive in GPUI 0.3.5; a human screen-reader walkthrough and state-by-state matrix remain open.
4. **Distribution:** a fresh external Cargo project successfully vended `Button` from a temporary release-candidate registry entry and passed `cargo check --offline` plus `cargo-mkit doctor`. Production registry entries remain `implementation_in_progress` until API, spec, visual, and book review. No crate has been published.

This sample proves a separate application can compile against mkit and expose a realistic native accessibility tree. It is still a release candidate until the remaining reviews and accessibility checks pass.
