//! A read-only, navigable graph surface with a semantic node and port model.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Bounds, Context, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement,
    KeyBinding, KeyDownEvent, KeyUpEvent, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, PinchEvent, Pixels, Point, Render, Rgba, ScrollDelta, ScrollWheelEvent, Window,
    actions, canvas, div, point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
use std::{cell::Cell, rc::Rc};

pub const KEY_CONTEXT: &str = "MkitNodeEditor";
actions!(
    node_editor,
    [
        NextNode,
        PreviousNode,
        SelectFocused,
        AddFocused,
        ToggleFocused,
        EnterPorts,
        ExitPorts,
        PanLeft,
        PanRight,
        PanUp,
        PanDown,
        ZoomIn,
        ZoomOut,
        FitGraph,
        ActualSize,
        NudgeLeft,
        NudgeRight,
        NudgeUp,
        NudgeDown,
        BeginBoxSelection,
        CommitBoxSelection,
        BoxCornerLeft,
        BoxCornerRight,
        BoxCornerUp,
        BoxCornerDown,
        StartConnection,
        CommitConnection,
        NextConnectionCandidate,
        PreviousConnectionCandidate,
        RemoveFocusedConnection,
        ToggleMinimap,
        NextConnection,
        PreviousConnection,
        CenterFocusedNode
    ]
);

