# E8.11 Layer panel

The registry draft provides a reorderable nested layer tree with visibility and lock controls,
optional image thumbnails or colour swatches, group expansion, typed events, and controlled and uncontrolled modes.
The component uses only GPUI and `mkit-core`.

## State contract

`LayerPanel::new` owns its active row and layer metadata. Accepted interactions update local state
and then emit a typed event. `LayerPanel::controlled` leaves the active row and layer data with the
owner; events request changes, and the owner applies them with `set_active`, `set_layers`, or
`set_expanded`. Reorders report the node ID, destination parent ID, and sibling index.

## Current implementation limits

- Reordering is available through Alt+Up/Down, per-row move-up/move-down buttons, and pointer drag.
  A row can be dropped before another row in the same sibling list; the compatible target gets an
  accent border and elevated surface during the drag. Cross-parent drops and dropping into groups
  are intentionally unsupported in this version.
- `with_thumbnail` accepts a GPUI resource path, URI, or embedded asset name. GPUI loads and renders
  the image; callers provide the resource. `with_swatch` supports color-only previews.
- The conformance manifest is exercised with a 30-case screenshot matrix: five declared states,
  three themes, and 1×/2× scale. The dragging state is captured by dispatching a real pointer drag
  over a compatible same-parent target. Active platform accessibility snapshots remain open.
- Tree rows expose role, name, level, and selected state; group rows expose expanded state.
  Sibling position and set size are not yet surfaced in the rendered accessibility tree.

## Evidence

The package includes data tests for tree flattening and same-parent target validation, plus GPUI
pointer scripts for uncontrolled nested sibling moves, controlled reorder requests, cross-parent
drop rejection, pointer toggles, and keyboard reorder. Public API, event naming, keyboard behavior,
accessibility semantics, and visual treatment need maintainer review. The full keyboard matrix,
active platform accessibility snapshots, and fresh workspace coexistence check remain pending.
