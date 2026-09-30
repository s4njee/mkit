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
  - id: focused
    description: The collapsed trigger has keyboard focus and shows the focus ring.
    fixture: focused_fixture
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

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts`, styled by `.ui-card`
in `site/src/ui/ui.css` and `.e7-disclosure*` in `site/src/demos/e7_expansion.css`, with the shadcn
token mapping in `site/src/ui/tokens.ts`) and shadcn/ui's Collapsible/Accordion trigger. It is
resolved from the installed `Theme` in three variants. `high-contrast` is selected by theme name
(the convention other registry components use); every other theme is treated as dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived
colours use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no
mkit-core API or tokens are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Card fill (shadcn `card`), also the trigger fill | `surface` | `surface` | `background` |
| Card border | `border` | `text` at 10% over `surface`, composited opaque | `border` |
| Trigger label | `text`, medium weight | same | `text`, medium weight |
| Trigger hover | label turns `text_muted` (the preview's hover) | same | label underlined |
| Chevron | `text_muted` | `text_muted` | `text` |
| Panel text | `text` | `text` | `text` |
| Disabled label, chevron | `text`, `text_muted` at 50% over the card fill | same | solid `disabled` |
| Focus (`focus_visible`) | border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |

- **Card.** The `Disclosure` entity draws the preview's card: radius `radii.large`, a
  `borders.regular` border and no shadow (the preview removes the card shadow). The preview's card
  radius is `radius-lg + 4px` (12px); there is no such token in the shadcn themes, so the nearest,
  `radii.large`, is used. The `DisclosureTrigger` and `DisclosurePanel` parts do not draw the card,
  so composites such as PropertyInspector keep their own surfaces.
- **Trigger.** Full width, `controls.large + spacing.xsmall` tall (44px, the preview's
  `min-height`), with the preview's 16px card padding (`spacing.large`) moved onto the trigger so
  the focus ring surrounds the whole row; the reserved focus border counts toward that inset, so the
  label lines up with the panel text. Label and chevron are separated by at least
  `spacing.large` (shadcn `gap-4`). The label is `typography.body` (14px) at medium weight (500),
  shadcn's `font-medium`; there is no font-weight token yet. A transparent `borders.regular` border
  with `radii.medium` corners is reserved for focus so focusing does not shift layout.
- **Chevron.** A `spacing.large` (16px) Lucide `chevron-down` (6,9 → 12,15 → 18,9 on a 24-unit
  grid) drawn as a vector path while collapsed. shadcn rotates it half a turn while open; GPUI cannot
  rotate elements, so the expanded state draws the turned path (6,15 → 12,9 → 18,15) instead. The
  web preview shows its single expanded sample without the rotation. The chevron strokes at
  Lucide's 2/24 of its size in light and dark and at `borders.regular` in high contrast, and is
  decorative (no accessibility node).
- **Panel.** Padding `spacing.large` left, right and bottom and none on top, so the body text
  lines up with the label (the preview uses a 14px bottom padding; the shadcn content uses `pb-4`,
  16px, which is the nearest token). Text is `typography.body` in `text`.
- **Disabled.** The web preview and shadcn use `opacity: .5`. GPUI applies element opacity to each
  painted part separately, so the label and chevron colours are instead mixed 50% over the opaque
  card fill. High contrast keeps solid `disabled` so unavailable triggers stay legible.
- **Focus.** GPUI paints drop shadows as filled shapes that are not clipped to the element's
  outside, so the trigger always has an opaque fill (the card fill) under the ring. Ring corners use
  the trigger radius rather than CSS's radius-plus-spread. The 3px ring is the shadcn/ui ring width,
  a fixed component value.

Motion is optional and off by default. With `.motion(true)`, expanding fades the panel's opacity
from 0 to 1 over the E4.4 `TransitionKind::Open` duration (`Theme.motion.normal_ms`) using
`mkit_core::motion::transition_animation`. Panel presence, focusability, and the accessibility
tree change immediately; only paint opacity animates. Collapsing is immediate because the panel is
omitted. No animation element is created when `App::reduce_motion` is set or the resolved theme
duration is zero (the high-contrast theme sets motion tokens to zero), and GPUI's
`with_animation` independently renders the end state under reduced motion. The chevron swaps
orientation immediately; it does not animate.

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
