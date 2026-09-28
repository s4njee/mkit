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
    - name: controls
      value: not emitted; pinned GPUI exposes no element-to-element relation API
    - name: motion
      value: optional panel fade from theme Open duration; none when motion is reduced
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

The trigger and panel are also exported as stateless `RenderOnce` parts, `DisclosureTrigger` and
`DisclosurePanel`, so composite components that own their own expansion state (for example
PropertyInspector group rows) can reuse the same trigger semantics, key context, and panel
structure. The `Disclosure` entity renders through these parts. Both parts implement GPUI `Styled`;
caller refinements are applied after the token defaults so a composite can adjust surface or
padding without re-implementing the trigger.

## States

Collapsed, expanded, and disabled. Disabled disclosure ignores pointer and keyboard input. A
controlled disclosure can display either collapsed or expanded state.

## Props and events

`Disclosure::new(id, label, content_factory)` creates uncontrolled state, initially collapsed;
`.expanded(bool)` selects its initial state. `Disclosure::controlled(id, label, expanded,
content_factory)` leaves state authoritative to the owner. Uncontrolled toggles update local state
and emit `ExpandedChanged(bool)`. Controlled toggles emit the proposed value and wait for
`set_expanded`. Programmatic replacement emits no event; disabled and no-op interactions emit no
event. The content factory returns a new GPUI element each render. `.motion(bool)` opts in to the
panel reveal transition described under Theme tokens; it is off by default.

`DisclosureTrigger::new(id, label)` takes `.expanded(bool)`, `.disabled(bool)`,
`.track_focus(&FocusHandle)` (otherwise the trigger gets an element-owned focus handle through its
tab index), and `.on_toggle(Fn(&mut Window, &mut App))`. It emits no entity event: the owner
decides what a toggle means. `DisclosurePanel::new(id, label)` is a `ParentElement` with an
optional `.motion(bool)`. These parts are a draft public API addition for maintainer review.

## Keyboard map

The `Disclosure` key context is registered on the trigger element and binds the `Toggle` action to
Enter and Space. Either key toggles the focused enabled trigger. Because the context is on the
trigger rather than the whole disclosure, Enter and Space pressed on a control inside the panel go
to that control and do not collapse the panel. A composite that renders `DisclosureTrigger` inherits
the same context and rebindable action. Tab follows platform order through the trigger and, when open, its
content descendants. No arrow-key navigation is bound.

## Pointer behaviour

Clicking the trigger toggles its panel. Disabled triggers ignore clicks.

## Accessibility role and properties

The trigger has button role, its label as accessible name, and expanded state. The panel has group
role, the trigger label, and a stable panel ID; it is omitted from the tree while collapsed. The
disabled trigger is removed from tab traversal and exposed as unavailable.

The APG `aria-controls` relation cannot be emitted with pinned GPUI 0.3.5, and this component
documents that limit instead of approximating it. AccessKit 0.24 supports `Node::set_controls`, but
GPUI offers no `aria_controls` builder, derives each element's `NodeId` from a crate-private
`GlobalElementId::accesskit_node_id`, and its public `A11ySubtreeBuilder` can only mint IDs for
synthetic children of the current element. The trigger therefore cannot name the panel's node. As
far as the API allows, the relationship is conveyed structurally: the panel is the trigger's next
sibling in the tree, carries a stable element ID (`disclosure-panel-{id}`), and has the trigger's
label as its accessible name. When GPUI exposes a relation builder or public node IDs, the trigger
should set `controls` to the panel node.

## Theme tokens used

Read surface, text, disabled text, spacing, typography, control height, border, and radius from the
mkit-core Global `Theme`. No component colors or fixed sizes are introduced.

Motion is optional and off by default. With `.motion(true)`, expanding fades the panel's opacity
from 0 to 1 over the E4.4 `TransitionKind::Open` duration (`Theme.motion.normal_ms`) using
`mkit_core::motion::transition_animation`. Panel presence, focusability, and the accessibility
tree change immediately; only paint opacity animates. Collapsing is immediate because the panel is
omitted. No animation element is created when `App::reduce_motion` is set or the resolved theme
duration is zero (the high-contrast theme sets motion tokens to zero), and GPUI's
`with_animation` independently renders the end state under reduced motion. The chevron glyph stays
static because GPUI text elements cannot be rotated.

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
- Review the `DisclosureTrigger`/`DisclosurePanel` part API and the `Styled` override approach.
- Decide whether motion should also animate panel height once GPUI exposes measured layout
  interpolation; the draft fades opacity only.

[APG Disclosure pattern]: https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/
