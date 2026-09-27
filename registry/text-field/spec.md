---
spec_version: 1
component: text-field
states:
  - id: empty
    description: Empty single-line input, optionally showing placeholder.
    fixture: empty_fixture
  - id: filled
    description: Input contains editable text.
    fixture: filled_fixture
  - id: focused
    description: Input owns keyboard focus and native selection/caret.
    fixture: focused_fixture
  - id: invalid
    description: Input exposes invalid validation state and associated message.
    fixture: invalid_fixture
  - id: disabled
    description: Value is readable but input cannot be focused, edited, or changed by field actions.
    fixture: disabled_fixture
  - id: secure
    description: Password value is masked visually and in accessibility output; copy and cut are disabled.
    fixture: secure_fixture
    accessibility_role: password
keys:
  - key: Ctrl+A
    modifiers: [Ctrl]
    when: focused and enabled
    action: Select all text using the platform text-editing command.
    initial_state: filled
    expect:
      state: focused
      focus_target: input
  - key: Ctrl+Z
    modifiers: [Ctrl]
    when: focused and enabled and not composing
    action: Undo the latest edit when the fixture has undo history.
    initial_state: filled
    expect:
      state: filled
      event: input
      focus_target: input
  - key: Ctrl+Shift+Z
    modifiers: [Ctrl, Shift]
    when: focused and enabled and not composing
    action: Redo the latest undone edit when the fixture has redo history.
    initial_state: filled
    expect:
      state: filled
      event: input
      focus_target: input
  - key: Ctrl+C
    modifiers: [Ctrl]
    when: focused and enabled and selection is non-empty
    action: Copy the selected text to the system clipboard.
    initial_state: filled
    expect:
      state: focused
      focus_target: input
  - key: Ctrl+X
    modifiers: [Ctrl]
    when: focused and enabled and selection is non-empty and not composing
    action: Copy selection and delete it as one edit.
    initial_state: filled
    expect:
      state: filled
      event: input
      focus_target: input
  - key: Ctrl+V
    modifiers: [Ctrl]
    when: focused and enabled and not composing
    action: Replace selection with plain text from the system clipboard.
    initial_state: filled
    expect:
      state: filled
      event: input
      focus_target: input
accessibility:
  role: textbox
  properties:
    - name: aria-value
      value: current editor text
      except_when: [secure]
    - name: aria-value
      value: bullet-masked editor text
      when: secure
    - name: aria-invalid
      value: true
      when: invalid
    - name: aria-disabled
      value: true
      when: disabled
---

# Text field

## Purpose
A compact single-line text editor for short values. The draft uses GPUI's text input handler for IME composition and selection, with explicit select-all, clipboard, and undo/redo actions.

## Anatomy
One focusable text input with optional placeholder, description, and validation message. The caller supplies the accessible name through `with_label`.

## States
`empty`, `filled`, and `focused` describe editing/value presentation; `invalid` and `disabled` may overlay either value state. Placeholder is visible only when empty. The caret is visible only while the enabled field owns focus; an unfocused or disabled filled field shows its value without a caret. A validation message sets a danger border and message and requests invalid accessibility state. A disabled field preserves its displayed value, declines text input, and ignores select-all, undo, and redo actions.

`secure` masks each Unicode scalar as a bullet, uses the AccessKit password input role, and withholds the clear value from the accessibility node. Copy and cut are no-ops; paste, editing, selection, and IME continue to work. The owning application receives the clear value through `InputChanged` and must handle it as a secret. This presentation protects against casual screen and clipboard disclosure, not memory inspection.

## Screenshot fixtures
The screenshot matrix renders each fixture below at 1x and 2x using shadcn light, shadcn dark, and high-contrast themes. Every fixture uses one persistent `Entity<TextField>` created with `TextField::new(cx)`; fixture setup uses the public builder, value, focus, and fixture-state APIs. The shared frame has a `Text field` heading, then the state caption, the field, and a short muted visual note, in that order. The field label is `Workspace name` except in the secure fixture, which uses `Secret token`.

| Fixture | Caption / visual note | Placeholder | Description | Value / selection | Additional state |
|---|---|---|---|---|---|
| `empty_fixture` | `Empty` / `Placeholder with ordinary helper text` | `Enter a workspace name` | `Used in invitations and settings.` | Empty, caret at 0 | Enabled and unfocused |
| `filled_fixture` | `Filled` / `Unfocused value with ordinary helper text` | None | `Used in invitations and settings.` | `Northstar Studio`, caret at UTF-16 offset 9 | Enabled and unfocused |
| `focused_fixture` | `Focused` / `Caret at the middle of the value` | None | `Used in invitations and settings.` | `Northstar Studio`, caret at UTF-16 offset 9 | Enabled and focused, with caret visible |
| `invalid_fixture` | `Invalid` / `Validation message and danger border` | None | None | `northstar studio`, caret at end | Validation message `Use letters, numbers, and hyphens.` |
| `disabled_fixture` | `Disabled` / `Read-only value with organization note` | None | `This workspace is managed by your organization.` | `Northstar Studio`, caret at end | Disabled and unfocused |
| `secure_fixture` | `Secure` / `Value is masked` | None | None | `sample-token`, caret at end | Secure and unfocused |

