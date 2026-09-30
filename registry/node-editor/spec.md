---
spec_version: 1
component: node-editor
states:
  - id: empty
    description: The graph contains no nodes or connections and remains pannable and zoomable.
    fixture: empty_fixture
  - id: disconnected
    description: Nodes and labeled ports are shown without connections.
    fixture: disconnected_fixture
  - id: connected
    description: Existing directed connections are drawn and named with their endpoint ports.
    fixture: connected_fixture
  - id: selected
    description: One focused node is selected independently of keyboard focus.
    fixture: selected_fixture
  - id: port_navigation
    description: Keyboard navigation is within the focused node's ports.
    fixture: port_navigation_fixture
  - id: zoomed
    description: The graph is shown with a non-default pan and zoom transform.
    fixture: zoomed_fixture
  - id: drag_preview
    description: A live pointer drag of selected nodes shows displaced cards and an active-preview summary before release.
    fixture: selected_fixture
  - id: box_preview
    description: A live empty-canvas pointer drag shows the box-selection rectangle and preview summary before release.
    fixture: disconnected_fixture
  - id: connection_preview
    description: Keyboard focus on an output port starts a connection candidate preview before commit or cancellation.
    fixture: disconnected_fixture
  - id: connection_selected
    description: An existing connection has keyboard focus or pointer selection independent from node selection.
    fixture: connected_fixture
  - id: invalid_connection_target
    description: Releasing a real output-port drag on a mismatched input shows the rejection reason without changing graph data.
    fixture: disconnected_fixture
  - id: minimap_visible
    description: The optional overview shows graph bounds and current viewport position.
    fixture: connected_fixture
  - id: minimap_hidden
    description: The minimap is hidden while graph navigation remains available.
    fixture: connected_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: editor is focused and port navigation is inactive
    action: Focus the next node in deterministic reading order.
    initial_state: disconnected
    expect:
      event: FocusChanged
  - key: ArrowUp
    modifiers: []
    when: editor is focused and port navigation is inactive
    action: Focus the previous node in deterministic reading order.
    initial_state: disconnected
    expect:
      event: FocusChanged
  - key: Space
    modifiers: []
    when: a node is focused
    action: Select the focused node.
    initial_state: disconnected
    expect:
      event: SelectionChanged
  - key: Enter
    modifiers: []
    when: a node is focused
    action: Enter that node's port navigation path.
    initial_state: disconnected
    expect:
      state: port_navigation
  - key: ArrowDown
    modifiers: []
    when: port navigation is active
    action: Focus the next port in the current node.
    initial_state: port_navigation
    expect:
      event: FocusChanged
  - key: Escape
    modifiers: []
    when: port navigation is active
    action: Return focus to the node.
    initial_state: port_navigation
    expect:
      state: disconnected
  - key: ArrowLeft
    modifiers: [Alt]
    when: editor is focused
    action: Pan the graph viewport to the left.
    initial_state: zoomed
    expect:
      event: ViewChanged
  - key: "="
    modifiers: []
    when: editor is focused
    action: Zoom in at the viewport center.
    initial_state: connected
    expect:
      state: zoomed
  - key: "-"
    modifiers: []
    when: editor is focused
    action: Zoom out at the viewport center.
    initial_state: connected
    expect:
      state: zoomed
  - key: f
    modifiers: []
    when: editor is focused
    action: Fit all finite graph bounds into the viewport.
    initial_state: connected
    expect:
      event: ViewChanged
  - key: "1"
    modifiers: []
    when: editor is focused
    action: Set actual-size scale and center the graph bounds.
    initial_state: zoomed
    expect:
      event: ViewChanged
  - key: Space
    modifiers: [Shift]
    when: a node is focused
    action: Add the focused node to selection.
    initial_state: selected
    expect:
      event: SelectionChanged
  - key: Space
    modifiers: [Platform]
    when: a node is focused
    action: Toggle the focused node in selection.
    initial_state: selected
    expect:
      event: SelectionChanged
  - key: ArrowRight
    modifiers: [Shift, Alt]
    when: one or more nodes are selected
    action: Propose moving selected nodes one graph unit to the right.
    initial_state: selected
    expect:
      event: NodesMoveRequested
  - key: b
    modifiers: [Shift]
    when: a node is focused
    action: Begin keyboard box selection around the focused node.
    initial_state: selected
    expect:
      state: box_preview
  - key: ArrowRight
    modifiers: [Shift]
    when: keyboard box selection is active
    action: Move the active selection corner one graph unit to the right.
    initial_state: box_preview
    expect:
      state: box_preview
  - key: Enter
    modifiers: [Shift]
    when: keyboard box selection is active
    action: Commit the full-containment selection.
    initial_state: box_preview
    expect:
      event: SelectionChanged
  - key: Escape
    modifiers: []
    when: a movement or box preview is active
    action: Cancel the preview without emitting a graph edit.
    initial_state: drag_preview
    expect:
      event: none
  - key: c
    modifiers: []
    when: a port is focused
    action: Choose the focused output as connection source.
    initial_state: port_navigation
    expect:
      state: connection_preview
  - key: "]"
    modifiers: []
    when: a connection source is chosen
    action: Focus the next compatible input candidate in reading order.
    initial_state: connection_preview
    expect:
      event: FocusChanged
  - key: "["
    modifiers: []
    when: a source is chosen
    action: Focus the previous compatible input candidate.
    initial_state: connection_preview
    expect:
      event: FocusChanged
  - key: Ctrl+Enter
    modifiers: [Control]
    when: a valid input candidate is focused
    action: Request the connection and clear the preview.
    initial_state: connection_preview
    expect:
      event: ConnectionCreateRequested
  - key: Delete
    modifiers: []
    when: a connection is focused
    action: Request removal of the focused connection.
    initial_state: connection_selected
    expect:
      event: ConnectionRemoveRequested
  - key: m
    modifiers: []
    when: editor is focused
    action: Toggle the minimap visibility.
    initial_state: minimap_hidden
    expect:
      state: minimap_visible
  - key: g
    modifiers: []
    when: editor is focused
    action: Focus the next existing connection.
    initial_state: connection_selected
    expect:
      event: ConnectionFocusChanged
  - key: Delete
    modifiers: []
    when: a connection is focused
    action: Request removal of that connection.
    initial_state: connection_selected
    expect:
      event: ConnectionRemoveRequested
  - key: v
    modifiers: []
    when: a node is focused
    action: Center the viewport on that node.
    initial_state: zoomed
    expect:
      event: ViewChanged
  - key: Escape
    modifiers: []
    when: connection creation preview is active
    action: Cancel the source preview without a graph edit.
    initial_state: connection_preview
    expect:
      event: none
  - key: m
    modifiers: []
    when: editor is focused
    action: Toggle minimap visibility.
    initial_state: minimap_visible
    expect:
      state: minimap_hidden
  - key: "]"
    modifiers: []
    when: a source is chosen
    action: Focus the next compatible input candidate.
    initial_state: connection_preview
    expect:
      event: FocusChanged
