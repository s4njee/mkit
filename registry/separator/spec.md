---
spec_version: 1
component: separator
states:
  - id: horizontal
    description: A nondecorative horizontal rule between two related sections.
    fixture: horizontal_fixture
  - id: vertical
    description: A nondecorative vertical rule between adjacent panels.
    fixture: vertical_fixture
keys: []
accessibility:
  role: splitter
  properties:
    - name: orientation
      value: horizontal
      when: horizontal
    - name: orientation
      value: vertical
      when: vertical
---

# Separator

## Purpose

Separate related content with a quiet hairline rule that uses the active theme's border color and width tokens.

## Anatomy

The separator is a single line. Its horizontal axis fills the available width; its vertical axis fills the available height. It has no visible label.

## States

- `Horizontal` (default): a full-width horizontal rule.
- `Vertical`: a full-height vertical rule whose parent supplies its height.
- Decorative (default): no explicit accessibility role.
- Semantic (`decorative(false)`): exposes orientation and an optional accessible name.

## Props and events

`orientation`, `decorative`, and optional `label` (an accessible name only); no events. This is a stateless `RenderOnce` builder.

## Keyboard map

No keyboard behavior or key context is needed. The separator is not interactive.

## Pointer behaviour

None.

## Accessibility role and properties

Decorative separators do not set an accessibility role. Semantic separators request AccessKit's `Splitter` role and set orientation; `label`, when supplied, becomes the accessible name. The pinned AccessKit version does not expose a noninteractive separator role, so `Splitter` is the closest available role and can imply resizability on some platforms. This mismatch needs maintainer review before treating the role as a final public contract.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7.ts`, styled by `.ui-separator` in
`site/src/ui/ui.css` with the shadcn token mapping in `site/src/ui/tokens.ts`), which in turn follows
the shadcn/ui separator: a `borders.hairline` (1px) line in shadcn's `--border` colour that does not
shrink in a flex row (`shrink-0`).

| Theme | Line colour |
|---|---|
| Light (luminance of `background` ≥ 0.5) | `border` |
| Dark (luminance of `background` < 0.5) | `text` at 10% alpha, the docs-site `--border` mapping |
| High contrast (selected by theme name) | `border` (white), kept solid |

Dark themes use an alpha colour rather than an opaque mix so the rule reads the same over
`background`, `surface`, and elevated panels, as the web's `color-mix(… transparent)` does. The theme
is classified with `mkit_core::contrast::relative_luminance`, the convention Button, Select, and Tabs
use; no mkit-core API or tokens are added. No other colours or dimensions are hard-coded.

## WAI-ARIA pattern reference

[Separator](https://www.w3.org/WAI/ARIA/apg/patterns/separator/).

## Platform notes

Vertical height comes from its containing layout. Hairline rendering at 1x and 2x is covered by the screenshot matrix.

## Open questions

- Maintainer review is needed for the `Splitter` role mapping because the pinned bridge has no static separator role.
