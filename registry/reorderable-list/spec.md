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
theme_tokens: [surface, elevated_surface, text, text_muted, accent, accent_text, focus, border, disabled, spacing.xsmall, spacing.small, spacing.medium, controls.small, radii.small, radii.medium, borders.hairline, borders.strong]
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

## Props and events

Stateful GPUI `Entity<ReorderableList>` with typed `ReorderRequested` and `Reordered` events. Each `ReorderableItem` has a stable ID and label. Controlled mode receives the ordered item array and applies owner updates with `set_items`; uncontrolled mode changes local order. The active ID can be set by the owner. No custom row rendering or nested lists are included in this draft.

## Keyboard map

The list registers the rebindable `MkitReorderableList` key context. `Alt+ArrowUp` and `Alt+ArrowDown` move the active row one position and keep that row active. Boundary moves do nothing. This avoids taking plain Arrow keys away from row content.

## Pointer behaviour

Drag an enabled row and drop before an enabled sibling in the same flat list. A drag preview follows the pointer. The destination edge appears only for a valid target; dropping on itself or outside the list leaves order unchanged. Keyboard commands provide the complete alternative to pointer reordering.

## Accessibility role and properties

The container exposes `List`; each row exposes `ListItem`, a label, one-based position, and total set size. The active row receives the sole tab stop and visible focus styling. After a move, its position and set-size semantics update; this is the available announcement mechanism because the pinned GPUI bridge has no live-region property. Native screen-reader announcement timing needs platform verification.

## Theme tokens used

Uses `Theme.colors.surface`, `elevated_surface`, `text`, `text_muted`, `accent`, `accent_text`, `focus`, `border`, and `disabled`; `Theme.spacing.xsmall/small/medium`, `controls.small`, `radii.small/medium`, and `borders.hairline/strong`.

## WAI-ARIA pattern reference

Uses the WAI-ARIA `list` and `listitem` roles for static collection semantics and APG keyboard principles for the alternative move commands. This is not a listbox: rows are reorderable records rather than selectable options.

## Platform notes

The draft renders all items in one scrollable list and does not virtualize rows. It is intended for short and medium collections. Dragging across offscreen rows is not supported. VirtualList currently owns selection and viewport slicing and offers no row-level reorder hooks; integration requires an explicit shared data/row contract. LayerPanel supports nested groups and same-parent-only reordering with its own tree interactions; adopting this flat list's move helper is not a drop-in replacement.

## Open questions

- Maintainer review is needed for Alt+Arrow shortcuts, position description announcements, and event naming.
- VirtualList and LayerPanel integration remains a follow-up review; this package does not add a dependency or modify either component.
