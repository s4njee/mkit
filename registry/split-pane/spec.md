---
spec_version: 1
component: split-pane
states:
  - id: centered
    description: Two panels with a centered divider.
    fixture: centered_fixture
  - id: resized
    description: Divider moved from the initial ratio.
    fixture: resized_fixture
  - id: vertical-centered
    description: Vertical pane divider at the initial 50 percent ratio.
    fixture: vertical_centered_fixture
  - id: focused
    description: Horizontal divider has keyboard focus and uses the focus token.
    fixture: focused_fixture
  - id: disabled
    description: Horizontal divider is unavailable, visually muted, and excluded from focus.
    fixture: disabled_fixture
keys:
  - key: ArrowRight
    modifiers: []
    when: Horizontal split divider has focus.
    action: Increase leading pane size by one step.
    initial_state: centered
    expect:
      state: resized
      event: ratio_changed
  - key: ArrowLeft
    modifiers: []
    when: Horizontal split divider has focus.
    action: Decrease leading pane size by one step.
    initial_state: centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowRight
    modifiers: [Shift]
    when: Horizontal split divider has focus.
    action: Increase leading pane size by one fine step.
    initial_state: centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowLeft
    modifiers: [Shift]
    when: Horizontal split divider has focus.
    action: Decrease leading pane size by one fine step.
    initial_state: centered
    expect: { state: resized, event: ratio_changed }
  - key: Home
    modifiers: []
    when: Horizontal split divider has focus.
    action: Move leading pane to its minimum ratio.
    initial_state: centered
    expect: { state: resized, event: ratio_changed }
  - key: End
    modifiers: []
    when: Horizontal split divider has focus.
    action: Move leading pane to its maximum ratio.
    initial_state: centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowDown
    modifiers: []
    when: Vertical split divider has focus.
    action: Increase leading pane height by one step.
    initial_state: vertical-centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowUp
    modifiers: []
    when: Vertical split divider has focus.
    action: Decrease leading pane height by one step.
    initial_state: vertical-centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowDown
    modifiers: [Shift]
    when: Vertical split divider has focus.
    action: Increase leading pane height by one fine step.
    initial_state: vertical-centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowUp
    modifiers: [Shift]
    when: Vertical split divider has focus.
    action: Decrease leading pane height by one fine step.
    initial_state: vertical-centered
    expect: { state: resized, event: ratio_changed }
accessibility:
  role: splitter
  properties:
    - name: value-min
      value: 0
    - name: value-max
      value: 100
---

# Split pane

## Purpose

Let users resize two adjacent application panels.

## Anatomy

Leading pane, focusable divider, and trailing pane. Orientation chooses horizontal or vertical placement.

## States

Centered, resized, focused, dragging, vertical-centered, and disabled. The keyboard-focused handle uses the theme focus token with a focus ring around its grip. Disabled dims the handle and does not enter keyboard focus. Dragging changes the ratio continuously; the resized fixture represents its stable result.

## Props and events

`ratio` or `default_ratio`, `min_ratio`, `max_ratio`, `orientation`, `disabled`; typed `RatioChanged(f32)` event. Controlled mode emits a requested ratio and waits for `set_ratio`; uncontrolled mode updates itself. `set_ratio` emits no event.
Default bounds are 10–90% of the available space, keeping both panes reachable.

## Keyboard map

Horizontal: Left/Right; vertical: Up/Down. Home/End go to bounds, Shift modifies step. `MkitSplitPane` context exposes named resize actions.
Normal adjustment is 2% of the viewport; Shift adjustment is 0.5%.
Tab reaches the enabled divider in normal focus order; disabled dividers are omitted. Keyboard focus exposes the focus token on the divider.

## Pointer behaviour

Dragging the divider adjusts the ratio within bounds; releasing ends the drag. Disabled blocks drag. The pointer target is `spacing.xsmall` wide, centred on the hairline handle, and pressing it focuses the divider.
The root and divider expose stable `mkit-split-pane` and `mkit-split-pane-divider` GPUI debug selectors for pointer harness scripts.

