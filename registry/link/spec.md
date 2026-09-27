---
spec_version: 1
component: link
states:
  - id: enabled
    description: Focusable link with a visible underline and activation callback.
    fixture: enabled_fixture
  - id: hover
    description: Hovered enabled link with increased visual emphasis.
    fixture: hover_fixture
  - id: focused
    description: Keyboard-focused link with visible focus treatment.
    fixture: focused_fixture
  - id: visited
    description: Link explicitly marked visited by its owner.
    fixture: visited_fixture
  - id: disabled
    description: Unavailable link that cannot be focused or activated.
    fixture: disabled_fixture
keys:
  - key: Enter
    modifiers: []
    when: Enabled and focused
    action: Dispatch the rebindable Activate action to invoke the callback.
    initial_state: enabled
    expect:
      event: activate
accessibility:
  role: link
  properties:
    - name: label
      value: visible text or supplied accessible name
      when: enabled
    - name: disabled
      value: true
      when: disabled
---

# Link

## Purpose

Present navigable text with consistent theme styling and semantic link behavior.

## Anatomy

A visible text label is underlined and can receive focus when it has an activation callback. The application owns destination resolution and navigation.

## States

- `Enabled`: focusable, clickable, and Enter-activatable when a callback is supplied.
- `Hover`: enabled link under pointer.
- `Focused`: visible keyboard focus ring.
- `Visited`: owner-provided visited state changes color without implying navigation history storage.
- `Disabled`: muted and neither focusable nor activatable.

## Props and events

Stateless `RenderOnce` builder. Props include visible label, optional accessible name, visited/disabled flags, optional stable identity for parent rerenders, and optional activation callback. A callback is required for focus and activation; without it the node is not focusable or activatable. No built-in URL opening or history behavior is provided.

## Keyboard map

`Enter` invokes the callback when enabled and focused. Space retains normal text navigation behavior and does not activate. Enter is bound to `Activate` in the `MkitLink` key context so applications may rebind it.

## Pointer behaviour

Primary click invokes the callback when enabled. Disabled links ignore clicks.

## Accessibility role and properties

Exposes the `Link` role and an accessible name from visible text unless overridden. Disabled links expose disabled state and are removed from tab order. The parent must provide a meaningful label and callback for navigation semantics.

## Theme tokens used

Uses `Theme.colors.accent`, `focus`, `text_muted`, `disabled`, `border`, `surface`; `Theme.typography.body`, `spacing.xsmall`, and `borders.hairline`.

## WAI-ARIA pattern reference

ARIA link role guidance; links activate with Enter and remain distinct from buttons that perform actions.

## Platform notes

Pointer hover and focus-visible presentation use GPUI behavior. Destination opening remains application-specific.

## Open questions

- Maintainer review is needed for the callback-based link contract and disabled link semantics.
