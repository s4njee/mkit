---
spec_version: 1
component: scroll-area
states:
  - id: idle
    description: A scrollable region at its initial offset.
    fixture: idle_fixture
  - id: scrolled
    description: A scrollable region offset one theme-token line (32 px in the light-theme fixture) below the top.
    fixture: scrolled_fixture
  - id: focused
    description: The viewport at its initial offset with keyboard focus from Tab, showing the focus border and ring.
    fixture: focused_fixture
keys:
  - key: PageDown
    modifiers: []
    when: Scroll area has focus and content exceeds viewport.
    action: Scroll one viewport downward.
    initial_state: idle
    expect:
      state: scrolled
  - key: PageUp
    modifiers: []
    when: Scroll area has focus and is scrolled one line below the top.
    action: Scroll one viewport upward, clamping at the top.
    initial_state: scrolled
    expect:
      state: idle
  - key: End
    modifiers: []
    when: Scroll area has focus and content exceeds viewport.
    action: Scroll to the end of the content.
    initial_state: idle
    expect:
      state: scrolled
  - key: Home
    modifiers: []
    when: Scroll area has focus and is scrolled one line below the top.
    action: Scroll to the beginning of the content.
    initial_state: scrolled
    expect:
      state: idle
  - key: ArrowDown
    modifiers: []
    when: Scroll area has focus and content exceeds viewport.
    action: Scroll one theme-token line downward.
    initial_state: idle
    expect:
      state: scrolled
  - key: ArrowUp
    modifiers: []
    when: Scroll area has focus and is scrolled one line below the top.
    action: Scroll one theme-token line upward, clamping at the top.
    initial_state: scrolled
    expect:
      state: idle
accessibility:
  role: scroll-view
  properties: []
---

# Scroll area

## Purpose

Give a panel a bounded, independently scrollable viewport.

## Anatomy

A bordered viewport containing caller supplied content. Overlay scrollbar thumbs appear along the trailing (vertical) and bottom (horizontal) inner edges when the content overflows on that axis.

## States

Idle, focused, and scrolled; content and viewport dimensions determine overflow. The visual fixtures render the same eight-row activity list in a bounded viewport. The scrolled fixture advances the shared `ScrollHandle` by one theme-token line after initial layout, so the two captured states show a real content offset. The focused fixture moves keyboard focus to the viewport with Tab, so it captures the keyboard focus treatment.

## Props and events

`height`, optional `width`, `id`, optional reusable `ScrollHandle`, and child content; no component events. Stateless `RenderOnce` builder. Callers retain the handle and ID across rerenders to retain scroll position.

## Keyboard map

Page Up/Down move by one viewport, Home/End move to the top/bottom, and arrows move by twice `Theme.spacing.large` (32 px in the light-theme fixture). They clamp at the scroll bounds. The `MkitScrollArea` key context exposes named scroll actions so applications can rebind them.

## Pointer behaviour

Wheel and trackpad scrolling use GPUI scrolling, including platform momentum. The scrollbar thumbs are position indicators only: GPUI draws no native scrollbars, and the thumbs do not take pointer input, so dragging a thumb does nothing and the pointer events reach the content beneath.

## Accessibility role and properties

Expose a named scroll-view region when `label` is supplied. Content retains its own semantics.

## Theme tokens used

`Theme.colors.surface/border/text/focus`, `Theme.borders.hairline`, `Theme.radii.medium`, and
`Theme.spacing.small/large`.

The look follows the shadcn/ui scroll area (a `rounded-md border` viewport with a thin, rounded
`bg-border` thumb) with the docs-site token mapping in `site/src/ui/tokens.ts`. The docs-site
preview itself is a card with the browser's native scrollbar, so the thumb geometry comes from
shadcn/ui. Colours are resolved from the installed `Theme` in three variants: `high-contrast` is
selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. No mkit-core API
or tokens are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Viewport fill | `surface` | `surface` | `surface` |
| Viewport border | `border` | `text` at 10% (shadcn dark `--border`) | `border` |
| Thumb | `border` | `text` at 10% | `border` (white) |
| Keyboard focus | border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |

- **Geometry.** The viewport has radius `radii.medium` and a `borders.hairline` border. Each track
  is 10px thick (shadcn `w-2.5`): an 8px (`spacing.small`) thumb with a `borders.hairline` gap on
  either side (shadcn `p-px`), inside the border. Thumbs are fully rounded (`rounded-full`). A
  thumb's length is the visible fraction of the track, but never shorter than `spacing.large`
  (16px) so a very long document still has a visible thumb. When both axes overflow, each thumb stops
  short of the other's track.
- **Visibility.** Thumbs show whenever the content overflows on that axis. shadcn's default Radix
  `type="hover"` hides them until the pointer is over the viewport; always showing them keeps the
  scroll position visible to keyboard users and in screenshots.
- **Focus** follows `:focus-visible`: the border and ring show while the viewport has focus and the
  last input was from the keyboard. The viewport fill is opaque, so the ring (a GPUI drop shadow) is
  not visible through it. Ring corners use the viewport radius rather than CSS's radius-plus-spread.
  The 3px ring is the shadcn/ui ring width, a fixed component value.
- The thumbs are painted by a zero-size overlay canvas against the tracked `ScrollHandle` bounds, so
  they add nothing to the scrollable content size and do not move with the content.

## WAI-ARIA pattern reference

Follow the platform scroll-view convention; there is no dedicated WAI-ARIA scroll-area pattern.

## Platform notes

GPUI handles momentum. GPUI does not render native scrollbars, so the component draws its own thumbs on every platform.

## Open questions

Confirm keyboard action wiring on all supported platforms.

Whether the thumbs should become draggable scrollbars, and whether they should hide until hover as in shadcn's default, needs maintainer review; both are pointer-behaviour changes beyond the visual pass.