accessibility:
  role: group
  properties:
    - name: name
      value: caller-provided graph label
    - name: node
      value: Each node is named by its title and has a stable accessible identifier.
    - name: port
      value: Each port exposes its label, input/output direction, and declared type.
    - name: connection
      value: Each existing connection exposes source and target node and port names.
    - name: selected
      value: Selected nodes expose selected state independently from focus.
    - name: focused
      value: The keyboard-focused node or port has visible focus treatment and is identified to assistive technology.
    - name: invalid_target
      value: Rejected connection targets expose a textual reason through the editor description.
controlled: >-
  The app supplies graph data, view transform and selected node IDs. Controlled view and selection wait
  for owner setters; uncontrolled view and selection update locally before emitting. Node movement is
  always proposal-only. Pointer preview clears on pointer-up after emitting NodesMoveRequested. Positions
  change only through set_graph in both modes, so a rejected request cannot leave a stale preview.
  Minimap visibility is uncontrolled by default and updates locally before MinimapVisibilityChanged;
  with_controlled_minimap uses request-only toggles until set_minimap_visible applies the owner's value.
events: [ViewChanged, SelectionChanged, FocusChanged, NodesMoveRequested, ConnectionCreateRequested, ConnectionRemoveRequested, ConnectionFocusChanged, MinimapVisibilityChanged]
theme_tokens: [background, surface, text, text_muted, border, accent, accent_text, focus, spacing.xsmall, spacing.small, spacing.large, radii.small, radii.medium, radii.large, radii.pill, borders.hairline, borders.regular, borders.strong, controls.xsmall, controls.small, controls.large, shadows.small, typography.body, typography.caption]
open_questions: [Human review of modifier conventions, keyboard box-selection corner model, one-world-unit keyboard step, public event names and timing, node content composition, and accessibility scaling for large graphs.]
---