These fixed strings intentionally distinguish placeholder, ordinary description, focused caret, validation, and disabled rendering. The screenshot test loads the named fixture in every matrix case and stores images under `tests/baselines/<state>/<theme>-<scale>x.png`.

## Props and events
This is an `Entity<TextField>` view. Construct with `TextField::new(cx)` and configure `with_label`, `with_placeholder`, `with_description`, `with_validation_message`, `disabled`, and `secure`. By default the entity owns its initial empty value (uncontrolled). `controlled(value)` initializes controlled mode with the owner's current value; `set_value(value, cx)` is the owner-to-view synchronization API in either mode. There is no `default_value` builder or `on_input` callback. Subscribe to the typed `InputChanged(String)` event.

Both modes apply a user edit to the local editing buffer immediately, then emit exactly one `InputChanged` snapshot containing the resulting full text. This ordering applies to text replacement, each IME marked-text update and commit, and successful undo/redo; selection, caret movement, unmarking, and programmatic `set_value` emit no event. The caller should treat the event as a change request and synchronize its chosen value with `set_value`. In controlled mode, that call acknowledges the request or replaces it with the caller's authoritative value. If the owner does not call `set_value`, the optimistic edit remains visible until a later owner update. Calling `set_value` with text equal to the current buffer preserves selection and marked text, which permits owners to echo IME snapshots safely. A different value is authoritative: it replaces the buffer, moves the caret to the end, clears marked text and both history stacks, and calls `cx.notify()` so the view redraws. An equal value is a no-op and does not notify. Uncontrolled mode retains edits without owner intervention; `set_value` remains an explicit programmatic replacement with the same synchronization behavior. Validation text is caller owned.

## Keyboard map
The enabled field registers a GPUI tab stop and the `TextField` key context with default Command/Control+A, C, X, V, Z, and Command/Control+Shift+Z actions. The key handler handles Left/Right movement; select-all and undo/redo run through the registered actions so apps can rebind them. GPUI's input handler accepts platform text replacement and marked text. Select-all changes selection without emitting `InputChanged`; successful undo/redo emits once, while an empty history emits nothing. Copy writes the selected UTF-8 text to GPUI’s system clipboard without changing the value. Cut copies then removes the selection as one undoable edit and emits one `InputChanged`. Paste reads plain text, replaces the selection as one edit, normalizes line breaks to spaces, and emits one `InputChanged`; unavailable or non-text clipboard contents do nothing. Empty selections make copy/cut no-ops. Disabled fields ignore editing and clipboard actions. Secure fields decline Copy and Cut without changing the clipboard or value. Tab uses normal focus traversal.

## Pointer behaviour
The input canvas supplies hit-test and range callbacks for caret placement and selection. A left press sets the anchor and caret; dragging with the left button extends the selection from the anchor, including outside the field bounds. Hit testing returns UTF-16 offsets and clamps to valid UTF-8 character boundaries, including surrogate pairs. Pointer selection emits no `InputChanged`; disabled fields do not track focus or change selection.

## Accessibility role and properties
The root requests AccessKit `TextInput` with an accessible label and exposes the current editor text as its accessible value; placeholder text is not the value. In secure mode it requests `PasswordInput` and exposes bullets rather than the clear value. It sets a description from the validation message, or from the ordinary description when there is no validation message, and requests invalid/disabled state as applicable. These are generated conformance expectations, not verified platform snapshots. No explicit multiline property is set.

## Theme tokens used
Rendering reads `Theme.colors.surface`, `text`, `text_muted`, `border`, `focus`, `danger`, `accent`, and `accent_text`; `spacing.xsmall/medium`, `controls.medium`, `radii.medium`, `borders.strong`, and `typography.heading`. The root and editor use `w_full()` so the field fills its parent width. Focus changes border color. The editor height is `controls.medium`; the one-pixel border is fixed. The base text inset is `spacing.medium`; an optional additive `with_leading_inset` reserves space before text for a leading adornment. The resulting total inset is used consistently for rendering, pointer hit testing, and IME candidate bounds.

## WAI-ARIA pattern reference
Text field follows the textbox role guidance and native single-line text input convention.

## Platform notes
UTF-16 selection ranges are converted at the GPUI handler boundary. Marked text has a stored range; candidate bounds use a shaped single line and the same theme-derived total text inset as pointer hit testing. Modifier spelling differs by platform. Active-platform accessibility and screenshot checks, native IME interaction, cross-platform pointer selection and keyboard behavior remain pending. Clipboard calls use GPUI’s synchronous system clipboard API.

## Open questions
The draft `with_leading_inset(f32)` builder accepts an additive logical-pixel inset; callers should pass a theme token (for example `Theme.spacing.medium`) rather than a fixed dimension. Negative and non-finite inputs normalize to zero. This narrow API addition is pending maintainer API approval. Maintainer review is also required for the other public builder/event names, secure-mode clipboard and accessibility contract, controlled-mode buffer contract, and accessibility bridge behavior. Confirm candidate bounds against actual layout and drag selection on a live platform.
