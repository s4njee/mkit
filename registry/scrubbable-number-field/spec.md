---
spec_version: 1
component: scrubbable-number-field
states:
  - id: idle
    description: Focusable field showing its current formatted value.
    fixture: number_idle
    screenshot_status: captured_by_number_field_matrix
  - id: focused
    description: Field has keyboard focus and a collapsed caret or selection.
    fixture: number_focused
    screenshot_status: captured_by_number_field_matrix
  - id: editing
    description: Text entry is in progress with a valid numeric draft.
    fixture: number_editing
    screenshot_status: captured_by_number_field_matrix
  - id: invalid_edit
    description: Text entry is in progress with an incomplete or invalid draft.
    fixture: number_invalid_edit
    screenshot_status: captured_by_number_field_matrix
  - id: scrubbing
    description: Primary pointer drag is changing the value.
    fixture: number_scrubbing
    screenshot_status: captured_by_number_field_matrix
  - id: disabled
    description: Value is visible but the field cannot be focused or changed.
    fixture: number_disabled
    screenshot_status: captured_by_number_field_matrix
keys:
  - key: ArrowUp
    modifiers: []
    when: focused, outside IME composition
    action: Increase by one step and commit the resulting value.
    initial_state: focused
    expect:
      state: focused
      event: value_changed
      focus_target: number_field
  - key: ArrowDown
    modifiers: []
    when: focused, outside IME composition
    action: Decrease by one step and commit the resulting value.
    initial_state: focused
    expect:
      state: focused
      event: value_changed
      focus_target: number_field
  - key: PageUp
    modifiers: []
    when: focused, outside IME composition
    action: Increase by the configured page step and commit the resulting value.
    initial_state: focused
    expect:
      state: focused
      event: value_changed
      focus_target: number_field
  - key: PageDown
    modifiers: []
    when: focused, outside IME composition
    action: Decrease by the configured page step and commit the resulting value.
    initial_state: focused
    expect:
      state: focused
      event: value_changed
      focus_target: number_field
  - key: Home
    modifiers: []
    when: focused, outside text editing or IME composition
    action: Set to the configured minimum when present; otherwise preserve platform text navigation.
    initial_state: focused
    expect:
      state: focused
      event: value_changed
      focus_target: number_field
  - key: End
    modifiers: []
    when: focused, outside text editing or IME composition
    action: Set to the configured maximum when present; otherwise preserve platform text navigation.
    initial_state: focused
    expect:
      state: focused
      event: value_changed
      focus_target: number_field
  - key: Enter
    modifiers: []
    when: editing
    action: Parse and commit a valid draft; restore the last committed value for an invalid draft.
    initial_state: editing
    expect:
      state: focused
      event: value_committed
      focus_target: number_field
  - key: Escape
    modifiers: []
    when: editing
    action: Discard the uncommitted text draft and restore the last committed value.
    initial_state: invalid_edit
    expect:
      state: focused
      focus_target: number_field
  - key: ArrowUp
    modifiers: [Shift]
    when: focused, outside IME composition
    action: Increase by one tenth of the configured step and commit.
    initial_state: focused
    expect:
      state: focused
      event: value_changed
      focus_target: number_field
  - key: ArrowDown
    modifiers: [Shift]
    when: focused, outside IME composition
    action: Decrease by one tenth of the configured step and commit.
    initial_state: focused
    expect:
      state: focused
      event: value_changed
      focus_target: number_field
accessibility:
  role: spinbutton
  properties:
    - name: value
      value: 12.5
      when: idle
    - name: minimum
      value: 0
      when: idle
    - name: maximum
      value: 100
      when: idle
    - name: value
      value: 12.5
      when: focused
    - name: minimum
      value: 0
      when: focused
    - name: maximum
      value: 100
      when: focused
    - name: value
      value: 12.5
      when: editing
    - name: minimum
      value: 0
      when: editing
    - name: maximum
      value: 100
      when: editing
    - name: value
      value: 12.5
      when: invalid_edit
    - name: minimum
      value: 0
      when: invalid_edit
    - name: maximum
      value: 100
      when: invalid_edit
    - name: value
      value: 12.5
      when: scrubbing
    - name: minimum
      value: 0
      when: scrubbing
    - name: maximum
      value: 100
      when: scrubbing
    - name: value
      value: 12.5
      when: disabled
    - name: minimum
      value: 0
      when: disabled
    - name: maximum
      value: 100
      when: disabled
    - name: disabled
      value: true
      when: disabled