pub fn default_key_bindings() -> [KeyBinding; 35] {
    [
        KeyBinding::new("down", NextNode, Some(KEY_CONTEXT)),
        KeyBinding::new("up", PreviousNode, Some(KEY_CONTEXT)),
        KeyBinding::new("space", SelectFocused, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-space", AddFocused, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-space", ToggleFocused, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-space", ToggleFocused, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", EnterPorts, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", ExitPorts, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-left", PanLeft, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-right", PanRight, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-up", PanUp, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-down", PanDown, Some(KEY_CONTEXT)),
        KeyBinding::new("=", ZoomIn, Some(KEY_CONTEXT)),
        KeyBinding::new("-", ZoomOut, Some(KEY_CONTEXT)),
        KeyBinding::new("f", FitGraph, Some(KEY_CONTEXT)),
        KeyBinding::new("1", ActualSize, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-alt-left", NudgeLeft, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-alt-right", NudgeRight, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-alt-up", NudgeUp, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-alt-down", NudgeDown, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-b", BeginBoxSelection, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-enter", CommitBoxSelection, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-left", BoxCornerLeft, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-right", BoxCornerRight, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-up", BoxCornerUp, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-down", BoxCornerDown, Some(KEY_CONTEXT)),
        KeyBinding::new("c", StartConnection, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-enter", CommitConnection, Some(KEY_CONTEXT)),
        KeyBinding::new("]", NextConnectionCandidate, Some(KEY_CONTEXT)),
        KeyBinding::new("[", PreviousConnectionCandidate, Some(KEY_CONTEXT)),
        KeyBinding::new("delete", RemoveFocusedConnection, Some(KEY_CONTEXT)),
        KeyBinding::new("m", ToggleMinimap, Some(KEY_CONTEXT)),
        KeyBinding::new("g", NextConnection, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-g", PreviousConnection, Some(KEY_CONTEXT)),
        KeyBinding::new("v", CenterFocusedNode, Some(KEY_CONTEXT)),
    ]
}

const NODE_WIDTH: f32 = 220.0;
const HEADER_HEIGHT: f32 = 32.0;
const PORT_ROW_HEIGHT: f32 = 24.0;
const PORT_RADIUS: f32 = 6.0;
const EDGE_WIDTH: f32 = 2.0;
const MIN_SCALE: f32 = 0.00001;
const MAX_SCALE: f32 = 16.0;

/// Colours derived from theme tokens; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    high_contrast: bool,
    /// shadcn "muted": control and row hover fill.
    muted: Rgba,
    /// Frame, card, toolbar, and list dividers.
    divider: Rgba,
    /// Opaque outline-button border.
    control_border: Rgba,
    /// Node card border under the pointer.
    hover_border: Rgba,
    /// Selected node title band and focused connection row.
    selected_bg: Rgba,
    selected_text: Rgba,
    ring: Rgba,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            high_contrast: true,
            muted: c.background,
            divider: c.border,
            control_border: c.border,
            hover_border: c.accent,
            selected_bg: c.accent,
            selected_text: c.accent_text,
            ring: c.focus,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    Look {
        high_contrast: false,
        muted,
        divider: if dark { c.text.opacity(0.1) } else { c.border },
        control_border: if dark { mix(c.text, c.background, 0.1) } else { c.border },
        hover_border: mix(c.text, c.background, 0.3),
        selected_bg: muted,
        selected_text: c.text,
        ring: c.focus.opacity(0.5),
    }
}
fn box_shadow(shadow: ShadowToken) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: shadow.color.into(),
        offset: point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}
/// shadcn/ui focus ring width, drawn outside the focused or selected element in screen pixels
/// so it stays legible at every zoom.
const FOCUS_RING_WIDTH: f32 = 3.0;
fn focus_ring(color: Rgba) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: color.into(),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(FOCUS_RING_WIDTH),
        inset: false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphTransform {
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for GraphTransform {
    fn default() -> Self {
        Self { scale: 1.0, offset_x: 0.0, offset_y: 0.0 }
    }
}

impl GraphTransform {
    pub fn valid(self) -> bool {
        self.scale.is_finite()
            && (MIN_SCALE..=MAX_SCALE).contains(&self.scale)
            && self.offset_x.is_finite()
            && self.offset_y.is_finite()
    }
    pub fn world_to_screen(self, world: Point<f32>) -> Point<f32> {
        point(self.offset_x + world.x * self.scale, self.offset_y + world.y * self.scale)
    }
    pub fn screen_to_world(self, screen: Point<f32>) -> Point<f32> {
        point((screen.x - self.offset_x) / self.scale, (screen.y - self.offset_y) / self.scale)
    }
    pub fn panned(self, dx: f32, dy: f32) -> Self {
        Self { offset_x: self.offset_x + dx, offset_y: self.offset_y + dy, ..self }
    }
    pub fn zoomed_at(self, screen: Point<f32>, factor: f32) -> Self {
        if !factor.is_finite() || factor <= 0.0 {
            return self;
        }
        let world = self.screen_to_world(screen);
        let scale = (self.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        Self { scale, offset_x: screen.x - world.x * scale, offset_y: screen.y - world.y * scale }
    }
    pub fn fitted(bounds: GraphBounds, viewport: Point<f32>, padding: f32) -> Self {
        if !bounds.is_finite() || viewport.x <= 0.0 || viewport.y <= 0.0 {
            return Self::default();
        }
        let width = bounds.max_x - bounds.min_x;
        let height = bounds.max_y - bounds.min_y;
        if width <= 0.0 || height <= 0.0 {
            return Self::default();
        }
        let scale = (((viewport.x - 2.0 * padding).max(1.0) / width)
            .min((viewport.y - 2.0 * padding).max(1.0) / height))
        .clamp(MIN_SCALE, MAX_SCALE);
        Self {
            scale,
            offset_x: (viewport.x - width * scale) / 2.0 - bounds.min_x * scale,
            offset_y: (viewport.y - height * scale) / 2.0 - bounds.min_y * scale,
        }
    }
    pub fn actual_size(bounds: Option<GraphBounds>, viewport: Point<f32>) -> Self {
        match bounds.filter(|bounds| bounds.is_finite()) {
            Some(bounds) => Self {
                scale: 1.0,
                offset_x: (viewport.x - bounds.width()) / 2.0 - bounds.min_x,
                offset_y: (viewport.y - bounds.height()) / 2.0 - bounds.min_y,
            },
            None => Self::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}
impl GraphBounds {
    pub fn width(self) -> f32 {
        self.max_x - self.min_x
    }
    pub fn height(self) -> f32 {
        self.max_y - self.min_y
    }
    pub fn is_finite(self) -> bool {
        [self.min_x, self.min_y, self.max_x, self.max_y].into_iter().all(f32::is_finite)
            && self.max_x > self.min_x
            && self.max_y > self.min_y
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPort {
    pub id: String,
    pub label: String,
    pub data_type: String,
    pub direction: PortDirection,
}
impl GraphPort {
    pub fn input(
        id: impl Into<String>,
        label: impl Into<String>,
        data_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            data_type: data_type.into(),
            direction: PortDirection::Input,
        }
    }
    pub fn output(
        id: impl Into<String>,
        label: impl Into<String>,
        data_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            data_type: data_type.into(),
            direction: PortDirection::Output,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub position: Point<f32>,
    pub inputs: Vec<GraphPort>,
    pub outputs: Vec<GraphPort>,
}
impl GraphNode {
    pub fn new(id: impl Into<String>, title: impl Into<String>, position: Point<f32>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            position,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }
    pub fn inputs(mut self, ports: Vec<GraphPort>) -> Self {
        self.inputs = ports;
        self
    }
    pub fn outputs(mut self, ports: Vec<GraphPort>) -> Self {
        self.outputs = ports;
        self
    }
    pub fn height(&self) -> f32 {
        HEADER_HEIGHT + self.inputs.len().max(self.outputs.len()).max(1) as f32 * PORT_ROW_HEIGHT
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphConnection {
    pub id: String,
    pub from_port_id: String,
    pub to_port_id: String,
    pub label: Option<String>,
}
impl GraphConnection {
    pub fn new(
        id: impl Into<String>,
        from_port_id: impl Into<String>,
        to_port_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            from_port_id: from_port_id.into(),
            to_port_id: to_port_id.into(),
            label: None,
        }
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub connections: Vec<GraphConnection>,
}
impl GraphData {
    pub fn new(nodes: Vec<GraphNode>, connections: Vec<GraphConnection>) -> Self {
        Self { nodes, connections }
    }
    pub fn bounds(&self) -> Option<GraphBounds> {
        let mut bounds: Option<GraphBounds> = None;
        for node in self
            .nodes
            .iter()
            .filter(|node| node.position.x.is_finite() && node.position.y.is_finite())
        {
            let next = GraphBounds {
                min_x: node.position.x,
                min_y: node.position.y,
                max_x: node.position.x + NODE_WIDTH,
                max_y: node.position.y + node.height(),
            };
            bounds = Some(match bounds {
                Some(current) => GraphBounds {
                    min_x: current.min_x.min(next.min_x),
                    min_y: current.min_y.min(next.min_y),
                    max_x: current.max_x.max(next.max_x),
                    max_y: current.max_y.max(next.max_y),
                },
                None => next,
            });
        }
        bounds.filter(|bounds| bounds.is_finite())
    }
    fn node_for_port(&self, id: &str) -> Option<(&GraphNode, &GraphPort)> {
        self.nodes.iter().find_map(|node| {
            node.inputs
                .iter()
                .chain(&node.outputs)
                .find(|port| port.id == id)
                .map(|port| (node, port))
        })
    }
    fn port_position(node: &GraphNode, port: &GraphPort) -> Point<f32> {
        let ports = match port.direction {
            PortDirection::Input => &node.inputs,
            PortDirection::Output => &node.outputs,
        };
        let index = ports.iter().position(|candidate| candidate.id == port.id).unwrap_or(0);
        point(
            node.position.x + if port.direction == PortDirection::Input { 0.0 } else { NODE_WIDTH },
            node.position.y
                + HEADER_HEIGHT
                + index as f32 * PORT_ROW_HEIGHT
                + PORT_ROW_HEIGHT / 2.0,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewChanged {
    pub transform: GraphTransform,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionChanged {
    pub node_ids: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusChanged {
    pub node_id: Option<String>,
    pub port_id: Option<String>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct NodePosition {
    pub node_id: String,
    pub position: Point<f32>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct NodesMoveRequested {
    pub positions: Vec<NodePosition>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionCreateRequested {
    pub from_port_id: String,
    pub to_port_id: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionRemoveRequested {
    pub connection_id: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MinimapVisibilityChanged {
    pub visible: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionFocusChanged {
    pub connection_id: Option<String>,
}
#[derive(Clone, Debug)]
struct NodeDrag {
    start: Point<f32>,
    originals: Vec<NodePosition>,
    delta: Point<f32>,
}
#[derive(Clone, Debug)]
struct BoxDrag {
    start: Point<f32>,
    current: Point<f32>,
    original_selection: Vec<String>,
    additive: bool,
    toggle: bool,
}
impl EventEmitter<ViewChanged> for NodeEditor {}
impl EventEmitter<SelectionChanged> for NodeEditor {}
impl EventEmitter<FocusChanged> for NodeEditor {}
impl EventEmitter<NodesMoveRequested> for NodeEditor {}
impl EventEmitter<ConnectionCreateRequested> for NodeEditor {}
impl EventEmitter<ConnectionRemoveRequested> for NodeEditor {}
impl EventEmitter<MinimapVisibilityChanged> for NodeEditor {}
impl EventEmitter<ConnectionFocusChanged> for NodeEditor {}

pub struct NodeEditor {
    label: String,
    graph: GraphData,
    transform: GraphTransform,
    selected: Vec<String>,
    controlled_view: bool,
    controlled_selection: bool,
    focused_node: Option<String>,
    focused_port: Option<String>,
    ports_active: bool,
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    pointer_down: bool,
    last_pointer: Option<Point<f32>>,
    space_held: bool,
    focus: Option<FocusHandle>,
    node_drag: Option<NodeDrag>,
    box_drag: Option<BoxDrag>,
    connection_source: Option<String>,
    selected_connection: Option<String>,
    minimap_visible: bool,
    controlled_minimap: bool,
    connection_status: Option<String>,
}

impl NodeEditor {
    pub fn new(label: impl Into<String>, graph: GraphData) -> Self {
        let focused_node = reading_order(&graph.nodes).first().map(|node| node.id.clone());
        Self {
            label: label.into(),
            graph,
            transform: GraphTransform::default(),
            selected: Vec::new(),
            controlled_view: false,
            controlled_selection: false,
            focused_node,
            focused_port: None,
            ports_active: false,
            bounds: Rc::new(Cell::new(None)),
            pointer_down: false,
            last_pointer: None,
            space_held: false,
            focus: None,
            node_drag: None,
            box_drag: None,
            connection_source: None,
            selected_connection: None,
            minimap_visible: false,
            controlled_minimap: false,
            connection_status: None,
        }
    }
    pub fn controlled(
        label: impl Into<String>,
        graph: GraphData,
        transform: GraphTransform,
        selected: Vec<String>,
    ) -> Self {
        let mut editor = Self::new(label, graph);
        editor.controlled_view = true;
        editor.controlled_selection = true;
        if transform.valid() {
            editor.transform = transform;
        }
        editor.selected = editor.valid_selection(selected);
        editor
    }
    pub fn graph(&self) -> &GraphData {
        &self.graph
    }
    pub fn transform(&self) -> GraphTransform {
        self.transform
    }
    pub fn selection(&self) -> &[String] {
        &self.selected
    }
    pub fn focused_node(&self) -> Option<&str> {
        self.focused_node.as_deref()
    }
    pub fn focused_port(&self) -> Option<&str> {
        self.focused_port.as_deref()
    }
    pub fn ports_active(&self) -> bool {
        self.ports_active
    }
    pub fn minimap_visible(&self) -> bool {
        self.minimap_visible
    }
    pub fn connection_status(&self) -> Option<&str> {
        self.connection_status.as_deref()
    }
    pub fn with_controlled_minimap(mut self, visible: bool) -> Self {
        self.controlled_minimap = true;
        self.minimap_visible = visible;
        self
    }
    fn valid_connection_pair(&self, from: &str, to: &str) -> bool {
        self.connection_error(from, to).is_none()
    }
    fn connection_error(&self, from: &str, to: &str) -> Option<String> {
        let (Some((from_node, from_port)), Some((to_node, to_port))) =
            (self.graph.node_for_port(from), self.graph.node_for_port(to))
        else {
            return Some("Connection target is missing from the current graph.".into());
        };
        if from_port.direction != PortDirection::Output || to_port.direction != PortDirection::Input
        {
            return Some("Connections must run from an output port to an input port.".into());
        }
        if from_port.data_type != to_port.data_type {
            return Some(format!(
                "Cannot connect {} to {}: declared types do not match.",
                from_port.data_type, to_port.data_type
            ));
        }
        if from_node.id == to_node.id {
            return Some("Connections between ports on the same node are not supported.".into());
        }
        if self.graph.connections.iter().any(|c| c.from_port_id == from && c.to_port_id == to) {
            return Some("That endpoint pair is already connected.".into());
        }
        None
    }
    fn request_connection(&mut self, from: String, to: String, cx: &mut Context<Self>) {
        if let Some(reason) = self.connection_error(&from, &to) {
            self.connection_status = Some(reason);
        } else {
            self.connection_status = None;
            cx.emit(ConnectionCreateRequested { from_port_id: from, to_port_id: to });
        }
        self.connection_source = None;
        cx.notify();
    }
    fn toggle_minimap(&mut self, cx: &mut Context<Self>) {
        self.request_minimap_visible(!self.minimap_visible, cx);
    }
    pub fn set_minimap_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        if self.minimap_visible == visible {
            return;
        }
        self.minimap_visible = visible;
        cx.notify();
    }
    fn request_minimap_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        if self.minimap_visible == visible {
            return;
        }
        if !self.controlled_minimap {
            self.minimap_visible = visible;
        }
        cx.emit(MinimapVisibilityChanged { visible });
        cx.notify();
    }
    fn minimap_jump(&mut self, local: Point<f32>, theme: Theme, cx: &mut Context<Self>) {
        let (Some(bounds), Some(size)) = (self.graph.bounds(), self.viewport_size()) else {
            return;
        };
        let mini_w = 168.0;
        let mini_h = 104.0;
        let pad = 8.0;
        let origin_x = size.x - mini_w - theme.spacing.small;
        let origin_y = size.y - mini_h - theme.spacing.small;
        let scale =
            ((mini_w - pad * 2.0) / bounds.width()).min((mini_h - pad * 2.0) / bounds.height());
        let drawn_w = bounds.width() * scale;
        let drawn_h = bounds.height() * scale;
        let map_x = (mini_w - drawn_w) / 2.0;
        let map_y = (mini_h - drawn_h) / 2.0;
        let x = ((local.x - origin_x - map_x).clamp(0.0, drawn_w) / scale) + bounds.min_x;
        let y = ((local.y - origin_y - map_y).clamp(0.0, drawn_h) / scale) + bounds.min_y;
        let transform = GraphTransform {
            offset_x: size.x / 2.0 - x * self.transform.scale,
            offset_y: size.y / 2.0 - y * self.transform.scale,
            ..self.transform
        };
        self.request_view(transform, cx);
    }
    fn start_connection(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.focused_port.clone() else {
            return;
        };
        if self.graph.node_for_port(&id).is_some_and(|(_, p)| p.direction == PortDirection::Output)
        {
            self.connection_source = Some(id);
            self.connection_status = None;
            cx.notify();
        } else {
            self.connection_status = Some("Choose an output port to start a connection.".into());
            cx.notify();
        }
    }
    fn move_connection_candidate(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some(source) = self.connection_source.as_ref() else {
            return;
        };
        let candidates: Vec<&GraphPort> = reading_order(&self.graph.nodes)
            .into_iter()
            .flat_map(|n| n.inputs.iter())
            .filter(|p| self.valid_connection_pair(source, &p.id))
            .collect();
        if candidates.is_empty() {
            self.connection_status =
                Some("No compatible input ports are available for this output.".into());
            cx.notify();
            return;
        }
        let current =
            self.focused_port.as_ref().and_then(|id| candidates.iter().position(|p| &p.id == id));
        let index = current
            .map(|i| (i as isize + delta).rem_euclid(candidates.len() as isize) as usize)
            .unwrap_or(0);
        let id = candidates[index].id.clone();
        if let Some((node, _)) = self.graph.node_for_port(&id) {
            self.focused_node = Some(node.id.clone());
            self.focused_port = Some(id.clone());
            self.ports_active = true;
            cx.emit(FocusChanged { node_id: Some(node.id.clone()), port_id: Some(id) });
            cx.notify();
        }
    }
    fn remove_focused_connection(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_connection.take() {
            cx.emit(ConnectionRemoveRequested { connection_id: id });
            cx.notify();
        }
    }
    fn step_connection(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.graph.connections.is_empty() {
            return;
        }
        let current = self
            .selected_connection
            .as_ref()
            .and_then(|id| self.graph.connections.iter().position(|c| &c.id == id));
        let next = current
            .map(|i| {
                (i as isize + delta).rem_euclid(self.graph.connections.len() as isize) as usize
            })
            .unwrap_or(0);
        let id = self.graph.connections[next].id.clone();
        if self.selected_connection.as_deref() != Some(&id) {
            self.selected_connection = Some(id.clone());
            cx.emit(ConnectionFocusChanged { connection_id: Some(id) });
            cx.notify();
        }
    }
    fn center_focused_node(&mut self, cx: &mut Context<Self>) {
        let Some(node) =
            self.focused_node.as_ref().and_then(|id| self.graph.nodes.iter().find(|n| &n.id == id))
        else {
            return;
        };
        let size = self.viewport_size().unwrap_or(point(0.0, 0.0));
        let center =
            point(node.position.x + NODE_WIDTH / 2.0, node.position.y + node.height() / 2.0);
        self.request_view(
            GraphTransform {
                offset_x: size.x / 2.0 - center.x * self.transform.scale,
                offset_y: size.y / 2.0 - center.y * self.transform.scale,
                ..self.transform
            },
            cx,
        );
    }
    fn port_at(&self, world: Point<f32>, direction: Option<PortDirection>) -> Option<String> {
        self.graph
            .nodes
            .iter()
            .flat_map(|node| node.inputs.iter().chain(&node.outputs).map(move |port| (node, port)))
            .filter(|(_, port)| direction.is_none_or(|d| port.direction == d))
            .find(|(node, port)| {
                let p = GraphData::port_position(node, port);
                (p.x - world.x).abs() <= 10.0 && (p.y - world.y).abs() <= 10.0
            })
            .map(|(_, port)| port.id.clone())
    }
    pub fn set_graph(&mut self, graph: GraphData, cx: &mut Context<Self>) {
        self.cancel_preview(cx);
        self.graph = graph;
        if !self
            .graph
            .nodes
            .iter()
            .any(|node| Some(node.id.as_str()) == self.focused_node.as_deref())
        {
            self.focused_node =
                reading_order(&self.graph.nodes).first().map(|node| node.id.clone());
            self.focused_port = None;
            self.ports_active = false;
        }
        self.selected.retain(|id| self.graph.nodes.iter().any(|node| node.id == *id));
        cx.notify();
    }
    pub fn set_transform(&mut self, transform: GraphTransform, cx: &mut Context<Self>) {
        if transform.valid() {
            self.transform = transform;
            cx.notify();
        }
    }
    pub fn set_selection(&mut self, node_ids: Vec<String>, cx: &mut Context<Self>) {
        self.selected = self.valid_selection(node_ids);
        cx.notify();
    }
    fn valid_selection(&self, ids: Vec<String>) -> Vec<String> {
        let mut result = Vec::new();
        for id in ids {
            if self.graph.nodes.iter().any(|node| node.id == id) && !result.contains(&id) {
                result.push(id);
            }
        }
        result
    }
    fn viewport_size(&self) -> Option<Point<f32>> {
        self.bounds
            .get()
            .map(|bounds| point(bounds.size.width.as_f32(), bounds.size.height.as_f32()))
    }
    fn pointer_local(&self, pointer: Point<Pixels>) -> Option<Point<f32>> {
        self.bounds.get().map(|bounds| {
            point(
                pointer.x.as_f32() - bounds.origin.x.as_f32(),
                pointer.y.as_f32() - bounds.origin.y.as_f32(),
            )
        })
    }
    fn pointer_local_inside(&self, pointer: Point<Pixels>) -> Option<Point<f32>> {
        let local = self.pointer_local(pointer)?;
        let size = self.viewport_size()?;
        (local.x >= 0.0 && local.y >= 0.0 && local.x <= size.x && local.y <= size.y)
            .then_some(local)
    }
    fn request_view(&mut self, transform: GraphTransform, cx: &mut Context<Self>) {
        if !transform.valid() || transform == self.transform {
            return;
        }
        if !self.controlled_view {
            self.transform = transform;
        }
        cx.emit(ViewChanged { transform });
        cx.notify();
    }
    fn pan(&mut self, dx: f32, dy: f32, cx: &mut Context<Self>) {
        self.request_view(self.transform.panned(dx, dy), cx);
    }
    fn zoom(&mut self, factor: f32, at: Point<f32>, cx: &mut Context<Self>) {
        self.request_view(self.transform.zoomed_at(at, factor), cx);
    }
    fn fit(&mut self, theme: Theme, cx: &mut Context<Self>) {
        let size = self.viewport_size().unwrap_or(point(0.0, 0.0));
        let transform = self
            .graph
            .bounds()
            .map(|bounds| GraphTransform::fitted(bounds, size, theme.spacing.large))
            .unwrap_or_default();
        self.request_view(transform, cx);
    }
    fn actual_size(&mut self, cx: &mut Context<Self>) {
        let size = self.viewport_size().unwrap_or(point(0.0, 0.0));
        self.request_view(GraphTransform::actual_size(self.graph.bounds(), size), cx);
    }
    fn center(&self) -> Point<f32> {
        self.viewport_size()
            .map(|size| point(size.x / 2.0, size.y / 2.0))
            .unwrap_or(point(0.0, 0.0))
    }
    fn request_selection(&mut self, mut ids: Vec<String>, cx: &mut Context<Self>) {
        ids = self.valid_selection(ids);
        if ids == self.selected {
            return;
        }
        if !self.controlled_selection {
            self.selected = ids.clone();
        }
        cx.emit(SelectionChanged { node_ids: ids });
        cx.notify();
    }
    fn nudge_selection(&mut self, dx: f32, dy: f32, cx: &mut Context<Self>) {
        let positions: Vec<_> = self
            .graph
            .nodes
            .iter()
            .filter(|n| self.selected.contains(&n.id))
            .map(|n| NodePosition {
                node_id: n.id.clone(),
                position: point(n.position.x + dx, n.position.y + dy),
            })
            .collect();
        self.request_move(positions, cx);
    }
    fn request_move(&mut self, positions: Vec<NodePosition>, cx: &mut Context<Self>) {
        if positions.is_empty()
            || positions.iter().any(|p| !p.position.x.is_finite() || !p.position.y.is_finite())
        {
            return;
        }
        let mut positions = positions;
        positions.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        if positions.iter().any(|p| !self.graph.nodes.iter().any(|n| n.id == p.node_id)) {
            return;
        }
        if positions
            .iter()
            .all(|p| self.graph.nodes.iter().any(|n| n.id == p.node_id && n.position == p.position))
        {
            return;
        }
        cx.emit(NodesMoveRequested { positions });
        cx.notify();
    }
    fn cancel_preview(&mut self, cx: &mut Context<Self>) -> bool {
        let active = self.node_drag.take().is_some() || self.box_drag.take().is_some();
        if active {
            self.pointer_down = false;
            self.last_pointer = None;
            cx.notify();
        }
        active
    }
    fn focus_node(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        if let Some(ref id) = id
            && !self.graph.nodes.iter().any(|node| node.id == *id)
        {
            return;
        }
        if self.focused_node == id && self.focused_port.is_none() {
            return;
        }
        self.focused_node = id.clone();
        self.focused_port = None;
        self.ports_active = false;
        cx.emit(FocusChanged { node_id: id, port_id: None });
        cx.notify();
    }
    fn step_node(&mut self, delta: isize, cx: &mut Context<Self>) {
        let nodes = reading_order(&self.graph.nodes);
        if nodes.is_empty() {
            return;
        }
        let current = self
            .focused_node
            .as_ref()
            .and_then(|id| nodes.iter().position(|node| &node.id == id))
            .unwrap_or(0);
        let next = (current as isize + delta).clamp(0, nodes.len() as isize - 1) as usize;
        self.focus_node(Some(nodes[next].id.clone()), cx);
    }
    fn select_focused(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.focused_node.clone() else {
            return;
        };
        self.request_selection(vec![id], cx);
    }
    fn enter_ports(&mut self, cx: &mut Context<Self>) {
        let Some(node_id) = self.focused_node.as_ref() else {
            return;
        };
        let Some(node) = self.graph.nodes.iter().find(|node| &node.id == node_id) else {
            return;
        };
        let Some(port) = node.inputs.iter().chain(&node.outputs).next() else {
            return;
        };
        self.ports_active = true;
        self.focused_port = Some(port.id.clone());
        cx.emit(FocusChanged { node_id: Some(node.id.clone()), port_id: Some(port.id.clone()) });
        cx.notify();
    }
    fn exit_ports(&mut self, cx: &mut Context<Self>) {
        if !self.ports_active {
            return;
        }
        self.ports_active = false;
        self.focused_port = None;
        cx.emit(FocusChanged { node_id: self.focused_node.clone(), port_id: None });
        cx.notify();
    }
    fn step_port(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some(node_id) = self.focused_node.as_ref() else {
            return;
        };
        let Some(node) = self.graph.nodes.iter().find(|node| &node.id == node_id) else {
            return;
        };
        let ports: Vec<&GraphPort> = node.inputs.iter().chain(&node.outputs).collect();
        if ports.is_empty() {
            return;
        }
        let index = self
            .focused_port
            .as_ref()
            .and_then(|id| ports.iter().position(|port| &port.id == id))
            .unwrap_or(0);
        let next = (index as isize + delta).clamp(0, ports.len() as isize - 1) as usize;
        let port_id = ports[next].id.clone();
        if self.focused_port.as_deref() != Some(&port_id) {
            self.focused_port = Some(port_id.clone());
            cx.emit(FocusChanged { node_id: Some(node.id.clone()), port_id: Some(port_id) });
            cx.notify();
        }
    }
    fn pan_action(&mut self, dx: f32, dy: f32, cx: &mut Context<Self>) {
        let step = 32.0;
        self.pan(dx * step, dy * step, cx);
    }
    fn on_next(&mut self, _: &NextNode, _: &mut Window, cx: &mut Context<Self>) {
        if self.ports_active { self.step_port(1, cx) } else { self.step_node(1, cx) }
    }
    fn on_previous(&mut self, _: &PreviousNode, _: &mut Window, cx: &mut Context<Self>) {
        if self.ports_active { self.step_port(-1, cx) } else { self.step_node(-1, cx) }
    }
    fn on_select(&mut self, _: &SelectFocused, _: &mut Window, cx: &mut Context<Self>) {
        self.select_focused(cx);
    }
    fn on_add_focused(&mut self, _: &AddFocused, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.focused_node.clone() {
            let mut ids = self.selected.clone();
            if !ids.contains(&id) {
                ids.push(id);
            }
            self.request_selection(ids, cx);
        }
    }
    fn on_toggle_focused(&mut self, _: &ToggleFocused, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.focused_node.clone() {
            let mut ids = self.selected.clone();
            if let Some(i) = ids.iter().position(|v| v == &id) {
                ids.remove(i);
            } else {
                ids.push(id);
            }
            self.request_selection(ids, cx);
        }
    }
    fn on_enter_ports(&mut self, _: &EnterPorts, _: &mut Window, cx: &mut Context<Self>) {
        self.enter_ports(cx);
    }
    fn on_exit_ports(&mut self, _: &ExitPorts, _: &mut Window, cx: &mut Context<Self>) {
        if self.connection_source.take().is_some() {
            cx.notify();
            return;
        }
        if self.cancel_preview(cx) {
            return;
        }
        self.exit_ports(cx);
    }
    fn on_start_connection(&mut self, _: &StartConnection, _: &mut Window, cx: &mut Context<Self>) {
        self.start_connection(cx);
    }
    fn on_commit_connection(
        &mut self,
        _: &CommitConnection,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let (Some(from), Some(to)) = (self.connection_source.clone(), self.focused_port.clone())
        {
            self.request_connection(from, to, cx);
        }
    }
    fn on_next_connection_candidate(
        &mut self,
        _: &NextConnectionCandidate,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_connection_candidate(1, cx);
    }
    fn on_previous_connection_candidate(
        &mut self,
        _: &PreviousConnectionCandidate,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_connection_candidate(-1, cx);
    }
    fn on_remove_connection(
        &mut self,
        _: &RemoveFocusedConnection,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.remove_focused_connection(cx);
    }
    fn on_toggle_minimap(&mut self, _: &ToggleMinimap, _: &mut Window, cx: &mut Context<Self>) {
        self.toggle_minimap(cx);
    }
    fn on_next_connection(&mut self, _: &NextConnection, _: &mut Window, cx: &mut Context<Self>) {
        self.step_connection(1, cx);
    }
    fn on_previous_connection(
        &mut self,
        _: &PreviousConnection,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.step_connection(-1, cx);
    }
    fn on_center_focused_node(
        &mut self,
        _: &CenterFocusedNode,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.center_focused_node(cx);
    }
    fn on_nudge_left(&mut self, _: &NudgeLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge_selection(-1.0, 0.0, cx);
    }
    fn on_nudge_right(&mut self, _: &NudgeRight, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge_selection(1.0, 0.0, cx);
    }
    fn on_nudge_up(&mut self, _: &NudgeUp, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge_selection(0.0, -1.0, cx);
    }
    fn on_nudge_down(&mut self, _: &NudgeDown, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge_selection(0.0, 1.0, cx);
    }
    fn on_begin_box(&mut self, _: &BeginBoxSelection, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(node) =
            self.focused_node.as_ref().and_then(|id| self.graph.nodes.iter().find(|n| &n.id == id))
        {
            let p = point(node.position.x + NODE_WIDTH, node.position.y + node.height());
            let current = node.position;
            self.box_drag = Some(BoxDrag {
                start: p,
                current,
                original_selection: self.selected.clone(),
                additive: false,
                toggle: false,
            });
            cx.notify();
        }
    }
    fn on_commit_box(&mut self, _: &CommitBoxSelection, _: &mut Window, cx: &mut Context<Self>) {
        self.finish_box(cx);
    }
    fn move_box_corner(&mut self, dx: f32, dy: f32, cx: &mut Context<Self>) {
        if let Some(b) = self.box_drag.as_mut() {
            b.current.x += dx;
            b.current.y += dy;
            cx.notify();
        }
    }
    fn on_box_left(&mut self, _: &BoxCornerLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.move_box_corner(-1.0, 0.0, cx)
    }
    fn on_box_right(&mut self, _: &BoxCornerRight, _: &mut Window, cx: &mut Context<Self>) {
        self.move_box_corner(1.0, 0.0, cx)
    }
    fn on_box_up(&mut self, _: &BoxCornerUp, _: &mut Window, cx: &mut Context<Self>) {
        self.move_box_corner(0.0, -1.0, cx)
    }
    fn on_box_down(&mut self, _: &BoxCornerDown, _: &mut Window, cx: &mut Context<Self>) {
        self.move_box_corner(0.0, 1.0, cx)
    }
    fn finish_box(&mut self, cx: &mut Context<Self>) {
        let Some(box_drag) = self.box_drag.take() else {
            return;
        };
        let selected: Vec<String> = self
            .graph
            .nodes
            .iter()
            .filter(|n| node_fully_contained(n, box_drag.start, box_drag.current))
            .map(|n| n.id.clone())
            .collect();
        let mut result = if box_drag.additive || box_drag.toggle {
            box_drag.original_selection
        } else {
            Vec::new()
        };
        for id in selected {
            if box_drag.toggle {
                if let Some(i) = result.iter().position(|v| v == &id) {
                    result.remove(i);
                } else {
                    result.push(id);
                }
            } else if !result.contains(&id) {
                result.push(id);
            }
        }
        self.request_selection(result, cx);
        cx.notify();
    }
    fn on_pan_left(&mut self, _: &PanLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.pan_action(-1.0, 0.0, cx);
    }
    fn on_pan_right(&mut self, _: &PanRight, _: &mut Window, cx: &mut Context<Self>) {
        self.pan_action(1.0, 0.0, cx);
    }
    fn on_pan_up(&mut self, _: &PanUp, _: &mut Window, cx: &mut Context<Self>) {
        self.pan_action(0.0, -1.0, cx);
    }
    fn on_pan_down(&mut self, _: &PanDown, _: &mut Window, cx: &mut Context<Self>) {
        self.pan_action(0.0, 1.0, cx);
    }
    fn on_zoom_in(&mut self, _: &ZoomIn, _: &mut Window, cx: &mut Context<Self>) {
        self.zoom(1.2, self.center(), cx);
    }
    fn on_zoom_out(&mut self, _: &ZoomOut, _: &mut Window, cx: &mut Context<Self>) {
        self.zoom(1.0 / 1.2, self.center(), cx);
    }
    fn on_fit(&mut self, _: &FitGraph, _: &mut Window, cx: &mut Context<Self>) {
        let theme = *cx.global::<Theme>();
        self.fit(theme, cx);
    }
    fn on_actual_size(&mut self, _: &ActualSize, _: &mut Window, cx: &mut Context<Self>) {
        self.actual_size(cx);
    }
    fn on_mouse_down(&mut self, event: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if event.button == MouseButton::Middle
            || (event.button == MouseButton::Left && self.space_held)
        {
            self.pointer_down = true;
            self.last_pointer = self.pointer_local(event.position);
            return;
        }
        if event.button == MouseButton::Left {
            let Some(local) = self.pointer_local(event.position) else {
                return;
            };
            let world = self.transform.screen_to_world(local);
            if let Some(port_id) = self.port_at(world, Some(PortDirection::Output)) {
                self.connection_source = Some(port_id);
                self.connection_status = None;
                self.pointer_down = true;
                self.last_pointer = Some(local);
                cx.notify();
                return;
            }
            let hit = self
                .graph
                .nodes
                .iter()
                .rev()
                .find(|n| {
                    n.position.x.is_finite()
                        && n.position.y.is_finite()
                        && world.x >= n.position.x
                        && world.x <= n.position.x + NODE_WIDTH
                        && world.y >= n.position.y
                        && world.y <= n.position.y + n.height()
                })
                .map(|n| n.id.clone());
            if let Some(id) = hit {
                self.focus_node(Some(id.clone()), cx);
                let mut selection = self.selected.clone();
                if event.modifiers.platform {
                    if let Some(i) = selection.iter().position(|v| v == &id) {
                        selection.remove(i);
                    } else {
                        selection.push(id.clone());
                    }
                } else if event.modifiers.shift {
                    if !selection.contains(&id) {
                        selection.push(id.clone());
                    }
                } else if !selection.contains(&id) {
                    selection = vec![id.clone()];
                }
                let move_selection = selection.clone();
                self.request_selection(selection, cx);
                let ids = if move_selection.contains(&id) { move_selection } else { Vec::new() };
                let originals: Vec<NodePosition> = self
                    .graph
                    .nodes
                    .iter()
                    .filter(|n| ids.contains(&n.id))
                    .map(|n| NodePosition { node_id: n.id.clone(), position: n.position })
                    .collect();
                if !originals.is_empty() {
                    self.node_drag =
                        Some(NodeDrag { start: local, originals, delta: point(0.0, 0.0) });
                }
            } else {
                self.box_drag = Some(BoxDrag {
                    start: world,
                    current: world,
                    original_selection: self.selected.clone(),
                    additive: event.modifiers.shift,
                    toggle: event.modifiers.platform,
                });
            }
            self.pointer_down = true;
        }
    }
    fn on_minimap_down(&mut self, event: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if event.button == MouseButton::Left
            && let Some(local) = self.pointer_local(event.position)
        {
            self.minimap_jump(local, *cx.global::<Theme>(), cx);
        }
    }
    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.pointer_down || !event.dragging() {
            return;
        }
        let current = self.pointer_local(event.position);
        if self.connection_source.is_some() {
            self.last_pointer = current;
            cx.notify();
            return;
        }
        if let Some(current) = current {
            if let Some(drag) = self.node_drag.as_mut() {
                drag.delta = point(
                    (current.x - drag.start.x) / self.transform.scale,
                    (current.y - drag.start.y) / self.transform.scale,
                );
                cx.notify();
                return;
            }
            if let Some(drag) = self.box_drag.as_mut() {
                drag.current = self.transform.screen_to_world(current);
                cx.notify();
                return;
            }
        }
        if let (Some(previous), Some(current)) = (self.last_pointer, current) {
            self.pan(current.x - previous.x, current.y - previous.y, cx);
        }
        self.last_pointer = current;
    }
    fn on_mouse_up(&mut self, event: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        let release = self.pointer_local_inside(event.position);
        if let Some(from) = self.connection_source.take() {
            if let Some(local) = release {
                let world = self.transform.screen_to_world(local);
                if let Some(to) = self.port_at(world, None) {
                    self.request_connection(from, to, cx);
                } else {
                    self.connection_status =
                        Some("Connection cancelled: release over an input port.".into());
                }
            } else {
                self.connection_status =
                    Some("Connection cancelled: release inside the canvas.".into());
            }
            self.pointer_down = false;
            self.last_pointer = None;
            cx.notify();
            return;
        }
        if let Some(drag) = self.node_drag.take()
            && let Some(local) = release
        {
            let delta = point(
                (local.x - drag.start.x) / self.transform.scale,
                (local.y - drag.start.y) / self.transform.scale,
            );
            let positions = drag
                .originals
                .into_iter()
                .map(|p| NodePosition {
                    node_id: p.node_id,
                    position: point(p.position.x + delta.x, p.position.y + delta.y),
                })
                .collect();
            self.request_move(positions, cx);
        }
        if let Some(local) = release {
            if let Some(box_drag) = self.box_drag.as_mut() {
                box_drag.current = self.transform.screen_to_world(local);
            }
            self.finish_box(cx);
        } else {
            self.box_drag = None;
        }
        self.pointer_down = false;
        self.last_pointer = None;
        cx.notify();
    }
    fn on_wheel(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        match event.delta {
            ScrollDelta::Lines(delta) => {
                if let Some(pointer) = self.pointer_local(event.position) {
                    self.zoom(1.12_f32.powf(delta.y), pointer, cx);
                }
            }
            ScrollDelta::Pixels(delta) => self.pan(delta.x.as_f32(), delta.y.as_f32(), cx),
        }
    }
    fn on_pinch(&mut self, event: &PinchEvent, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(pointer) = self.pointer_local(event.position) {
            self.zoom(1.0 + event.delta, pointer, cx);
        }
    }
    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, _: &mut Context<Self>) {
        if event.keystroke.key == "space" {
            self.space_held = true;
        }
    }
    fn on_key_up(&mut self, event: &KeyUpEvent, _: &mut Window, _: &mut Context<Self>) {
        if event.keystroke.key == "space" {
            self.space_held = false;
        }
    }
    /// A small outline button (the restyled Button's `outline` variant) at the dense
    /// `controls.xsmall` toolbar height.
    fn toolbar_button(
        id: &'static str,
        label: &'static str,
        theme: Theme,
        click: impl Fn(&gpui_pre::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> impl IntoElement {
        let look = look(&theme);
        let c = theme.colors;
        div()
            .id(id)
            .debug_selector(move || id.into())
            .role(gpui_pre::accesskit::Role::Button)
            .aria_label(label)
            .tab_index(0)
            .h(px(theme.controls.xsmall))
            .min_w(px(theme.controls.small))
            .px(px(theme.spacing.small))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.regular))
            .border_color(look.control_border)
            .bg(c.background)
            .shadow(vec![box_shadow(theme.shadows.small)])
            .text_color(c.text)
            .text_size(px(theme.typography.body))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .hover(
                move |s| {
                    if look.high_contrast { s.border_color(c.accent) } else { s.bg(look.muted) }
                },
            )
            .focus_visible(move |s| {
                s.border_color(c.focus).bg(c.background).shadow(vec![focus_ring(look.ring)])
            })
            .on_click(click)
            .child(label)
    }
}

impl Focusable for NodeEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone().expect("NodeEditor focus initialized during render")
    }
}

impl Render for NodeEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let look = look(&theme);
        let transparent = theme.colors.background.opacity(0.);
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(0)).clone();
        let transform = self.transform;
        let bounds_cell = self.bounds.clone();
        let draw_graph = self.graph.clone();
        let draw_graph = if let Some(drag) = &self.node_drag {
            let mut g = draw_graph;
            for p in drag.originals.iter() {
                if let Some(n) = g.nodes.iter_mut().find(|n| n.id == p.node_id) {
                    n.position = point(p.position.x + drag.delta.x, p.position.y + drag.delta.y);
                }
            }
            g
        } else {
            draw_graph
        };
        let draw_theme = theme;
        let connection_preview = self.connection_source.as_ref().and_then(|source| {
            if self.pointer_down {
                self.last_pointer
            } else {
                self.focused_port.as_ref().and_then(|port_id| {
                    self.graph.node_for_port(port_id).map(|(node, port)| {
                        self.transform.world_to_screen(GraphData::port_position(node, port))
                    })
                })
            }
            .map(|target| (source.clone(), target))
        });
        let canvas_view = canvas(
            move |bounds, _, _| {
                bounds_cell.set(Some(bounds));
                (bounds, transform, draw_graph.clone())
            },
            move |_, (bounds, transform, graph), window, _| {
                paint_connections(
                    bounds,
                    transform,
                    &graph,
                    connection_preview.clone(),
                    draw_theme,
                    window,
                )
            },
        )
        .size_full();
        let focused_port_description = self.focused_port.as_ref().and_then(|id| {
            self.graph.node_for_port(id).map(|(_, port)| format!(
                "Focused {} port {}, type {}",
                match port.direction { PortDirection::Input => "input", PortDirection::Output => "output" },
                port.label, port.data_type,
            ))
        }).unwrap_or_else(|| "Arrow Up and Down navigate nodes. Space selects. Enter enters ports. Alt+arrows pan; equals and minus zoom; F fits; 1 shows actual size.".into());
        let preview_description = if let Some(drag) = &self.node_drag {
            format!("Moving {} selected nodes.", drag.originals.len())
        } else if self.box_drag.is_some() {
            "Box selection preview active.".to_string()
        } else {
            String::new()
        };
        let selection_description = format!(
            "{} nodes selected. {} {} {}",
            self.selected.len(),
            preview_description,
            focused_port_description,
            self.connection_status.as_deref().unwrap_or("")
        );
        let mut root = div()
            .id("mkit-node-editor")
            .debug_selector(|| "mkit-node-editor".into())
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .tab_index(0)
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.label.clone())
            .aria_description(selection_description)
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .bg(theme.colors.background)
            .border(px(theme.borders.hairline))
            .border_color(look.divider)
            .rounded(px(theme.radii.large))
            .focus_visible(move |element| {
                element.border_color(theme.colors.focus).shadow(vec![focus_ring(look.ring)])
            })
            .on_action(cx.listener(Self::on_next))
            .on_action(cx.listener(Self::on_previous))
            .on_action(cx.listener(Self::on_select))
            .on_action(cx.listener(Self::on_add_focused))
            .on_action(cx.listener(Self::on_toggle_focused))
            .on_action(cx.listener(Self::on_enter_ports))
            .on_action(cx.listener(Self::on_exit_ports))
            .on_action(cx.listener(Self::on_pan_left))
            .on_action(cx.listener(Self::on_pan_right))
            .on_action(cx.listener(Self::on_pan_up))
            .on_action(cx.listener(Self::on_pan_down))
            .on_action(cx.listener(Self::on_zoom_in))
            .on_action(cx.listener(Self::on_zoom_out))
            .on_action(cx.listener(Self::on_fit))
            .on_action(cx.listener(Self::on_actual_size))
            .on_action(cx.listener(Self::on_nudge_left))
            .on_action(cx.listener(Self::on_nudge_right))
            .on_action(cx.listener(Self::on_nudge_up))
            .on_action(cx.listener(Self::on_nudge_down))
            .on_action(cx.listener(Self::on_begin_box))
            .on_action(cx.listener(Self::on_commit_box))
            .on_action(cx.listener(Self::on_box_left))
            .on_action(cx.listener(Self::on_box_right))
            .on_action(cx.listener(Self::on_box_up))
            .on_action(cx.listener(Self::on_box_down))
            .on_action(cx.listener(Self::on_start_connection))
            .on_action(cx.listener(Self::on_commit_connection))
            .on_action(cx.listener(Self::on_next_connection_candidate))
            .on_action(cx.listener(Self::on_previous_connection_candidate))
            .on_action(cx.listener(Self::on_remove_connection))
            .on_action(cx.listener(Self::on_toggle_minimap))
            .on_action(cx.listener(Self::on_next_connection))
            .on_action(cx.listener(Self::on_previous_connection))
            .on_action(cx.listener(Self::on_center_focused_node));
        let zoom_label = format!("{:.0}%", self.transform.scale * 100.0);
        root = root.child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .gap(px(theme.spacing.xsmall))
                .h(px(theme.controls.small))
                .px(px(theme.spacing.small))
                .bg(theme.colors.background)
                .border_b(px(theme.borders.hairline))
                .border_color(look.divider)
                .text_color(theme.colors.text_muted)
                .text_size(px(theme.typography.caption))
                .child(zoom_label)
                .child(Self::toolbar_button(
                    "mkit-node-editor-fit",
                    "Fit",
                    theme,
                    cx.listener(|this, _, _, cx| {
                        let theme = *cx.global::<Theme>();
                        this.fit(theme, cx);
                    }),
                ))
                .child(Self::toolbar_button(
                    "mkit-node-editor-actual",
                    "100%",
                    theme,
                    cx.listener(|this, _, _, cx| this.actual_size(cx)),
                ))
                .child(Self::toolbar_button(
                    "mkit-node-editor-zoom-out",
                    "Zoom out",
                    theme,
                    cx.listener(|this, _, _, cx| this.zoom(1.0 / 1.2, this.center(), cx)),
                ))
                .child(Self::toolbar_button(
                    "mkit-node-editor-zoom-in",
                    "Zoom in",
                    theme,
                    cx.listener(|this, _, _, cx| this.zoom(1.2, this.center(), cx)),
                ))
                .child(Self::toolbar_button(
                    "mkit-node-editor-minimap-toggle",
                    if self.minimap_visible { "Hide map" } else { "Show map" },
                    theme,
                    cx.listener(|this, _, _, cx| this.toggle_minimap(cx)),
                )),
        );
        let transform = self.transform;
        let mut canvas_container = div()
            .id("mkit-node-editor-canvas")
            .debug_selector(|| "mkit-node-editor-canvas".into())
            .relative()
            .flex_1()
            .overflow_hidden()
            .on_mouse_down(MouseButton::Middle, cx.listener(Self::on_mouse_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Middle, cx.listener(Self::on_mouse_up))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_scroll_wheel(cx.listener(Self::on_wheel))
            .on_pinch(cx.listener(Self::on_pinch))
            .on_key_down(cx.listener(Self::on_key_down))
            .on_key_up(cx.listener(Self::on_key_up))
            .child(canvas_view);
        for node in self
            .graph
            .nodes
            .iter()
            .filter(|node| node.position.x.is_finite() && node.position.y.is_finite())
        {
            let position =
                self.node_drag
                    .as_ref()
                    .and_then(|drag| {
                        drag.originals.iter().find(|p| p.node_id == node.id).map(|p| {
                            point(p.position.x + drag.delta.x, p.position.y + drag.delta.y)
                        })
                    })
                    .unwrap_or(node.position);
            let screen = transform.world_to_screen(position);
            let scale = transform.scale;
            let height = node.height();
            let selected = self.selected.contains(&node.id);
            let focused = self.focused_node.as_deref() == Some(node.id.as_str())
                && self.focused_port.is_none();
            let card_radius = theme.radii.large * scale;
            let mut card = div()
                .id(format!("node-{}", node.id))
                .debug_selector({
                    let selector = format!("node-{}", node.id);
                    move || selector.clone()
                })
                .absolute()
                .left(px(screen.x))
                .top(px(screen.y))
                .w(px(NODE_WIDTH * scale))
                .h(px(height * scale))
                .role(gpui_pre::accesskit::Role::Group)
                .aria_label(format!("{} node", node.title))
                .aria_selected(selected)
                // Keyboard focus: a `focus` border at the strong width. Selection: the focus
                // ring outside the card plus a filled title band. Hover: a darker border.
                .border(px(if focused {
                    theme.borders.strong * scale
                } else {
                    theme.borders.hairline * scale
                }))
                .border_color(if focused { theme.colors.focus } else { look.divider })
                .when(!focused, |card| card.hover(move |s| s.border_color(look.hover_border)))
                .rounded(px(card_radius))
                .bg(theme.colors.surface)
                .shadow(if selected {
                    vec![focus_ring(look.ring)]
                } else {
                    vec![box_shadow(theme.shadows.small)]
                })
                .text_color(theme.colors.text)
                .flex()
                .flex_col()
                .overflow_hidden();
            card = card.child(
                div()
                    .h(px(HEADER_HEIGHT * scale))
                    .px(px(theme.spacing.small * scale))
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .border_b(px(theme.borders.hairline * scale))
                    .border_color(look.divider)
                    .text_size(px(theme.typography.body * scale))
                    .font_weight(FontWeight::MEDIUM)
                    .when(selected, |header| {
                        header
                            .bg(look.selected_bg)
                            .text_color(look.selected_text)
                            .rounded_t(px(card_radius))
                    })
                    .child(node.title.clone()),
            );
            let port_rows = node.inputs.len().max(node.outputs.len()).max(1);
            for row_index in 0..port_rows {
                let input = node.inputs.get(row_index);
                let output = node.outputs.get(row_index);
                let mut row = div()
                    .h(px(PORT_ROW_HEIGHT * scale))
                    .px(px(theme.spacing.xsmall * scale))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(theme.spacing.xsmall * scale))
                    .text_size(px(theme.typography.caption * scale));
                if let Some(port) = input {
                    let port_focused = self.focused_port.as_deref() == Some(port.id.as_str());
                    row = row.child(
                        div()
                            .id(format!("port-{}", port.id))
                            .flex()
                            .items_center()
                            .gap(px(theme.spacing.xsmall * scale))
                            .role(gpui_pre::accesskit::Role::Label)
                            .aria_label(format!("Input {}, type {}", port.label, port.data_type))
                            .child(port_handle(theme, look, scale, port_focused))
                            .child(port.label.clone()),
                    );
                } else {
                    row = row.child(div().flex_1());
                }
                if let Some(port) = output {
                    let port_focused = self.focused_port.as_deref() == Some(port.id.as_str());
                    row = row.child(
                        div()
                            .id(format!("port-{}", port.id))
                            .flex()
                            .items_center()
                            .gap(px(theme.spacing.xsmall * scale))
                            .role(gpui_pre::accesskit::Role::Label)
                            .aria_label(format!("Output {}, type {}", port.label, port.data_type))
                            .child(port.label.clone())
                            .child(port_handle(theme, look, scale, port_focused)),
                    );
                }
                card = card.child(row);
            }
            canvas_container = canvas_container.child(card);
        }
        if self.minimap_visible
            && let (Some(graph_bounds), Some(viewport)) =
                (self.graph.bounds(), self.viewport_size())
        {
            let mini_w = 168.0;
            let mini_h = 104.0;
            let inner = 8.0;
            let map_scale = ((mini_w - inner * 2.0) / graph_bounds.width())
                .min((mini_h - inner * 2.0) / graph_bounds.height());
            let map_w = graph_bounds.width() * map_scale;
            let map_h = graph_bounds.height() * map_scale;
            let map_x = (mini_w - map_w) / 2.0;
            let map_y = (mini_h - map_h) / 2.0;
            let top_left = transform.screen_to_world(point(0.0, 0.0));
            let view_world_w = viewport.x / transform.scale;
            let view_world_h = viewport.y / transform.scale;
            let mut minimap = div()
                    .id("mkit-node-editor-minimap")
                    .debug_selector(|| "mkit-node-editor-minimap".into())
                    .absolute()
                    .right(px(theme.spacing.small))
                    .bottom(px(theme.spacing.small))
                    .w(px(mini_w))
                    .h(px(mini_h))
                    .overflow_hidden()
                    .role(gpui_pre::accesskit::Role::Button)
                    .tab_index(0)
                    .aria_label("Graph minimap. Click to center the viewport on a graph region. Use Alt+Arrow keys to pan.")
                    .aria_description(format!("Current viewport: x {:.0} to {:.0}, y {:.0} to {:.0}.", top_left.x, top_left.x + view_world_w, top_left.y, top_left.y + view_world_h))
                    .border(px(theme.borders.hairline))
                    .border_color(look.divider)
                    .rounded(px(theme.radii.large))
                    .bg(theme.colors.surface)
                    .shadow(vec![box_shadow(theme.shadows.small)])
                    .focus_visible(move |element| {
                        element
                            .border_color(theme.colors.focus)
                            .shadow(vec![focus_ring(look.ring)])
                    })
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::on_minimap_down));
            for node in &self.graph.nodes {
                if !node.position.x.is_finite() || !node.position.y.is_finite() {
                    continue;
                }
                minimap = minimap.child(
                    div()
                        .absolute()
                        .left(px(map_x + (node.position.x - graph_bounds.min_x) * map_scale))
                        .top(px(map_y + (node.position.y - graph_bounds.min_y) * map_scale))
                        .w(px((NODE_WIDTH * map_scale).max(2.0)))
                        .h(px((node.height() * map_scale).max(2.0)))
                        .rounded(px(theme.radii.small * map_scale))
                        .bg(theme.colors.text_muted),
                );
            }
            minimap = minimap.child(
                div()
                    .id("mkit-node-editor-minimap-viewport")
                    .absolute()
                    .left(px(map_x + (top_left.x - graph_bounds.min_x) * map_scale))
                    .top(px(map_y + (top_left.y - graph_bounds.min_y) * map_scale))
                    .w(px((view_world_w * map_scale).max(2.0)))
                    .h(px((view_world_h * map_scale).max(2.0)))
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.strong))
                    .border_color(theme.colors.accent),
            );
            canvas_container = canvas_container.child(minimap);
        }
        if let Some(box_drag) = &self.box_drag {
            let a = transform.world_to_screen(box_drag.start);
            let b = transform.world_to_screen(box_drag.current);
            canvas_container = canvas_container.child(
                div()
                    .id("node-editor-box-selection")
                    .absolute()
                    .left(px(a.x.min(b.x)))
                    .top(px(a.y.min(b.y)))
                    .w(px((a.x - b.x).abs().max(1.0)))
                    .h(px((a.y - b.y).abs().max(1.0)))
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.hairline))
                    .border_color(theme.colors.accent)
                    .bg(theme.colors.accent.opacity(0.1))
                    .role(gpui_pre::accesskit::Role::Status)
                    .aria_label("Box selection preview"),
            );
        }
        root = root.child(canvas_container);
        if let Some(status) = self.connection_status.as_ref() {
            root = root.child(
                div()
                    .id("node-editor-connection-status")
                    .role(gpui_pre::accesskit::Role::Status)
                    .px(px(theme.spacing.small))
                    .py(px(theme.spacing.xsmall))
                    .text_color(theme.colors.text_muted)
                    .text_size(px(theme.typography.caption))
                    .child(status.clone()),
            );
        }
        let summaries =
            self.graph.connections.iter().map(|connection| self.connection_summary(connection));
        let mut summary_row = div()
            .id("mkit-node-editor-connections")
            .role(gpui_pre::accesskit::Role::List)
            .aria_label("Graph connections")
            .max_h(px(theme.controls.large * 2.0))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .px(px(theme.spacing.small))
            .py(px(theme.spacing.xsmall))
            .bg(theme.colors.surface)
            .border_t(px(theme.borders.hairline))
            .border_color(look.divider)
            .text_color(theme.colors.text_muted)
            .text_size(px(theme.typography.caption));
        for (index, summary) in summaries.enumerate() {
            let connection_id = self.graph.connections[index].id.clone();
            let selected = self.selected_connection.as_deref() == Some(connection_id.as_str());
            summary_row = summary_row.child(
                div()
                    .id(format!("connection-summary-{index}"))
                    .role(gpui_pre::accesskit::Role::Button)
                    .tab_index(0)
                    .aria_selected(selected)
                    .aria_label(format!("{}{}", summary, if selected { ", selected" } else { "" }))
                    // The restyled Tree row: radius-small, a reserved hairline border, accent
                    // fill with a `focus` outline for the focused connection, muted hover.
                    .px(px(theme.spacing.small))
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.hairline))
                    .border_color(if selected { theme.colors.focus } else { transparent })
                    .when(selected, |row| row.bg(look.selected_bg).text_color(look.selected_text))
                    .when(!selected, |row| {
                        row.hover(move |s| {
                            if look.high_contrast {
                                s.border_color(theme.colors.border)
                            } else {
                                s.bg(look.muted)
                            }
                        })
                    })
                    .focus_visible(move |row| row.border_color(theme.colors.focus))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.selected_connection = Some(connection_id.clone());
                        cx.notify();
                    }))
                    .child(summary),
            );
        }
        root.child(summary_row)
    }
}

