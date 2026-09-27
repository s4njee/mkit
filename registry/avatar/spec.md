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

Uses `Theme.colors.surface`, `text_muted`, `border`, and `accent`; `Theme.typography.body_emphasis`, `radii.pill`, and `borders.hairline`. Dimensions map to `Theme.controls.xsmall`, `medium`, and `large`.

## WAI-ARIA pattern reference

No interactive APG pattern applies. Semantics follow the ARIA image role and alternative text guidance.

## Platform notes

The image is clipped to a circle. Fallback initials are supplied by the app; grapheme-aware truncation/localization is outside this primitive. Image loading errors are owned by the app because GPUI image-source failures are not observable by this primitive.

## Open questions

- Maintainer review is needed for image source ownership and unlabeled fallback semantics.
