---
spec_version: 1
component: empty-state
states:
  - id: basic
    description: Empty state shows title and description without an icon or actions.
    fixture: basic_fixture
  - id: illustrated
    description: Empty state includes an application-provided icon.
    fixture: illustrated_fixture
  - id: actionable
    description: Empty state includes one or more caller-provided actions.
    fixture: actionable_fixture
keys:
  - key: Tab
    modifiers: []
    when: Empty state contains actions.
    action: Move through caller-provided action controls in normal platform order.
    initial_state: actionable
    expect:
      focus_target: first_action
accessibility:
  role: group
  properties:
    - name: label
      value: title
    - name: description
      value: optional explanatory text
    - name: actions
      value: caller-provided controls, in insertion order
theme_tokens: [background, text, text_muted, spacing.small, spacing.large, spacing.xlarge, spacing.xxlarge, radii.large, controls.large, typography.heading_small, typography.body]
open_questions: [Review layout API and whether the empty state should impose a minimum height.]
---

# Empty state

## Purpose

Explain why a view has no content and offer a clear path forward when one exists.

## Anatomy

A stateless centered group with optional caller-provided icon, required title, optional description, and zero or more caller-provided action elements. The component does not create data or trigger application operations itself.

## States

Basic title and description, illustrated with an icon, and actionable with one or more controls. Missing description, icon, or actions are valid. The component has no internal state.

## Props and events

`EmptyState::new(title)` creates a `RenderOnce` builder. `.description(text)` adds supporting copy, `.icon(element)` adds an app-owned visual, and `.action(element)` appends an action control. Caller-provided controls own their interaction and events; EmptyState does not re-emit them.

## Keyboard map

The component binds no keys. Tab traverses caller-provided action controls in normal platform order; actions must supply their own accessible names and focus behavior.

## Pointer behaviour

Pointer behavior is delegated to caller-provided action elements. The surrounding empty state is not interactive.

## Accessibility role and properties

The root has group role and is labelled by its title. Description text is exposed as supporting text. The icon is decorative unless the caller's element adds a meaningful accessible name. Action controls remain child elements in insertion order with their own semantics.

## Theme tokens used

The docs site has no separate EmptyState preview, so the look follows the shadcn/ui `Empty`
composition (`Empty`, `EmptyHeader`, `EmptyMedia variant="icon"`, `EmptyTitle`,
`EmptyDescription`, `EmptyContent`) with the site's shadcn token mapping in
`site/src/ui/tokens.ts`, resolved from the installed `Theme` the same way Button, Select, and Tabs
do it. `high-contrast` is selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" is shadcn's `muted`: `text` mixed 4% (light) or 12% (dark) into
`background`.

| Part | Light / dark | High contrast |
|---|---|---|
| Icon tile fill | muted | `background`, with a `borders.hairline` `border` outline |
| Icon tile content colour | `text` (shadcn `text-foreground`) | `text` |
| Title | `text` | `text` |
| Description | `text_muted` | `text_muted` |

- **Icon tile.** When an icon is supplied it sits in a `controls.large` (40px, shadcn `size-10`)
  square with radius `radii.large` (`rounded-lg`) and a muted fill; the tile sets the colour its
  content inherits. The icon remains application supplied; shadcn sizes it to 24px
  (`spacing.xlarge`), and the screenshot fixtures pass a Lucide icon drawn as a vector path at that
  size.
- **Geometry.** The group is centred with `spacing.xlarge` (24px, shadcn `p-6`) padding and no
  border (shadcn's default `Empty`). The icon tile has `spacing.small` (8px, `mb-2`) extra space
  below it, and the header items are `spacing.small` (8px, `gap-2`) apart; the actions sit
  `spacing.xlarge` (24px, `gap-6`) below the header, `spacing.small` apart. The title is
  `typography.heading_small` (16px) at medium weight (500); shadcn's `text-lg` (18px) has no token.
  The description is `typography.body` (14px) in `text_muted`, centred, at most `spacing.xxlarge ×
  12` (384px, shadcn `max-w-sm`) wide. There is no font-weight token yet.
- EmptyState has no focusable parts of its own (actions are caller supplied and styled by the
  caller), so its screenshot matrix has no focus state.

## WAI-ARIA pattern reference

Empty state has no dedicated APG pattern. It uses the [WAI-ARIA Group Role] for a related title, description, and optional actions; contained buttons follow their own patterns.

## Platform notes

GPUI maps the named group through AccessKit. Screen readers should encounter the title, supporting description, then action controls in visual order. Verify this order in active platform accessibility snapshots.

## Open questions

- Maintainers should review the public builder methods and default alignment.
- Decide whether gallery layouts should provide minimum height externally or through an optional token-based size.

[WAI-ARIA Group Role]: https://www.w3.org/TR/wai-aria-1.2/#group
