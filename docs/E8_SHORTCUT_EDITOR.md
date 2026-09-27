# E8.13 — Shortcut editor

The shortcut editor is an `Entity<ShortcutEditor>` view for settings surfaces that need to inspect and rebind an application's action map. Each row shows an action label, optional category, and assigned chord. Enter starts capture; Escape cancels; an already-used chord announces the conflicting action and Enter moves that chord to the selected action. Delete clears the selected action. Up and Down move through rows.

`ShortcutEditor::new` owns a draft map and emits `BindingChanged` after edits. `ShortcutEditor::controlled` leaves the supplied map authoritative and emits `BindingChangeRequested` with the proposed full map; the owner applies it with `set_bindings`. Both modes emit typed selection, capture, conflict, and save events. `save_to_path` writes version 1 JSON (`{"version":1,"bindings":{"action-id":"cmd-k"}}`) and reports success or I/O failure through events and status text.

The key context is `MkitShortcutEditor`. Install `default_key_bindings()` in the app's keymap so applications can rebind row navigation, capture, and clear actions. Captured chords serialize GPUI modifier fields to parseable strings such as `cmd-o`; GPUI's display glyphs are not written to keymaps. Conflict comparison is case-insensitive and trims surrounding whitespace. The E8 example crate includes a compiling host adapter that maps the saved stable action IDs to statically typed GPUI actions; unknown IDs are ignored by that host adapter.

The component reads colors, typography, spacing, radius, and border values from `mkit_core::theme::Theme`. Its root exposes grid semantics and its rows expose action label, chord, and selection. The status row announces capture and save feedback; conflict status is announced separately.

## Verification status

- `cargo check -p mkit-registry-shortcut-editor` passes.
- Five tests pass: four data tests cover normalized conflict lookup, stable escaped JSON output, and clearing an assignment; a GPUI keyboard script captures a conflicting Command chord, resolves it, and verifies the saved keymap file.
- The harness captures six states (viewing, capturing, conflict, dirty, saved, save error) in light, dark, and high-contrast at 1× and 2×: 36 screenshots. Capturing/conflict/dirty/saved/error cases dispatch real GPUI key chords and save calls. Candidates were inspected before baselines were written; the matrix compares all cases. Capture requires macOS Metal.
- The example host adapter compiles and has a focused test confirming it maps only known action IDs. Native platform accessibility snapshots and manual keymap installation review remain pending.
- Public API, keymap format, conflict policy, keyboard contract, and accessibility contract require maintainer review per the component agreement.
