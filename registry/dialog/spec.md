---
spec_version: 1
component: dialog
states:
  - id: closed
    description: The surface has zero size and no content children.
    fixture: dialog_closed
  - id: open
    description: A host-viewport modal layer blocks host pointer input and centers the titled surface.
    fixture: dialog_open
  - id: busy
    description: The open surface shows an accessible busy status and ignores dismissal requests.
    fixture: dialog_busy
  - id: interactive
    description: Caller-built content supplies two enabled focus stops in visual order.
    fixture: dialog_interactive
keys:
  - key: Escape
    modifiers: []
    when: surface is open and not busy
    action: Request dismissal.
    initial_state: open
    expect:
      state: closed
      event: OpenChanged(false)
  - key: Tab
    modifiers: []
    when: surface is open
    action: Keep focus within the surface.
    initial_state: open
    expect:
      focus_target: surface
  - key: Tab
    modifiers: [Shift]
    when: surface is open
    action: Keep focus within the surface.
    initial_state: open
    expect:
      focus_target: surface
  - key: Tab
    modifiers: []
    when: interactive surface is open with the first content control focused
    action: Move focus to the second content control.
    initial_state: interactive
    expect:
      focus_target: second_control
  - key: Tab
    modifiers: [Shift]
    when: interactive surface is open with the first content control focused
    action: Wrap focus backward to the second content control.
    initial_state: interactive
    expect:
      focus_target: second_control
accessibility:
  role: dialog
  properties:
    - name: aria-label
      value: title
      when: open
    - name: aria-label
      value: title
      when: busy
    - name: aria-label
      value: title
      when: interactive
controlled: Owner supplies open state through controlled constructor and set_open; uncontrolled dismissal updates state before emitting.
events: [OpenChanged]
theme_tokens: [background, surface, text, text_muted, border, accent, spacing.xsmall, spacing.small, spacing.large, spacing.xlarge, radii.medium, radii.large, borders.regular, typography.body, typography.heading_small, typography.heading, shadows.large]
open_questions: []
---

# Dialog

## Purpose

A titled modal dialog that manages initial focus, Escape/outside dismissal, a blocking backdrop, and a focus trap across rendered or caller-provided focus stops.

## Anatomy

An `Entity<Dialog>` renders a viewport-sized modal layer with a centered dialog card containing a title and either string content or caller-built content. The layer consumes pointer interaction outside the card and requests dismissal. Closed state renders a zero-size root. The card keeps a `key_context` and Escape action.

## States

`new(title, content)` starts open. `controlled(title, content, open)` takes owner state. `busy(true)` shows a token-styled status message explaining that work is in progress and dismissal is blocked. It also suppresses Escape and outside-pointer dismissal while open. Busy is a construction-time builder setting; there is no separate disabled or urgency variant.

## Props and events

`set_open` applies a supplied boolean; `is_open` reads it. In uncontrolled mode, Escape closes the surface and emits `OpenChanged(false)`. In controlled mode, Escape emits the same request but leaves `open` unchanged until `set_open`. The title and string content are fixed at construction. `with_content` accepts an element factory for interactive content. `focus_stops` optionally supplies exact focus handles in visual order for custom traversal; the caller must track those handles on the corresponding controls. Without explicit stops, the dialog uses GPUI's rendered tab order and wraps when traversal would leave its focus subtree.

## Keyboard map

`Dialog` key context binds Escape to `Dismiss`, Tab to `FocusForward`, and Shift+Tab to `FocusBackward`. Actions are rebindable. Escape has no effect when closed or busy. Without explicit focus stops, Tab and Shift+Tab follow GPUI's rendered tab order inside the dialog, wrapping to the root when the next stop is outside. With supplied stops, traversal wraps across those handles in visual order, starting on the first stop at open.

## Pointer behaviour

The open backdrop fills the available host viewport. Pointer down outside the card requests dismissal unless busy, and is consumed by the backdrop so host controls cannot receive the press. The card is centered in that host-provided viewport; callers should mount the dialog at the window content root for viewport placement.

## Accessibility role and properties

The open card exposes AccessKit `dialog` role with the supplied title as its accessible name. While busy, a visible child exposes AccessKit `status` role, the name `Work in progress`, and the text `Dismissal is paused until this work finishes.` The root receives initial focus without explicit stops; the first supplied focus stop receives initial focus with them. Tab is trapped within the dialog's focus subtree, and the previously focused handle is restored on close when it remains alive. The host must hide background content from its accessibility subtree while a modal is open. Backdrop blocks host pointer interaction. Closed root has no `dialog` role. An active-platform snapshot must verify the name and focus relationship.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7.ts`, styled by `.ui-overlay` and
``.ui-card`` in `site/src/ui/ui.css` with the shadcn token mapping in `site/src/ui/tokens.ts`) and is resolved
from the installed `Theme` in three variants. `high-contrast` is selected by theme name (the
convention other registry components use); every other theme is treated as dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Hairline" is shadcn's `border`: the `border` token in light themes and `text` at
10% in dark themes. "Muted" is shadcn's `muted`: `text` mixed 4% (light) or 12% (dark) into
`background`. "Ink" is the theme's near-black: `text` in light themes and `background` in dark and
high-contrast themes (the web uses literal black).

| Part | Light / dark | High contrast |
|---|---|---|
| Backdrop | ink at 50% (shadcn `bg-black/50`) | ink (`background`) at 50% |
| Card fill | `surface` (the preview's `card`; shadcn's `background` is identical in `shadcn-light`) | `background` |
| Card border | hairline | `border` |
| Card shadow | `shadows.large` (shadcn `shadow-lg`) | none (the token is transparent) |
| Title | `text`, semibold | `text`, semibold |
| String content | `text_muted` (shadcn `DialogDescription`) | `text_muted` |
| Caller-built content | inherits `text`; the caller styles it | same |
| Busy status | muted fill, `text_muted` text, `radii.medium`, `accent` dot | `background` fill, `border` outline, `text_muted` text, `accent` dot |

GPUI paints drop shadows as filled shapes that are not clipped to the element's outside, so the card
fill is always opaque.

Geometry: the card has radius `radii.large` (shadcn `rounded-lg`), `spacing.xlarge` padding (shadcn
`p-6`), a `borders.regular` border, and a `spacing.large` gap between the header and caller-built
content or the busy status (shadcn `gap-4`). Title and string content form shadcn's header with a gap
at the midpoint of `spacing.xsmall` and `spacing.small` (6px, the preview's and shadcn's
`gap-1.5`), because there is no 6px token. The title uses the midpoint of `typography.heading_small`
and `typography.heading` (18px, shadcn `text-lg`), since there is no 18px type token and either
neighbour visibly changes the heading's weight in the card; string content uses `typography.body`
(14px, shadcn `text-sm`). Semibold (600) is shadcn's `font-semibold`; there is no font-weight token
yet. The card width follows its content: shadcn's `max-w-lg` and the preview's 420px have no size
token. The preview's close button and footer buttons are caller content (or a future API) rather
than part of this surface; adding a built-in close button would add a focus stop and an
accessibility node, which needs a spec decision.

## WAI-ARIA pattern reference

[WAI-ARIA APG Dialog (Modal) Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/).

## Platform notes

Placement is relative to the containing host viewport, which should be the window content root. The generated conformance manifest defines cases rather than reporting executed harness results.

## Open questions

The caller must keep `focus_stops` synchronized with rendered and enabled controls. GPUI view composition does not expose a cross-window/global modal stack; nested overlays must be mounted and ordered by the host.
