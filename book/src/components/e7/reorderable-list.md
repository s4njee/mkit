# ReorderableList

A stateful flat list for ordered records. It supports pointer drag-and-drop and reordering the active row with `Alt+ArrowUp` and `Alt+ArrowDown`. Each row has a stable ID and caller-provided label.

![ReorderableList drop target in the dark theme](../../images/e7/reorderable-list.png)

Use uncontrolled mode when the component may apply reorders locally. Controlled mode emits a move request and waits for the owner to call `set_items`; this keeps the displayed order aligned with app state. A move request identifies the item and its source and destination positions.

The list exposes list/list-item roles and row position/set-size metadata. Empty text and labels come from the app so callers can localize them. Live-region announcement timing still needs native accessibility verification.

This draft targets short and medium flat collections. It does not virtualize, and drag across offscreen rows is unsupported. VirtualList and LayerPanel have different row/tree contracts; integration remains a maintainer review gate.
