---
spec_version: 1
component: progress
states:
  - id: zero
    description: No progress.
    fixture: zero_fixture
  - id: partial
    description: Progress between bounds.
    fixture: partial_fixture
  - id: complete
    description: Progress reached maximum.
    fixture: complete_fixture
  - id: indeterminate
    description: Progress amount is unknown.
    fixture: indeterminate_fixture
keys: []
accessibility:
  role: progressbar
  properties:
    - name: aria-valuemin
      value: 0
    - name: aria-valuemax
      value: 100
    - name: description
      value: In progress
      when: indeterminate
---

# Progress

## Purpose

Show completion of a task without accepting input.

## Anatomy

Thin rounded track and filled segment, optionally accompanied by caller-provided label/value text.

## States

Zero, partial, complete, or indeterminate. The component is stateless; caller supplies progress or indeterminate mode. The range requires finite bounds; invalid bounds fall back to 0–100. Determinate values clamp to those bounds, and a nonfinite value falls back to the minimum. Indeterminate mode draws a centered, static short segment to distinguish unknown progress from a determinate fill; animation and reduced-motion behavior remain open.

The screenshot fixtures use a 0–100 range and values of 0, 42, and 100 for zero, partial, and complete. The indeterminate fixture omits the current value. Every fixture uses the same labelled, 320-point-wide track so the fill length is directly comparable across shadcn light, dark, and high-contrast themes at 1× and 2×.

## Props and events

`label`, min/max/value, or indeterminate. No events; caller updates props.

## Keyboard map

Not focusable and has no keyboard actions.

## Pointer behaviour

Decorative and ignores pointer input.

## Accessibility role and properties

Progressbar role with accessible name and min/max bounds. Determinate values expose the clamped current value. Indeterminate mode omits the current value and supplies the description “In progress”; it does not set a live-region or busy property. Active-platform announcement behavior remains unverified.

## Theme tokens used
The look follows the shadcn/ui progress bar and is resolved from the installed `Theme` in three
variants, the same way Tabs does it. `high-contrast` is selected by theme name; every other theme
is dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light.
Derived colours use `mkit_core::contrast::composite`; no new core tokens are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Track | `accent` at 20% over `background` | `accent` at 20% over `background` | `background` with a `borders.hairline` border in `border` |
| Fill | `accent` | `accent` | `accent` |

The track uses shadcn/ui's `bg-primary/20`, as the web preview does, composited opaque over
`background`. High contrast outlines the track instead so the yellow fill never sits on a white
track. The bar is `spacing.small` (8px) tall, shadcn/ui's `h-2`, with radius `radii.pill` on both
the track and the fill. The static indeterminate segment is 35% of the track width and centred, so
it stays distinguishable from a determinate fill, which always starts at the leading edge; this
fraction is a deliberate visual placeholder pending animation and reduced-motion design review.

## WAI-ARIA pattern reference

[WAI-ARIA progressbar role](https://www.w3.org/TR/wai-aria/#progressbar) and [APG guidance for range properties](https://www.w3.org/WAI/ARIA/apg/practices/range-related-properties/). Progress is read-only and omits `aria-valuenow` when unknown.

## Platform notes

Avoid frequent accessibility announcements; caller controls update cadence.

## Open questions

Confirm indeterminate animation policy, reduced-motion integration, and whether the “In progress” description should be translated or supplied by the host.
