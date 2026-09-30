---
spec_version: 1
component: precision-slider
states:
  - id: minimum
    description: Value is at its lower bound.
    fixture: minimum_fixture
  - id: middle
    description: Value is in range; bipolar mode renders zero at the track center.
    fixture: middle_fixture
  - id: maximum
    description: Value is at its upper bound.
    fixture: maximum_fixture
  - id: dragging
    description: Pointer drag updates the value and displays its formatted tooltip.
    fixture: dragging_fixture
  - id: disabled
    description: Slider cannot receive focus or change its value.
    fixture: disabled_fixture
  - id: focused
    description: Slider owns keyboard focus; the thumb shows the keyboard focus ring.
    fixture: focused_fixture
keys:
  - key: ArrowRight
    modifiers: []
    when: Enabled slider is focused.
    action: Increase by one regular step.
    initial_state: middle
    expect:
      event: change_requested
  - key: ArrowLeft
    modifiers: []
    when: Enabled slider is focused.
    action: Decrease by one regular step.
    initial_state: middle
    expect:
      event: change_requested
  - key: ArrowRight
    modifiers: [Shift]
    when: Enabled slider is focused.
    action: Increase by one tenth of a regular step.
    initial_state: middle
    expect:
      event: change_requested
  - key: r
    modifiers: []
    when: Enabled slider is focused.
    action: Request the configured reset value.
    initial_state: middle
    expect:
      event: change_requested
accessibility:
  role: slider
  properties:
    - name: aria-valuemin
      value: configured minimum
    - name: aria-valuemax
      value: configured maximum
    - name: aria-valuenow
      value: current value
    - name: aria-disabled
      value: true
      when: disabled
---

# Precision slider

## Purpose

Adjust one bounded numeric value with direct pointer scrubbing, precise modifier movement,
optional bipolar zero-centered presentation, and a quick reset affordance.

## Anatomy

One horizontal track, selected fill, one focusable thumb, and a value tooltip shown during dragging.
The tooltip uses the restyled Tooltip look and is centred above the thumb, offset by the small
control-height token. Two decorative marks are drawn as vector shapes, never text glyphs:

- In bipolar mode a **centre mark** (a vertical tick) marks zero. It is taller than the thumb, so it
  stays visible above and below the thumb when the value is zero.
- A **reset marker** (a small upward-pointing triangle below the track) marks the reset value
  whenever the current value differs from it, showing where double click or `r` returns. In bipolar
  mode, when the reset value is zero the centre mark already shows it and the triangle is omitted.

Neither mark adds a hit target, focus stop, or accessibility node.

## States

Minimum, middle, maximum, dragging, focused, and disabled. In bipolar mode, the neutral zero
position is the fill origin. The fixture matrix uses bounds −100 to 100, step 1, and endpoint/middle
values; each fixture's reset value is its initial value (the `new` default), so the dragging fixture
shows the reset marker at its drag-start value of 0.

## Conformance fixtures

The dedicated screenshot matrix renders all six states with shadcn light, shadcn dark, and high
contrast themes at 1× and 2×. Minimum, middle, maximum, and disabled use stable builder values.
The middle fixture enables bipolar fill. The focused fixture is bipolar at 40, so the fill grows
from the centre mark; it focuses the slider, then dispatches an unbound Escape keystroke so GPUI
records keyboard modality and paints the keyboard focus ring. The dragging fixture starts with the middle value,
dispatches a real primary-button pointer drag across the track, and captures before pointer up so
the value tooltip is visible. The test asserts the drag changed the numeric value before comparing
its screenshot. This matrix is visual evidence; the generated keyboard and active-platform
accessibility cases remain separate checks.

## Props and events

`label`, current/default value, min, max, positive step, reset value, optional bipolar mode,
disabled, and optional value formatter. `new` is uncontrolled; `controlled` treats its value as
owner-supplied. Uncontrolled interaction updates the displayed value before emitting
`ChangeRequested(value)`. Controlled interaction emits the proposed value without changing its
display; the owner applies accepted values with `set_value`. Reset uses the same value event and
contract as other changes. No event is emitted for a no-op.

## Pointer behaviour

Pointer down on the track begins scrubbing and maps horizontal position linearly to the numeric
domain; dragging continues through the parent pointer event path. Hold Shift while dragging to
reduce pointer sensitivity to one tenth of regular drag sensitivity; the result quantizes to one
tenth of the regular step. A double click requests the configured
reset value. While dragging, show a compact value tooltip. In bipolar mode min/max are expected to be
symmetric around zero, the neutral point is centered, and the fill grows from zero toward the value.

## Keyboard map

