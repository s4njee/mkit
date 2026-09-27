# E8.10 node editor — proposed story split

The original proposal below has now been implemented through N4. Source details and verified scope are in [N3/N4 completion notes](E8_NODE_EDITOR_N3_N4.md). Public contracts, visual baselines, book accuracy, and the outstanding manual checks still need maintainer review.

## Shared boundary and contract

The node editor is a stateful `Entity<NodeEditor>` view. The app supplies stable `NodeId`, `PortId`, and `ConnectionId` values, node titles, positions, port directions/types, and connection endpoints. It owns graph evaluation, validation beyond declared port compatibility, persistence, undo, and any inspector or node catalog. The component lays out and navigates the graph, draws edges, hit-tests ports and nodes, and emits typed requests. It must not silently alter app graph data. Reuse E8.1 pan/zoom transform math and token conventions, with a semantic node/port/connection model over the canvas.

Proposed typed events are `ViewChanged { transform }`, `SelectionChanged { nodes, connections }`, `NodesMoveRequested { positions }`, `ConnectionCreateRequested { from_port, to_port }`, and `ConnectionRemoveRequested { connection_id }`. No event fires for a no-op. In uncontrolled mode, accepted view and selection changes update locally before emission; in controlled mode they request a change and wait for owner setters. For graph edits, define whether the component ever owns a graph copy. The safer initial contract is proposal-only even in uncontrolled view mode: the app confirms moved positions and connection changes through updated graph data. Drag previews can be transient but must cancel cleanly. Decide event timing and rejection feedback in the spec.

