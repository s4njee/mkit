---
spec_version: 1
component: checkbox
states:
  - id: unchecked
    description: Checkbox is off.
    fixture: unchecked_fixture
  - id: checked
    description: Checkbox is on.
    fixture: checked_fixture
  - id: indeterminate
    description: Mixed selection.
    fixture: indeterminate_fixture
  - id: disabled
    description: Checkbox is disabled.
    fixture: disabled_fixture
keys:
  - key: Space
    modifiers: []
    when: Checkbox has focus and is enabled.
    action: Toggle checked state.
    initial_state: unchecked
    expect:
      state: checked
      event: change
accessibility:
  role: checkbox
  properties:
    - name: aria-checked
      value: false
      when: unchecked
    - name: aria-checked
      value: true
      when: checked
    - name: aria-checked
      value: mixed
      when: indeterminate
    - name: aria-disabled
      value: true
      when: disabled
    - name: description
      value: Unavailable
      when: disabled
---

# Checkbox

## Purpose

A single binary choice that may also represent a mixed aggregate state.

## Anatomy

A compact square indicator and caller-provided persistent label. The label is the hit target.

## States

Unchecked, checked, indeterminate, and disabled. Activating indeterminate requests checked. A disabled control never changes.

## Props and events

`label`, `checked`/`default_checked`, `indeterminate`, and `disabled`. `is_checked()` and `is_indeterminate()` expose current local state for host logic and tests. Controlled mode emits `ChangeRequested(bool)` and waits for `set_checked(value, cx)`; uncontrolled mode updates internally and emits the same event. `set_checked` clears a mixed state and requests a redraw without emitting. Props never emit events.

## Keyboard map

Space toggles when focused. Register the `Checkbox` key context and named `Toggle` action; apps may replace its default `space` binding. An enabled checkbox is one tab stop; disabled is omitted from tab order.

## Pointer behaviour

Clicking the indicator or label toggles once. Disabled suppresses pointer actions.

## Accessibility role and properties

Checkbox role, accessible name from label, and checked boolean or mixed state. The disabled state sets the AccessKit disabled property, supplies an "Unavailable" description, and omits the tab stop. Label and indicator share one focus stop. On the pinned macOS bridge, AccessKit's mixed state appears as `AXValue=1`; the component adds a "Mixed" description so the state remains distinguishable from checked when this bridge is used.

## Theme tokens used
The look follows the shadcn/ui checkbox and is resolved from the installed `Theme` in three
variants, the same way Tabs does it. `high-contrast` is selected by theme name; every other theme
is dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light.
Derived colours use `mkit_core::contrast::composite`; no new core tokens are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Box border ("input") | `border` | `text` at 15% | `border` |
| Box fill, unchecked | `background` | `background` | `background` |
| Box fill and border, checked or mixed | `accent` | `accent` | `accent` |
| Check and dash | `accent_text` | `accent_text` | `accent_text` |
| Box shadow | `shadows.small` | `shadows.small` | `shadows.small` (transparent in this theme) |
| Label | `text` | `text` | `text` |
| Keyboard focus | box border `focus` plus a 3px ring of `focus` at 50% | same | box border `focus` plus a 3px ring of opaque `focus` |
| Disabled | box, mark and label at 50% over `background` | same | same |

Geometry: the box is `spacing.large` (16px) square with radius `radii.small` (4px) and a
`borders.hairline` border, matching shadcn/ui's `size-4 rounded-[4px] border`. The check and the
mixed dash are vector strokes painted in a `spacing.medium` (12px) square centred in the box,
following Lucide `check` (20,6 → 9,17 → 4,12) and `minus` (5,12 → 19,12) on a 24-unit grid with a
3-unit stroke (1.5px), as in the web preview. A vector path stays crisp at every scale and does not
depend on the system font carrying a check glyph. Checked and mixed share the filled box and differ
only in the mark, as in shadcn/ui; the "Mixed" description keeps them distinct for assistive
technology. The unchecked fill is opaque `background` rather than transparent because GPUI paints
the drop shadow as a filled shape inside the element. The label is `typography.body` at medium
weight (500), shadcn/ui's `font-medium`, with a `spacing.small` gap. The 3px focus ring is the
shadcn/ui ring width and a fixed component value. The row keeps a trailing `spacing.xsmall` of
padding so the pointer target is at least as wide as the earlier 17px indicator row.

Disabled matches the web preview's `opacity: .5` applied to the control as one layer: every part's
colour is composited opaque over `background` and then mixed 50% with it, and the shadow alpha is
halved. GPUI element opacity is not used because it dims each painted part separately, so a thumb
or mark would show the part beneath it.

## WAI-ARIA pattern reference

[Checkbox](https://www.w3.org/WAI/ARIA/apg/patterns/checkbox/).

## Platform notes

Space activation follows desktop checkbox conventions; preserve system focus visibility.

## Open questions

Confirm public event naming, the `set_checked(value, cx)` redraw contract, and whether indeterminate activation should be configurable. Review disabled accessibility semantics on an active platform.
