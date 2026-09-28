---
spec_version: 1
component: radio-group
states:
  - id: first_selected
    description: First enabled option selected.
    fixture: first_selected_fixture
  - id: second_selected
    description: Second option selected.
    fixture: second_selected_fixture
  - id: disabled
    description: Group is disabled.
    fixture: disabled_fixture
  - id: controlled
    description: A controlled group awaits its owner after requesting a value.
    fixture: controlled_fixture
  - id: no_selection
    description: No option is selected; the first enabled option is the roving tab stop.
    fixture: no_selection_fixture
  - id: horizontal_first_selected
    description: Horizontal group with the first enabled option selected.
    fixture: horizontal_first_selected_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: An enabled radio option is focused.
    action: Select and focus the next enabled option, wrapping.
    initial_state: first_selected
    expect:
      state: second_selected
      event: ChangeRequested(second)
      focus_target: radio
  - key: Home
    modifiers: []
    when: An enabled radio option is focused.
    action: Select and focus the first enabled option.
    initial_state: second_selected
    expect:
      state: first_selected
      event: ChangeRequested(first)
      focus_target: radio
  - key: End
    modifiers: []
    when: An enabled radio option is focused.
    action: Select and focus the last enabled option.
    initial_state: first_selected
    expect:
      state: second_selected
      event: ChangeRequested(second)
      focus_target: radio
  - key: ArrowDown
    modifiers: []
    when: The group is disabled.
    action: Keep selection and focus unchanged and emit no event.
    initial_state: disabled
    expect:
      state: disabled
      event: none
      focus_target: none
  - key: ArrowDown
    modifiers: []
    when: A controlled group has its first option selected.
    action: Request the second value and focus it without changing the controlled value.
    initial_state: controlled
    expect:
      state: first_selected
      event: ChangeRequested(second)
      focus_target: radio
  - key: ArrowUp
    modifiers: []
    when: The first enabled option is focused in a vertical group.
    action: Wrap to the last enabled option, skipping the disabled option.
    initial_state: first_selected
    expect:
      state: second_selected
      event: ChangeRequested(second)
      focus_target: radio
  - key: Space
    modifiers: []
    when: The group has no selection and its first enabled option is focused.
    action: Select the focused option.
    initial_state: no_selection
    expect:
      state: first_selected
      event: ChangeRequested(first)
      focus_target: radio
  - key: ArrowRight
    modifiers: []
    when: The first enabled option is focused in a horizontal group.
    action: Select and focus the next enabled option, skipping the disabled option.
    initial_state: horizontal_first_selected
    expect:
      state: second_selected
      event: ChangeRequested(second)
      focus_target: radio
accessibility:
  role: radiogroup
  properties:
    - name: aria-orientation
      value: vertical
    - name: aria-disabled
      value: true
      when: disabled
---

# Radio group

## Purpose

Choose exactly one option from a related set.

## Anatomy

Named group label and vertically arranged radio indicators with option labels.

## States

One enabled option selected, another selected, optional no selection, and disabled group/individual options. Arrow navigation skips disabled options and wraps.

The screenshot fixtures use First, Unavailable (disabled), and Second options. The first and second fixtures select the corresponding enabled option; the disabled fixture disables the entire group with First selected. The controlled fixture starts with First selected and shows that authoritative value before owner apply. The no-selection fixture leaves every option unchecked. The horizontal fixture selects First and places options along the row. Each declared fixture is captured in shadcn light, dark, and high contrast at 1× and 2×.

## Props and events

`label`, stable `(id,label,disabled)` options, `value` or `default_value`, `disabled`, `orientation` (vertical by default). A `None` value is allowed. Controlled mode emits `ChangeRequested(id)` and keeps the current value until its owner calls `set_value`; uncontrolled mode updates selected id. No prop-driven events. A disabled group suppresses selection requests and keyboard navigation; a disabled option is skipped.

## Keyboard map

Up/Down follow vertical orientation; Left/Right follow horizontal orientation; the active pair wraps and skips disabled options. Home/End select first/last enabled option; Space selects the focused option. Each enabled option has roving tabindex, with the selected enabled option as initial tab stop, or the first enabled option if selection is absent. Tab enters at that stop and exits the group. Register `RadioGroup` key context and named actions with replaceable default bindings.

## Pointer behaviour

Click option or its label selects it and focuses that option. Controlled clicks request a value without changing the rendered selection until `set_value`; disabled options and disabled groups ignore clicks.

## Accessibility role and properties

Group has radiogroup role/name and orientation. Each child has radio role/name, checked and disabled state. Roving focus exposes one tab stop among enabled options; a disabled group exposes no tab stops.

## Theme tokens used
The look follows the shadcn/ui radio group and is resolved from the installed `Theme` in three
variants, the same way Tabs does it. `high-contrast` is selected by theme name; every other theme
is dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light.
Derived colours use `mkit_core::contrast::composite`; no new core tokens are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Circle border, unselected ("input") | `border` | `text` at 15% | `border` |
| Circle border, selected | `accent` | `accent` | `accent` |
| Circle fill | `background` | `background` | `background` |
| Selected dot | `accent` | `accent` | `accent` |
| Circle shadow | `shadows.small` | `shadows.small` | `shadows.small` (transparent in this theme) |
| Group and option labels | `text` | `text` | `text` |
| Keyboard focus | circle border `focus` plus a 3px ring of `focus` at 50% | same | circle border `focus` plus a 3px ring of opaque `focus` |
| Disabled option or group | circle, dot and label at 50% over `background` | same | same |

Geometry: each circle is `spacing.large` (16px) with radius `radii.pill` and a `borders.hairline`
border, matching shadcn/ui's `size-4 rounded-full border`; the selected dot is `spacing.small`
(8px), shadcn/ui's `size-2`. The selected circle takes an `accent` border as well as the dot (the
classic shadcn/ui radio), so selection does not rest on the 8px dot alone. The circle fill is
opaque `background` because GPUI paints the drop shadow as a filled shape inside the element. The
group label and option labels use `typography.body` at medium weight (500), shadcn/ui's
`font-medium`; rows are separated by `spacing.small` because GPUI's default line height already
makes each row taller than the web preview's. The 3px focus ring is the shadcn/ui ring width and a
fixed component value. Each option row keeps a trailing `spacing.xsmall` of padding so its pointer
target stays at least as wide as the earlier 16.2px indicator row.

Disabled matches the web preview's `opacity: .5` applied to the control as one layer: every part's
colour is composited opaque over `background` and then mixed 50% with it, and the shadow alpha is
halved. GPUI element opacity is not used because it dims each painted part separately, so a thumb
or mark would show the part beneath it.

## WAI-ARIA pattern reference

[Radio Group](https://www.w3.org/WAI/ARIA/apg/patterns/radio/).

## Platform notes

Arrow behavior follows APG desktop radio groups.

## Open questions

None. Default orientation is vertical, and no-selection is permitted by the public API.
