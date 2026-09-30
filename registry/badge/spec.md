---
spec_version: 1
component: badge
states:
  - id: count
    description: Compact numeric count, clamped to the configured maximum display.
    fixture: count_fixture
  - id: count_overflow
    description: Count above the configured maximum uses the caller-provided overflow label.
    fixture: overflow_fixture
  - id: status_success
    description: Success status with a visible dot and caller label.
    fixture: success_fixture
  - id: status_warning
    description: Warning status with a visible dot and caller label.
    fixture: warning_fixture
  - id: status_danger
    description: Danger status with a visible dot and caller label.
    fixture: danger_fixture
  - id: disabled
    description: Muted informational badge.
    fixture: disabled_fixture
keys: []
accessibility:
  role: generic
  properties:
    - name: label
      value: supplied label or visible text
      when: count
---

# Badge

## Purpose

Show small count or status metadata beside another piece of content.

## Anatomy

A small rounded badge contains either a count string or a status dot and label. Count badges can cap a displayed number and use an app-supplied overflow label. Status variants are neutral, success, warning, and danger.

## States

- `Count`: numeric value; `max_count` defaults to 99 and values above it display the supplied overflow label or `99+`.
- `Status`: neutral, success, warning, or danger dot with visible caller label.
- `Disabled`: muted visual treatment for unavailable metadata.

## Props and events

Stateless `RenderOnce` builder; no events. Props include count/status constructors, maximum count, overflow text, visible status label, and optional accessible name. Count formatting is numeric and locale formatting remains app-owned.

## Keyboard map

No key context. Badge is not interactive or focusable.

## Pointer behaviour

None.

## Accessibility role and properties

Noninteractive generic content. The visible count/status text supplies its accessible text; an optional accessible name overrides it. Status color is never the only status cue: status badges include a visible dot and text label. No live region is implied.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts`, styled by `.ui-badge*`
in `site/src/ui/ui.css` and `.e7-status-badge` in `site/src/demos/e7_expansion.css`, with the shadcn
token mapping in `site/src/ui/tokens.ts`), which in turn follows the shadcn/ui `Badge`. It is
resolved from the installed `Theme` in three variants, the same way Button, Select, and Tabs do it.
`high-contrast` is selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" below is shadcn's `secondary`: `text` mixed 4% (light) or 12% (dark) into
`background`. "Border" is `border` in light themes and `text` at 10% over `background` in dark
themes, as in the site token mapping.

The existing kinds map onto the shadcn variants without an API change:

| Kind | shadcn variant | Light / dark fill | Text and dot | Border | High contrast (fill, text, border, dot) |
|---|---|---|---|---|---|
| Count | default | `accent` | `accent_text` (no dot) | the fill | `accent`, `accent_text`, `accent` |
| Neutral status | secondary | muted | `text`; dot `text_muted` | the fill | `background`, `text`, `border`, `text_muted` |
| Success, warning status | the preview's tinted status badge | the status colour mixed 12% into `background` | `text`; dot the status colour | the status colour mixed 30% into the border | `background`, `text`, `border`, the status colour |
| Danger status | destructive | `danger` | the theme's near-white (light `background`, dark `text`; the web uses `#fff`), dot the same | the fill | `background`, `text`, `border`, `danger` |

- **Status colour.** The web preview tints its status badge with `--ui-ring`; mkit uses the
  badge's own status colour in the same 12% fill / 30% border formula so success and warning stay
  distinct. The dot and the visible label keep status from depending on colour alone.
- **Disabled** matches shadcn's `opacity: .5` as one layer, the way Select and TextField do it:
  every colour is composited over `background` and mixed 50% with it. GPUI element opacity is not
  used because it dims each painted part separately. High contrast keeps solid colours instead:
  `background` fill, and `disabled` text, border, and dot.
- **Geometry.** Height is 22px, shadcn's `h-[22px]`: a 20px content box (`spacing.large +
  spacing.xsmall`) plus a `borders.regular` border on each side (24px in high contrast, whose
  border is 2px). Horizontal padding is `spacing.small` (8px, `px-2`), the dot gap
  `spacing.xsmall` (4px, `gap-1`), and the radius `radii.medium` (shadcn `rounded-md`). Text is
  `typography.caption` (12px, `text-xs`) at medium weight (500, `font-medium`; there is no
  font-weight token yet). The status dot sits in a `typography.caption` (12px) square, shadcn's
  `[&>svg]:size-3` icon slot, and is half that size, close to the preview's `●` glyph.
- Badge is not focusable, so its screenshot matrix has no focus state.

## WAI-ARIA pattern reference

No dedicated APG pattern applies to static badge content. Semantics follow generic text content and avoid live-region announcements for static values.

## Platform notes

The status dot is accompanied by text for color-independent meaning. Layout is checked at 1× and 2× in light, dark, and high-contrast themes.

## Open questions

- Maintainer review is needed for the public count overflow and status variant API.
