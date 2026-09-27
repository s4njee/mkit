---
spec_version: 1
component: toggle-button
states:
  - id: "off"
    description: Toggle is not pressed.
    fixture: toggle_off
  - id: "on"
    description: Toggle is pressed.
    fixture: toggle_on
  - id: disabled
    description: Toggle cannot change state.
    fixture: toggle_disabled
screenshots:
  themes: [shadcn-light, shadcn-dark, high-contrast]
  scales: [1, 2]
  matrix: every state × theme × scale
keys:
  - key: Enter
    modifiers: []
    when: enabled and focused
    action: Toggle the pressed state.
    initial_state: "off"
    expect:
      state: "on"
      event: value_changed
  - key: Space
    modifiers: []
    when: enabled and focused
    action: Toggle the pressed state using platform button behavior.
    initial_state: "off"
    expect:
      state: "on"
      event: value_changed
accessibility:
  role: button
  properties:
    - name: pressed
      value: false
      when: "off"
    - name: pressed
      value: true
      when: "on"
    - name: description
      value: Unavailable
      when: disabled
    - name: aria-disabled
      value: true
      when: disabled
---

# Toggle button

## Purpose

A two-state button for a persistent on/off choice such as formatting or view mode.

## Anatomy

A label and optional leading/trailing icon content in a button surface.

## States

`off`, `on`, and `disabled`. State remains controlled by a parent when `controlled` is used.

## Screenshot matrix

The gallery captures each declared state (`off`, `on`, `disabled`) with the shadcn light and dark
themes and the high-contrast theme at 1x and 2x. The JSON conformance manifest names each baseline;
the matrix test reads those cases and fails if a declared case has no captured image.

## Props and events

Stateful `Entity<ToggleButton>` with typed `ValueChanged(bool)` event. `new(label, default_pressed)` is uncontrolled: activation updates the stored value, notifies the view, then emits `ValueChanged(next)`. `controlled(label, pressed)` emits the requested next value without changing its stored value; the parent applies an accepted value with `set_pressed`, which updates and notifies the view without emitting another change request. Disabled activation emits no event and changes no state in either mode. Variant, size, optional icon, disabled, and accessible label are configurable.

## Keyboard map

Enter and Space dispatch the `Toggle` action through the `MkitToggleButton` key context, which hosts bind and may rebind. Both request the opposite value when enabled and focused. Native tab traversal applies; disabled toggles are not focus targets.

## Pointer behaviour

Primary click requests a changed value if enabled. Disabled suppresses state changes and events.

## Accessibility role and properties

Button role with accessible label and toggled state mapped to `aria-pressed`; disabled sets the AccessKit disabled property and supplies an unavailable description.

## Theme tokens used

`Theme.colors` background/surface/text/text_muted/border/accent/accent_text/focus/disabled; `Theme.spacing` small/medium; `Theme.radii.small`; `Theme.borders.regular`.

## WAI-ARIA pattern reference

[Button Pattern: toggle button](https://www.w3.org/WAI/ARIA/apg/patterns/button/).

## Platform notes

The visual pressed state and semantic pressed state are synchronized from one value.

## Open questions

None.
