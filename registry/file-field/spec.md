---
spec_version: 1
component: file-field
states:
  - id: empty
    description: No files are selected and the field offers Browse.
    fixture: empty_fixture
  - id: selected
    description: Selected file names and sizes are shown with remove actions.
    fixture: selected_fixture
  - id: multiple
    description: Multiple selected files are shown when multiple selection is enabled.
    fixture: multiple_fixture
  - id: disabled
    description: Browse and remove actions are disabled.
    fixture: disabled_fixture
  - id: invalid
    description: Rejected files are reported while accepted files remain selected.
    fixture: invalid_fixture
    screenshot_status: unsupported_headless_capture
    screenshot_limitation: Requires a real picker result or a dedicated public validation-state API; the screenshot matrix does not fabricate picker behavior.
keys:
  - key: Enter
    modifiers: []
    when: The Browse button is focused and enabled.
    action: Open the platform file picker.
    initial_state: empty
    expect: { event: browse_requested, focus_target: same_button }
  - key: Space
    modifiers: []
    when: The Browse button is focused and enabled.
    action: Open the platform file picker.
    initial_state: empty
    expect: { event: browse_requested, focus_target: same_button }
  - key: Enter
    modifiers: []
    when: A focused Remove file button is enabled.
    action: Remove that file from the selection proposal.
    initial_state: selected
    expect: { event: files_change, focus_target: browse_or_next_remove }
  - key: Space
    modifiers: []
    when: A focused Remove file button is enabled.
    action: Remove that file from the selection proposal.
    initial_state: selected
    expect: { event: files_change, focus_target: browse_or_next_remove }
accessibility:
  role: group
  properties:
    - name: name
      value: caller-provided field label
    - name: description
      value: accepted file types and selection limit
    - name: disabled
      value: true
      when: disabled
    - name: live
      value: polite for validation and selection updates
---

# FileField

## Purpose

Represent a file selection as a labeled form field, with a platform Browse button and removable selected-file rows. It may be used independently or alongside `FileDropZone`.

## Anatomy

The field contains a label, accepted-type hint, Browse button, zero or more file rows, and a polite validation/status message. Each file row contains a name, optional size, and a Remove button.

## States

`empty` has no rows. `selected` contains one accepted path. `multiple` contains several accepted paths. `invalid` announces rejected paths while preserving accepted selection. `disabled` disables Browse and row removal.

## Props and events

The stateful `FileField` accepts label, `accept` suffix filters, `multiple`, `disabled`, and optional `files`. Omitting `.files(...)` selects uncontrolled mode and mutates internal selection; `.default_files(paths)` seeds its initial value. `.files(paths)` selects controlled mode: `FilesChange` emits a proposed complete selection and the component waits for the parent to call `set_files`. A second typed event, `BrowseRequested`, reports that the user asked to browse; the component invokes GPUI's platform picker and reports its result as `FilesChange`. `FileRejected` reports the rejected paths and reason. Canceling the picker changes no files. Each item carries a path, display name, and optional caller-provided size; file contents are never opened by the component.

## Keyboard map

Tab traverses Browse followed by enabled Remove buttons in row order. Enter and Space activate Browse; Enter and Space activate a focused Remove button. After removing a row, focus moves to the next row's Remove button, otherwise the previous row, otherwise Browse. Escape has no field-specific behavior.

## Pointer behaviour

Browse opens the platform path picker. Remove actions remove one path and emit a full selection proposal. In controlled mode the visual selection remains parent-owned until new `files` are supplied. File type acceptance is suffix based; the field does not inspect file contents or trust extensions as a security boundary.

## Accessibility role and properties

The field is a labeled Group. Browse and Remove are ordinary named buttons, with Remove names containing the file name. The accepted-type hint and selection limit are associated description text. Rejections and changes are exposed in a polite status region. Disabled state applies to Browse and all Remove buttons. Native screen-reader output must be verified on supported platforms.

## Theme tokens used

Use `Theme.colors.surface`, `border`, `text`, `text_muted`, `accent`, `danger`, and `disabled`, plus spacing, typography, radii, and control-height tokens. Rejected status uses both text/icon and danger styling; color is not the only signal.

## WAI-ARIA pattern reference

Use standard labeled-group, button, and polite-status semantics. File rows are not a listbox and do not claim selection-widget semantics.

## Platform notes

The pinned GPUI `0.3.5` API exposes `App::prompt_for_paths(PathPromptOptions)` with files/multiple configuration and an asynchronous oneshot result. It does not expose extension filters in the picker options, so accepted suffixes are checked after selection. Headless tests may simulate picker completion; a native OS picker should not be invoked by screenshot fixtures. Native assistive-technology behavior remains a platform check.

## Open questions

- Review file metadata exposed publicly and whether selection limits belong on this component.
- Decide whether future fields need an async loading state for file reads; this draft only presents paths.
- Verify focus restoration following removal and polite announcement timing with native assistive technology.
