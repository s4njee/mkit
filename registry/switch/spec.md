---
spec_version: 1
component: switch
states:
  - id: "off"
    description: Switch is off.
    fixture: off_fixture
  - id: "on"
    description: Switch is on.
    fixture: on_fixture
  - id: disabled
    description: Switch is disabled.
    fixture: disabled_fixture
    checked: false
keys:
  - key: Space
    modifiers: []
    when: Enabled switch has focus.
    action: Toggle switch value.
    initial_state: "off"
    expect:
      state: "on"
      event: change
accessibility:
  role: switch
  properties:
    - name: aria-checked
      value: false
      when: "off"
    - name: aria-checked
      value: true
      when: "on"
    - name: aria-checked
      value: false
      when: disabled
    - name: aria-disabled
      value: true
      when: disabled
    - name: aria-description
      value: Unavailable
      when: disabled
---

# Switch

## Purpose

Immediately turn a setting on or off.

## Anatomy

Compact pill track, circular thumb, and persistent caller label.

## States

Off, on, and disabled-off. Disabled cannot be reached by sequential keyboard focus or activated. Disabling a switch preserves its current checked value; the declared disabled screenshot fixture starts off (`checked: false`). A disabled-on value is the same disabled modifier applied to the on state and is not a separate declared fixture in this version.

## Props and events

`label`, `checked`/`default_checked`, `disabled`. Controlled mode emits `ChangeRequested(bool)` and leaves the displayed value unchanged until `set_checked`; uncontrolled mode updates itself and emits the same event. `set_checked` applies a value silently. Disabled switches do not emit events.

## Keyboard map

Space toggles. Enter has no default activation binding. Register `Switch` context with named `Toggle` action and replaceable `space` binding; enabled switches participate in standard tab order and disabled switches are skipped.

## Pointer behaviour

Clicking track, thumb, or label toggles. Disabled suppresses interaction.

## Accessibility role and properties

Switch role with stable accessible name, checked state (`aria-checked`), an AccessKit disabled property, and a disabled description. The disabled-off fixture reports `aria-checked: false`; a disabled-on switch must continue to report `aria-checked: true`. Do not encode on/off in the accessible name.

## Theme tokens used

Global surface, text, border, accent, accent_text, focus, disabled colors; spacing.xsmall/small, radii.pill, border.hairline, typography.body.

## WAI-ARIA pattern reference

[Switch](https://www.w3.org/WAI/ARIA/apg/patterns/switch/).

## Platform notes

Space activation and focus indication follow desktop switch conventions.

## Open questions

None.
