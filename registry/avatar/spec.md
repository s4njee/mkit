---
spec_version: 1
component: avatar
states:
  - id: image
    description: User-provided image fills a circular or rounded avatar frame.
    fixture: image_fixture
  - id: initials
    description: Caller-provided initials render when there is no image.
    fixture: initials_fixture
  - id: fallback
    description: Generic accessible visual fallback when neither image nor initials is available.
    fixture: fallback_fixture
  - id: image_alt
    description: Image with a caller-provided accessible name.
    fixture: image_alt_fixture
keys: []
accessibility:
  role: img
  properties:
    - name: label
      value: caller-provided accessible name, or initials
      when: initials
---

# Avatar

## Purpose

Represent a person or account with an image or initials fallback.

## Anatomy

A fixed token-sized circular frame shows an optional supplied image, initials, or a neutral generic fallback. The component does not fetch remote assets or derive initials from a name.

## States

- `Image`: application-supplied GPUI image source.
- `Initials`: caller-provided fallback text when no image is present.
- `Fallback`: neutral visual marker when neither image nor initials is supplied.
- `Size`: small, medium, or large theme control dimensions.

## Props and events

Stateless `RenderOnce` builder; no events. Props include optional image source, fallback initials, accessible name, and size. Image source loading and name-to-initials localization are app-owned.

## Keyboard map

No key context. Avatar is not interactive or focusable.

## Pointer behaviour

None.

## Accessibility role and properties

Exposes the `Image` role with the caller's accessible name when provided; otherwise initials are used as the accessible label. For a generic fallback without initials or a supplied name, the component is decorative and omits the role. Image remains a single accessible node rather than exposing implementation details.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts`, styled by `.e7-avatar*`
in `site/src/demos/e7_expansion.css` with the shadcn token mapping in `site/src/ui/tokens.ts`), which
in turn follows the shadcn/ui `Avatar`. It is resolved from the installed `Theme` in three variants,
the same way Button, Select, and Tabs do it. `high-contrast` is selected by theme name; every other
theme is dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise
light. Derived colours use a crate-local `color-mix` helper built on
`mkit_core::contrast::composite`; no mkit-core API or tokens are added. "Muted" below is shadcn's
`muted`/`secondary`: `text` mixed 4% (light) or 12% (dark) into `background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Frame fill (initials and fallback) | muted | muted | `background` |
| Frame border | `border` | `text` at 10% over `background` | `border` |
| Initials | `text` | `text` | `text` |
| Fallback icon | `text_muted` | `text_muted` | `text` |

- **Fallback.** The generic fallback is the Lucide `user` icon (a 4-unit-radius head at 12,7 and
  shoulders `M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2` on a 24-unit grid, 2-unit stroke) drawn as a
  vector path in a `spacing.large` (16px) square at every size, as in the preview; there are no
  icon assets.
- **Geometry.** The frame is a circle (`radii.pill`) with a `borders.hairline` border; the image is
  clipped to the same circle and covers the frame. Sizes are `controls.xsmall` (28px, the
  preview's small avatar), `controls.medium` (36px, the preview's default), and `controls.large`
  (40px). Initials use `typography.caption` (12px) at medium weight (500; there is no font-weight
  token yet) at every size; the preview's 10px small-avatar text has no token.
- Avatar is not focusable, so its screenshot matrix has no focus state.

## WAI-ARIA pattern reference

No interactive APG pattern applies. Semantics follow the ARIA image role and alternative text guidance.

## Platform notes

The image is clipped to a circle. Fallback initials are supplied by the app; grapheme-aware truncation/localization is outside this primitive. Image loading errors are owned by the app because GPUI image-source failures are not observable by this primitive.

## Open questions

- Maintainer review is needed for image source ownership and unlabeled fallback semantics.
