---
spec_version: 1
component: shortcut-editor
states:
  - id: viewing
    description: Current action bindings are listed and none is being edited.
    fixture: viewing_fixture
    screenshot_status: captured_by_shortcut_editor_matrix
  - id: capturing
    description: One action is selected and the next non-cancel key chord is captured.
    fixture: capturing_fixture
    screenshot_status: captured_by_shortcut_editor_matrix
  - id: conflict
    description: The captured chord is already assigned to another action and needs resolution.
    fixture: conflict_fixture
    screenshot_status: captured_by_shortcut_editor_matrix
  - id: dirty
    description: Bindings differ from the loaded keymap and can be saved.
    fixture: dirty_fixture
    screenshot_status: captured_by_shortcut_editor_matrix
  - id: saved
    description: The current bindings were serialized successfully to the requested keymap path.
    fixture: saved_fixture
    screenshot_status: captured_by_shortcut_editor_matrix
  - id: save_error
    description: Saving failed and the unsaved bindings remain available for retry.
    fixture: save_error_fixture
    screenshot_status: captured_by_shortcut_editor_matrix
keys:
  - key: ArrowDown
    modifiers: []
    when: editor is focused
    action: Select the next binding row, wrapping at the end.
    initial_state: viewing
    expect:
      state: viewing
  - key: ArrowUp
    modifiers: []
    when: editor is focused
    action: Select the previous binding row, wrapping at the start.
    initial_state: viewing
    expect:
      state: viewing
  - key: Enter
    modifiers: []
    when: a binding row is selected
    action: Begin capturing a replacement chord.
    initial_state: viewing
    expect:
      state: capturing
  - key: Escape
    modifiers: []
    when: capturing or a conflict is pending
    action: Cancel capture and keep the existing binding.
    initial_state: capturing
    expect:
      state: viewing
  - key: Escape
    modifiers: []
    when: capturing or a conflict is pending
    action: Cancel capture and keep the existing binding.
    initial_state: conflict
    expect:
      state: viewing
  - key: Enter
    modifiers: []
    when: a conflicting chord is pending
    action: Move the chord from the conflicting action to the selected action.
    initial_state: conflict
    expect:
      state: dirty
  - key: Delete
    modifiers: []
    when: a binding row is selected and not capturing
    action: Clear the selected binding.
    initial_state: viewing
    expect:
      state: dirty
accessibility:
  role: grid
  properties:
    - name: name
      value: Keyboard shortcuts
    - name: row
      value: Each action row exposes its label, current chord, and conflict status.
    - name: selected
      value: The selected row is exposed as selected.
    - name: live-status
      value: Capture, conflict, save success, and save errors are announced as status text.
controlled: Owner supplies the actions and binding map. In controlled mode edits emit BindingChangeRequested and remain pending until the owner calls set_bindings; uncontrolled mode updates the local binding map before emitting BindingChanged. Save serializes the current effective map and never writes during a binding change.
events: [SelectionChanged, CaptureStarted, BindingChangeRequested, BindingChanged, ConflictDetected, SaveRequested, Saved, SaveFailed]
theme_tokens: [surface, elevated_surface, text, text_muted, border, focus, danger, spacing.small, spacing.medium, spacing.large, radii.small, borders.hairline, borders.strong, typography.caption]
open_questions: [Confirm the default on-conflict policy and keymap file format before stabilizing the public API.]
---

# Shortcut editor

## Purpose

Let users inspect and change an application's action-to-key-chord map in a settings surface.

## Anatomy

- A focused list with one row per action, optional category, action label, and current chord.
- A status message for capture, conflict, save success, and save failure.
- `viewing`, `capturing`, `conflict`, `dirty`, `saved`, and `save_error` states.

## States

- `viewing`: action rows are shown and no capture is active.
- `capturing`: the editor waits for one chord; Escape cancels without changing the binding.
- `conflict`: the captured chord belongs to another action; Enter moves it to the selected action and clears it from the previous action.
- `dirty`: the effective binding map differs from its loaded value and may be saved.
- `saved`: writing the map to the requested file succeeded.
- `save_error`: writing failed; the draft remains available for retry.

Bindings use GPUI keymap strings such as `cmd-k` or `ctrl-shift-p`. Capture serializes GPUI modifier fields to these parseable names rather than display glyphs. Comparison is case-insensitive after surrounding whitespace is trimmed. JSON output uses version 1 and an action-to-chord object. The selected row is independent from focus, which remains on the editor while keys are navigated.

## Props and events

Each action has a unique stable ID, label, and optional category. The host supplies the action list and binding map. In controlled mode the component emits `BindingChangeRequested` containing the target ID, chord, and proposed full map; it changes displayed bindings only after the host calls `set_bindings`. In uncontrolled mode it applies the local map change and emits `BindingChanged`. A confirmed conflict clears the former owner of the chord. `save_to_path` serializes the currently effective map and emits `SaveRequested`, then `Saved` or `SaveFailed`. Saving does not happen as a side effect of editing.

## Keyboard map

Register named actions under `MkitShortcutEditor` so hosts can rebind row navigation, capture/resolve, and clear. Up and Down wrap through rows. Enter begins capture; during a conflict the same Enter action confirms moving the chord. Escape cancels capture. Delete clears the selected chord. The next non-Escape keystroke becomes the replacement chord. The key context provides app-level rebinding, while the editor's capture handler reads the raw GPUI keystroke.

## Pointer behaviour

Clicking a row selects it and starts capture for that row. Clicking does not change a binding until a chord is captured. There are no pointer-only save controls: the settings owner chooses a path and calls `save_to_path`.

## Accessibility role and properties

The root uses grid role and the accessible name “Keyboard shortcuts”. Each row exposes an accessible name containing the action label and current chord, plus selected state and conflict text. Capture, conflict, save, and error feedback is exposed as status content. The component retains focus while selection moves; a conflict is named in the status content as well as on its row.

## Theme tokens used

Read colors, typography, spacing, radii, and borders from the GPUI `Theme` global. This draft uses surface/elevated surface, text/text-muted, border, a focus-coloured selection edge, danger, caption typography, small/medium spacing, small radius, and hairline border tokens.

## WAI-ARIA pattern reference

The list uses the [WAI-ARIA Grid Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/grid/) for named rows and selected-row state. This pilot keeps DOM-like focus on the grid container and does not implement cell navigation or a complete grid-cell model; assistive technology behavior needs platform snapshot review.

## Platform notes

Capture is based on GPUI's normalized `Keystroke` display string, so app keymaps should use GPUI's platform conventions (`cmd` maps to the platform modifier). Escape remains reserved for cancelling capture. The saved JSON is an mkit data format; the host remains responsible for loading it and installing GPUI key bindings.

## Open questions

- Human review is required for public API and naming, the conflict-resolution contract, keymap format, keyboard behavior, accessibility contract, and visual baseline.
- Confirm whether chords may contain multiple keystrokes or only one physical chord in the first version.
