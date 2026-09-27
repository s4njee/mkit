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
The tooltip uses a theme surface, border, caption size, and spacing so its compact label is visible
against both light and dark canvas backgrounds. An offset derived from the small control-height
token places it near the thumb.

## States

Minimum, middle, maximum, dragging, and disabled. In bipolar mode, the neutral zero position is the
fill origin. The fixture matrix uses bounds −100 to 100, step 1, reset 0, and endpoint/middle values.

## Conformance fixtures

The dedicated screenshot matrix renders all five states with shadcn light, shadcn dark, and high
contrast themes at 1× and 2×. Minimum, middle, maximum, and disabled use stable builder values.
The middle fixture enables bipolar fill. The dragging fixture starts with the middle value,
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

Use GPUI Global `Theme` colors for surface, border, selected fill, thumb, focus, text, and disabled
state. Use theme spacing/control sizing and pill radius tokens. Avoid fixed colors; any thumb size
ratio is an implementation detail to review with the visual baseline.

## WAI-ARIA pattern reference

[Slider](https://www.w3.org/WAI/ARIA/apg/patterns/slider/).

## Platform notes

Support keyboard and pointer input. Applications should retain a numeric text entry alternative
where direct numeric editing or touch accessibility is needed.

## Open questions

- Is Shift the preferred fine-drag modifier on all supported platforms?
- Should bipolar mode reject asymmetric bounds or compute its neutral point as `(min + max) / 2`?
- Should reset be visually indicated on hover as well as through double click and the `r` action?
- What formatter and tooltip precision defaults best fit consumer apps?
- Public builder naming and event semantics need maintainer review.
