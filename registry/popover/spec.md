---
spec_version: 1
component: popover
states:
  - id: closed
    description: The trigger is rendered and the floating surface is absent.
    fixture: popover_closed
  - id: open
    description: The trigger and anchored floating surface are rendered.
    fixture: popover_open
  - id: busy
    description: The open surface ignores Escape and outside-pointer dismissal.
    fixture: popover_busy
keys:
  - key: Escape
    modifiers: []
    when: surface is open and not busy
    action: Request dismissal and restore focus to the trigger.
    initial_state: open
    expect:
      event: OpenChanged
accessibility:
  role: dialog
  properties: []
controlled: Owner supplies open state through controlled constructor and set_open; uncontrolled dismissal updates state before emitting.
events: [OpenChanged]
theme_tokens: [background, surface, text, text_muted, border, accent, focus, spacing.xsmall, spacing.small, spacing.medium, spacing.large, controls.medium, radii.medium, borders.regular, typography.body, shadows.small, shadows.medium]
open_questions: []
---

# Popover

## Purpose

A nonmodal supplemental surface opened from a component-owned trigger. The trigger is a keyboard-focusable button; the floating surface is rendered with GPUI's `deferred` and `anchored` primitives.

## Anatomy

The entity renders an outline-button trigger and, while open, a deferred dialog surface. A trigger activation places the surface below the trigger's measured left edge with a `spacing.small` gap when it fits. If the measured surface will not fit below with the `spacing.medium` viewport margin, it flips above the trigger with the same gap. GPUI keeps the surface within the viewport.

## Anatomy and API

`Popover::new(title, content).trigger(label)` creates an uncontrolled, initially closed popover. `Popover::controlled(title, content, open).trigger(label)` creates the controlled form. The trigger label is fixed at construction. `set_open` updates state. GPUI prepaint exposes the trigger child's bounds; trigger activation uses those bounds for a below-start anchor. `anchor_at(point)` supplies a window-coordinate fallback for an initially-open controlled surface and is replaced by the measured trigger anchor after trigger activation. When no surface measurement exists, the first layout/prepaint frame is visually transparent while the deferred surface is measured. The first visible frame uses that measured height to choose above or below placement.

## States

The closed state renders only the trigger. The open state renders a deferred dialog surface. Its initial measurement frame is transparent to prevent a visibly misplaced first frame; after prepaint measures it, the next frame is visible at the selected placement. `busy(true)` suppresses Escape and outside-pointer dismissal. There is no separate disabled or urgency variant.

## Props and events

The `title`, `content`, and trigger label are fixed strings. `OpenChanged(bool)` reports trigger and dismissal requests. Controlled mode is authoritative until the owner calls `set_open`; uncontrolled mode applies the request immediately. Placement uses the last measured surface height and trigger bounds. Until the first surface measurement, the surface remains visually transparent; its layout and measurement still run.

## Events and state contract

The trigger requests open in uncontrolled mode and emits `OpenChanged(true)`. Escape, outside pointer down, and trigger activation while open request close and emit `OpenChanged(false)`. Uncontrolled mode applies the request immediately; controlled mode waits for `set_open` from the owner. Closing restores focus to the trigger when its handle remains alive.

## Keyboard map

The `Popover` key context binds Escape to `Dismiss`. The action is rebindable. Escape has no effect when closed or busy. The trigger uses button semantics and GPUI click activation. Tab remains in the ordinary document order; this nonmodal popover does not trap focus.

## Pointer behaviour

Clicking the trigger opens or closes the surface. A pointer press outside the rendered surface requests dismissal. The trigger is treated as part of the popover interaction and toggles it directly. Nested overlay routing remains the host's responsibility; dispatch dismissal to the topmost surface first.

## Accessibility role and properties

The open surface has AccessKit `dialog` role. The trigger has button role and is keyboard focusable. The component returns focus to the trigger after close. It is nonmodal and does not trap focus.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7.ts`, styled by `.ui-btn--outline` and
`.ui-popover` in `site/src/ui/ui.css` with the shadcn token mapping in `site/src/ui/tokens.ts`) and is
resolved from the installed `Theme` in three variants. `high-contrast` is selected by theme name (the
convention other registry components use); every other theme is treated as dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" is shadcn's `accent`: `text` mixed 4% (light) or 12% (dark) into
`background`. "Hairline" is shadcn's `border`: the `border` token in light themes and `text` at 10%
in dark themes.

| Part | Light / dark | High contrast |
|---|---|---|
| Trigger fill | `background` (shadcn outline button) | `background` |
| Trigger text | `text`, medium weight | `text`, medium weight |
| Trigger border | hairline | `border` |
| Trigger shadow | `shadows.small` | none (the token is transparent) |
| Trigger hover | muted fill | `accent` border |
| Trigger focus | border `focus` plus a 3px ring of `focus` at 50% | border `focus` plus a 3px ring of opaque `focus` |
| Surface fill | `surface` (shadcn `popover`) | `background` |
| Surface border | hairline | `border` |
| Surface shadow | `shadows.medium` (shadcn `shadow-md`) | none (the token is transparent) |
| Title | `text`, semibold | `text`, semibold |
| Body | `text_muted` (shadcn `muted-foreground`) | `text_muted` |

GPUI paints drop shadows as filled shapes that are not clipped to the element's outside, so both
the trigger and the surface fills are opaque. The trigger matches the Button component's outline
variant; it is drawn locally because registry components depend only on mkit-core and GPUI.

Geometry: the trigger is `controls.medium` tall (36px, shadcn `h-9`) with `spacing.large`
horizontal padding (shadcn `px-4`), radius `radii.medium`, and a `borders.regular` border, and it
sizes to its label rather than stretching across its container. The surface has radius
`radii.medium` (shadcn `rounded-md`), `spacing.large` padding (shadcn `p-4`), a `borders.regular`
border, and a `spacing.xsmall` gap between title and body (the web preview's 4px header gap). The
surface width follows its content: shadcn's fixed `w-72` (288px) and the preview's 300px have no
size token, and fixed widths would clip longer localized strings. Title and body use
`typography.body` (14px; the preview's 13px description has no token). Font weights are shadcn's
`font-medium` (500) and `font-semibold` (600); there is no font-weight token yet. The 3px focus ring
is the shadcn/ui ring width, a fixed component value shared with Button.

## WAI-ARIA pattern reference

[WAI-ARIA APG Dialog Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog/) informs the dialog semantics. This primitive is nonmodal and does not implement dialog focus trapping.

## Platform notes

GPUI's child-prepaint callback provides the rendered trigger bounds before a pointer activation. The surface aligns with that left edge, with `spacing.small` gap, below or above according to available viewport space. A previously measured surface height chooses placement; an initially open surface can correct placement after its first measurement. Initial focus remains on the trigger until keyboard navigation moves into the surface.

The generated conformance manifest defines cases rather than reporting executed harness results. The first visible open frame is checked by a GPUI test after the transparent measurement frame.

## Open questions

The public trigger builder, initial-open `anchor_at` fallback, and placement behavior are pending maintainer review.
