---
spec_version: 1
component: reorderable-list
states:
  - id: default
    description: Ordered rows with one active item and drag affordances.
    fixture: default_fixture
  - id: empty
    description: Empty list with its accessible name and owner-provided empty text.
    fixture: empty_fixture
  - id: active
    description: Keyboard-active row with position and size semantics.
    fixture: active_fixture
  - id: dragging
    description: One row is being dragged and shown with a drag preview.
    fixture: dragging_fixture
  - id: drop_target
    description: A valid insertion edge is visibly indicated for the dragged item.
    fixture: drop_target_fixture
  - id: reordered
    description: A row has moved and its position semantics describe the new order.
    fixture: reordered_fixture
  - id: controlled
    description: A move request is pending owner acknowledgement; order is unchanged.
    fixture: controlled_fixture
  - id: focused
    description: The active row has keyboard focus from Tab and shows the focus outline.
    fixture: focused_fixture
keys:
  - key: ArrowUp
    modifiers: [Alt]
    when: An enabled row is active and has a predecessor.
    action: Move the active row up one position.
    initial_state: default
    expect:
      event: ReorderRequested
  - key: ArrowDown
    modifiers: [Alt]
    when: An enabled row is active and has a successor.
    action: Move the active row down one position.
    initial_state: default
    expect:
      event: ReorderRequested
accessibility:
  role: list
  properties:
    - name: label
      value: caller-provided list label
    - name: child_role
      value: Each row is exposed as a list item.
    - name: position_in_set
      value: one-based row position
    - name: size_of_set
      value: total item count
    - name: description
      value: moved row position and total count
      when: reordered
controlled: Owner supplies the ordered item array. Reorder attempts emit `ReorderRequested` and leave the visible order unchanged until `set_items` receives the owner's order. Uncontrolled mode updates the array before emitting `Reordered`.
events: [ReorderRequested, Reordered]
theme_tokens: [background, surface, text, text_muted, accent, accent_text, focus, border, disabled, shadows.medium, spacing.xsmall, spacing.small, spacing.large, controls.medium, radii.small, radii.medium, radii.large, borders.hairline, borders.strong, typography.body]
open_questions: [Maintainer review of keyboard modifiers, role semantics, controlled events, virtualization limits, and position announcements.]
---

# ReorderableList

## Purpose

Let users reorder a short or medium ordered collection with either pointer drag-and-drop or keyboard commands.

## Anatomy

A labeled list contains stable-ID rows. Each row shows caller-owned text. During drag, a floating preview follows the pointer and a clear insertion edge marks the destination. The active row receives visible emphasis.

## States

- `Default`: ordered rows with one active row when nonempty.
- `Empty`: no rows; optional app-supplied empty text.
- `Active`: keyboard-active row with current one-based position.
- `Dragging`: a row is dragged within the list.
- `Drop target`: a valid insertion location is highlighted; invalid/self targets are ignored.
- `Reordered`: new row order and position semantics are reflected.
- `Controlled`: a reorder request is emitted without optimistic change until the owner updates items.
- `Focused`: the active row has keyboard focus (from Tab) and shows the focus outline.

## Props and events

Stateful GPUI `Entity<ReorderableList>` with typed `ReorderRequested` and `Reordered` events. Each `ReorderableItem` has a stable ID and label. Controlled mode receives the ordered item array and applies owner updates with `set_items`; uncontrolled mode changes local order. The active ID can be set by the owner. No custom row rendering or nested lists are included in this draft.

## Keyboard map

The list registers the rebindable `MkitReorderableList` key context. `Alt+ArrowUp` and `Alt+ArrowDown` move the active row one position and keep that row active. Boundary moves do nothing. This avoids taking plain Arrow keys away from row content.

## Pointer behaviour

Drag an enabled row and drop before an enabled sibling in the same flat list. A drag preview follows the pointer. The destination edge appears only for a valid target; dropping on itself or outside the list leaves order unchanged. Keyboard commands provide the complete alternative to pointer reordering.

## Accessibility role and properties

The container exposes `List`; each row exposes `ListItem`, a label, one-based position, and total set size. The active row receives the sole tab stop, a persistent active outline, and a solid focus outline while it has keyboard focus. After a move, its position and set-size semantics update; this is the available announcement mechanism because the pinned GPUI bridge has no live-region property. Native screen-reader announcement timing needs platform verification.

## Theme tokens used

