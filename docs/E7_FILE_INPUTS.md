# E7.19 FileDropZone and FileField evidence

The two stateful components accept filesystem paths and emit typed selection proposals. Neither reads
file contents. Both use GPUI's pinned `gpui-pre 0.3.5` platform APIs rather than pretending a
synthetic drag fixture proves OS integration.

## Platform API findings

The pinned GPUI API includes `FileDropEvent`, which translates OS file drag paths to an
`ExternalPaths` drag payload and dispatches it to element `on_drop` handlers. The element `drag_over`
style callback can inspect that payload for accepted/rejected preview styling. `App::prompt_for_paths`
opens the platform file picker and returns an asynchronous oneshot channel; the test platform can
simulate selection/cancel results. `PathPromptOptions` supports files and `multiple`, but has no
extension or MIME filter field. Components therefore filter returned and dropped paths by suffix
(`.pdf`, `.txt`) or a small common suffix map for `image/*`, `audio/*`, and `video/*`. This is UI
filtering only; it does not inspect file content or provide a security boundary.

## Ownership and event contract

Omitting `.files(...)` selects uncontrolled state. Supplying `.files(paths)` selects controlled state;
parents apply the complete `FilesChange` proposal with `set_files`. `FileDropZone` includes rejected
paths in `FilesChange`; `FileField` emits one typed `FileRejected` per rejected path. With `multiple: false`, only the first accepted path is proposed and additional paths are rejected. Both emit
`BrowseRequested` before asking GPUI to display the native picker. Picker cancellation does not change
selection. Both components expose named Browse and per-file Remove buttons and keep keyboard focus
moving to a neighboring Remove action or Browse after uncontrolled removal.

## Evidence

- `cargo test -p mkit-registry-file-drop-zone -p mkit-registry-file-field --offline -j 2`: 8 tests
  passed (per-package tests cover suffix rules, native-picker requests from Enter, accepted/rejected
  selection, controlled ownership/removal, keyboard removal, and byte-size formatting).
- `cargo clippy -p mkit-registry-file-drop-zone -p mkit-registry-file-field --offline -j 2 -- -D warnings`:
  passed.
- `cargo test -p mkit-gallery --test e7_file_inputs_matrix --offline -j 2 -- --nocapture`: 42 supported
  screenshots across 3 declared states for FileDropZone and 4 states for FileField, × 3 themes × 2
  scales. Candidate baselines were inspected at 1× in all states/themes and at 2× in a representative
  multi-file case; three file rows fit after raising the fixture height to 360 logical pixels.
- The generated manifests contain 60 screenshot cases total. 42 are rendered and compared. 12
  FileDropZone accepted/rejected drag hover cases are explicitly marked unsupported in headless mode
  because they require an OS-owned drag session. Six FileField invalid-result images are skipped
  because there is no real picker result or public validation-state fixture; no fake OS interaction
  is claimed.
- Native OS drag enter/leave/drop, native picker UI/filters/cancellation, and native screen-reader
  output remain pending on macOS, Windows, and Linux. The GPUI test validates that Enter reaches the
  actual picker API on its test backend, not that an OS dialog was opened.

The spec and generated adapter manifests are `registry/file-drop-zone/spec.md`,
`registry/file-drop-zone/tests/conformance.json`, `registry/file-field/spec.md`, and
`registry/file-field/tests/conformance.json`. Public selection model, suffix filters, and platform
fallback behavior need maintainer review.
