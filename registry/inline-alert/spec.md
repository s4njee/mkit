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
  - id: focused
    description: Informational alert whose action button has keyboard focus after Tab.
    fixture: focused_fixture
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
theme_tokens: [background, text, text_muted, border, focus, success, warning, danger, disabled, spacing.xsmall, spacing.small, spacing.medium, spacing.large, radii.medium, radii.large, borders.regular, typography.body, controls.xsmall, controls.small, shadows.small]
open_questions: [Review urgency mapping for warning messages and whether info/success should ever be assertive.]
---

# Inline alert

## Purpose

Show a persistent contextual message near the content it explains, with an optional next action or dismissal.

## Anatomy

An alert region has a severity marker, message, optional title, optional compact action button, and optional dismiss button. The component supplies layout and styling; callers supply icon elements and action labels. The icon slot is a fixed square beside the title that sets the severity colour for its content. The action button sizes to its label rather than stretching across the message column.

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

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts`, styled by `.e7-alert*`
in `site/src/demos/e7_expansion.css` with the shadcn token mapping in `site/src/ui/tokens.ts`), which
in turn follows the shadcn/ui `Alert`; the action and dismiss controls follow Button's `outline` and
`ghost` variants. It is resolved from the installed `Theme` in three variants, the same way Button,
Select, and Tabs do it. `high-contrast` is selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Border" is `border` in light themes and `text` at 10% over `background` in dark
themes; "muted" is `text` mixed 4% (light) or 12% (dark) into `background`.

| Part | Light / dark | High contrast |
|---|---|---|
| Card fill | `background` | `background` |
| Card border | info and success: border; warning and error: the severity colour mixed 45% into the border (the preview's `.e7-alert--warning`) | `border` |
| Icon slot colour | info `text`, success `success`, warning `warning`, error `danger` | the same tokens |
| Title | `text`; error `danger` (shadcn `destructive`) | `text` |
| Message | `text_muted`; error `danger` 90% over `background` (shadcn `text-destructive/90`) | `text` |
| Action button | `background` fill, border, `text`, `shadows.small`; hover muted | `background` fill, `border` border, `text` |
| Dismiss button | transparent, `text_muted`; hover muted fill and `text` | transparent, `text` |
| Keyboard focus (either button) | border `focus` plus a 3px ring of `focus` at 50%, over an opaque fill | border `focus` plus a 3px ring of opaque `focus` |
| Disabled controls | each control colour mixed 50% over `background`; shadow alpha halved; no hover | `background` fill, `disabled` text and border |

- **Severity** is never carried by colour alone: the role, the caller's title and message, and the
  caller's icon carry it too. The icon slot only sets the text colour its content inherits; icons
  stay application supplied (the screenshot fixtures pass Lucide icons drawn as vector paths that
  read the inherited colour).
- **Rings and shadows.** GPUI paints drop shadows as filled shapes that are not clipped to the
  element's outside, so both buttons use an opaque fill while focused (`background` for the
  transparent dismiss button). Ring corners use the button radius rather than CSS's
  radius-plus-spread.
- **Disabled** matches shadcn's `opacity: .5` as one layer, the way Select and TextField do it; GPUI
  element opacity is not used. Only the controls are dimmed: the message stays readable, as the
  alert content itself is not unavailable. High contrast keeps solid colours instead.
- **Geometry.** The card has radius `radii.large` (shadcn `rounded-lg`), a `borders.regular` border,
  `spacing.medium` (12px, the preview's padding) padding, and a `spacing.medium` gap between the
  icon slot and the text column (shadcn `gap-x-3`; the preview uses 10px, which has no token). Text
  is `typography.body` (14px) with a 20px line height (`spacing.large + spacing.xsmall`, Tailwind's
  `text-sm` line box); the title is semibold (600, the preview's `b`) and sits `spacing.xsmall`
  (4px) above the message (the preview uses 3px). The icon slot is `spacing.large` (16px, shadcn
  `size-4`) wide and one 20px line tall, so the icon centres on the title line. The action button is
  `controls.small` (32px, Button's small size) tall with `spacing.medium` padding, radius
  `radii.medium`, medium weight (500), and sits `spacing.small` (8px) below the message; the dismiss
  button is `controls.xsmall` (28px) tall with `spacing.small` padding and radius `radii.medium`,
  centred on the title line by an equal negative vertical margin.
  There is no font-weight token yet. The 3px focus ring is the shadcn/ui ring width, a fixed
  component value.
- The `focused` screenshot state dispatches a keyboard Tab in the `info` fixture and moves focus to
  the first tab stop, the action button.

## WAI-ARIA pattern reference

Follow the [WAI-ARIA Alert Pattern] for urgency and live announcement. The optional action and dismiss controls remain ordinary buttons.

## Platform notes

GPUI maps status/alert roles through AccessKit. Some assistive technology announces assertive alerts immediately, while polite status updates may wait for a pause. Platform announcement behavior needs a native accessibility run; headless snapshots alone do not prove speech output.

## Open questions

- Maintainers should review warning urgency and the public builder/event naming.
- Confirm whether content-only inline alerts should share this component or remain a `RenderOnce` variant.

[WAI-ARIA Alert Pattern]: https://www.w3.org/WAI/ARIA/apg/patterns/alert/
