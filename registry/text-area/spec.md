---
spec_version: 1
component: text-area
states:
  - id: empty
    description: Empty multi-line editor, optionally showing placeholder.
    fixture: empty_fixture
  - id: filled
    description: Editor contains editable text.
    fixture: filled_fixture
  - id: focused
    description: Editor owns keyboard focus and native selection/caret.
    fixture: focused_fixture
  - id: composing
    description: An IME marked-text range is present and can be committed or cancelled.
    fixture: composing_fixture
  - id: selected
    description: A non-empty range is selected, including reversed selections.
    fixture: selected_fixture
  - id: invalid
    description: Editor exposes invalid validation state and associated message.
    fixture: invalid_fixture
  - id: disabled
    description: Value is readable but editor cannot be focused or edited.
    fixture: disabled_fixture
keys:
  - key: Ctrl+A
    modifiers: [Ctrl]
    when: focused
    action: Select all text using the platform text-editing command.
    initial_state: filled
    expect:
      state: focused
      focus_target: input
  - key: Ctrl+Z
    modifiers: [Ctrl]
    when: focused and not composing
    action: Undo the latest edit when the fixture has undo history.
    initial_state: filled
    expect:
      state: filled
      event: input
      focus_target: input
  - key: Ctrl+Shift+Z
    modifiers: [Ctrl, Shift]
    when: focused and not composing
    action: Redo the latest undone edit when the fixture has redo history.
    initial_state: filled
    expect:
      state: filled
      event: input
      focus_target: input
  - key: Ctrl+C
    modifiers: [Ctrl]
    when: focused and enabled and selection is non-empty
    action: Copy the selected UTF-8 text to the GPUI system clipboard without changing the value.
    initial_state: selected
    expect:
      state: selected
      event: none
      focus_target: input
  - key: Ctrl+X
    modifiers: [Ctrl]
    when: focused and enabled and selection is non-empty
    action: Copy the selection then remove it and emit the resulting input snapshot.
    initial_state: selected
    expect:
      state: filled
      event: input
      focus_target: input
  - key: Ctrl+V
    modifiers: [Ctrl]
    when: focused and enabled and clipboard contains text
    action: Replace the selection/caret with clipboard text and emit the resulting input snapshot.
    initial_state: focused
    expect:
      state: filled
      event: input
      focus_target: input
  - key: Enter
    modifiers: []
    when: focused
    action: Insert a newline at the selection/caret; Enter does not submit a form.
    initial_state: focused
    expect:
      state: filled
      event: input
      focus_target: input
  - key: Ctrl+A
    modifiers: [Ctrl]
    when: disabled
    action: Do not change selection or emit an event.
    initial_state: disabled
    expect:
      state: disabled
      event: none
      focus_target: none
accessibility:
  role: textbox
  properties:
    - name: aria-multiline
      value: true
    - name: aria-value
      value: current editor text
    - name: aria-invalid
      value: true
      when: invalid
    - name: aria-disabled
      value: true
      when: disabled
---

# Text area

## Purpose
A draft editor for newline-delimited text. Text is shaped and painted with GPUI's multiline text layout, and the native input handler maps pointer and IME geometry to visual lines.

## Anatomy
One focusable text input with optional placeholder, description, and validation message. The caller supplies the accessible name through `with_label`.

## States
`empty`, `filled`, and `focused` describe editing/value presentation; `invalid` and `disabled` may overlay these states. Placeholder is visible only while empty. The caret is visible only while the enabled editor owns focus; an unfocused or disabled filled editor shows its value without a caret. A validation message sets a danger border and message and requests invalid accessibility state.

## Props and events
This is an `Entity<TextArea>` view. Construct with `TextArea::new(cx)` and configure `with_label`, `with_placeholder`, `with_description`, `with_validation_message`, and `disabled`. By default the entity owns its initial empty value (uncontrolled). `controlled(value)` initializes controlled mode with the owner's current value; `set_value(value, cx)` is the owner-to-view synchronization API in either mode. There is no `default_value` builder or `on_input` callback. Subscribe to the typed `InputChanged(String)` event.

Both modes apply a user edit to the local editing buffer immediately, then emit exactly one `InputChanged` snapshot containing the resulting full text. This ordering applies to text replacement, each IME marked-text update and commit, and successful undo/redo; selection, caret movement, unmarking, and programmatic `set_value` emit no event. The caller should treat the event as a change request and synchronize its chosen value with `set_value`. In controlled mode, that call acknowledges the request or replaces it with the caller's authoritative value. If the owner does not call `set_value`, the optimistic edit remains visible until a later owner update. Calling `set_value` with text equal to the current buffer preserves selection, marked text, and undo grouping, which permits owners to echo IME snapshots safely. A different value is authoritative: it replaces the buffer, moves the caret to the end, clears marked text and both history stacks, ends typing coalescence, and calls `cx.notify()` so the view redraws. An equal value is a no-op and does not notify. Uncontrolled mode retains edits without owner intervention; `set_value` remains an explicit programmatic replacement with the same synchronization behavior. Validation text is caller owned.

Consecutive character insertions at the active caret coalesce into one undo step. Selection changes, caret movement, replacement of a selected range, and IME composition end that typing group. A marked-text sequence records one pre-composition undo state, and its final commit completes that same step. If composition is cancelled via `unmark_text`, the current composed text remains and that undo step is available. Each intermediate marked-text snapshot still emits `InputChanged`; this component does not defer events until IME commit.

