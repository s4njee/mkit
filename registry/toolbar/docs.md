# Toolbar

`Toolbar` groups common actions behind one keyboard tab stop. Its stateful API supports push
buttons, toggle buttons, toggle groups, separators, horizontal or vertical orientation, and an
overflow menu. `visible_capacity` provides an explicit number of overflow-eligible controls to keep
inline; never-overflow controls remain visible even when this limit is exceeded. Toggle groups move
as one unit if all choices do not fit in the remaining capacity.

Create the view with `Toolbar::new(label, items)` for uncontrolled values or
`Toolbar::controlled(label, items)` for request-only toggle updates. Subscribe to `ToolbarEvent` to
handle actions and value requests. Controlled owners apply the accepted values through
`set_toggle` and `set_group_value`.

Use Left/Right in a horizontal toolbar and Up/Down in a vertical toolbar. Home and End move to the
first and last enabled toolbar item. Inside a toggle group, the arrow axis marks a choice and
Enter/Space commits it. Disabled actions are skipped. The overflow menu lists moved actions,
including disabled actions, and skips disabled rows during navigation. Pixel-based automatic overflow is pending a GPUI layout
measurement contract; this version lets the app select an action capacity explicitly.

Keyboard, pointer, controlled-state, and overflow behavior is covered by focused crate tests. The
conformance manifest records the full light/dark/high-contrast and 1x/2x screenshot matrix as
pending until the supported platform capture is run. Native accessibility snapshots are also
pending.
