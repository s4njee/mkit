# E8.10 N1 — Navigable graph surface

N1 adds a read-only node graph view in `registry/node-editor`. The app supplies node IDs, titles,
world positions, ordered input/output ports, port types, and existing connection endpoints. The
component renders node cards and directed cubic paths, exposes a text summary for each connection,
and leaves graph data unchanged.

## Navigation and view

The `MkitNodeEditor` key context exposes named, rebindable actions. Up/Down move through nodes in a
stable top-to-bottom, left-to-right, then ID order. Space selects the focused node. Enter enters its
port list; Up/Down traverse inputs followed by outputs; Escape returns focus to the node. Alt+arrow
keys pan, `=`/`-` zoom at the viewport center, F fits graph bounds, and 1 sets scale to 100% and
centers the graph. The mouse wheel zooms around its pointer location, trackpad pinch zooms at the
gesture center, and middle-button drag pans. Fit on an empty graph resets to the default transform.

`NodeEditor::new` owns view and selection state. `NodeEditor::controlled` treats the supplied
transform and selected node IDs as authoritative; interactions emit `ViewChanged` or
`SelectionChanged` requests and take effect after `set_transform` or `set_selection`. Both modes
emit `FocusChanged` for local keyboard navigation. The app can replace the read-only graph through
`set_graph`; no graph edits are emitted by N1.

## Semantics and theme

The root is a labeled group. Node cards expose title and selected state. Port names include
direction and declared type. Every connection is drawn with a direction marker and named in an
accessible text list by source and target node/port. Unresolved endpoints are skipped for painting
and receive an “Unresolved connection” summary.

The view reads palette, border, spacing, radii, typography, and controls from `mkit_core::theme::Theme`.
Node width and port spacing are fixed world geometry so they scale with the graph transform; their
values and rationale are recorded in the spec.

## Verification status

- `cargo check -p mkit-registry-node-editor` passes.
- Thirteen package tests pass, including node traversal, selection, port navigation, pan, zoom, fit,
  actual-size, typed connection editing, minimap visibility, and stable pointer connection preview.
- The screenshot manifest declares 13 states × 3 themes × 2 scales. All 78 NodeEditor captures were
  rendered on macOS and compared with the checked-in baselines. The accessibility cases are listed
  in the manifest, but native platform accessibility snapshots have not been captured.
- Native trackpad and screen-reader traversal need manual review. N1 public API, keymap, semantics,
  and visual treatment require maintainer review.

N1 itself remains the navigation slice; proposed movement and box selection are covered in N2,
typed connection editing in N3, and optional minimap navigation in N4.
