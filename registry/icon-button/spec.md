---
spec_version: 1
component: icon-button
states:
  - id: idle
    description: Enabled icon action ready for activation.
    fixture: icon_button_idle
  - id: disabled
    description: Icon action is unavailable and inert.
    fixture: icon_button_disabled
keys:
  - key: Enter
    modifiers: []
    when: enabled and focused
    action: Activate the icon button.
    initial_state: idle
    expect:
      event: activate
  - key: Space
    modifiers: []
    when: enabled and focused
    action: Activate the icon button using platform button behavior.
    initial_state: idle
    expect:
      event: activate
accessibility:
  role: button
  properties:
    - name: description
      value: Unavailable
      when: disabled
    - name: aria-disabled
      value: true
      when: disabled
---

# Icon button

## Purpose

A compact button for a single icon action, such as close, search, or more options.

## Anatomy

A square control containing one icon element and a required accessible label.

## States

`idle` and `disabled`.

## Screenshot matrix

The gallery screenshot matrix is generated from `tests/conformance.json`: every declared state is rendered with the shadcn light, shadcn dark, and high-contrast themes at 1x and 2x. Each state fixture presents all five variants (`default`, `secondary`, `outline`, `ghost`, `destructive`) across small, default, and large sizes. The disabled state applies to every sample in that fixture.

## Props and events

Stateless `RenderOnce` builder with required accessible label, icon `AnyElement`, variant (`default`, `secondary`, `outline`, `ghost`, `destructive`), size (`sm`, `default`, `lg`), and disabled. `on_activate` runs for a primary pointer click or the named keyboard `Activate` action. `on_click` is pointer-only and receives the GPUI click event; when both callbacks are set, pointer clicks call `on_click` first and `on_activate` second. Disabled suppresses both callbacks.

## Keyboard map

Enter and Space are bound to the named `Activate` action in `MkitIconButton`; hosts can rebind the action. Standard tab navigation is native.

## Pointer behaviour

Primary click over the full square control calls `on_click` and then `on_activate` when enabled. A control with only `on_activate` still activates on click. Disabled is inert.

## Accessibility role and properties

Button role and required accessible name. The icon is decorative to assistive technology. Disabled controls have tab index -1, an AccessKit disabled property, and an accessible description of “Unavailable”.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7.ts`, styled by `.ui-btn--icon` in
`site/src/ui/ui.css` with the shadcn token mapping in `site/src/ui/tokens.ts`) and is resolved from
the installed `Theme` in three variants. `high-contrast` is selected by theme name (the convention
other registry components use); every other theme is treated as dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" below is shadcn's `secondary`/`accent`/`muted`: `text` mixed 4% (light) or
12% (dark) into `background`.

Colours, hover, focus, and high-contrast treatment are identical to `Button` (see its spec) for the
shared variants (`default`, `secondary`, `outline`, `ghost`, `destructive`): accent, muted, and
danger fills; an outline border of `border` (light) or `text` at 10% (dark); `shadows.small` on
every variant except ghost; a `focus` border plus a 3px ring of `focus` at 50% (opaque in high
contrast) over an opaque fill; disabled at 50% opacity, or solid `background`/`disabled`
colours in high contrast. The destructive glyph uses the theme's near-white (light `background`,
dark `text`) and `accent_text` in high contrast.

Geometry: a square of `controls.small`/`controls.medium`/`controls.large` (32/36/40, shadcn
`size-8`, `size-9`, `size-10`), radius `radii.medium`, a `borders.regular` border (transparent unless
the variant draws one), and `typography.body` text size so text-glyph icons match 14px labels. The
caller's icon element is centred and keeps its own size (the web preview uses 16px icons).

## WAI-ARIA pattern reference

[Button Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/).

## Platform notes

Native GPUI focus and click behavior; caller is responsible for choosing an icon whose meaning matches the accessible label.

## Open questions

None.