# Node editor — N1 navigable graph surface

## Purpose

Render and navigate an app-supplied graph with N2 movement proposals and box selection. Graph edits remain app-owned.

## Anatomy

- A graph viewport with a token-driven background and compact fit, actual-size, and zoom controls.
- Node cards positioned in graph coordinates with a title and labeled input/output ports.
- Directed cubic connection paths between resolved ports, with an arrow marker and endpoint description.
- A separate accessible textual summary for every existing connection.

## States

`empty`, `disconnected`, and `connected` cover graph data shape. `selected` is app selection, `port_navigation` is the composite focus path through a node's ports, and `zoomed` is a non-default view transform. `drag_preview` and `box_preview` are captured between real pointer-down and pointer-up events. `connection_preview` and `connection_selected` are reached through their keyboard actions. `invalid_connection_target` is captured after releasing an output drag on an incompatible input. `minimap_visible` and `minimap_hidden` capture the optional overview in both states. An invalid/missing endpoint is skipped for painting and exposed in the connection summary as unresolved; it never panics.

## Props and events

Each node has a stable ID, title, finite world position, and ordered input/output ports. Each port has a stable ID, label, and declared type. Connections reference source and target port IDs. The host supplies all graph data and remains authoritative; IDs should be unique. `NodeEditor::new` is uncontrolled for view and selection. `NodeEditor::controlled` receives the current transform and selected node IDs; `set_graph`, `set_transform`, and `set_selection` apply owner updates. `ViewChanged` and `SelectionChanged` are emitted in both modes; `FocusChanged` reports local keyboard focus. Empty, identical, invalid, and otherwise no-op requests emit no event. This component never modifies graph data.

## Keyboard map

Named actions use the `MkitNodeEditor` key context and are registered by `default_key_bindings`. Arrow Up/Down traverse a stable reading order (top, then left, then ID); Space selects the focused node, Shift+Space adds it, and platform+Space toggles it. Enter enters that node's ports; Up/Down move among its inputs then outputs; Escape returns to its node. Alt+Arrow keys pan; Shift+Alt+Arrow nudges selected nodes by one graph unit; Shift+B begins keyboard box selection and Shift+Arrow moves its active corner by one graph unit; Shift+Enter commits. `=`/`-` zoom at center, F fits all finite graph bounds, and 1 sets scale to 100% while centering the graph. Hosts can rebind every named action.

## Pointer behaviour

Mouse wheel zooms at the pointer location and middle-button drag pans. Plain node click replaces selection; Shift adds and platform-command toggles. Empty-canvas drag previews box selection; dragging a selected node moves the selected set and emits a proposal on pointer-up. Toolbar buttons provide Fit, 100%, Zoom out, and Zoom in.

## Accessibility role and properties

The root is a labeled group. Node cards expose group role, stable name, and selected state. Port labels remain visible and expose direction and type in their accessible names. Existing connection summaries identify both endpoint nodes and ports. The focused node or port uses visible focus treatment. The connection canvas is supplemented with text because path graphics do not convey graph relationships to assistive technology.

## Theme tokens used

Canvas and node surface colors, text, borders, selection, and focus come from `mkit_core::theme::Theme`. Spacing, radii, border widths, and typography use GPUI theme tokens. The 220 logical-pixel node width, 32-pixel title band, 24-pixel port row, 6-pixel port radius, and 2-pixel connection stroke are graph geometry constants justified for N1's compact layout and scale with the view transform.

### Visual design (E13.2 P4)

