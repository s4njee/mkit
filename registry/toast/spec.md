---
spec_version: 1
component: toast
states:
  - id: visible
    description: The status surface renders title and content and requests dismissal after five seconds.
    fixture: toast_visible
  - id: dismissed
    description: The surface has zero size and no content children.
    fixture: toast_dismissed
  - id: busy
    description: The visible surface ignores dismissal requests.
    fixture: toast_busy
  - id: controlled
    description: The visible surface requests dismissal while waiting for its owner to close it.
    fixture: toast_controlled
keys:
  - key: Escape
    modifiers: []
    when: surface is open and not busy
    action: Request dismissal.
    initial_state: visible
    expect:
      state: dismissed
      event: OpenChanged(false)
  - key: Escape
    modifiers: []
    when: surface is open and busy
    action: Keep the surface open and emit no event.
    initial_state: busy
    expect:
      state: busy
      event: none
  - key: Escape
    modifiers: []
    when: controlled surface is open
    action: Request dismissal while leaving the supplied open value unchanged.
    initial_state: controlled
    expect:
      state: controlled
      event: OpenChanged(false)
accessibility:
  role: status
  properties:
    - name: live
      value: polite
      when: visible
controlled: Owner supplies open state through controlled constructor and set_open; uncontrolled dismissal updates state before emitting.
events: [OpenChanged]
theme_tokens: [background, surface, text, text_muted, border, spacing.xsmall, spacing.large, radii.large, borders.regular, typography.body, shadows.large]
open_questions: []
---

# Toast

## Purpose

A titled status surface for short feedback.

## Anatomy

An `Entity<Toast>` renders a root with a title text element and content text element while open. Closed state renders the root at zero size. The root keeps a `key_context` and Escape action.

## States

`new(title, content)` starts open and requests dismissal after five seconds. `controlled(title, content, open)` takes owner state; the timer emits `OpenChanged(false)` and waits for the owner to call `set_open(false)`. `busy(true)` suppresses Escape and timed dismissal. `set_busy(false)` starts a fresh five-second interval after a busy period. Closing invalidates the pending timer; reopening with `set_open(true)` starts a fresh interval. In controlled mode, a dismissal request is emitted once until the owner calls `set_open` again. `alert(true)` selects assertive `alert` semantics; the default is polite `status`.

The screenshot fixtures place one toast in a 460 × 220 logical-point panel. `visible` and `controlled` show the same “Changes saved” title and “Your workspace settings were updated.” content before timer expiry; `controlled` remains owner-held. `busy` shows a “Syncing changes” title with a waiting message. `dismissed` uses a closed controlled toast, leaving only the panel caption. All four are captured with shadcn light, dark, and high-contrast themes at 1× and 2×.

## Props and events

`set_open` applies a supplied boolean; `is_open` reads it. `set_busy` updates suppression and restarts the timeout when leaving busy state. In uncontrolled mode, Escape closes the surface and emits `OpenChanged(false)`. In controlled mode, Escape emits the same request but leaves `open` unchanged until `set_open`. The title and content are strings fixed at construction.

## Keyboard map

`Toast` key context binds Escape to `Dismiss`. The action is rebindable. It has no effect when closed or busy. A five-second timer requests dismissal. No Enter, Space, Tab, or focus-loop action is implemented.

## Pointer behaviour

The surface has no pointer handlers, close button, trigger, outside-click dismissal, or positioning logic. The host must provide any of these behaviors.

## Accessibility role and properties

The open root exposes AccessKit `status` role with polite live-region priority by default, or `alert` role with assertive priority when `alert(true)` is set. Its accessible name includes the title and content; it tracks a focus handle for Escape. Closed root has no live-region role. Actual assistive-technology announcements still need an active-platform check.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7.ts`, a `.ui-popover` surface styled in
`site/src/ui/ui.css` with the shadcn token mapping in `site/src/ui/tokens.ts`) and is resolved from the
installed `Theme` in three variants. `high-contrast` is selected by theme name (the convention other
registry components use); every other theme is treated as dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. No mkit-core
API or tokens are added. "Hairline" is shadcn's `border`: the `border` token in light themes and
`text` at 10% in dark themes.

| Part | Light / dark | High contrast |
|---|---|---|
| Fill | `surface` (shadcn `popover`) | `background` |
| Border | hairline | `border` |
| Shadow | `shadows.large` (shadcn `shadow-lg`) | none (the token is transparent) |
| Title | `text`, semibold | `text`, semibold |
| Content | `text_muted` (shadcn `muted-foreground`) | `text_muted` |

GPUI paints drop shadows as filled shapes that are not clipped to the element's outside, so the
fill is always opaque. The dismissed state renders nothing, as before.

Geometry: radius `radii.large` (the preview overrides `.ui-popover`'s medium radius with
`--ui-radius-lg`), `spacing.large` padding (shadcn `p-4`; the preview's 14px vertical
padding has no token), a `borders.regular` border, and a `spacing.xsmall` gap between title and
content (the nearest token to the preview's 2px gap; shadcn's `gap-1`). Title and content use
`typography.body` (14px); the preview's 13px description has no token. Semibold (600) is shadcn's
`font-semibold`; there is no font-weight token yet. Width is host-owned, like placement. The
preview's status icon and Undo action are not part of this component's API; adding an action button
would add a focus stop and an accessibility node, which needs a spec decision.

## WAI-ARIA pattern reference

[WAI-ARIA APG Alert Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/alert/) informs the optional assertive alert role; default notifications use the polite status role.

## Platform notes

Window-level placement is host-owned. The generated conformance manifest defines cases rather than reporting executed harness results.

## Open questions

The five-second timeout is fixed in this API. A deterministic GPUI test-clock check covers elapsed-time behavior, but platform live-region announcements remain unverified.