impl NodeEditor {
    fn connection_summary(&self, connection: &GraphConnection) -> String {
        let endpoint = |id: &str| {
            self.graph
                .node_for_port(id)
                .map(|(node, port)| format!("{} / {}", node.title, port.label))
        };
        match (endpoint(&connection.from_port_id), endpoint(&connection.to_port_id)) {
            (Some(from), Some(to)) => format!(
                "{} to {}{}",
                from,
                to,
                connection.label.as_ref().map(|label| format!(" ({label})")).unwrap_or_default()
            ),
            _ => format!(
                "Unresolved connection {}: {} to {}",
                connection.id, connection.from_port_id, connection.to_port_id
            ),
        }
    }
}

/// A port drawn in the restyled Slider thumb look at the graph's port size: an opaque
/// `background` fill, a hairline `accent` border, and the small shadow. The keyboard-focused port
/// fills with `accent` and gains a `focus` border and the focus ring in place of the shadow.
fn port_handle(theme: Theme, look: Look, scale: f32, focused: bool) -> impl IntoElement {
    let size = PORT_RADIUS * scale;
    // GPUI draws a box-shadow ring with the element's clamped corner radius, which squares off a
    // ring around so small a circle, so the focus ring is a round layer painted behind the port.
    // Both layers are absolutely positioned, so the port's layout footprint is unchanged.
    div()
        .flex_none()
        .relative()
        .size(px(size))
        .when(focused, |port| {
            port.child(
                div()
                    .absolute()
                    .top(px(-FOCUS_RING_WIDTH))
                    .left(px(-FOCUS_RING_WIDTH))
                    .size(px(size + FOCUS_RING_WIDTH * 2.0))
                    .rounded(px(size / 2.0 + FOCUS_RING_WIDTH))
                    .bg(look.ring),
            )
        })
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded(px(size / 2.0))
                .border(px(theme.borders.hairline * scale))
                .border_color(if focused { theme.colors.focus } else { theme.colors.accent })
                .bg(if focused { theme.colors.accent } else { theme.colors.background })
                .when(!focused, |port| port.shadow(vec![box_shadow(theme.shadows.small)])),
        )
}

