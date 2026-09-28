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
theme_tokens: [accent, accent_text, background, text, border, spacing.xsmall, spacing.small, spacing.medium, radii.medium, borders.hairline, typography.caption]
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

The popup follows the docs-site web preview (`site/src/demos/e7.ts`, styled by `.ui-tooltip` in
`site/src/ui/ui.css` with the shadcn token mapping in `site/src/ui/tokens.ts`), which is shadcn/ui's
tooltip: a primary-filled label. `high-contrast` is selected by theme name (the convention other
registry components use); no light/dark derivation is needed because the popup reads the same
tokens in both. No mkit-core API or tokens are added.

| Part | Light and dark | High contrast |
|---|---|---|
| Fill | `accent` (shadcn `primary`) | `background` |
| Text | `accent_text` (shadcn `primary-foreground`) | `text` |
| Border | none | `borders.hairline` in `border` |
| Shadow | none (shadcn draws none) | none |

- **Geometry**: horizontal padding `spacing.medium` (12px, shadcn `px-3`); vertical padding is the
  midpoint of `spacing.xsmall` and `spacing.small` (6px, shadcn `py-1.5`), because there is no 6px
  token and either neighbour visibly changes the label's proportions; radius `radii.medium`
  (shadcn `rounded-md`); label `typography.caption` (12px, shadcn `text-xs`). The label does not wrap
  in the web preview; GPUI sizes the popup to its content. The optional shadcn arrow is not drawn:
  GPUI owns the popup placement and does not report which side it chose.
- **High contrast** keeps the popup legible against any content by using the theme's solid
  `background` and `text` with a `border` outline (2px there) instead of the yellow `accent` fill.
- The focused child's own focus cue is unchanged; the tooltip does not style the child.

## WAI-ARIA pattern reference

[WAI-ARIA APG Tooltip Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/tooltip/) informs the hover delay and descriptive semantics. Focus-triggered visual display and an explicit described-by relationship remain unsupported by this API.

## Platform notes

GPUI owns popup placement. The host does not receive visibility events from this component. The generated conformance manifest defines cases rather than reporting executed harness results.

## Open questions

The pinned GPUI 0.3.5 `Div` API exposes pointer/long-press tooltip construction and focus styling, but does not expose element focus-in/focus-out callbacks. Focus subscriptions live on a `Context<T>` and require an `Entity<T>` with a persistent `FocusHandle`; converting this `RenderOnce` wrapper to that model would also require preserving/rebuilding its arbitrary, potentially non-cloneable `AnyElement` child. Adding a focusable wrapper tab stop would change navigation and could steal focus from the child. Therefore keyboard-triggered display and a focused-descendant description relationship are deferred until GPUI supports them without changing child focus semantics.

The fixed trigger id is replaced with a per-instance id so sibling Tooltip wrappers do not reuse the same element id. This preserves GPUI's native pointer/long-press behavior while preventing identity collisions.

## Gallery screenshot fixtures

`tooltip_visible` is a declared state, but its screenshot cases are marked `unsupported_headless_capture`: GPUI 0.3.5's headless screenshot contains only the trigger after synthetic pointer hover, deterministic timer advance, executor pump, and a fresh draw. A platform screenshot that includes the native tooltip popup is still pending. `tooltip_keyboard_focused_child` captures the focused child with a visible focus-token border; GPUI 0.3.5 does not show the popup on focus, and this fixture must not be interpreted as keyboard-triggered tooltip coverage. `tooltip_disabled` captures the child with tooltip behavior omitted. The harness cannot capture platform accessibility in headless mode.
