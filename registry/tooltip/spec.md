---
spec_version: 1
component: tooltip
states:
  - id: visible
    description: GPUI shows the label after a 500 ms pointer hover or long press.
    fixture: tooltip_visible
    screenshot_status: unsupported_headless_capture
    screenshot_limitation: GPUI 0.3.5 headless screenshots omit the native tooltip popup after a synthetic hover, deterministic timer advance, executor pump, and fresh draw; active-platform popup capture remains pending.
  - id: keyboard_focused_child
    description: The focused child shows its own focus cue; GPUI 0.3.5 does not trigger a tooltip on focus.
    fixture: tooltip_keyboard_focused_child
    screenshot_status: captured_by_e7_gallery_matrix
  - id: disabled
    description: Only the child is rendered.
    fixture: tooltip_disabled
    screenshot_status: captured_by_e7_gallery_matrix
keys: []
accessibility:
  role: none
  properties:
    - name: description
      value: label
controlled: Stateless RenderOnce builder; no controlled or uncontrolled state contract.
events: []
theme_tokens: [elevated_surface, text, border, focus, spacing.small, spacing.xsmall, radii.small, borders.hairline, typography.caption]
open_questions: []
---

# Tooltip

## Purpose

Display a short label beside a child element.

## Anatomy

A wrapper contains the supplied child and uses GPUI's native tooltip popup. The wrapper exposes the label as its accessible description.

## States

The tooltip is hidden until pointer hover/long press, appears after 500 ms, and disappears when the pointer leaves. `disabled(true)` omits tooltip behavior and the description.

## Props and events

`Tooltip::new(label, child)` is a stateless `RenderOnce` builder; `disabled` controls tooltip behavior. It emits no events.

## Keyboard map

No key context or action is registered. The child retains its keyboard behavior. The visual tooltip is not triggered by keyboard focus.

## Pointer behaviour

GPUI owns hover/leave timing and popup placement. The delay is fixed at 500 ms.

## Accessibility role and properties

The wrapper sets an accessible description with the label; the child keeps its own role and name. GPUI 0.3.5 exposes a description setter but no described-by node relationship builder. A description placed on a generic wrapper is not evidence that the focused descendant receives that description, so the current implementation does not claim a keyboard-focus description relationship.

## Theme tokens used

`elevated_surface`, `text`, `border`, `spacing.small`, `spacing.xsmall`, `radii.small`, `borders.hairline`, and `typography.caption` come from GPUI `Theme`.

## WAI-ARIA pattern reference

[WAI-ARIA APG Tooltip Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/tooltip/) informs the hover delay and descriptive semantics. Focus-triggered visual display and an explicit described-by relationship remain unsupported by this API.

## Platform notes

GPUI owns popup placement. The host does not receive visibility events from this component. The generated conformance manifest defines cases rather than reporting executed harness results.

## Open questions

The pinned GPUI 0.3.5 `Div` API exposes pointer/long-press tooltip construction and focus styling, but does not expose element focus-in/focus-out callbacks. Focus subscriptions live on a `Context<T>` and require an `Entity<T>` with a persistent `FocusHandle`; converting this `RenderOnce` wrapper to that model would also require preserving/rebuilding its arbitrary, potentially non-cloneable `AnyElement` child. Adding a focusable wrapper tab stop would change navigation and could steal focus from the child. Therefore keyboard-triggered display and a focused-descendant description relationship are deferred until GPUI supports them without changing child focus semantics.

The fixed trigger id is replaced with a per-instance id so sibling Tooltip wrappers do not reuse the same element id. This preserves GPUI's native pointer/long-press behavior while preventing identity collisions.

## Gallery screenshot fixtures

`tooltip_visible` is a declared state, but its screenshot cases are marked `unsupported_headless_capture`: GPUI 0.3.5's headless screenshot contains only the trigger after synthetic pointer hover, deterministic timer advance, executor pump, and a fresh draw. A platform screenshot that includes the native tooltip popup is still pending. `tooltip_keyboard_focused_child` captures the focused child with a visible focus-token border; GPUI 0.3.5 does not show the popup on focus, and this fixture must not be interpreted as keyboard-triggered tooltip coverage. `tooltip_disabled` captures the child with tooltip behavior omitted. The harness cannot capture platform accessibility in headless mode.