fn reading_order(nodes: &[GraphNode]) -> Vec<&GraphNode> {
    let mut nodes: Vec<&GraphNode> = nodes
        .iter()
        .filter(|node| node.position.x.is_finite() && node.position.y.is_finite())
        .collect();
    nodes.sort_by(|a, b| {
        a.position
            .y
            .total_cmp(&b.position.y)
            .then_with(|| a.position.x.total_cmp(&b.position.x))
            .then_with(|| a.id.cmp(&b.id))
    });
    nodes
}

fn node_fully_contained(node: &GraphNode, start: Point<f32>, end: Point<f32>) -> bool {
    let x1 = start.x.min(end.x);
    let x2 = start.x.max(end.x);
    let y1 = start.y.min(end.y);
    let y2 = start.y.max(end.y);
    node.position.x.is_finite()
        && node.position.y.is_finite()
        && node.position.x >= x1
        && node.position.y >= y1
        && node.position.x + NODE_WIDTH <= x2
        && node.position.y + node.height() <= y2
}

fn paint_connections(
    bounds: Bounds<Pixels>,
    transform: GraphTransform,
    graph: &GraphData,
    preview: Option<(String, Point<f32>)>,
    theme: Theme,
    window: &mut Window,
) {
    for connection in &graph.connections {
        let Some((from_node, from_port)) = graph.node_for_port(&connection.from_port_id) else {
            continue;
        };
        let Some((to_node, to_port)) = graph.node_for_port(&connection.to_port_id) else {
            continue;
        };
        if !from_node.position.x.is_finite()
            || !from_node.position.y.is_finite()
            || !to_node.position.x.is_finite()
            || !to_node.position.y.is_finite()
        {
            continue;
        }
        let from = transform.world_to_screen(GraphData::port_position(from_node, from_port));
        let to = transform.world_to_screen(GraphData::port_position(to_node, to_port));
        let start = bounds.origin + point(px(from.x), px(from.y));
        let end = bounds.origin + point(px(to.x), px(to.y));
        let control_distance = px(((to.x - from.x).abs() * 0.5).max(24.0 * transform.scale));
        let mut path = gpui_pre::PathBuilder::stroke(px((EDGE_WIDTH * transform.scale).max(0.75)));
        path.move_to(start);
        path.cubic_bezier_to(
            end,
            start + point(control_distance, px(0.0)),
            end - point(control_distance, px(0.0)),
        );
        if let Ok(path) = path.build() {
            window.paint_path(path, theme.colors.text_muted);
        }
        let direction = if to.x >= from.x { 1.0 } else { -1.0 };
        let tip = end;
        let arrow = [
            tip,
            tip - point(px(8.0 * transform.scale * direction), px(4.0 * transform.scale)),
            tip - point(px(8.0 * transform.scale * direction), px(-4.0 * transform.scale)),
        ];
        let mut arrow_path = gpui_pre::PathBuilder::fill();
        arrow_path.move_to(arrow[0]);
        arrow_path.line_to(arrow[1]);
        arrow_path.line_to(arrow[2]);
        arrow_path.close();
        if let Ok(path) = arrow_path.build() {
            window.paint_path(path, theme.colors.text_muted);
        }
    }
    if let Some((source_id, pointer)) = preview
        && let Some((node, port)) = graph.node_for_port(&source_id)
    {
        let from = transform.world_to_screen(GraphData::port_position(node, port));
        let start = bounds.origin + point(px(from.x), px(from.y));
        let end = bounds.origin + point(px(pointer.x), px(pointer.y));
        let control_distance = px(((pointer.x - from.x).abs() * 0.5).max(24.0 * transform.scale));
        let mut path = gpui_pre::PathBuilder::stroke(px((EDGE_WIDTH * transform.scale).max(0.75)));
        path.move_to(start);
        path.cubic_bezier_to(
            end,
            start + point(control_distance, px(0.0)),
            end - point(control_distance, px(0.0)),
        );
        if let Ok(path) = path.build() {
            window.paint_path(path, theme.colors.accent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Modifiers, MouseButton, TestAppContext};
    use std::{cell::RefCell, rc::Rc};

    fn fixture() -> GraphData {
        let source = GraphNode::new("source", "Source", point(30.0, 40.0))
            .outputs(vec![GraphPort::output("out", "Image", "rgba")]);
        let sink = GraphNode::new("sink", "Sink", point(360.0, 160.0))
            .inputs(vec![GraphPort::input("in", "Texture", "rgba")]);
        GraphData::new(vec![sink, source], vec![GraphConnection::new("wire", "out", "in")])
    }

    #[test]
    fn transform_zoom_preserves_pointer_anchor() {
        let transform = GraphTransform { scale: 1.5, offset_x: -20.0, offset_y: 10.0 };
        let pointer = point(180.0, 95.0);
        let world = transform.screen_to_world(pointer);
        let zoomed = transform.zoomed_at(pointer, 1.2);
        assert_eq!(zoomed.world_to_screen(world), pointer);
    }

    #[test]
    fn fit_handles_empty_and_large_graph_bounds() {
        assert_eq!(
            GraphTransform::default(),
            GraphTransform::fitted(
                GraphBounds { min_x: 0.0, min_y: 0.0, max_x: 0.0, max_y: 0.0 },
                point(800.0, 600.0),
                16.0,
            )
        );
        let fit = GraphTransform::fitted(
            GraphBounds { min_x: -50_000.0, min_y: 10.0, max_x: 50_000.0, max_y: 70_000.0 },
            point(800.0, 600.0),
            16.0,
        );
        assert!(fit.valid());
        assert!((fit.scale - 0.00768).abs() < 0.00001);
    }

    #[test]
    fn invalid_connection_endpoint_has_accessible_fallback_and_no_paint_endpoint() {
        let graph =
            GraphData::new(fixture().nodes, vec![GraphConnection::new("broken", "missing", "in")]);
        let editor = NodeEditor::new("Graph", graph);
        assert!(
            editor
                .connection_summary(&editor.graph.connections[0])
                .contains("Unresolved connection broken")
        );
    }

    #[test]
    fn box_selection_uses_full_containment_with_negative_world_coordinates() {
        let node = GraphNode::new("n", "Node", point(-40.0, -20.0));
        assert!(node_fully_contained(&node, point(-50.0, -30.0), point(250.0, 100.0)));
        assert!(!node_fully_contained(&node, point(-50.0, -30.0), point(100.0, 100.0)));
        assert!(node_fully_contained(&node, point(250.0, 100.0), point(-50.0, -30.0)));
    }

    #[test]
    fn connection_policy_requires_output_input_matching_types_and_new_endpoints() {
        let mut graph = fixture();
        graph.connections.clear();
        let editor = NodeEditor::new("Graph", graph);
        assert!(editor.valid_connection_pair("out", "in"));
        assert!(!editor.valid_connection_pair("in", "out"));
        assert!(!editor.valid_connection_pair("missing", "in"));
        let mut duplicate = editor.graph.clone();
        duplicate.connections.push(GraphConnection::new("existing", "out", "in"));
        assert!(!NodeEditor::new("Graph", duplicate).valid_connection_pair("out", "in"));
    }

    #[gpui_pre::test]
    fn keyboard_connection_creation_and_removal_are_typed_requests(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let mut graph = fixture();
        graph.connections.clear();
        let (editor, visual) = cx.add_window_view(|_, _| NodeEditor::new("Graph", graph));
        let creates = Rc::new(RefCell::new(Vec::<ConnectionCreateRequested>::new()));
        let removes = Rc::new(RefCell::new(Vec::<ConnectionRemoveRequested>::new()));
        let create_log = creates.clone();
        let remove_log = removes.clone();
        let _subscriptions = visual.update(|_, app| {
            let a = app.subscribe(&editor, move |_, e: &ConnectionCreateRequested, _| {
                create_log.borrow_mut().push(e.clone())
            });
            let b = app.subscribe(&editor, move |_, e: &ConnectionRemoveRequested, _| {
                remove_log.borrow_mut().push(e.clone())
            });
            (a, b)
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            editor.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("enter c ] ctrl-enter");
        assert_eq!(
            creates.borrow().as_slice(),
            &[ConnectionCreateRequested { from_port_id: "out".into(), to_port_id: "in".into() }]
        );
        editor.update(visual, |view, cx| {
            view.set_graph(
                GraphData::new(
                    view.graph.nodes.clone(),
                    vec![GraphConnection::new("wire", "out", "in")],
                ),
                cx,
            );
        });
        visual.simulate_keystrokes("g delete");
        assert_eq!(
            removes.borrow().as_slice(),
            &[ConnectionRemoveRequested { connection_id: "wire".into() }]
        );
    }

    #[gpui_pre::test]
    fn pointer_connection_drag_keeps_view_stable_and_proposes_edge(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let mut graph = fixture();
        graph.connections.clear();
        let (editor, visual) = cx.add_window_view(|_, _| NodeEditor::new("Graph", graph));
        let creates = Rc::new(RefCell::new(Vec::<ConnectionCreateRequested>::new()));
        let create_log = creates.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &ConnectionCreateRequested, _| {
                create_log.borrow_mut().push(event.clone());
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let canvas = visual.debug_bounds("mkit-node-editor-canvas").expect("canvas bounds");
        let (start, end, before) = editor.read_with(visual, |view, _| {
            let from_node = view.graph.nodes.iter().find(|node| node.id == "source").unwrap();
            let from_port = from_node.outputs.iter().find(|port| port.id == "out").unwrap();
            let to_node = view.graph.nodes.iter().find(|node| node.id == "sink").unwrap();
            let to_port = to_node.inputs.iter().find(|port| port.id == "in").unwrap();
            let transform = view.transform;
            let from = transform.world_to_screen(GraphData::port_position(from_node, from_port));
            let to = transform.world_to_screen(GraphData::port_position(to_node, to_port));
            (
                canvas.origin + point(px(from.x), px(from.y)),
                canvas.origin + point(px(to.x), px(to.y)),
                transform,
            )
        });
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        assert_eq!(editor.read_with(visual, |view, _| view.transform), before);
        assert!(editor.read_with(visual, |view, _| view.connection_source.is_some()));
        visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        assert_eq!(
            creates.borrow().as_slice(),
            &[ConnectionCreateRequested { from_port_id: "out".into(), to_port_id: "in".into() }]
        );
    }

    #[test]
    fn minimap_mapping_uses_uniform_graph_bounds_with_negative_coordinates() {
        let bounds = GraphBounds { min_x: -100.0, min_y: -50.0, max_x: 500.0, max_y: 250.0 };
        let scale = ((168.0 - 16.0) / bounds.width()).min((104.0 - 16.0) / bounds.height());
        let drawn_w = bounds.width() * scale;
        let drawn_h = bounds.height() * scale;
        assert!(drawn_w <= 152.0 && drawn_h <= 88.0);
        assert!((bounds.min_x + drawn_w / scale - bounds.max_x).abs() < 0.001);
        assert!((bounds.min_y + drawn_h / scale - bounds.max_y).abs() < 0.001);
    }

    #[gpui_pre::test]
    fn keyboard_can_toggle_minimap_and_reports_visibility(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (editor, visual) = cx.add_window_view(|_, _| NodeEditor::new("Graph", fixture()));
        let changes = Rc::new(RefCell::new(Vec::<MinimapVisibilityChanged>::new()));
        let log = changes.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &MinimapVisibilityChanged, _| {
                log.borrow_mut().push(*event)
            })
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            editor.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("m");
        assert!(editor.read_with(visual, |view, _| view.minimap_visible()));
        assert_eq!(changes.borrow().as_slice(), &[MinimapVisibilityChanged { visible: true }]);
    }

    #[gpui_pre::test]
    fn controlled_minimap_visibility_waits_for_owner_setter(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (editor, visual) = cx.add_window_view(|_, _| {
            NodeEditor::new("Graph", fixture()).with_controlled_minimap(false)
        });
        let changes = Rc::new(RefCell::new(Vec::<MinimapVisibilityChanged>::new()));
        let log = changes.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &MinimapVisibilityChanged, _| {
                log.borrow_mut().push(*event)
            })
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            editor.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("m");
        assert!(!editor.read_with(visual, |view, _| view.minimap_visible()));
        assert_eq!(changes.borrow().as_slice(), &[MinimapVisibilityChanged { visible: true }]);
        editor.update(visual, |view, cx| view.set_minimap_visible(true, cx));
        assert!(editor.read_with(visual, |view, _| view.minimap_visible()));
    }

    #[gpui_pre::test]
    fn keyboard_nudge_emits_move_request_without_mutating_authoritative_graph(
        cx: &mut TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (editor, visual) = cx.add_window_view(|_, _| NodeEditor::new("Graph", fixture()));
        let moves = Rc::new(RefCell::new(Vec::<NodesMoveRequested>::new()));
        let log = moves.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &NodesMoveRequested, _| {
                log.borrow_mut().push(event.clone())
            })
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            editor.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("space shift-alt-right");
        assert_eq!(moves.borrow().len(), 1);
        assert_eq!(
            moves.borrow()[0].positions,
            vec![NodePosition { node_id: "source".into(), position: point(31.0, 40.0) }]
        );
        assert_eq!(
            editor.read_with(visual, |view, _| view
                .graph()
                .nodes
                .iter()
                .find(|n| n.id == "source")
                .unwrap()
                .position),
            point(30.0, 40.0)
        );
    }

    #[gpui_pre::test]
    fn pointer_release_uses_final_position_without_move_event(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (editor, visual) = cx.add_window_view(|_, _| NodeEditor::new("Graph", fixture()));
        let moves = Rc::new(RefCell::new(Vec::<NodesMoveRequested>::new()));
        let log = moves.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &NodesMoveRequested, _| {
                log.borrow_mut().push(event.clone())
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let canvas = visual.debug_bounds("mkit-node-editor-canvas").expect("canvas bounds");
        let start = canvas.origin + point(px(80.0), px(60.0));
        let release = start + point(px(24.0), px(18.0));
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_up(release, MouseButton::Left, Modifiers::default());
        assert_eq!(
            moves.borrow().as_slice(),
            &[NodesMoveRequested {
                positions: vec![NodePosition {
                    node_id: "source".into(),
                    position: point(54.0, 58.0),
                }],
            }]
        );
        assert_eq!(
            editor.read_with(visual, |view, _| view
                .graph()
                .nodes
                .iter()
                .find(|n| n.id == "source")
                .unwrap()
                .position),
            point(30.0, 40.0)
        );
    }

    #[gpui_pre::test]
    fn delivered_release_outside_canvas_cancels_move_and_box(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (editor, visual) = cx.add_window_view(|_, _| NodeEditor::new("Graph", fixture()));
        let moves = Rc::new(RefCell::new(Vec::<NodesMoveRequested>::new()));
        let log = moves.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &NodesMoveRequested, _| {
                log.borrow_mut().push(event.clone())
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let canvas = visual.debug_bounds("mkit-node-editor-canvas").expect("canvas bounds");
        let start = canvas.origin + point(px(80.0), px(60.0));
        let preview = start + point(px(24.0), px(18.0));
        let outside = canvas.origin + point(canvas.size.width + px(10.0), px(60.0));
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(preview, Some(MouseButton::Left), Modifiers::default());
        visual.update(|window, cx| {
            editor.update(cx, |view, cx| {
                view.on_mouse_up(
                    &MouseUpEvent {
                        button: MouseButton::Left,
                        position: outside,
                        modifiers: Modifiers::default(),
                        click_count: 1,
                    },
                    window,
                    cx,
                )
            });
        });
        assert!(moves.borrow().is_empty());
        assert!(editor.read_with(visual, |view, _| view.node_drag.is_none()));

        editor.update(visual, |view, cx| view.set_selection(Vec::new(), cx));
        let box_start = canvas.origin + point(px(10.0), px(10.0));
        let box_preview = canvas.origin + point(px(330.0), px(300.0));
        visual.simulate_mouse_down(box_start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(box_preview, Some(MouseButton::Left), Modifiers::default());
        assert!(editor.read_with(visual, |view, _| view.box_drag.is_some()));
        visual.update(|window, cx| {
            editor.update(cx, |view, cx| {
                view.on_mouse_up(
                    &MouseUpEvent {
                        button: MouseButton::Left,
                        position: outside,
                        modifiers: Modifiers::default(),
                        click_count: 1,
                    },
                    window,
                    cx,
                )
            });
        });
        assert!(editor.read_with(visual, |view, _| view.box_drag.is_none()));
        assert!(editor.read_with(visual, |view, _| view.selection().is_empty()));
    }

    #[gpui_pre::test]
    fn keyboard_navigation_selection_ports_and_view_commands(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (editor, visual) = cx.add_window_view(|_, _| NodeEditor::new("Graph", fixture()));
        let views = Rc::new(RefCell::new(Vec::new()));
        let selections = Rc::new(RefCell::new(Vec::new()));
        let focuses = Rc::new(RefCell::new(Vec::new()));
        let v = views.clone();
        let s = selections.clone();
        let f = focuses.clone();
        let _subscriptions = visual.update(|_, app| {
            let view_subscription = app.subscribe(&editor, move |_, event: &ViewChanged, _| {
                v.borrow_mut().push(event.transform)
            });
            let selection_subscription = app
                .subscribe(&editor, move |_, event: &SelectionChanged, _| {
                    s.borrow_mut().push(event.node_ids.clone())
                });
            let focus_subscription = app.subscribe(&editor, move |_, event: &FocusChanged, _| {
                f.borrow_mut().push(event.clone())
            });
            (view_subscription, selection_subscription, focus_subscription)
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            editor.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("down space enter down escape alt-left = f 1");
        assert_eq!(
            editor.read_with(visual, |view, _| view.focused_node().map(str::to_owned)),
            Some("sink".into())
        );
        assert_eq!(editor.read_with(visual, |view, _| view.selection().to_vec()), vec!["sink"]);
        assert!(!views.borrow().is_empty());
        assert_eq!(selections.borrow().as_slice(), &[vec!["sink".to_string()]]);
        assert!(focuses.borrow().iter().any(|event| event.port_id.as_deref() == Some("in")));
    }

    #[gpui_pre::test]
    fn controlled_selection_and_view_wait_for_owner_setters(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (editor, visual) = cx.add_window_view(|_, _| {
            NodeEditor::controlled("Graph", fixture(), GraphTransform::default(), Vec::new())
        });
        let view_requests = Rc::new(RefCell::new(Vec::new()));
        let selection_requests = Rc::new(RefCell::new(Vec::new()));
        let view_log = view_requests.clone();
        let selection_log = selection_requests.clone();
        let _subscriptions = visual.update(|_, app| {
            let view_subscription = app.subscribe(&editor, move |_, event: &ViewChanged, _| {
                view_log.borrow_mut().push(event.transform);
            });
            let selection_subscription =
                app.subscribe(&editor, move |_, event: &SelectionChanged, _| {
                    selection_log.borrow_mut().push(event.node_ids.clone());
                });
            (view_subscription, selection_subscription)
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            editor.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("space alt-left");
        assert!(editor.read_with(visual, |view, _| view.selection().is_empty()));
        assert_eq!(editor.read_with(visual, |view, _| view.transform()), GraphTransform::default());
        assert_eq!(selection_requests.borrow().as_slice(), &[vec!["source".to_string()]]);
        assert_eq!(view_requests.borrow().len(), 1);
        editor.update(visual, |view, cx| view.set_selection(vec!["sink".into()], cx));
        assert_eq!(editor.read_with(visual, |view, _| view.selection().to_vec()), vec!["sink"]);
    }
}
