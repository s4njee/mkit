---
spec_version: 1
component: file-drop-zone
states:
  - id: idle
    description: The zone accepts files and offers a keyboard reachable Browse action.
    fixture: idle_fixture
  - id: drag-accepted
    description: A compatible OS file drag is over the zone.
    fixture: drag_accepted_fixture
    screenshot_status: unsupported_headless_capture
    screenshot_limitation: Requires an OS-owned file drag session; a synthetic fixture would not prove native drag integration.
  - id: drag-rejected
    description: An incompatible OS file drag is over the zone.
    fixture: drag_rejected_fixture
    screenshot_status: unsupported_headless_capture
    screenshot_limitation: Requires an OS-owned file drag session; a synthetic fixture would not prove native drag integration.
  - id: selected
    description: Accepted file paths are represented by the controlled or internal selection.
    fixture: selected_fixture
  - id: disabled
    description: The zone rejects drops and Browse activation while disabled.
    fixture: disabled_fixture
keys:
  - key: Enter
    modifiers: []
    when: The Browse button is focused and enabled.
    action: Open the platform file picker configured by the accept and multiple props.
    initial_state: idle
    expect: { event: browse_requested, focus_target: same_browse_button }
  - key: Space
    modifiers: []
    when: The Browse button is focused and enabled.
    action: Open the platform file picker configured by the accept and multiple props.
    initial_state: idle
    expect: { event: browse_requested, focus_target: same_browse_button }
  - key: Enter
    modifiers: []
    when: A focused enabled Remove file button is selected.
    action: Propose removal of that file.
    initial_state: selected
    expect: { event: files_change, focus_target: next_remove_or_browse }
  - key: Space
    modifiers: []
    when: A focused enabled Remove file button is selected.
    action: Propose removal of that file.
    initial_state: selected
    expect: { event: files_change, focus_target: next_remove_or_browse }
accessibility:
  role: group
  properties:
    - name: name
      value: caller-provided accessible label
    - name: description
      value: accepted file types and browse instruction
    - name: live
      value: polite for accepted/rejected feedback
    - name: disabled
      value: true
      when: disabled
---

# FileDropZone

## Purpose

Accept on-disk paths dragged from the operating system and provide a keyboard-accessible platform Browse action. This component does not read file contents; consumers own validation and loading.

## Anatomy

The zone contains an instruction/feedback region, a Browse button, and an optional selected-files presentation. The separate `FileField` component can render the same selected-file model with removable rows.

## States

`idle` is ready for files. `drag-accepted` and `drag-rejected` are based on real platform file paths and the configured extension/MIME-like suffix filter. `selected` presents current file paths. `disabled` makes the zone noninteractive. Programmatic path changes follow the same filtering rules as dropped or browsed paths.

## Props and events

The stateful `FileDropZone` accepts a label, optional description, `accept` filters (extensions such as `.png` and suffix/type tokens such as `image/*`), `multiple`, `disabled`, and an optional `files` value. Omitting `.files(...)` is uncontrolled and updates internal selected paths; `.default_files(paths)` can seed that uncontrolled selection. `.files(paths)` is controlled: the component emits a typed `FilesChange` proposal and waits for the parent to call `set_files` with the new paths. Rejected paths are included in the change event and are never silently added. When `multiple` is false, the first accepted path is proposed and further dropped/picker paths are rejected. Each selected item is a path and display name; actual file reads remain the caller's responsibility.

## Keyboard map

A labeled Group contains the Browse button followed by one Remove button per selected file. Tab reaches enabled controls in order; Enter and Space activate Browse or Remove through separate rebindable key contexts. Removing an item focuses the next row, previous row, or Browse. The Browse button opens GPUI's platform path picker. The picker supports file selection and the `multiple` setting. Escape belongs to the native picker. The component does not intercept unrelated shortcuts.

## Pointer behaviour

GPUI converts OS file drops to `ExternalPaths` and dispatches the typed drop payload to the target element. Drag-over styling is active only for compatible paths; incompatible paths show a reject cue and are reported, not selected. Leaving the target clears hover feedback. Clicking Browse opens `App::prompt_for_paths`; cancel leaves selection unchanged. No synthetic internal drag payload is treated as an OS file.

## Accessibility role and properties

The zone is a labeled Group, with a visible instruction and a native button named Browse files. Accepted/rejected feedback is exposed through a polite status announcement. Selection rows expose file names and removal controls are named buttons. Disabled zones remove Browse from activation and report disabled semantics. Headless tests can exercise component-produced state and button actions; actual OS drag gestures and native assistive-technology announcements require platform testing.

## Theme tokens used

Use `Theme.colors.surface`, `border`, `text`, `text_muted`, `accent`, `danger`, and `disabled`; spacing, control-size, radii, and typography tokens for layout. The accepted/rejected distinction includes text/icon feedback and does not rely on color alone.

## WAI-ARIA pattern reference

The group uses ordinary browse button semantics and a polite status region; there is no established APG composite pattern for desktop file drop targets. Follow platform file picker and drag/drop conventions.

## Platform notes

The pinned GPUI `0.3.5` API exposes `FileDropEvent` translated to `ExternalPaths`, element `on_drop`/`drag_over` handlers, and `App::prompt_for_paths(PathPromptOptions)`. The picker returns asynchronously through a oneshot receiver. GPUI headless tests simulate drops but do not represent OS integration; macOS/Windows/Linux native drag and picker paths need real-platform verification. GPUI's picker API does not expose extension filters in `PathPromptOptions`; this draft filters returned paths and dropped paths itself, so picker filtering is advisory/unavailable.

## Open questions

- Review public `accept` token syntax and whether MIME type detection beyond file suffixes belongs in this UI component or app code.
- Decide whether the component should accept dropped directories; this draft accepts files only.
- Native OS drag state, live announcement timing, and picker cancellation need platform integration review.