The focusable control registers a `PrecisionSlider` key context. Left/Down and Right/Up request one
step down/up; Shift variants request one tenth step; Home and End request min and max; `r` requests
reset. Bindings are public defaults and can be replaced by applications. Disabled controls are
omitted from tab order and ignore actions.

## Accessibility role and properties

Expose one slider node named from `label` with numeric min, max, and current value. Disabled state is
exposed through AccessKit and the value remains readable. The tooltip is visual feedback; it does
not replace the slider's accessible numeric value.

## Theme tokens used

The track, fill and thumb are the restyled Slider's (see `registry/slider/spec.md`, "Theme tokens
used"), resolved from the installed `Theme` in three variants. `high-contrast` is selected by theme
name; every other theme is dark when `mkit_core::contrast::relative_luminance(colors.background) <
0.5`, otherwise light. Derived colours use `mkit_core::contrast::composite`; no mkit-core API or
tokens are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Track ("muted") | `text` at 4% over `background` | `text` at 12% over `background` | `background` with a `borders.hairline` border in `border` |
| Selected fill | `accent` | `accent` | `accent` |
| Thumb fill | `background` | `background` | `background` |
| Thumb border | `accent` | `accent` | `accent` |
| Thumb shadow | `shadows.small` | `shadows.small` | `shadows.small` (transparent in this theme) |
| Keyboard focus | a `spacing.xsmall` (4px) ring of `focus` at 50% around the thumb | same | the same ring in opaque `focus` |
| Centre mark and reset marker | `text_muted` | `text_muted` | `text_muted` |
| Drag tooltip | `accent` fill, `accent_text` label | same | `background` fill, `text` label, `borders.hairline` border in `border` |
| Disabled | track, fill, thumb border and marks at 50% over `background`; shadow alpha halved | same | same |

- **Geometry (maintainer density decision, provisional).** The pointer row keeps its existing
  `controls.small` (32px) height, so the hit target and the pointer-to-value mapping are unchanged
  (see `docs/E13.2_SPLIT.md`). Inside it the visible track is `spacing.xsmall + borders.strong`
  tall with radius `radii.pill` (6px in the shadcn themes, shadcn/ui's `h-1.5`; 7px in high
  contrast). The thumb is a `spacing.large` (16px) circle with a `borders.hairline` border.
- **Focus.** Only keyboard focus (GPUI's last input was a keyboard event) shows the ring, which
  follows Slider's `ring-4` and replaces the thumb shadow. The ring is drawn as a separate
  `spacing.large + 2 × spacing.xsmall` circle behind the opaque thumb, because a GPUI spread shadow
  keeps the element's own corner radius and would square the ring's corners. Pointer focus while dragging shows no
  ring. The track no longer changes colour on focus.
- **Centre mark.** A `borders.regular` wide (1px; 2px in high contrast) vertical tick,
  `spacing.large + 2 × spacing.xsmall` (24px) tall and centred on the track, drawn below the thumb
  so its ends stay visible above and below a thumb at zero.
- **Reset marker.** A filled triangle `spacing.small` (8px) wide and `spacing.xsmall` (4px) tall,
  apex up, `borders.strong` below the thumb's bottom edge, centred on the reset value.
- **Tooltip.** The restyled Tooltip look: horizontal padding `spacing.medium`, vertical padding the
  midpoint of `spacing.xsmall` and `spacing.small` (6px), radius `radii.medium`, and
  `typography.caption` text that does not wrap. It is centred horizontally on the thumb and its
  bottom edge sits `controls.small` above the row's bottom.
- **Disabled** matches the web preview's `opacity: .5` applied to the control as one layer: each
  colour is composited opaque over `background` and mixed 50% with it, and the shadow alpha is
  halved. GPUI element opacity is not used because it dims each painted part separately. Thumb
  fills are opaque because GPUI paints the drop shadow as a filled shape inside the element.

## WAI-ARIA pattern reference

[Slider](https://www.w3.org/WAI/ARIA/apg/patterns/slider/).

## Platform notes

Support keyboard and pointer input. Applications should retain a numeric text entry alternative
where direct numeric editing or touch accessibility is needed.

## Open questions

- Is Shift the preferred fine-drag modifier on all supported platforms?
- Should bipolar mode reject asymmetric bounds or compute its neutral point as `(min + max) / 2`?
- Is the always-visible reset marker (shown only when the value differs from the reset value) the
  right reset affordance, or should it appear on hover only?
- Confirm the provisional E13.2 density decision (keep the 32px pointer row) and whether fine drag
  should get its own visual cue.
- What formatter and tooltip precision defaults best fit consumer apps?
- Public builder naming and event semantics need maintainer review.
