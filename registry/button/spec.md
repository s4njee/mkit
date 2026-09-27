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

`idle`, `disabled`, and `loading` are the declared states. Loading is unavailable for activation, exposes a "Loading" description, and is disabled in the accessibility tree; a platform busy-state relationship remains to be verified.

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

`Theme.colors` background/surface/text/border/accent/accent_text/focus/danger/disabled/elevated_surface; `Theme.spacing` small/medium; `Theme.radii.medium`; `Theme.borders.regular`; `Theme.typography.body`; `Theme.controls` xsmall/medium/large.

## WAI-ARIA pattern reference

[Button Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/).

## Platform notes

Use GPUI's native focus and click semantics. Host applications may replace the named default key bindings.

## Open questions

The component API's icon type is `AnyElement`; maintainers should decide whether a future icon registry warrants a typed icon abstraction. Review whether the pointer-only `on_click` callback should remain alongside unified `on_activate`. Review disabled and busy accessibility semantics on an active platform.
