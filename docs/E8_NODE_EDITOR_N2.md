# E8.10 N2 — Node movement and box selection

## Contract

N2 extends the N1 read-only graph surface with selection gestures and proposed node moves. The graph remains app-owned in both view/selection modes: accepted positions arrive only through `set_graph`. `NodesMoveRequested { positions: Vec<NodePosition> }` reports final positions for stable node IDs. Drag positions exist only as a visual preview; pointer-up emits one request, and the preview is discarded immediately. If the owner rejects or accepts different positions, the next authoritative graph remains the displayed truth. Escape cancels previews without emission. Invalid or non-finite positions are rejected as a whole.

View and selection retain N1's independent controlled flags. Uncontrolled selection updates locally before `SelectionChanged`; controlled selection requests and waits for `set_selection`. Move requests are always proposal-only and never mutate graph positions in either mode.

## States

- `selected`: one or more nodes selected separately from keyboard focus.
- `drag_preview`: selected node(s) follow the pointer with their relative offsets preserved.
- `box_preview`: a screen-space rectangle and live node set are shown.
- `box_committed`: pointer-up or keyboard commit sends the resulting selection through the usual selection contract.

Node hit testing uses reverse paint order; a node is hit when the pointer lies inside its card bounds. Box selection uses full containment of each node card in the world-space rectangle. Nodes partially intersecting the rectangle are excluded. Negative coordinates and non-finite nodes are handled deterministically.

## Pointer and keyboard

Plain click focuses and replaces selection with that node. Shift-click adds a node; platform-command-click toggles it. Dragging a selected node moves the whole selected set; dragging an unselected node first replaces selection and then moves it. Empty-canvas left drag starts box selection; Shift adds the resulting set and platform-command toggles it. Middle drag and Space+left drag continue to pan.

A delivered pointer release inside the canvas supplies the final coordinates for node movement and box selection, including when no final move event arrived. A delivered release outside the canvas discards either preview without a move or selection request. Connection drags also cancel outside the canvas and report that status. Native delivery of releases after the pointer has left the canvas remains a platform check.

Keyboard actions are registered in `MkitNodeEditor`: Space selects/replaces, Shift+Space adds, platform+Space toggles, arrows retain N1 focus navigation, and Shift+Alt+arrow nudges the selected set by a fixed 1 world unit. `Escape` cancels a drag or box preview. Box selection is keyboard accessible through `BeginBoxSelection` (initial rectangle encloses the focused node), Shift+Arrow actions move the active corner by one world unit, `CommitBoxSelection` applies full-containment selection, and `Escape` cancels. These commands can be rebound. The 1-unit nudge/corner step is a graph-space interaction constant; pointer movement remains continuous.

## Accessibility and theme

The root remains a labeled group. Nodes expose selected state and the focused node remains predictable after selection changes. The live box preview is a labeled status description of the selected-node count; the rectangle uses the theme accent with reduced opacity and a theme border. Selected cards use the existing accent/focus tokens. No meaning depends on color alone.

## Open questions and review

Maintainer review is required for modifier conventions, keyboard box-selection corner model, 1-unit graph-space keyboard step, event naming/timing, and public API. N2 does not provide undo, graph persistence, connection selection, or move rejection callback; owner `set_graph` is authoritative. Pointer capture outside the canvas and native screen-reader announcements require manual checks if GPUI's test platform does not expose them.


## Verification

The N2 drag and box-preview fixtures are included in the shared 13-state × 3-theme × 2-scale NodeEditor screenshot matrix (78 captures). The E8 example harness exercises real pointer movement before capture. Focused GPUI tests assert that a release without a final move event uses its coordinates and that a delivered outside release discards a node-move preview. Native pointer capture outside the canvas and screen-reader announcements remain manual checks. Public event, modifier, keyboard-box-selection, and API review remain maintainer gates.