## Screenshot fixtures
The screenshot matrix loads these deterministic fixture contents before rendering. All selection and marked ranges are UTF-16 offsets, matching the platform input handler. The gallery captures a 760 × 430 logical-pixel frame with a titled panel, field label, and supporting caption; the field fills a 696-pixel content column. The entity is constructed once per capture and retained by the gallery root.

| Fixture | Initial value | Selection / marked range | Presentation |
|---|---|---|---|
| `empty_fixture` | empty | caret at 0 | Placeholder `Write a note…`; description `A short project note.` |
| `filled_fixture` | `Ship the first draft\nReview with the design team\nPublish the release notes` | caret at end (UTF-16 74) | Description `Three-line release checklist.` |
| `focused_fixture` | `The quick brown fox\njumps over the quiet dog.` | caret at UTF-16 10 | Enabled and focused; description `Editing the opening line.` |
| `composing_fixture` | `Café notes\nDraft in progress` (the accent is `e` plus U+0301) | caret at UTF-16 5; marked range `3..5` | Enabled and focused; marked text receives the component composition underline. |
| `selected_fixture` | `Selected text` | selection `0..13` | Enabled, unfocused full-value selection presentation. |
| `invalid_fixture` | `This note is missing a title.` | caret at end | Validation message `Add a title before saving.` |
| `disabled_fixture` | `Archived notes are read-only.\nContact the project owner to make changes.` | caret at end (UTF-16 72) | Disabled; description `This archived note cannot be edited.` |

The selected fixture's range covers its entire value; focus is deliberately absent so it distinguishes selection fill from caret focus. The composing fixture is focused to expose both focus border and marked-range underline. Marker ranges stay on one visual line, while the composing, filled, and disabled fixtures demonstrate multiline marker/edit layout.

## Keyboard map
The `TextArea` key context has default Command/Control+A, Command/Control+Z, and Command/Control+Shift+Z actions. Its key handler handles Left/Right movement. GPUI's input handler accepts platform text replacement and marked text, including newlines in the ordinary replacement path. Undo requires a fixture with a preceding edit. Tab uses normal focus traversal. Enter inserts a newline. Copy, cut, and paste use GPUI clipboard APIs and are covered by the focused adapter tests. Clipboard shortcuts are Command/Control C/X/V. Clipboard read failures and non-text clipboard items leave the value unchanged. Line navigation needs platform verification. All component actions are inert while disabled.

## Multiline visual layout contract
Every explicit newline starts a new visual line at the same content left edge, including when a caret, selected range, or IME marked range splits that line into styled pieces. Soft-wrapped lines keep the same left edge and preserve the same selection, caret, and marked-text positions as the input handler’s shaped-line geometry. A trailing newline produces a final empty visual line. Marker spans may style only the text range they cover; they must not shift subsequent lines or change the editor’s content width.

## Pointer behaviour
The input canvas supplies hit-test and range callbacks using GPUI wrapped text layouts. Pointer hit testing accounts for explicit newlines and soft wraps. Range geometry is a bounding box spanning the start and end visual caret positions (the platform callback supports one rectangle). The content viewport scrolls vertically when content exceeds its fixed height. After caret or selection changes from keyboard navigation, text replacement, or a native input-handler range update, the viewport scrolls only as far as needed to keep the active caret line visible. The disabled editor does not track focus and rejects text input. Its select-all, undo, and redo actions are inert while disabled.

## Accessibility role and properties
The root requests AccessKit `MultilineTextInput` with an accessible label and exposes the current editor text as its accessible value; placeholder text is not the value. This role maps to a native text area on macOS. It sets a description from the validation message, or from the ordinary description when there is no validation message, and requests invalid/disabled state as applicable. The generated accessibility cases are pending active platform snapshots.

## Theme tokens used
Rendering reads `Theme.colors.surface`, `text`, `text_muted`, `border`, `focus`, `danger`, `accent`, and `accent_text`; `spacing.xsmall/small/medium`, `controls.large`, `radii.medium`, `borders.strong`, and `typography.heading`. The root and editor use `w_full()` so the area fills its parent width. Focus changes border color. The editor height is fixed at `controls.large * 3`; the one-pixel border and `FIELD_TEXT_INSET = 12` used for hit testing and candidate bounds are fixed values. The inset should become a spacing token or be justified and checked against the render inset.

## WAI-ARIA pattern reference
Follow textbox role guidance. The multiline role, line-aware rendering, and caret scrolling require platform verification.

## Platform notes
GPUI handler ranges use UTF-16; internal offsets are UTF-8 byte boundaries. Marked text has a stored range. Candidate bounds, pointer hit testing, visible content, and vertical caret scrolling use multiline/wrapped layout. GPUI's input handler does not expose a portable horizontal scroll-to-caret API here; long unwrapped lines may extend horizontally. Native IME candidate placement still requires active-platform verification. Pointer drag selection starts from the hit-tested UTF-16 boundary and updates the selection anchor-to-pointer range across explicit newlines and soft-wrapped visual rows; reversed selection direction is preserved. Dragging outside the editor clamps to the nearest text boundary. Disabled editors do not start a drag. The full keyboard, pointer, IME, accessibility, and screenshot matrices remain pending harness or platform verification.

## Open questions
Maintainer review is required for public builder/event names, controlled-mode buffer contract, fixed hit-test inset, and multiline AccessKit semantics. Native IME candidate placement, active-platform drag selection, and broader accessibility and visual behavior remain unverified.
