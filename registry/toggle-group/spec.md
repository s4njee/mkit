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

The look follows the docs-site web preview (`site/src/demos/e7.ts`, styled by `.ui-toggle-group--outline` in
`site/src/ui/ui.css` with the shadcn token mapping in `site/src/ui/tokens.ts`) and is resolved from
the installed `Theme` in three variants. `high-contrast` is selected by theme name (the convention
other registry components use); every other theme is treated as dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" below is shadcn's `secondary`/`accent`/`muted`: `text` mixed 4% (light) or
12% (dark) into `background`.

The group renders shadcn's outline toggle group: items are joined edge to edge, each item has a
`borders.hairline` border but every item after the first drops its leading edge (left when
horizontal, top when vertical), and only the outer corners use `radii.medium`. The group draws
`shadows.small` (shadcn `shadow-xs`) and therefore an opaque `background` fill, because GPUI fills
the inside of drop shadows.

| Part | Light / dark | High contrast |
|---|---|---|
| Item border | light `border`; dark `text` at 10% | `border`; `disabled` for unavailable items |
| Unselected item | transparent, `text`; hover muted fill with `text_muted` | `background`, `text`; hover border `accent` |
| Selected item | muted fill (shadcn `accent`), `text` | `accent` fill, `accent_text` |
| Disabled | item at 50% opacity; a disabled group is dimmed once at the group level | unselected `disabled` text; selected `disabled` fill with `accent_text` |
| Focus | border `focus` plus a 3px ring of `focus` at 50%, over an opaque fill (muted or `background`) | border `focus` plus a 3px ring of opaque `focus` |
| Group shadow | `shadows.small` | `shadows.small` (transparent in this theme) |

Item geometry follows shadcn's toggle (`h-9 min-w-9 px-2`): height and minimum width
`controls.medium`, horizontal padding `spacing.small`, icon gap `spacing.small`, and
`typography.body` at medium weight (500). GPUI paints later siblings over the focus ring, so the
next item's top and bottom borders overlap the ring by up to 3px where CSS would raise the focused
item with `z-index`. The borderless default (non-outline) group in the web preview is not
implemented because `ToggleGroup` has no variant API; see open questions.

## WAI-ARIA pattern reference

[Toolbar Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/) informs roving focus; toggle items use the [Button Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/). This group is single selection rather than a toolbar command set.

## Platform notes

Orientation determines arrow navigation. Host applications may rebind the named direction actions. The group selects as focus moves, a documented toolbar-pattern deviation for a mutually exclusive choice set.

## Open questions

Maintainer review should confirm whether empty-selection is the desired default for all future use cases and whether selecting during arrow navigation matches the intended accessibility contract. Visual proposal needing an API decision: the web preview also shows a borderless default group variant; `ToggleGroup` renders only the outline look until a variant API is approved.