## Accessibility role and properties

The divider exposes splitter role, name, orientation, current value, and bounds; children preserve their own semantics.

## Theme tokens used

`Theme.colors.background/border/text/focus/disabled`, `Theme.borders.hairline`,
`Theme.spacing.xsmall/medium/large`, `Theme.radii.small`.

The look follows the docs-site web preview (`site/src/demos/e7.ts`, the `split-pane` demo, with the
shadcn token mapping in `site/src/ui/tokens.ts`), which in turn follows shadcn/ui's resizable handle
with `withHandle`: a 1px line in the border colour with a small grip box centred on it. Colours are
resolved from the installed `Theme` in three variants: `high-contrast` is selected by theme name;
every other theme is dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`,
otherwise light. Derived colours use a crate-local `color-mix` helper built on
`mkit_core::contrast::composite`; no mkit-core API or tokens are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Handle line, grip fill and grip border | `border` | `text` at 10% (shadcn dark `--border`) over `surface`, made opaque | line and grip border `border` (white), grip fill `background` |
| Grip dots | `text` | `text` | `text` |
| Keyboard focus | line `focus`; grip border `focus` plus a 3px ring of `focus` at 50% | same | line and grip border `focus`, ring of opaque `focus` |
| Disabled | every colour above composited over `background` and mixed 50% with it | same | line, grip border and dots `disabled` |

- **Opaque colours.** The web preview draws the handle on a card (`surface`). Compositing the
  dark theme's 10% `text` over `surface` gives the same colour in GPUI without depending on what the
  owner draws behind the split pane, and keeps the grip opaque over two differently filled panels
  and under the focus ring.
- **Geometry.** The handle is `borders.hairline` (1px) wide in the layout, so the panels meet at a
  single line as in the web preview. A transparent pointer target `spacing.xsmall` (4px, shadcn's
  `after:w-1`) wide is centred on the line and drawn above both panels, so the handle stays as easy
  to grab as before; it shows the column-resize (row-resize when vertical) cursor. The grip is a
  12 × 18px box (the web's `width: 12px; height: 18px`): `spacing.medium` wide and `spacing.large`
  plus two hairline borders tall, rotated for a vertical split. It has radius `radii.small` (the
  web's 3px has no token; 4px is the nearest) and a `borders.hairline` border, and shows Lucide
  `grip-vertical` (`grip-horizontal` when vertical): six dots on a 24-unit grid drawn as filled
  circles in a 10px square (the grip's width inside its borders), as the web's 10px icon.
- **The grip is always shown.** shadcn makes it opt-in with `withHandle`; an opt-out would be a new
  public builder and needs maintainer approval (see Open questions).
- **Focus** follows `:focus-visible`: the focus colours show while the divider has focus and the last
  input was from the keyboard. GPUI paints drop shadows as filled shapes that are not clipped to the
  element's outside, so the grip fill under the ring is opaque. The 3px ring is the shadcn/ui ring
  width, a fixed component value; its corners use the grip radius.
- **Disabled** matches a web `opacity: .5` as one layer, the way Select and Checkbox do it: each
  colour is composited over `background` and mixed 50% with it. GPUI element opacity is not used.
  High contrast keeps solid `disabled` colours instead.
- The focusable, labelled `Splitter` node stays the hairline between the two panels in the element
  and accessibility tree order. The wider pointer target and grip are an undecorated overlay with no
  accessibility role; pressing it focuses the divider and starts a drag, as pressing the divider
  does.

## WAI-ARIA pattern reference

[Window splitter](https://www.w3.org/WAI/ARIA/apg/patterns/windowsplitter/).

## Platform notes

The initial implementation uses pointer coordinates in the rendered split bounds.

## Open questions

Confirm pointer capture behavior when dragging beyond the application window.

Whether to add a builder that hides the grip (shadcn's `withHandle` is opt-in) is a public API decision for the maintainer.
