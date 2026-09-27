---
spec_version: 1
component: disclosure
states:
  - id: collapsed
    description: Trigger is focused and the labelled content is hidden.
    fixture: collapsed_fixture
  - id: expanded
    description: Labelled content is visible and trigger reports expanded state.
    fixture: expanded_fixture
  - id: disabled
    description: Trigger is unavailable and cannot change expanded state.
    fixture: disabled_fixture
keys:
  - key: Enter
    modifiers: []
    when: Disclosure trigger is focused and enabled.
    action: Toggle expanded state.
    initial_state: collapsed
    expect:
      event: ExpandedChanged
  - key: Space
    modifiers: []
    when: Disclosure trigger is focused and enabled.
    action: Toggle expanded state.
    initial_state: collapsed
    expect:
      event: ExpandedChanged
accessibility:
  role: button
  properties:
    - name: expanded
      value: false
      when: collapsed
    - name: panel
      value: stable content id
    - name: disabled
      value: true
      when: disabled
---

# Disclosure

## Purpose

Show and hide one optional labelled region so dense app panels stay scannable.

## Anatomy

A focusable button trigger and a content panel with a stable ID. The panel is omitted while
collapsed and rebuilt from its content factory when expanded. Stateful content should be owned by
an application entity outside the panel.

## States

Collapsed, expanded, and disabled. Disabled disclosure ignores pointer and keyboard input. A
controlled disclosure can display either collapsed or expanded state.

## Props and events

`Disclosure::new(id, label, content_factory)` creates uncontrolled state, initially collapsed;
`.expanded(bool)` selects its initial state. `Disclosure::controlled(id, label, expanded,
content_factory)` leaves state authoritative to the owner. Uncontrolled toggles update local state
and emit `ExpandedChanged(bool)`. Controlled toggles emit the proposed value and wait for
`set_expanded`. Programmatic replacement emits no event; disabled and no-op interactions emit no
event. The content factory returns a new GPUI element each render.

## Keyboard map

The `Disclosure` key context registers the `Toggle` action on Enter and Space. Either key toggles
the focused enabled trigger. Tab follows platform order through the trigger and, when open, its
content descendants. No arrow-key navigation is bound.

## Pointer behaviour

Clicking the trigger toggles its panel. Disabled triggers ignore clicks.

## Accessibility role and properties

The trigger has button role, its label as accessible name, and expanded state. The panel has group
role, the trigger label, and a stable panel ID; it is omitted from the tree while collapsed. The
disabled trigger is removed from tab traversal and exposed as unavailable. GPUI 0.3.5 has no
`aria_controls` builder, so the trigger-to-panel relation cannot currently be emitted.

## Theme tokens used

Read surface, text, disabled text, spacing, typography, control height, border, and radius from the
mkit-core Global `Theme`. No component colors or fixed sizes are introduced. Motion is deferred:
there is no animation builder/path in this implementation. A future opt-in chevron animation can
use shared motion duration and easing tokens; panel visibility should still change immediately.

## WAI-ARIA pattern reference

Follow the [APG Disclosure pattern]: Enter and Space activate the header button, whose expanded
property reflects panel visibility.

## Platform notes

GPUI maps the trigger's button role and expanded state through AccessKit. Relation metadata is
limited by the lack of an `aria_controls` builder. A collapsed panel is omitted, so controls inside
it cannot remain focusable.

## Open questions

- Maintainers should review public constructor and builder naming.
- Confirm how to expose trigger-to-panel relation when GPUI adds supported AccessKit metadata.
- Add optional chevron motion after the shared E4.4 motion API is available.

[APG Disclosure pattern]: https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/
