---
spec_version: 1
component: scroll-area
states:
  - id: idle
    description: A scrollable region at its initial offset.
    fixture: idle_fixture
  - id: scrolled
    description: A scrollable region offset one theme-token line (32 px in the light-theme fixture) below the top.
    fixture: scrolled_fixture
keys:
  - key: PageDown
    modifiers: []
    when: Scroll area has focus and content exceeds viewport.
    action: Scroll one viewport downward.
    initial_state: idle
    expect:
      state: scrolled
  - key: PageUp
    modifiers: []
    when: Scroll area has focus and is scrolled one line below the top.
    action: Scroll one viewport upward, clamping at the top.
    initial_state: scrolled
    expect:
      state: idle
  - key: End
    modifiers: []
    when: Scroll area has focus and content exceeds viewport.
    action: Scroll to the end of the content.
    initial_state: idle
    expect:
      state: scrolled
  - key: Home
    modifiers: []
    when: Scroll area has focus and is scrolled one line below the top.
    action: Scroll to the beginning of the content.
    initial_state: scrolled
    expect:
      state: idle
  - key: ArrowDown
    modifiers: []
    when: Scroll area has focus and content exceeds viewport.
    action: Scroll one theme-token line downward.
    initial_state: idle
    expect:
      state: scrolled
  - key: ArrowUp
    modifiers: []
    when: Scroll area has focus and is scrolled one line below the top.
    action: Scroll one theme-token line upward, clamping at the top.
    initial_state: scrolled
    expect:
      state: idle
accessibility:
  role: scroll-view
  properties: []
---

# Scroll area

## Purpose

Give a panel a bounded, independently scrollable viewport.

## Anatomy

A viewport containing caller supplied content; platform scrollbars appear as needed.

## States

Idle, focused, and scrolled; content and viewport dimensions determine overflow. The visual fixtures render the same eight-row activity list in a bounded viewport. The scrolled fixture advances the shared `ScrollHandle` by one theme-token line after initial layout, so the two captured states show a real content offset.

## Props and events

`height`, optional `width`, `id`, optional reusable `ScrollHandle`, and child content; no component events. Stateless `RenderOnce` builder. Callers retain the handle and ID across rerenders to retain scroll position.

## Keyboard map

Page Up/Down move by one viewport, Home/End move to the top/bottom, and arrows move by twice `Theme.spacing.large` (32 px in the light-theme fixture). They clamp at the scroll bounds. The `MkitScrollArea` key context exposes named scroll actions so applications can rebind them.

## Pointer behaviour

Wheel or trackpad scrolling and native scrollbar dragging use GPUI scrolling.

## Accessibility role and properties

Expose a named scroll-view region when `label` is supplied. Content retains its own semantics.

## Theme tokens used

`Theme.colors.surface`, `Theme.colors.border`, `Theme.borders.hairline`, and `Theme.radii.medium`.

## WAI-ARIA pattern reference

Follow the platform scroll-view convention; there is no dedicated WAI-ARIA scroll-area pattern.

## Platform notes

GPUI handles momentum and platform scrollbar rendering.

## Open questions

Confirm keyboard action wiring on all supported platforms.
