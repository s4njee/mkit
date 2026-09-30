---
spec_version: 1
component: link
states:
  - id: enabled
    description: Focusable link with a visible underline and activation callback.
    fixture: enabled_fixture
  - id: hover
    description: Hovered enabled link with increased visual emphasis.
    fixture: hover_fixture
  - id: focused
    description: Keyboard-focused link with visible focus treatment.
    fixture: focused_fixture
  - id: visited
    description: Link explicitly marked visited by its owner.
    fixture: visited_fixture
  - id: disabled
    description: Unavailable link that cannot be focused or activated.
    fixture: disabled_fixture
keys:
  - key: Enter
    modifiers: []
    when: Enabled and focused
    action: Dispatch the rebindable Activate action to invoke the callback.
    initial_state: enabled
    expect:
      event: activate
accessibility:
  role: link
  properties:
    - name: label
      value: visible text or supplied accessible name
      when: enabled
    - name: disabled
      value: true
      when: disabled
---

# Link

## Purpose

Present navigable text with consistent theme styling and semantic link behavior.

## Anatomy

A visible text label is underlined and can receive focus when it has an activation callback. The application owns destination resolution and navigation.

## States

- `Enabled`: focusable, clickable, and Enter-activatable when a callback is supplied.
- `Hover`: enabled link under pointer.
- `Focused`: visible keyboard focus ring.
- `Visited`: owner-provided visited state changes color without implying navigation history storage.
- `Disabled`: muted and neither focusable nor activatable.

## Props and events

Stateless `RenderOnce` builder. Props include visible label, optional accessible name, visited/disabled flags, optional stable identity for parent rerenders, and optional activation callback. A callback is required for focus and activation; without it the node is not focusable or activatable. No built-in URL opening or history behavior is provided.

## Keyboard map

`Enter` invokes the callback when enabled and focused. Space retains normal text navigation behavior and does not activate. Enter is bound to `Activate` in the `MkitLink` key context so applications may rebind it.

## Pointer behaviour

Primary click invokes the callback when enabled. Disabled links ignore clicks.

## Accessibility role and properties

Exposes the `Link` role and an accessible name from visible text unless overridden. Disabled links expose disabled state and are removed from tab order. The parent must provide a meaningful label and callback for navigation semantics.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts`, styled by `.e7-link*` in
`site/src/demos/e7_expansion.css` with the shadcn token mapping in `site/src/ui/tokens.ts`). It is
resolved from the installed `Theme` the same way Button, Select, and Tabs do it. `high-contrast` is
selected by theme name; the light and dark themes share one set of rules because no link colour
depends on the background's luminance. Derived colours use a crate-local `color-mix` helper built on
`mkit_core::contrast::composite`; no mkit-core API or tokens are added.

| State | Light / dark | High contrast |
|---|---|---|
| Enabled text and underline | `accent` (shadcn `text-primary`) | `accent` |
| Hover | `text_muted` (the preview's `hover:text-muted-foreground`); a visited link hovers to `text` | `focus` |
| Visited | `text_muted` | `text_muted` |
| Keyboard focus | border `focus` plus a 3px ring of `focus` at 50%, over a `background` fill | border `focus` plus a 3px ring of opaque `focus` |
| Disabled | the resting colour mixed 50% over `background` | `disabled` |

- **Visited.** The preview's muted link uses `text`, but in the shadcn themes `text` and `accent`
  are both near-black (light) or near-white (dark), so a visited link uses `text_muted` to stay
  distinguishable.
- **Focus.** The ring follows `:focus-visible` and replaces nothing else; GPUI paints drop shadows as
  filled shapes that are not clipped to the element's outside, so the link draws an opaque
  `background` fill while focused. The resting border is transparent, so focus does not shift the
  layout. Ring corners use the link radius rather than CSS's radius-plus-spread.
- **Disabled** matches shadcn's `opacity: .5` as one layer, the way Select and TextField do it: the
  text colour is composited over `background` and mixed 50% with it; GPUI element opacity is not
  used. High contrast keeps the solid `disabled` colour.
- **Geometry.** Text is `typography.body` (14px) with a GPUI underline in the text colour; GPUI has
  no underline-offset style, so the preview's `text-underline-offset: 4px` is not reproduced. The
  link has a `borders.hairline` border for the focus treatment and radius `radii.small`, with
  `spacing.xsmall` horizontal padding offset by an equal negative margin, so the focus ring clears
  the text while the text stays where an unpadded link would put it. The
  preview's trailing chevron is decorative page content, not part of the Link API. The 3px focus
  ring is the shadcn/ui ring width, a fixed component value.

## WAI-ARIA pattern reference

ARIA link role guidance; links activate with Enter and remain distinct from buttons that perform actions.

## Platform notes

Pointer hover and focus-visible presentation use GPUI behavior. Destination opening remains application-specific.

## Open questions

- Maintainer review is needed for the callback-based link contract and disabled link semantics.
