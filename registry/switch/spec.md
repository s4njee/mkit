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
The look follows the shadcn/ui switch and is resolved from the installed `Theme` in three variants,
the same way Tabs does it. `high-contrast` is selected by theme name; every other theme is dark
when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived
colours use `mkit_core::contrast::composite`; no new core tokens are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Track, off ("input") | `border` | `text` at 15% over `background`, opaque | `background` |
| Track, on | `accent` | `accent` | `accent` |
| Track border | transparent | transparent | `border` when off, `accent` when on |
| Thumb, off | `background` | `text` | `text` |
| Thumb, on | `background` | `background` | `accent_text` |
| Track and thumb shadow | `shadows.small` | `shadows.small` | `shadows.small` (transparent in this theme) |
| Label | `text` | `text` | `text` |
| Keyboard focus | track border `focus` plus a 3px ring of `focus` at 50% | same | track border `focus` plus a 3px ring of opaque `focus` |
| Disabled | track, thumb and label at 50% over `background` | same | same |

Geometry: the thumb is a `spacing.large` (16px) circle. The track is `spacing.xxlarge` (32px) wide
and one thumb plus two `borders.hairline` borders tall (18px; 20px in high contrast, whose hairline
is 2px), matching shadcn/ui's `h-[1.15rem] w-8` pill with a transparent 1px border. The thumb sits
at the start of the track when off and at the end when on. Track fills are opaque because GPUI
paints the track shadow as a filled shape inside the element. The label is `typography.body` at
medium weight (500) with a `spacing.small` gap. The 3px focus ring is the shadcn/ui ring width and a
fixed component value. The row keeps a trailing `spacing.xsmall` of padding so its pointer target
stays at least as wide as the earlier 36px-track row.

Disabled matches the web preview's `opacity: .5` applied to the control as one layer: every part's
colour is composited opaque over `background` and then mixed 50% with it, and the shadow alpha is
halved. GPUI element opacity is not used because it dims each painted part separately, so a thumb
or mark would show the part beneath it.

## WAI-ARIA pattern reference

[Switch](https://www.w3.org/WAI/ARIA/apg/patterns/switch/).

## Platform notes

Space activation and focus indication follow desktop switch conventions.

## Open questions

None.