Register `MkitNodeEditor` key context with named actions; apps can rebind defaults. A single editor Tab stop enters a composite focus path; arrows navigate spatially among nodes, a command moves into/out of ports, and Tab reaches normal surrounding controls. Keys for a text field inside a node belong to that field while editing. No APG pattern covers arbitrary graphs. Use a labeled group with semantic nodes and navigable ports, and [APG Grid](https://www.w3.org/WAI/ARIA/apg/patterns/grid/) only if an actual row/column cell model is adopted. Follow the [component pattern map](component-pattern-map.md#planned-component-mapping). Announce node title, selection, port direction/type, connected endpoints, and connection count. Every essential pointer operation needs a keyboard route.

Paint uses GPUI `Global` theme tokens: `background` for the canvas, `surface` for nodes/minimap, text/muted text for labels, `accent` for selected items, `focus` for keyboard focus, border/spacing/radius/control-size tokens for frames and port targets. Connection type cannot be conveyed by color alone. Any grid spacing, port radius, edge width, minimum node size, or minimap size must be a theme token or justified in the slice spec. Capture light/dark and 1×/2× behavior.

## N1 — Navigable graph surface

**Scope.** Render app-supplied nodes, ports, and existing connections on a pan/zoom canvas. Show the connection direction and port labels. Support fit-all and actual-size commands plus cursor-centered zoom. Graph data is read-only here; focus/navigation and selection are first-class.

**Acceptance.** Node and edge locations are deterministic from supplied graph data and transform. Invalid endpoints have a defined fallback and do not crash. Panning/zooming preserve world-to-screen mapping and the pointer anchor; fit-all handles empty and very large graphs. Keyboard actions `Pan`, `ZoomIn`, `ZoomOut`, `FitGraph`, `ActualSize`, `NextNode`, `PreviousNode`, and `EnterPorts` navigate without modifying graph data. Selection is separate from focus and emits `SelectionChanged` in both state modes. The accessibility tree names nodes, ports, and existing connections, including their relationships; important graph information is not canvas-only.

**Evidence.** Transform and edge-routing tests; keyboard scripts for navigation, selection, fit/zoom and controlled rejection; accessibility snapshots for empty, disconnected, and connected graphs; screenshots of each in light/dark at 1×/2× and at two zoom levels. Inspect snapshot diffs before baseline updates. Physical trackpad and native screen-reader traversal need manual checks until harness support exists.

## N2 — Node movement and box selection

**Scope.** Add single/multiple node selection, node dragging, keyboard nudging, and rectangular box selection. Box selection affects nodes only in this slice. Define additive/toggle modifiers according to platform convention and a deterministic hit rule for partially enclosed nodes.

**Acceptance.** Pointer and keyboard produce the same selected node IDs and proposed positions. A drag preserves relative positions of a multiple-node selection; movement is cancellable with Escape and never commits on an invalid/non-finite position. Keyboard actions select/toggle a node, nudge selection in four directions, begin/end box selection, commit, and cancel. Box selection has a keyboard equivalent such as setting two corners with arrow actions or a range of nodes in navigation order; the exact model must be approved before code. `NodesMoveRequested` includes stable IDs and final positions. A controlled owner rejection restores the authoritative graph with no stale preview. Focus remains on a predictable node after a selection change.

**Evidence.** Geometry tests for containment, overlap, zoom, and negative coordinates; keyboard/pointer interaction scripts for additive selection, box selection, move, cancel and owner rejection; accessibility snapshots for multi-selection and movement; screenshots of box rectangle, selected set, drag preview, and keyboard focus in both themes/scales. Check pointer capture outside the canvas manually if the harness cannot.

## N3 — Connection editing

**Scope.** Create a connection by dragging between ports or by a keyboard source/destination flow. Select and remove an existing connection. The app remains the authority for compatibility and cycle rules; the component can reject clearly impossible direction or missing-port pairs and show an invalid-target state.

**Acceptance.** A source port can be chosen without a pointer; keyboard focus can reach compatible candidate ports and commit or cancel. Pointer and keyboard submit the same `ConnectionCreateRequested` endpoints. Escape cancels a preview without mutation. Repeated identical endpoints or self-connections follow explicit policy. Removing a selected edge emits `ConnectionRemoveRequested` once and is keyboard accessible. Connected-port names and endpoint relationships update after the owner confirms data. Invalid targets have a text/accessibility explanation, not only a color change.

**Evidence.** Model tests for endpoint identity/direction and duplicate policy; keyboard and pointer scripts for create, invalid target, cancel and delete; accessibility snapshots before/after connection and invalid preview; screenshots for normal, hovered, selected, invalid and focus states in light/dark at 1×/2×. Manually test native screen-reader announcements and pointer drag capture if harness gaps remain.

## N4 — Minimap and large graph navigation

**Scope.** Add an optional overview minimap showing graph bounds, nodes, and the current viewport rectangle. Clicking or dragging the minimap changes the view; keyboard users get equivalent pan-to-region or jump-to-node commands. Do not make the minimap the only route to distant content.

**Acceptance.** Minimap-to-world mapping remains correct under pan, zoom, negative coordinates and bounds changes. It has a caller-visible toggle and a named accessible control/description of viewport position. Pointer and keyboard actions move to the same graph region and emit `ViewChanged` under both state modes. Focus can leave the minimap; no focus trap or duplicate, confusing node list appears in the accessibility tree. A large graph can be navigated to any node without a pointer or excessive sequential Tab presses.

**Evidence.** Mapping tests and keyboard scripts for jump, pan, toggle and controlled rejection; accessibility snapshots with minimap visible/hidden; screenshots at small/large graph extents in light/dark at 1×/2×. Manually inspect precision on physical trackpads and spoken minimap/graph navigation.

## Review gates and open questions

Each slice needs a registry spec (states, events, keyboard map, role/properties, tokens, platform notes), compiling example with mdBook `{{#include}}` anchors, relevant build/tests/lint, book build, and coexistence check. Record commands and actual results in its PR. Harness evidence above is required where adapters exist; document missing adapters or inactive accessibility and manual checks without claiming a pass. Request human review of the spec, public names and event timing, keyboard/AX contract, visual baselines, and book accuracy before closing a slice.

- Is graph data immutable input with all edits confirmed by the app, even when view/selection are uncontrolled?
- How should nodes containing ordinary editable controls enter/leave graph navigation without stealing text/IME keys?
- What is the keyboard model for box selection and spatial node order, especially when nodes overlap?
- Should ports expose type compatibility from app data, and who decides cycles, duplicates and multi-edge connections?
- How many off-screen nodes/edges should the accessibility tree expose on large graphs, and how does focus survive virtualization?
- Does edge routing belong to the component, or should the app provide route points for special graph styles?
