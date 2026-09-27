---
spec_version: 1
component: inline-alert
states:
  - id: info
    description: Informational message is visible with polite status semantics.
    fixture: info_fixture
  - id: success
    description: Successful result is visible with polite status semantics.
    fixture: success_fixture
  - id: warning
    description: Caution is visible with assertive alert semantics.
    fixture: warning_fixture
    accessibility_role: alert
  - id: error
    description: Error is visible with assertive alert semantics.
    fixture: error_fixture
    accessibility_role: alert
  - id: dismissed
    description: Dismissible alert is removed after dismissal.
    fixture: dismissed_fixture
keys:
  - key: Enter
    modifiers: []
    when: Dismiss button is focused and enabled.
    action: Request dismissal.
    initial_state: info
    expect:
      event: DismissRequested
accessibility:
  role: status
  properties:
    - name: live
      value: polite
      when: info
    - name: live
      value: polite
      when: success
    - name: live
      value: assertive
      when: warning
    - name: live
      value: assertive
      when: error
controlled: Owner controls dismissed state with controlled constructor and set_dismissed; uncontrolled dismissal hides before emitting.
events: [ActionInvoked, DismissRequested]
theme_tokens: [surface, elevated_surface, text, text_muted, border, accent, danger, disabled, spacing.small, spacing.medium, spacing.large, radii.medium, borders.hairline, typography.body, typography.body_emphasis, controls.xsmall]
open_questions: [Review urgency mapping for warning messages and whether info/success should ever be assertive.]
---

# Inline alert

## Purpose

Show a persistent contextual message near the content it explains, with an optional next action or dismissal.

## Anatomy

An alert region has a severity marker, message, optional title, optional compact action button, and optional dismiss button. The component supplies layout and styling; callers supply icon elements and action labels. The action width follows the label using the body typography token for its text estimate and token-based inline padding rather than stretching across the message column.

## States

Severity is info, success, warning, or error. Dismissible instances can be visible or dismissed. Disabled action and dismiss controls are omitted from tab traversal. `InlineAlert` is an `Entity<InlineAlert>` because optional action and dismiss controls emit typed events and dismissal can be controlled.

## Props and events

Builder props include stable ID, severity, title, message, optional icon factory, optional action label, and an optional dismiss control. Action activation emits `ActionInvoked`; it does not execute caller code internally. Dismissible mode is uncontrolled by default: `new` starts visible, clicking dismiss hides it and emits `DismissRequested`. `controlled(..., dismissed)` leaves visibility to the owner; the same event is emitted as a proposal and the owner applies it with `set_dismissed`. Programmatic updates do not emit events. No-op and disabled interactions emit none.

## Keyboard map

The `InlineAlert` key context binds Enter and Space to the focused action/dismiss button through normal button activation. No region-level keyboard handler is added. Tab follows platform order among optional controls.

## Pointer behaviour

Action and dismiss buttons activate independently. The action emits its event. Dismiss hides or requests hiding the alert. The message surface itself has no click action.

## Accessibility role and properties

Info and success use status role with polite live-region priority. Warning and error use alert role with assertive priority. The message is the region's accessible name/description; action and dismiss controls are named buttons. Dismissed alerts are omitted from the accessibility tree. Actual announcement timing depends on the host platform and needs active-platform validation.

## Theme tokens used

Read surface, elevated surface, text, muted text, border, accent, danger, disabled, spacing, radii, hairline, control height, and typography from the mkit-core Global `Theme`. The severity marker uses the matching semantic token and does not introduce literal colors. Fixed icon glyphs are not imposed; icons are application supplied.

## WAI-ARIA pattern reference

Follow the [WAI-ARIA Alert Pattern] for urgency and live announcement. The optional action and dismiss controls remain ordinary buttons.

## Platform notes

GPUI maps status/alert roles through AccessKit. Some assistive technology announces assertive alerts immediately, while polite status updates may wait for a pause. Platform announcement behavior needs a native accessibility run; headless snapshots alone do not prove speech output.

## Open questions

- Maintainers should review warning urgency and the public builder/event naming.
- Confirm whether content-only inline alerts should share this component or remain a `RenderOnce` variant.

[WAI-ARIA Alert Pattern]: https://www.w3.org/WAI/ARIA/apg/patterns/alert/
