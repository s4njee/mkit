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

Colors, borders, spacing, control height, and typography come from the mkit-core Global `Theme`.
Motion is deferred: there is no animation builder/path in this implementation. A future opt-in
chevron animation can use shared motion tokens; panel visibility changes immediately. Separator and
content insets use theme tokens.

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

[APG Accordion pattern]: https://www.w3.org/WAI/ARIA/apg/patterns/accordion/
