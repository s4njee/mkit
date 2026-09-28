---
spec_version: 1
component: button
states:
  - id: idle
    description: Enabled action button ready for pointer or keyboard activation.
    fixture: button_idle
  - id: disabled
    description: Button is unavailable and does not receive focus or activate.
    fixture: button_disabled
  - id: loading
    description: Button describes loading, remains layout-stable, and cannot activate again.
    fixture: button_loading
keys:
  - key: Enter
    modifiers: []
    when: enabled and focused
    action: Activate the button.
    initial_state: idle
    expect:
      event: activate
  - key: Space
    modifiers: []
    when: enabled and focused
    action: Activate the button according to platform button behavior.
    initial_state: idle
    expect:
      event: activate
accessibility:
  role: button
  properties:
    - name: name
      value: Save
    - name: disabled
      value: true
      when: disabled
    - name: disabled
      value: true
      when: loading
    - name: description
      value: Unavailable
      when: disabled
    - name: description
      value: Loading
      when: loading
---

# Button

## Purpose

A compact action control for submitting, confirming, and triggering commands.

## Anatomy

A button has an accessible name, optional leading/trailing icon elements, and a label. Loading replaces the leading icon with a progress glyph supplied by the caller, when supplied; the component does not invent a spinner asset.

## States

`idle`, `disabled`, and `loading` are the declared states. Disabled and loading share the same unavailable look (see "Theme tokens used"). Loading is unavailable for activation, exposes a "Loading" description, and is disabled in the accessibility tree; a platform busy-state relationship remains to be verified.

## Screenshot matrix

The gallery screenshot matrix is generated from `tests/conformance.json`: every declared state is rendered with the shadcn light, shadcn dark, and high-contrast themes at 1x and 2x. Each state fixture presents all six variants (`default`, `secondary`, `outline`, `ghost`, `destructive`, `link`) across small, default, and large sizes, so the same baseline set also checks the full visual API. The disabled and loading state applies to every sample in that fixture. Representative optional leading/trailing icon affordances are included where space allows.

## Props and events

Stateless `RenderOnce` builder: variant (`default`, `secondary`, `outline`, `ghost`, `destructive`, `link`), size (`sm`, `default`, `lg`), optional leading/trailing `AnyElement`, label child, disabled, loading, and accessible label override. `on_activate` runs for either primary pointer click or the named keyboard `Activate` action. `on_click` is a pointer-only callback with the GPUI click event; when both callbacks are set, pointer clicks call `on_click` first and `on_activate` second. Stateless buttons do not emit a separate entity event. Loading and disabled suppress both callbacks.

## Keyboard map

Enter and Space dispatch the `Activate` action through the `MkitButton` key context, which the host binds and may rebind. The component uses GPUI's tab-index and focus handling; key-down/keyup timing follows GPUI action dispatch. The generated GPUI keyboard adapter covers both keys, including disabled and loading suppression.

## Pointer behaviour

Primary click activates when enabled and calls `on_activate` even if no `on_click` callback is set. Disabled and loading buttons are inert. Hit target is the rendered control bounds. The test renderer exposes the bounds as `mkit-button` for pointer scripts.

## Accessibility role and properties

Use a button role and accessible name. The visible label is the default accessible name; `aria_label` overrides it when supplied. Decorative leading and trailing icons do not replace that name, and icon-only labels are not supported because `Button` always renders its visible label. Disabled and loading set the AccessKit disabled state and descriptions "Unavailable" and "Loading", and leave the tab order. The active-platform disabled semantics and loading announcements remain open for review.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7.ts`, styled by `.ui-btn*` in
`site/src/ui/ui.css` with the shadcn token mapping in `site/src/ui/tokens.ts`) and is resolved from
the installed `Theme` in three variants. `high-contrast` is selected by theme name (the convention
other registry components use); every other theme is treated as dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" below is shadcn's `secondary`/`accent`/`muted`: `text` mixed 4% (light) or
12% (dark) into `background`.

| Variant | Light / dark fill | Text | Border | Shadow | Hover | High contrast (fill, text, border; hover border) |
|---|---|---|---|---|---|---|
| default | `accent` | `accent_text` | transparent | `shadows.small` | `accent` 90% over `background` | `accent`, `accent_text`, `accent`; `text` |
| secondary | muted | `text` | transparent | `shadows.small` | the web's `color-mix(secondary 80%, text 6%)` (86% total becomes alpha over `background`) | `background`, `text`, `border`; `accent` |
| outline | `background` | `text` | light `border`; dark `text` at 10% | `shadows.small` | muted | `background`, `text`, `border`; `accent` |
| ghost | transparent | `text` | transparent | none | muted | transparent, `text`, transparent; `accent` |
| destructive | `danger` | light `background`, dark `text` (the theme's near-white; the web uses `#fff`) | transparent | `shadows.small` | the theme's near-black (light `text`, dark `background`) at 10% over `danger` | `danger`, `accent_text`, `danger`; `text` |
| link | transparent | `accent` (shadcn `text-primary`) | transparent | none | underline, no fill | transparent, `accent`, transparent; underline |

- **Focus** (`focus_visible`): border `focus` plus a 3px ring of `focus` at 50% alpha (opaque
  `focus` in high contrast); the ring replaces the resting shadow, as in CSS. GPUI paints drop
  shadows as filled shapes that are not clipped to the element's outside, so the fill under the ring
  is always opaque: the variant fill, or `background` for the transparent ghost and link variants.
  Ring corners use the button radius rather than CSS's radius-plus-spread.
- **Disabled and loading**: the resting look at 50% opacity with hover suppressed, matching the web
  preview (which renders its loading sample as a disabled button). High contrast instead keeps solid
  colours: `background` fill, `disabled` text, and a `disabled` border on every variant except ghost
  and link, so unavailable buttons stay legible without relying on transparency.
- **Geometry**: heights `controls.small`/`controls.medium`/`controls.large` (32/36/40, shadcn `h-8`,
  `h-9`, `h-10`); horizontal padding `spacing.medium`/`spacing.large`/`spacing.xlarge` (12/16/24,
  shadcn `px-3`, `px-4`, `px-6`); radius `radii.medium`; a `borders.regular` border that is
  transparent unless the variant draws one, so every variant has the same box; icon gap
  `spacing.small` (8px) at every size (shadcn's small size uses 6px; there is no 6px token, and the
  8px default gap is kept rather than crowding icons at 4px). Labels use `typography.body` (14px) at
  medium weight (500), shadcn's `font-medium`; there is no font-weight token yet. The 3px focus ring
  is the shadcn/ui ring width, a fixed component value.

## WAI-ARIA pattern reference

[Button Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/).

## Platform notes

Use GPUI's native focus and click semantics. Host applications may replace the named default key bindings.

## Open questions

The component API's icon type is `AnyElement`; maintainers should decide whether a future icon registry warrants a typed icon abstraction. Review whether the pointer-only `on_click` callback should remain alongside unified `on_activate`. Review disabled and busy accessibility semantics on an active platform. Visually, loading and disabled currently render identically because the component has no progress glyph; the web preview shows a spinner, which needs either a loading-glyph slot or the E9.2 icon set.
