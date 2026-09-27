---
spec_version: 1
component: toggle-group
states:
  - id: selection
    description: Enabled group with zero or one selected item.
    fixture: toggle_group_selection
  - id: disabled
    description: Group cannot change selection.
    fixture: toggle_group_disabled
screenshots:
  themes: [shadcn-light, shadcn-dark, high-contrast]
  scales: [1, 2]
  matrix: every state × theme × scale
keys:
  - key: ArrowRight
    modifiers: []
    when: a group item is focused in a horizontal group
    action: Move focus to and select the next enabled item, wrapping at the end.
    initial_state: selection
    expect:
      event: value_changed
      focus_target: next_enabled_item
  - key: ArrowLeft
    modifiers: []
    when: a group item is focused in a horizontal group
    action: Move focus to and select the previous enabled item, wrapping at the beginning.
    initial_state: selection
    expect:
      event: value_changed
      focus_target: previous_enabled_item
  - key: ArrowDown
    modifiers: []
    when: a group item is focused in a vertical group
    action: Move focus to and select the next enabled item, wrapping at the end.
    initial_state: selection
    expect:
      event: value_changed
      focus_target: next_enabled_item
  - key: ArrowUp
    modifiers: []
    when: a group item is focused in a vertical group
    action: Move focus to and select the previous enabled item, wrapping at the beginning.
    initial_state: selection
    expect:
      event: value_changed
      focus_target: previous_enabled_item
  - key: Home
    modifiers: []
    when: a group item is focused
    action: Move focus to and select the first enabled item.
    initial_state: selection
    expect:
      event: value_changed
      focus_target: first_enabled_item
  - key: End
    modifiers: []
    when: a group item is focused
    action: Move focus to and select the last enabled item.
    initial_state: selection
    expect:
      event: value_changed
      focus_target: last_enabled_item
accessibility:
  role: group
  properties:
    - name: label
      value: Group label
      when: selection
    - name: aria-disabled
      value: true
      when: disabled
---

# Toggle group

## Purpose

A related set of mutually exclusive on/off controls, such as text alignment choices.

## Anatomy

A labelled group containing toggle items. Each item has a stable value, label, and disabled state.

## States

`selection` and `disabled`. Selection is optional; empty selection is allowed. Per-item disabled values are skipped during keyboard movement.

## Screenshot matrix

The gallery captures each declared state (`selection`, `disabled`) with the shadcn light and dark
themes and the high-contrast theme at 1x and 2x. The JSON conformance manifest names each baseline;
the matrix test reads those cases and fails if a declared case has no captured image.

## Props and events

Stateful `Entity<ToggleGroup>` with typed `ValueChanged(Option<String>)`. `new(label, items, default_value)` is uncontrolled and updates on activation. `controlled(label, items, value)` emits requested next values while retaining the authoritative value until `set_value`. Single selection only; no deselection once selected unless `allow_empty` is enabled. Orientation may be horizontal or vertical.

## Keyboard map

One item participates in tab order. Arrows move focus and select the next enabled item with wrapping (Right/Left for horizontal, Down/Up for vertical); the cross-axis arrows leave the group unchanged. Home and End move to and select the first and last enabled item. Bindings live in the `MkitToggleGroup` key context for host rebinding.

## Pointer behaviour

Primary click on an enabled item requests selection. Disabled items and a disabled group ignore pointer input.

## Accessibility role and properties

Use a labelled group containing button-role toggle items with toggled state mapped to `aria-pressed`. A disabled group and its items set the AccessKit disabled property; disabled items also announce an unavailable description. Focus stays on the active item.

## Theme tokens used

`Theme.colors` surface/text/border/accent/accent_text/focus/disabled; `Theme.spacing.small`; `Theme.radii.medium`; `Theme.borders.regular/hairline`; `Theme.controls.medium`; `Theme.typography.body`.

## WAI-ARIA pattern reference

[Toolbar Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/) informs roving focus; toggle items use the [Button Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/). This group is single selection rather than a toolbar command set.

## Platform notes

Orientation determines arrow navigation. Host applications may rebind the named direction actions. The group selects as focus moves, a documented toolbar-pattern deviation for a mutually exclusive choice set.

## Open questions

Maintainer review should confirm whether empty-selection is the desired default for all future use cases and whether selecting during arrow navigation matches the intended accessibility contract.