The look follows the restyled everyday components (Button, Toolbar, Slider, Tree) and is resolved from the installed `Theme` in three variants. `high-contrast` is selected by theme name; every other theme is dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or tokens are added. "Muted" is `text` mixed 4% (light) or 12% (dark) into `background`. "Divider" is `border` in light and `text` at 10% alpha in dark.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Frame | `background` canvas, divider border, radius `radii.large` | same | `background`, `border` |
| Frame keyboard focus | `focus` border plus a 3px ring of `focus` at 50% | same | `focus` border plus a 3px ring of opaque `focus` |
| Toolbar | `background`, divider underneath; zoom readout `text_muted` caption | same | `background`, `border` |
| Toolbar buttons | the Button `outline` variant: `background`, `text`, outline border, `shadows.small`; hover muted | same with `text` at 10% over `background` as the outline | `background`, `text`, `border`; hover border `accent` |
| Toolbar button focus | `focus` border, opaque `background`, 3px ring of `focus` at 50% | same | ring in opaque `focus` |
| Node card | a bordered rounded card: opaque `surface`, divider border, radius `radii.large`, `shadows.small`; title `typography.body` at medium weight | same | `surface`, `border` |
| Node hover | border `text` mixed 30% into `background` | same | border `accent` |
| Keyboard-focused node | `focus` border at `borders.strong` | same | same |
| Selected node | the 3px focus ring of `focus` at 50% in place of the shadow, and a muted title band | same | opaque `focus` ring and an `accent` title band with `accent_text` |
| Port | the Slider thumb look at the port size: opaque `background` fill, hairline `accent` border, `shadows.small` | same | same (no shadow) |
| Keyboard-focused port | `accent` fill, `focus` border, round 3px ring of `focus` at 50% | same | ring in opaque `focus` |
| Connection wires and arrowheads | `text_muted` (the wire colour role; not re-tokenised); the connection preview stays `accent` | same | same |
| Box selection | `accent` at 10% alpha with a hairline `accent` border, radius `radii.small` | same | same |
| Minimap | a bordered rounded card: opaque `surface`, divider border, radius `radii.large`, `shadows.small`; nodes `text_muted`; viewport rectangle `borders.strong` `accent` | same | `surface`, `border` |
| Minimap focus | `focus` border plus the 3px ring | same | opaque ring |
| Connection list | `surface` with a divider above | same | `surface`, `border` |
| Connection rows | the Tree row: radius `radii.small`, `text_muted` caption, reserved hairline border; hover muted | same | hover border `border` |
| Focused connection row | muted fill, `text`, `focus` hairline outline inside the row | same | `accent` fill, `accent_text`, `focus` outline |

- **Density (maintainer decision, provisional).** Pro components keep their existing dense geometry: node width, title band, port rows, port size, port hit tolerance, wire width, minimap size, list height, and the `controls.xsmall` toolbar buttons (`controls.small` minimum width) are unchanged. Colours, borders, radii, shadows, focus, hover and typography follow the everyday components.
- **Selection, focus and hover stay distinct.** Selection draws the focus ring outside the card and fills the title band (a non-colour cue); keyboard focus draws a strong `focus` border; hover only darkens the resting border. A focused selected node shows both. Rings and shadows are drawn in screen pixels, so they stay legible at any zoom, while borders and radii scale with the graph like the rest of the card.
- The card radius scales with the view transform. The selected title band rounds its top corners to match, because GPUI clips children to a rectangle. A box-shadow ring around a 6px port takes the port's clamped corner radius and renders square, so the focused port's ring is a round layer painted behind the port instead; it does not change layout.
- GPUI paints drop shadows as filled shapes, so cards, ports, buttons and the minimap keep opaque fills under shadows and rings. The 3px focus ring is the shadcn/ui ring width.
- The node editor has no disabled state. The matrix's keyboard-driven states (`port_navigation`, `connection_preview`, `connection_selected`) and the pointer-driven states (`drag_preview`, `box_preview`, `invalid_connection_target`) focus the editor, so the frame focus ring is covered; no separate focused state is added.

## WAI-ARIA pattern reference

Arbitrary node graphs have no direct WAI-ARIA pattern. The component uses a labeled group containing named node groups and port text; keyboard focus is a composite path independent from the selected set. It does not claim grid semantics because nodes do not occupy a regular row/column model.

## Platform notes

GPUI AccessKit captures are required to verify role/name/selection and relationship summaries on active platforms. The graph transform is local to this crate because registry dependencies stay limited to GPUI and `mkit-core`; its coordinate behavior follows E8.1 viewport conventions. Trackpad gestures and native screen-reader traversal need manual checks where harness support is unavailable.

## Screenshot and interaction evidence

