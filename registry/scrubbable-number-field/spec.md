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
- A decorative trailing scrub affordance (a left/right chevron pair drawn as a vector path) marks the
  field as draggable. It is presentation only: it adds no hit target, focus stop, or accessibility
  node, and the whole field remains the scrub and caret target.

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

The field chrome is the restyled TextField's (see `registry/text-field/spec.md`, "Theme tokens
used"), resolved from the installed `Theme` in three variants. `high-contrast` is selected by theme
name; every other theme is dark when `mkit_core::contrast::relative_luminance(colors.background) <
0.5`, otherwise light. Derived colours use a crate-local `color-mix` helper built on
`mkit_core::contrast::composite`; no mkit-core API or tokens are added. "Input" is shadcn's
`--input`: `border` in light themes and `text` at 15% in dark themes.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Field fill | `background` (opaque) | `text` at 4.5% over `background` | `background` |
| Border | input (`border`) | input (`text` at 15%) | `border` |
| Value text | `text` | `text` | `text` |
| Unit suffix and scrub affordance | `text_muted` | `text_muted` | `text_muted` |
| Resting shadow | `shadows.small` | `shadows.small` | none (`shadows.small` is transparent) |
| Keyboard focus or IME composition | border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Invalid draft (`invalid_edit`) | border `danger` plus a 3px ring of `danger` at 20% | same with the ring at 40% | border `danger`; focus still adds the opaque `focus` ring |
| Scrubbing | border `focus`; scrub affordance `accent` | same | same |
| Selection | `accent` fill, `accent_text` text | same | same |
| Caret and IME underline | `accent` | `accent` | `accent` |
| Disabled | fill, border, value, suffix and affordance at 50% over `background`; shadow alpha halved | same | `background` fill; `disabled` border, value, suffix and affordance |

- **Rings and shadows.** A ring replaces the resting shadow, as in CSS. The field fill is always
  opaque because GPUI paints drop shadows as filled shapes inside the element. Ring corners use the
  field radius. The 3px ring is the shadcn/ui ring width, a fixed component value shared with
  TextField.
- **Invalid draft.** The danger border and ring are visual only: they show while an uncommitted,
  non-composing draft does not parse (including an empty draft). The accessibility tree is
  unchanged, so the committed numeric value remains the announced value, as described under
  States. Enter, Escape and focus loss behave as before.
- **Scrubbing.** A pointer scrub is drag feedback, not keyboard focus: the border takes `focus` and
  the scrub affordance turns `accent`, but no ring is drawn unless the field also has keyboard
  focus. Over an enabled field the pointer shows the platform left/right resize cursor as the scrub
  hint; while the field owns keyboard focus and is not scrubbing it shows the text I-beam instead.
- **Selection and caret.** The caret, selection highlight and IME underline render only while the
  enabled field owns focus (or is composing), matching platform text fields that hide an inactive
  selection. The rendered range is clamped to the draft so a scrub that shortens the text never
  indexes past it.
- **Disabled.** Each colour is composited opaque over `background` and mixed 50% with it instead of
  using GPUI element opacity, which would dim each painted part separately. High contrast keeps
  solid `disabled` colours so the value stays legible.
- **Geometry (maintainer density decision, provisional).** Pro-app controls keep today's denser
  size under the E9.1 dense, dark-first principle (see `docs/E13.2_SPLIT.md`): the field stays
  `controls.small` (32px) tall with a `spacing.small` (8px) text inset on both sides, so pointer,
  caret and IME geometry and hit targets are unchanged. Colours, border, radius, shadow, focus and
  typography follow TextField: a `borders.regular` border (1px; 2px in high contrast), radius
  `radii.medium` (shadcn `rounded-md`), and `typography.body` (14px) text. The text size is applied
  explicitly and the same size shapes text for pointer hit testing and IME candidate bounds, so
  geometry does not depend on the inherited text style where a platform callback runs.
- **Marks.** The caret is a `borders.hairline` wide bar `typography.heading` (20px) tall; the IME
  underline is `borders.strong` thick. The unit suffix sits `spacing.xsmall` after the value. The
  scrub affordance is Lucide's `chevrons-left-right` (24-unit grid, 2-unit stroke) drawn as a
  vector path in a `spacing.large` (16px) box at the trailing edge.
- Scrub scale is 4 logical pixels per step with a 3 pixel activation threshold; the pointer
  threshold and modifier multipliers remain subject to maintainer review.

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
- Confirm the provisional E13.2 density decision (keep the 32px height and 8px text inset while
  adopting TextField's chrome) and the always-visible trailing scrub affordance.
