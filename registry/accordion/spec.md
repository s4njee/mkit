---
spec_version: 1
component: accordion
states:
  - id: none_open
    description: All enabled accordion items are collapsed.
    fixture: none_open_fixture
  - id: one_open
    description: Exactly one item is expanded in single mode, or one item is expanded in multiple mode.
    fixture: one_open_fixture
  - id: multiple_open
    description: Multiple items are expanded when multiple mode is enabled.
    fixture: multiple_open_fixture
  - id: disabled_item
    description: Disabled item retains its current state and ignores activation.
    fixture: disabled_item_fixture
  - id: focused
    description: All items are collapsed and the first trigger has keyboard focus with its focus ring.
    fixture: focused_fixture
keys:
  - key: Enter
    modifiers: []
    when: An enabled accordion trigger is focused.
    action: Toggle that item's expanded state.
    initial_state: one_open
    expect:
      event: ExpandedChanged
  - key: Space
    modifiers: []
    when: An enabled accordion trigger is focused.
    action: Toggle that item's expanded state.
    initial_state: one_open
    expect:
      event: ExpandedChanged
  - key: Tab
    modifiers: []
    when: A trigger or expanded panel descendant is focused.
    action: Move to the next focusable control in normal order.
    initial_state: one_open
    expect:
      focus_target: next_focusable
  - key: Tab
    modifiers: [Shift]
    when: A trigger or expanded panel descendant is focused.
    action: Move to the previous focusable control in normal order.
    initial_state: one_open
    expect:
      focus_target: previous_focusable
accessibility:
  role: group
  properties:
    - name: item-trigger-role
      value: button
    - name: expanded
      value: per-item boolean
    - name: panel
      value: corresponding stable panel id
    - name: disabled
      value: true
      when: disabled_item
---

# Accordion

## Purpose

Group related disclosures under a shared single-open or multiple-open policy.

## Anatomy

A named group of items. Every item has a labelled button trigger and a stable-ID content panel.
The panel is omitted while collapsed and rebuilt from its content factory when expanded.

## States

No items open, one item open, multiple items open in multiple mode, and disabled items. Single mode
allows zero or one open item. Disabled items retain state and ignore interaction.

## Props and events

Each `Item` has a unique ID, label, content factory, initial `expanded` state, and optional
`disabled` state. `Accordion::new(items, Mode)` is uncontrolled and uses item initial states.
`Accordion::controlled(items, Mode, expanded_ids)` leaves state owner-controlled. Uncontrolled
interaction updates the local open IDs and emits `ExpandedChanged(Vec<String>)`; controlled
interaction emits the proposed full ID set and waits for `set_expanded`. Replacing state emits no
event. IDs not present in the item list are discarded, duplicates are removed, and single mode
keeps the first supplied open ID. Activating the open item closes it; an item is not required to
remain open.

## Keyboard map

The `Accordion` key context registers `Toggle` on Enter and Space. Either key toggles the focused
enabled trigger. Tab follows platform order through triggers and expanded descendants. Arrow keys,
Home, and End are not bound because APG header navigation is optional and standard Tab order is
used here.

## Pointer behaviour

Clicking an enabled trigger toggles its item. In single mode, opening one item closes the previous
item. In multiple mode, other open items stay open. Disabled items ignore clicks.

## Accessibility role and properties

The root has group role and the name “Accordion”. Triggers have button role, item label, and
expanded state. Panels have group role, item label, and stable panel IDs; they are omitted while
collapsed. Disabled triggers are unavailable and removed from tab traversal. GPUI 0.3.5 does not
provide `aria_controls`, so the trigger-to-panel relation cannot currently be emitted.

## Theme tokens used

The docs site shows Accordion on the Disclosure page without a separate web preview, so the look
follows shadcn/ui's Accordion and shares Disclosure's trigger, chevron and focus treatment. It is
resolved from the installed `Theme` in three variants. `high-contrast` is selected by theme name
(the convention other registry components use); every other theme is treated as dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived
colours use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no
mkit-core API or tokens are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Item separator | `border` | `text` at 10% over `background`, composited opaque | `border` |
| Trigger fill | `background` | `background` | `background` |
| Trigger label | `text`, medium weight | same | same |
| Trigger hover | underline (shadcn `hover:underline`) | same | same |
| Chevron | `text_muted` | `text_muted` | `text` |
| Panel text | `text` | `text` | `text` |
| Disabled label, chevron | `text`, `text_muted` at 50% over `background` | same | solid `disabled` |
| Focus (`focus_visible`) | border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |

- **Items.** The accordion has no card or outer border. Each item except the last ends with a
  `borders.hairline` bottom border (shadcn `border-b last:border-b-0`). The item wrapper has no
  role, so the accessibility tree is unchanged.
- **Trigger.** Full width with no horizontal padding (shadcn), `controls.large + spacing.medium`
  tall (52px: shadcn's `py-4` around a 20px line), label and chevron separated by at least
  `spacing.large` (`gap-4`). The label is `typography.body` (14px) at medium weight (500),
  shadcn's `font-medium`; there is no font-weight token yet. A transparent `borders.regular` border
  with `radii.medium` corners is reserved for focus so focusing does not shift layout.
- **Chevron.** A `spacing.large` (16px) Lucide `chevron-down` (6,9 → 12,15 → 18,9 on a 24-unit
  grid) drawn as a vector path while collapsed and the half-turned path (6,15 → 12,9 → 18,15) while
  open, since GPUI cannot rotate elements. It strokes at Lucide's 2/24 of its size in light and
  dark and at `borders.regular` in high contrast, and is decorative.
- **Panel.** Bottom padding `spacing.large` (shadcn `pb-4`) and a `borders.regular` side inset
  that matches the trigger's reserved focus border, so text lines up; `typography.body` text.
- **Disabled.** shadcn uses `opacity: .5`; GPUI applies element opacity per painted part, so the
  label and chevron colours are mixed 50% over `background` instead. High contrast keeps solid
  `disabled`. The panel of an expanded disabled item keeps its normal text.
- **Focus.** GPUI paints drop shadows as filled shapes that are not clipped to the element's
  outside, so the trigger has an opaque `background` fill under the ring; the accordion is meant to
  sit on the window background. Ring corners use the trigger radius. The 3px ring is the shadcn/ui
  ring width, a fixed component value.

Motion is deferred: there is no animation builder/path in this implementation. The chevron swaps
orientation immediately and panel visibility changes immediately.

## WAI-ARIA pattern reference

Follow the [APG Accordion pattern]. Triggers use button semantics and expose their expanded state.
Arrow-key header movement is optional in APG and is not implemented by this component.

## Platform notes

GPUI maps button role and expanded state through AccessKit. Stable panel IDs are present, but
relation metadata is limited by the missing `aria_controls` builder. Collapsed panels are omitted
so hidden descendants cannot receive focus.

## Open questions

- Maintainers should review public item/content builder naming and duplicate-ID normalization.
- Confirm how to expose trigger-to-panel relation when GPUI adds supported AccessKit metadata.
- Add optional chevron motion after the shared E4.4 motion API is available.
- The docs site has no dedicated Accordion web preview; confirm the shadcn Accordion look.

[APG Accordion pattern]: https://www.w3.org/WAI/ARIA/apg/patterns/accordion/
