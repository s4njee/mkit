---
spec_version: 1
component: slider
states:
  - id: minimum
    description: Single-value slider is at minimum.
    fixture: minimum_fixture
  - id: middle
    description: Single-value slider is in range.
    fixture: middle_fixture
  - id: maximum
    description: Single-value slider is at maximum.
    fixture: maximum_fixture
  - id: range
    description: Two independently focusable thumbs bound an ordered selected interval.
    fixture: range_fixture
  - id: disabled
    description: Slider and every thumb are disabled.
    fixture: disabled_fixture
keys:
  - key: ArrowRight
    modifiers: []
    when: Enabled slider thumb has focus.
    action: Increase the focused thumb by one step, clamped to its neighbor or maximum.
    initial_state: middle
    expect:
      event: "interaction_started → change_requested → interaction_ended"
  - key: ArrowLeft
    modifiers: []
    when: Enabled slider thumb has focus.
    action: Decrease the focused thumb by one step, clamped to its neighbor or minimum.
    initial_state: range
    expect:
      event: "interaction_started → change_requested → interaction_ended"
  - key: ArrowRight
    modifiers: [Shift]
    when: Enabled slider thumb has focus.
    action: Increase the focused thumb by one tenth of a step.
    initial_state: middle
    expect:
      event: "interaction_started → change_requested → interaction_ended"
  - key: ArrowRight
    modifiers: [Alt]
    when: Enabled slider thumb has focus.
    action: Increase the focused thumb by one hundredth of a step.
    initial_state: middle
    expect:
      event: "interaction_started → change_requested → interaction_ended"
  - key: PageUp
    modifiers: []
    when: Enabled slider thumb has focus.
    action: Increase the focused thumb by ten steps.
    initial_state: middle
    expect:
      event: "interaction_started → change_requested → interaction_ended"
  - key: Home
    modifiers: []
    when: Enabled slider thumb has focus.
    action: Set the focused thumb to its allowed minimum.
    initial_state: middle
    expect:
      event: "interaction_started → change_requested → interaction_ended"
accessibility:
  role: slider
  properties:
    - name: aria-valuemin
      value: 0
    - name: aria-valuemax
      value: 100
    - name: aria-disabled
      value: true
      when: disabled
events:
  - id: interaction_started
    payload: Current complete one- or two-value vector at gesture start.
  - id: change_requested
    payload: Complete proposed one- or two-value vector.
  - id: interaction_ended
    payload: Last requested vector at pointer-up or after a discrete keyboard action; equals displayed values in uncontrolled mode.
  - id: interaction_cancelled
    payload: Original complete vector to restore after Escape cancels a pointer gesture.
---

# Slider

## Purpose

Select a numeric value or ordered interval from bounded, stepped values.

## Anatomy

Track, selected fill, and one or two independently focusable thumbs. A single-value slider fills from the minimum to its thumb. A range fills between its two endpoint thumbs; the active endpoint is the one most recently focused or selected by pointer input.

## States

Single value or ordered range, minimum, middle, maximum, dragging, and disabled. Constructor values are clamped and snapped to the regular step. Range endpoints stay ordered and cannot cross. Keyboard arrows move by one step; PageUp/PageDown move by ten steps; Home/End move the active thumb to its allowed bound. Shift+arrow changes by one tenth of a step and Alt+arrow by one hundredth. Fine-step requests quantize within the active endpoint's allowed interval using its lower bound as the quantum anchor, and arithmetic comparisons use floating-point tolerance.

The screenshot fixtures use a 0–100 range with a step of 1: minimum is 0, middle is 50, maximum is 100, range spans 25–75, and disabled holds 50. The same 320-point track is used for each state so the selected-fill length can be compared across shadcn light, dark, and high-contrast themes at 1× and 2×.

## Props and events

`label`, min/max, positive step, `value` or `default_value`, optional two-value range, and disabled. `new` creates a single uncontrolled value, `range` creates two ordered uncontrolled values, and `controlled` accepts one or two values. `values()` reads current displayed values. Controlled mode emits `InteractionStarted`, `ChangeRequested`, and `InteractionEnded` in that order for a discrete keyboard action that changes a value; it never mutates display state, and the owner applies accepted values through `set_value(values, cx)`. A no-op keyboard action emits no events. A pointer gesture emits `InteractionStarted` on pointer down, zero or more `ChangeRequested` events as values change, then `InteractionEnded` on pointer up. Escape during a pointer gesture emits `InteractionCancelled` with the original values; the owner restores them through `set_value`. Uncontrolled mode updates before each `ChangeRequested`, records the gesture's original vector, and restores it before emitting cancellation. All payloads carry complete one- or two-value vectors. Disabled sliders ignore keyboard and pointer input.

## Keyboard map

Each enabled thumb is a tab stop and has the `Slider` key context. Disabled thumbs are omitted from tab order. Focus selects the corresponding endpoint for named `Increase`, `Decrease`, `PageIncrease`, `PageDecrease`, `Minimum`, `Maximum`, `FineIncrease`, `FineDecrease`, `MicroIncrease`, and `MicroDecrease` actions. Escape is bound to `CancelGesture` and cancels an active pointer gesture. Default bindings are replaceable by applications. The lower thumb is limited to min and the upper thumb value; the upper thumb is limited to the lower thumb value and max.

## Pointer behaviour

Pointer down on a thumb selects that endpoint and starts dragging; clicking the track selects the nearest endpoint, choosing the lower endpoint on an exact tie. Dragging maps the full track width linearly to min/max, snaps to the regular step, and clamps at the other endpoint. Pointer capture continues movement outside the track. Pointer updates emit only when the target value changes. Disabled ignores pointer actions. The test renderer exposes the track as `mkit-slider-track` for pointer scripts.

## Accessibility role and properties

Every thumb is a separate slider accessibility node with a focus handle, role, and value min/max/now. For a single slider, the node name is `label`. For a range, names are `label minimum` and `label maximum`; each node's value reflects its own endpoint and each can receive keyboard focus independently. The selected fill spans the lower to upper endpoint. Disabled thumbs set the AccessKit disabled property, describe themselves as unavailable, and ignore input.

## Theme tokens used

Track, fill, thumb, and focus colors come from the GPUI Global `Theme`; an inactive thumb uses `text_muted` for its outline so it stays visible against the dark surface. A disabled slider renders its selected fill and thumb with the theme's disabled color rather than the active accent. Compact track/thumb dimensions use `controls` and `borders` tokens, and corners use `radii.pill`. The thumb is 0.6 times the xsmall control token to fit within the small track row; maintainers should review this fixed ratio.

## WAI-ARIA pattern reference

[Slider](https://www.w3.org/WAI/ARIA/apg/patterns/slider/).

## Platform notes

Support keyboard and pointer; provide numeric text alternatives in apps where touch accessibility is required.

## First application evaluation: Laika (E8.14)

Laika's Develop inspector uses persistent controlled Slider entities while keeping its typed value editor and host formatting. Typed lifecycle events connect pointer drags and discrete keyboard adjustments to Laika's existing undo transaction. This integration verifies that a host can compose the slider with row-specific presentation and reset behavior without taking over its state entity. The app and mkit align on `gpui-pre` 0.3.5 in the E8.14 adoption worktree.

## Open questions

Public naming for lifecycle events, the `set_value(values, cx)` redraw contract, keyboard/pointer event ordering, and whether fine modifier semantics should vary by platform need maintainer review.