Uses `Theme.colors.background`, `surface`, `text`, `text_muted`, `accent`, `accent_text`, `focus`,
`border`, and `disabled`; `Theme.shadows.medium`, `Theme.spacing.xsmall/small/large`,
`controls.medium`, `radii.small/medium/large`, `borders.hairline/strong`, and `typography.body`.

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts`, the `reorderable-list`
demo, styled by `.e7-reorder-*` in `site/src/demos/e7_expansion.css` with the shadcn token mapping in
`site/src/ui/tokens.ts`). Colours are resolved from the installed `Theme` in three variants, the way
Button, Select, Tabs, and Sidebar do it: `high-contrast` is selected by theme name; every other theme
is dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light.
Derived colours use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no
mkit-core API or tokens are added. "Muted" below is `text` mixed 4% (light) or 12% (dark) into
`background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| List fill | `surface` | `surface` | `surface` |
| List border | `border` | `text` at 10% | `border` |
| Row text | `text` | `text` | `text` |
| Grip dots | `text_muted` | `text_muted` | `text` (`accent_text` on the active row) |
| Active row | muted fill, `borders.hairline` outline in `focus` at 50% | same | `accent` fill, `accent_text` |
| Active row with keyboard focus | muted fill, outline in `focus` | same | `accent` fill, outline in `focus` |
| Pointer hover (enabled, inactive row) | muted fill | muted fill | unchanged |
| Disabled row text and grip | composited over `surface` and mixed 50% with it | same | `disabled` |
| Drop indicator | `borders.strong` (2px) top edge in `accent` (shadcn `primary`) | same | same (`accent`) |
| Drag preview | `surface` fill, list border, `shadows.medium` | same | `surface` fill, `border`, no shadow |

- **Geometry.** The list is a card: radius `radii.large`, a `borders.hairline` border, and
  `spacing.xsmall` (4px, the web's `padding: 4px`) padding. Rows are `controls.medium` (36px, the
  web's `min-height: 36px`) tall with `spacing.small` (8px) horizontal padding and gap, radius
  `radii.small`, and `typography.body` (14px) labels that truncate. The web uses 13px row text; there
  is no 13px token, and 14px matches the other list components. Empty text uses the same row height
  and padding in `text_muted`.
- **Grip.** Each row starts with Lucide `grip-vertical`: six dots on a 24-unit grid drawn as filled
  circles in a 16px (`spacing.large`) square, the size of the other components' icons (the web uses
  14px). The grip is decorative; the whole enabled row remains the drag source.
- **Active row and focus.** The active row keeps the web's `is-active` look (muted fill and a 1px
  outline at about half the ring colour) at all times. While the list or its active row has keyboard
  focus (`:focus-visible`), the outline becomes solid `focus`. The outline is an overlay inside the
  row, so it neither shifts content nor overlaps neighbouring rows, and the row's own top border stays
  free for the drop indicator. In high contrast the solid `accent` fill marks the active row and the
  `focus` outline appears only with keyboard focus.
- **Drop indicator.** A valid drop target draws a 2px (`borders.strong`) line in `accent` (shadcn's
  primary) along its top edge, with square top corners so the line runs straight.
- **Drag preview.** The preview is a popover-style row: `controls.medium` tall, radius
  `radii.medium`, a hairline border, `shadows.medium` (none in high contrast), the grip, and the
  label. Its 180px width is a fixed preview width.
- **Disabled** rows match a web `opacity: .5` as one layer: the text and grip colours are composited
  over the list's `surface` and mixed 50% with it. GPUI element opacity is not used. High contrast
  uses the solid `disabled` token.

## WAI-ARIA pattern reference

Uses the WAI-ARIA `list` and `listitem` roles for static collection semantics and APG keyboard principles for the alternative move commands. This is not a listbox: rows are reorderable records rather than selectable options.

## Platform notes

The draft renders all items in one scrollable list and does not virtualize rows. It is intended for short and medium collections. Dragging across offscreen rows is not supported. VirtualList currently owns selection and viewport slicing and offers no row-level reorder hooks; integration requires an explicit shared data/row contract. LayerPanel supports nested groups and same-parent-only reordering with its own tree interactions; adopting this flat list's move helper is not a drop-in replacement.

## Open questions

- Maintainer review is needed for Alt+Arrow shortcuts, position description announcements, and event naming.
- VirtualList and LayerPanel integration remains a follow-up review; this package does not add a dependency or modify either component.