---

# Scrubbable number field (E5.5 pilot draft)

## Purpose

Edit a bounded or unbounded numeric value by typing, keyboard stepping, or dragging horizontally.
The same numeric value remains available through a keyboard path; scrubbing is an additional
desktop interaction. Intended for inspector properties, parameters, and other compact numeric
controls.

## Anatomy

- One focusable text-entry surface with a formatted numeric value and optional unit suffix.
- Optional accessible label and description supplied by the owner; the unit is included in the
  value text when useful and is not used as the control's name.
- A drag gesture on the field scrubs the value; there are no separate increment/decrement buttons
  in this pilot.

## States

- **idle**: displays the committed formatted value; pointer hover may show the scrub affordance.
- **focused**: keyboard focus is on the text field. Arrow and page keys adjust the value unless
  the text system is composing an IME sequence. A selection/caret remains available for typing.
- **editing**: typed text is a valid numeric draft; the committed numeric value remains the
  accessible value until commit.
- **invalid_edit**: typed text is empty, incomplete, or unparsable; the committed numeric value
  remains the accessible value until the draft is corrected, cancelled, or focus leaves.
- **scrubbing**: a primary-button drag past a 3 logical-pixel threshold updates the value from
  horizontal pointer delta. Ending the drag commits once; cancellation restores the drag-start
  value. The field retains focus if it had focus before the gesture and otherwise does not steal
  focus.
- **disabled**: shows the value, is omitted from keyboard focus traversal, and ignores keyboard
  and pointer changes.

Initial fixture values are `number_idle` = 12.5, `number_focused` = 12.5, `number_editing` = valid
draft `13.5`, `number_invalid_edit` = draft `-`, `number_scrubbing` = 12.5 with pointer down at
the scrub origin, and `number_disabled` = 12.5.
The standard numeric fixture uses step 1, page step 10, minimum 0, maximum 100, and accessible
name “Opacity”.

## Props and events

Stateful `Entity<ScrubbableNumberField>` with typed events. Builder properties: `value` (initial
value in uncontrolled mode), `min`, `max`, `step` (positive finite value, default 1), `page_step`
(default 10 × step), `precision_step` (default step / 10), `scrub_scale` (logical pixels per
step; proposed default 4), `format`, `parse`, `unit`, `label`, `description`, and `disabled`. The
current pilot uses default `f64` formatting; its optional unit is shown as a muted suffix and
accepted as a suffix when parsing a draft. Locale-aware parse/format callbacks remain pending.

Controlled mode supplies a value and receives `value_changed { previous, value, source }`; `value`
is a proposed value and the owner accepts it by updating the supplied value. The component does not
retain a divergent committed value, but keeps a local text draft while editing. Uncontrolled mode
stores committed numeric value internally. Both modes emit `value_changed` when a valid user action
produces a value different from the current value; `source` is `text`, `keyboard`, or `pointer`.
Values are clamped to configured bounds. No change event fires when clamping leaves the value
unchanged. `value_committed` reports the candidate accepted by local uncontrolled state, or the
requested candidate in controlled mode; controlled owners remain responsible for accepting or
rejecting the request.

`value_committed { value, source }` fires once when a valid text draft is accepted with Enter or
focus leaving, when a spin key changes the value, or when a scrub ends with a changed value.
Invalid text on Enter or focus loss restores the last committed value without a change/commit
event. Escape cancels only the active text draft or pointer scrub. Component emits typed
`EventEmitter` events; applications may rebind key actions through its registered key context.

## Keyboard map

