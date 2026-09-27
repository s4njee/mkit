# ReorderableList

`ReorderableList` is a stateful, flat collection for short and medium ordered sets. It supports pointer drag-and-drop and `Alt+ArrowUp` / `Alt+ArrowDown` reordering. Rows have stable IDs and caller-provided labels; the component supplies list/list-item semantics and one-based position metadata.

## State contract

Use `ReorderableList::new` for uncontrolled order, or `ReorderableList::controlled` when the owner stores the order. Both expose `ReorderRequested { id, from, to }`; uncontrolled mode also applies the move immediately and emits `Reordered`. Controlled mode leaves its displayed order unchanged until `set_items` receives the owner’s new sequence. Boundaries and disabled rows cannot be moved into or out of.

An active row is focusable. The list registers `MkitReorderableList`, and the default key bindings can be included in the host app’s binding setup. Pointer drag shows a row preview and highlights valid drop targets. Empty text and labels are supplied by the app so the component does not impose locale-specific wording.

## Accessibility

The root has the `list` role and the supplied accessible label. Each row has the `listitem` role, accessible label, one-based position, and set size. Position metadata updates when uncontrolled order changes. A live-region property is not exposed by the pinned accessibility bridge, so native announcement timing requires platform verification.

## Integration limits

This draft renders its flat collection directly and does not virtualize. It is designed for short and medium lists, not thousands of rows or dragging across offscreen items. `VirtualList` currently owns viewport slicing and selection and has no row reorder hook; integration needs a shared item/row contract. `LayerPanel` owns a nested tree and same-parent drop rules, so it is not a drop-in consumer. Both integrations require a maintainer review gate before a dependency is added.

## Review status

The public API, controlled event contract, keyboard modifiers, list/list-item semantics, and accessibility announcement behavior are draft decisions pending maintainer review. Screenshot and keyboard harness evidence is recorded in the conformance manifest; native accessibility snapshots remain pending platform support.