The conformance manifest declares every visual state in light, dark, and high-contrast themes at
1× and 2×. The E8 example harness must capture these 78 combinations. Active previews are captured
before pointer release, and the invalid target state after a real rejected port drag. Keyboard
states are reached by dispatching the documented key actions through the headless window. Screenshots
are evidence for visual state only; keyboard scripts, accessibility snapshots, and maintainer review
remain separate checks.

## Open questions

- Human review is required for public names, event semantics, keyboard and accessibility contracts, and visual baselines.
- Confirm node focus ordering, future node content composition, and accessibility scaling for very large graphs.

## N2 movement and box selection

Plain click replaces selection and focuses the hit node. Shift-click adds; platform-command-click toggles. Dragging selected nodes preserves their relative positions; dragging an unselected node first selects it. Empty-canvas drag previews a box. Box selection uses full node-card containment; partial intersections are excluded. Shift adds box results and platform-command toggles them. Hit testing follows reverse paint order. Escape cancels either preview. A release inside the canvas uses the release coordinates even if no final move event arrived; a release outside the canvas cancels node and box previews without a request. On pointer-up a changed valid drag emits one `NodesMoveRequested { positions: Vec<NodePosition> }`, containing stable IDs and finite final world positions; it does not mutate graph data. Owners confirm through `set_graph`. Invalid positions reject the whole request.

Space replaces selection, Shift+Space adds, platform+Space toggles. Shift+Alt+arrow nudges selected nodes by one graph unit. `BeginBoxSelection` starts with a rectangle around the focused node, and Shift+arrow actions move its top-left corner by one graph unit, `CommitBoxSelection` applies the result, and Escape cancels. These named actions are rebindable in `MkitNodeEditor`. The root group describes active previews and node selection count; nodes expose their selected state. Selected cards draw the focus ring and a filled title band, so selection is not conveyed by colour alone. The box preview uses a low-alpha accent fill with an accent border. Native screen-reader traversal and pointer capture outside the canvas need manual verification where the harness cannot exercise them. Human review is required for the public event/API and keyboard contract.

## N3 connection editing

Connection creation is proposal-only. Drag from an output port to an input port, or focus an output port and press `c`, cycle compatible input candidates with `[` and `]`, and commit with `Ctrl+Enter`. `Escape` cancels the active source preview. Direction, declared type, missing ports, same-node endpoints, and duplicate endpoint pairs are rejected locally; cycle rules and other graph validation remain app-owned. Invalid pointer targets clear the preview without an event. Create requests carry stable port IDs and never mutate `GraphData`; owners confirm through `set_graph`.

Connections appear as named buttons in the Graph connections list. `g` and `Shift+g` move connection focus through the list and emit `ConnectionFocusChanged`; clicking a summary focuses it as well. `Delete` emits one `ConnectionRemoveRequested` for the focused connection. Connection focus is independent of node focus and selection. Connected endpoint names are read from current owner-supplied graph data, so accepted changes update the accessible summaries.

## N4 minimap and large graph navigation

The minimap is hidden by default and can be shown with the toolbar button or `m`; hosts may use `set_minimap_visible`. Visibility is uncontrolled by default and updates before `MinimapVisibilityChanged`; `with_controlled_minimap` makes the control request-only until the host calls `set_minimap_visible`. It depicts finite graph bounds, nodes, and the viewport rectangle. Clicking it centers the view at the corresponding graph position using the same uniform scale and centered bounds used to draw nodes. The minimap button has a name and a text description of the visible world-coordinate range. `Alt+Arrow` pans; arrows navigate nodes in reading order and `v` centers the focused node, so every node remains reachable without the minimap. Focus can leave the minimap through normal Tab navigation. View requests follow the editor's existing controlled/uncontrolled transform contract and emit `ViewChanged` only when the transform changes.

The minimap is 168 by 104 logical pixels, justified as a compact overview target with enough inner space to show viewport bounds. Its inset, card (surface, divider border, `radii.large`, `shadows.small`), viewport border, and focus ring use GPUI theme spacing, surface, border, radius, shadow, accent, and focus tokens. Mapping accounts for negative graph coordinates and preserves a uniform aspect ratio. Large, extreme-range graph precision and native spoken feedback still need manual checks. Human review is required for the public event/API, key map, accessibility contract, and visual baselines.