The front block lists executable state, event, and focus assertions. `number_field` is the stable
fixture target ID. Up/Down change by one `step`; Shift+Up/Down use `precision_step`; PageUp/PageDown
change by `page_step`; each clamps to min/max. Repeated key events repeat the same operation. Home
and End set a configured bound. When the relevant bound is absent, Home/End retain the platform
text-field behavior. Typing and native selection, clipboard, undo, and caret navigation remain
text-system behavior; `simulate_input` supplies text in the harness.

Enter parses/commits a valid draft and keeps focus. Invalid drafts revert on Enter and focus loss.
Escape restores the last committed value and keeps focus from either editing state. Tab and Shift+Tab follow ordinary
desktop focus traversal. During IME composition, the text system owns keys and the field does not
interpret arrows or commit/cancel composition. All component key bindings use a registered
`scrubbable_number_field` key context and named actions.

## Pointer behaviour

Primary-button press records the pointer origin and current value. A horizontal move exceeding 3
logical pixels enters scrubbing; each horizontal delta changes value by `delta / scrub_scale ×
step`, preserving fractional values. Shift applies 0.1 precision; Alt applies 10× coarse movement.
The effective modifier factors multiply if both are held. Vertical motion has no effect. The
pointer may leave the field while dragging; release commits, while Escape or pointer cancellation
restores the drag-start value. A click without crossing the threshold places the text caret and
does not change the value. Secondary click follows the app's ordinary context-menu convention and
is not consumed by the field.

## Accessibility role and properties

Expose a spinbutton semantic with an owner-provided accessible name, current numeric value,
optional minimum and maximum, unit/value text where supported, and disabled state. The editable
surface is one focus stop. A screen reader can inspect and change the value through the standard
spinbutton keyboard path; dragging is never required. Draft text does not replace the announced
numeric value until valid. No live-region announcement is added in addition to native value
change notifications. The Accessibility bridge maps these concepts to supported AccessKit
properties rather than literal ARIA attribute names. The pilot suppresses focus and input,
dims the presentation, and requests AccessKit disabled state through GPUI's synthetic
accessibility builder. Live native output remains unverified.

## Theme tokens used

Read all values from the GPUI `Global` theme token set:

- field surface, text, and border tokens for default, hover, focused, disabled, and invalid draft
  presentation;
- focus-ring token for keyboard focus;
- accent/selection token for selected text and scrub affordance;
- spacing and control-height tokens for field geometry;
- typography token for numeric text.

This implementation maps these roles to the Global Theme's surface/text/border/focus/disabled/text-muted
colors, `spacing.small`/`spacing.xsmall`, `radii.small`, `borders.hairline`, `typography.body`, and
`controls.small`. Scrub scale is 4 logical pixels per step with a 3 pixel activation threshold.
The fixed pointer threshold and modifier multipliers remain subject to maintainer review; the
rendered field uses only the shared theme tokens for its visible geometry and color.

## WAI-ARIA pattern reference

[Spinbutton pattern](https://www.w3.org/WAI/ARIA/apg/patterns/spinbutton/), plus custom pointer
scrubbing as mapped in [component-pattern-map.md](../../docs/component-pattern-map.md). Preserve
direct text entry and native text editing in addition to the spinbutton keyboard contract.

## Platform notes

Use the host text system for selection, clipboard, undo, locale-aware numeric parsing, and IME
composition. Decimal separator and displayed unit formatting follow the supplied formatter and
locale; parsing must not silently reinterpret malformed input. Modifier-based drag factors are
app conventions and may need platform review; apps can rebind named keyboard actions. Respect
platform focus traversal and accessibility APIs. Pointer precision and logical-pixel scaling must
be checked on macOS, Windows, and Linux when their harness adapters are available.

## Open questions

- Maintainer approval needed for the public component name, prop/event names, and continuous
  `value_changed` versus commit event contract.
- Confirm whether Home/End should set bounds or always retain text-navigation semantics in focused
  mode; the draft assigns bounds when configured.
- Confirm Shift/Alt pointer precision factors, Shift+arrow precision, scrub scale and threshold.
- Define locale-aware parser/formatter defaults and behavior for NaN/infinity before implementation.
- Confirm concrete E9 token names and availability of AccessKit spinbutton value/min/max mappings.
- Confirm pointer cancellation representation and whether focus should be acquired on mouse-down.
