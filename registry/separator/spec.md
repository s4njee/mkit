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

`Theme.colors.border` provides the line color and `Theme.borders.hairline` provides its thickness. No colors or dimensions are hard-coded.

## WAI-ARIA pattern reference

[Separator](https://www.w3.org/WAI/ARIA/apg/patterns/separator/).

## Platform notes

Vertical height comes from its containing layout. Hairline rendering at 1x and 2x is covered by the screenshot matrix.

## Open questions

- Maintainer review is needed for the `Splitter` role mapping because the pinned bridge has no static separator role.
