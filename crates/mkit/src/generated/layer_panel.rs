//! A compact, keyboard-navigable layer tree for editor applications.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Pixels, Point,
    Render, Window, actions, div, img, prelude::*, px,
};
use mkit_core::theme::Theme;

pub const KEY_CONTEXT: &str = "MkitLayerPanel";
actions!(
    layer_panel,
    [Next, Previous, Expand, Collapse, ToggleVisibility, ToggleLock, MoveUp, MoveDown]
);

pub fn default_key_bindings() -> [KeyBinding; 8] {
    [
        KeyBinding::new("down", Next, Some(KEY_CONTEXT)),
        KeyBinding::new("up", Previous, Some(KEY_CONTEXT)),
        KeyBinding::new("right", Expand, Some(KEY_CONTEXT)),
        KeyBinding::new("left", Collapse, Some(KEY_CONTEXT)),
        KeyBinding::new("space", ToggleVisibility, Some(KEY_CONTEXT)),
        KeyBinding::new("l", ToggleLock, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-up", MoveUp, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-down", MoveDown, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Debug, PartialEq)]
pub enum LayerKind {
    Layer { visible: bool, locked: bool, thumbnail: Option<String>, swatch: Option<gpui_pre::Rgba> },
    Group { expanded: bool, children: Vec<LayerNode> },
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerNode {
    pub id: String,
    pub label: String,
    pub kind: LayerKind,
}

impl LayerNode {
    pub fn layer(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: LayerKind::Layer { visible: true, locked: false, thumbnail: None, swatch: None },
        }
    }
    /// Use a GPUI image resource path, URI, or embedded asset name as the layer thumbnail.
    pub fn with_thumbnail(mut self, source: impl Into<String>) -> Self {
        if let LayerKind::Layer { thumbnail, .. } = &mut self.kind {
            *thumbnail = Some(source.into());
        }
        self
    }
    /// Use a solid swatch when a raster image is unavailable.
    pub fn with_swatch(mut self, color: gpui_pre::Rgba) -> Self {
        if let LayerKind::Layer { swatch, .. } = &mut self.kind {
            *swatch = Some(color);
        }
        self
    }
    pub fn group(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<LayerNode>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: LayerKind::Group { expanded: true, children },
        }
    }
    pub fn is_group(&self) -> bool {
        matches!(self.kind, LayerKind::Group { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveChanged(pub Option<String>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibilityChanged {
    pub id: String,
    pub visible: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockChanged {
    pub id: String,
    pub locked: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupExpandedChanged {
    pub id: String,
    pub expanded: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReorderRequested {
    pub id: String,
    pub parent_id: Option<String>,
    pub index: usize,
}
impl EventEmitter<ActiveChanged> for LayerPanel {}
impl EventEmitter<VisibilityChanged> for LayerPanel {}
impl EventEmitter<LockChanged> for LayerPanel {}
impl EventEmitter<GroupExpandedChanged> for LayerPanel {}
impl EventEmitter<ReorderRequested> for LayerPanel {}

pub struct LayerPanel {
    label: String,
    layers: Vec<LayerNode>,
    active: Option<String>,
    controlled: bool,
    focus: Option<FocusHandle>,
}

#[derive(Clone, Debug)]
struct LayerDrag {
    id: String,
    parent_id: Option<String>,
    label: String,
}

struct LayerDragPreview {
    label: String,
    position: Point<Pixels>,
}

impl Render for LayerDragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let width = px(180.0);
        let height = px(theme.controls.small);
        div()
            .absolute()
            .left(self.position.x - width / 2.0)
            .top(self.position.y - height / 2.0)
            .w(width)
            .h(height)
            .px(px(theme.spacing.small))
            .flex()
            .items_center()
            .rounded(px(theme.radii.small))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.focus)
            .bg(theme.colors.elevated_surface)
            .text_color(theme.colors.text)
            .shadow_sm()
            .child(self.label.clone())
    }
}

impl LayerPanel {
    pub fn new(label: impl Into<String>, layers: Vec<LayerNode>) -> Self {
        let active = layers.first().map(|n| n.id.clone());
        Self { label: label.into(), layers, active, controlled: false, focus: None }
    }
    pub fn controlled(
        label: impl Into<String>,
        layers: Vec<LayerNode>,
        active: Option<String>,
    ) -> Self {
        Self { label: label.into(), layers, active, controlled: true, focus: None }
    }
    pub fn layers(&self) -> &[LayerNode] {
        &self.layers
    }
    pub fn active(&self) -> Option<&str> {
        self.active.as_deref()
    }
    pub fn set_layers(&mut self, layers: Vec<LayerNode>, cx: &mut Context<Self>) {
        self.layers = layers;
        cx.notify();
    }
    pub fn set_active(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        self.active = id;
        cx.notify();
    }
    pub fn set_expanded(&mut self, id: &str, expanded: bool, cx: &mut Context<Self>) {
        if let Some(node) = find_mut(&mut self.layers, id)
            && let LayerKind::Group { expanded: state, .. } = &mut node.kind
        {
            *state = expanded;
            cx.notify();
        }
    }
    fn visible_rows(&self) -> Vec<(String, String, usize, bool)> {
        let mut result = Vec::new();
        flatten(&self.layers, 1, &mut result);
        result
    }
    fn select_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some((id, ..)) = self.visible_rows().get(index) {
            self.select(id.clone(), cx);
        }
    }
    fn select(&mut self, id: String, cx: &mut Context<Self>) {
        if self.active.as_deref() == Some(&id) {
            return;
        }
        if !self.controlled {
            self.active = Some(id.clone());
        }
        cx.emit(ActiveChanged(Some(id)));
        cx.notify();
    }
    fn step(&mut self, step: isize, cx: &mut Context<Self>) {
        let rows = self.visible_rows();
        if rows.is_empty() {
            return;
        }
        let idx =
            self.active.as_ref().and_then(|a| rows.iter().position(|r| &r.0 == a)).unwrap_or(0);
        self.select_index((idx as isize + step).clamp(0, rows.len() as isize - 1) as usize, cx);
    }
    fn toggle_visibility_for(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(node) = find_mut(&mut self.layers, &id) else { return };
        if let LayerKind::Layer { visible, .. } = &mut node.kind {
            let next = !*visible;
            if !self.controlled {
                *visible = next;
            }
            cx.emit(VisibilityChanged { id, visible: next });
            cx.notify();
        }
    }
    fn toggle_lock_for(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(node) = find_mut(&mut self.layers, &id) else { return };
        if let LayerKind::Layer { locked, .. } = &mut node.kind {
            let next = !*locked;
            if !self.controlled {
                *locked = next;
            }
            cx.emit(LockChanged { id, locked: next });
            cx.notify();
        }
    }
    fn group_for(&mut self, id: String, expand: bool, cx: &mut Context<Self>) {
        let Some(node) = find_mut(&mut self.layers, &id) else { return };
        if let LayerKind::Group { expanded, .. } = &mut node.kind
            && *expanded != expand
        {
            if !self.controlled {
                *expanded = expand;
            }
            cx.emit(GroupExpandedChanged { id, expanded: expand });
            cx.notify();
        }
    }
    fn reorder(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some(id) = self.active.clone() else { return };
        self.reorder_id(id, delta, cx);
    }
    fn reorder_id(&mut self, id: String, delta: isize, cx: &mut Context<Self>) {
        let Some((parent, index, len)) = locate(&self.layers, &id, None) else { return };
        let target = index as isize + delta;
        if target < 0 || target >= len as isize {
            return;
        }
        let event = ReorderRequested { id: id.clone(), parent_id: parent, index: target as usize };
        if !self.controlled
            && let Some(siblings) = siblings_mut(&mut self.layers, event.parent_id.as_deref())
        {
            siblings.swap(index, target as usize);
        }
        cx.emit(event);
        cx.notify();
    }
    fn drop_before(&mut self, dragged: &LayerDrag, target_id: &str, cx: &mut Context<Self>) {
        let Some(event) = reorder_before(&self.layers, &dragged.id, target_id) else { return };
        if !self.controlled
            && let Some(siblings) = siblings_mut(&mut self.layers, event.parent_id.as_deref())
        {
            let source_index = siblings.iter().position(|node| node.id == dragged.id);
            if let Some(source_index) = source_index {
                let node = siblings.remove(source_index);
                siblings.insert(event.index.min(siblings.len()), node);
            }
        }
        cx.emit(event);
        cx.notify();
    }
    fn on_next(&mut self, _: &Next, _: &mut Window, cx: &mut Context<Self>) {
        self.step(1, cx);
    }
    fn on_previous(&mut self, _: &Previous, _: &mut Window, cx: &mut Context<Self>) {
        self.step(-1, cx);
    }
    fn on_expand(&mut self, _: &Expand, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.active.clone() {
            self.group_for(id, true, cx);
        }
    }
    fn on_collapse(&mut self, _: &Collapse, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.active.clone() {
            self.group_for(id, false, cx);
        }
    }
    fn on_visibility(&mut self, _: &ToggleVisibility, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.active.clone() {
            self.toggle_visibility_for(id, cx);
        }
    }
    fn on_lock(&mut self, _: &ToggleLock, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.active.clone() {
            self.toggle_lock_for(id, cx);
        }
    }
    fn on_move_up(&mut self, _: &MoveUp, _: &mut Window, cx: &mut Context<Self>) {
        self.reorder(-1, cx);
    }
    fn on_move_down(&mut self, _: &MoveDown, _: &mut Window, cx: &mut Context<Self>) {
        self.reorder(1, cx);
    }
}

impl Focusable for LayerPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone().expect("focus initialized")
    }
}

impl Render for LayerPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(0)).clone();
        let mut root = div()
            .id("mkit-layer-panel")
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .on_action(cx.listener(Self::on_next))
            .on_action(cx.listener(Self::on_previous))
            .on_action(cx.listener(Self::on_expand))
            .on_action(cx.listener(Self::on_collapse))
            .on_action(cx.listener(Self::on_visibility))
            .on_action(cx.listener(Self::on_lock))
            .on_action(cx.listener(Self::on_move_up))
            .on_action(cx.listener(Self::on_move_down))
            .role(gpui_pre::accesskit::Role::Tree)
            .aria_label(self.label.clone())
            .w_full()
            .flex()
            .flex_col()
            .rounded(px(theme.radii.small))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .bg(theme.colors.surface);
        let rows = self.visible_rows();
        for (id, label, level, group) in rows {
            let active = self.active.as_deref() == Some(&id);
            let node = find(&self.layers, &id).expect("visible nodes exist").clone();
            let expanded = matches!(&node.kind, LayerKind::Group { expanded: true, .. });
            let row_id = format!("layer-row-{id}");
            let drag = LayerDrag {
                id: id.clone(),
                parent_id: locate(&self.layers, &id, None).and_then(|(parent, _, _)| parent),
                label: label.clone(),
            };
            let drop_id = id.clone();
            let drop_parent = drag.parent_id.clone();
            let mut row = div()
                .id(row_id.clone())
                .debug_selector({
                    let s = row_id.clone();
                    move || s.clone()
                })
                .role(gpui_pre::accesskit::Role::TreeItem)
                .aria_label(label.clone())
                .aria_level(level)
                .aria_selected(active)
                .when(group, |el| {
                    el.aria_expanded(matches!(&node.kind, LayerKind::Group { expanded: true, .. }))
                })
                .tab_index(if active { 0 } else { -1 })
                .h(px(theme.controls.small))
                .pl(px(theme.spacing.small * level as f32))
                .pr(px(theme.spacing.small))
                .flex()
                .items_center()
                .gap(px(theme.spacing.small))
                .text_color(theme.colors.text)
                .when(active, |el| {
                    el.bg(theme.colors.elevated_surface)
                        .border_l(px(theme.borders.strong))
                        .border_color(theme.colors.focus)
                })
                .when(!active, |el| el.bg(theme.colors.surface))
                .drag_over::<LayerDrag>(move |style, dragged, _, _| {
                    if dragged.id != drop_id && dragged.parent_id == drop_parent {
                        style
                            .border_l(px(theme.borders.strong))
                            .border_color(theme.colors.accent)
                            .bg(theme.colors.elevated_surface)
                    } else {
                        style
                    }
                })
                .child(if group {
                    if matches!(node.kind, LayerKind::Group { expanded: true, .. }) {
                        "▾"
                    } else {
                        "▸"
                    }
                } else {
                    ""
                });
            if let LayerKind::Layer { visible, locked, thumbnail, swatch } = node.kind {
                let thumb = if let Some(source) = thumbnail {
                    div()
                        .w(px(theme.controls.xsmall))
                        .h(px(theme.controls.xsmall))
                        .rounded(px(theme.radii.small))
                        .overflow_hidden()
                        .child(img(source).w_full().h_full())
                } else {
                    div()
                        .w(px(theme.controls.xsmall))
                        .h(px(theme.controls.xsmall))
                        .rounded(px(theme.radii.small))
                        .bg(swatch.unwrap_or(theme.colors.elevated_surface))
                };
                row = row.child(
                    thumb.border(px(theme.borders.hairline)).border_color(theme.colors.border),
                );
                let visibility_label = if visible { "Hide layer" } else { "Show layer" };
                let lock_label = if locked { "Unlock layer" } else { "Lock layer" };
                let visibility_id = id.clone();
                let lock_id = id.clone();
                row = row
                    .child(div().flex_1().child(label.clone()))
                    .child(
                        div()
                            .id(format!("layer-visible-{id}"))
                            .debug_selector({
                                let s = format!("layer-visible-{id}");
                                move || s.clone()
                            })
                            .role(gpui_pre::accesskit::Role::Button)
                            .aria_label(visibility_label)
                            .aria_toggled(gpui_pre::accesskit::Toggled::from(visible))
                            .px(px(theme.spacing.xsmall))
                            .child(if visible { "◉" } else { "○" })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.select(visibility_id.clone(), cx);
                                this.toggle_visibility_for(visibility_id.clone(), cx);
                            })),
                    )
                    .child(
                        div()
                            .id(format!("layer-lock-{id}"))
                            .debug_selector({
                                let s = format!("layer-lock-{id}");
                                move || s.clone()
                            })
                            .role(gpui_pre::accesskit::Role::Button)
                            .aria_label(lock_label)
                            .aria_toggled(gpui_pre::accesskit::Toggled::from(locked))
                            .px(px(theme.spacing.xsmall))
                            .child(if locked { "◆" } else { "◇" })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.select(lock_id.clone(), cx);
                                this.toggle_lock_for(lock_id.clone(), cx);
                            })),
                    );
            } else {
                row = row.child(div().flex_1().child(label.clone()));
            }
            let up_id = id.clone();
            let down_id = id.clone();
            row = row
                .child(
                    div()
                        .id(format!("layer-move-up-{id}"))
                        .role(gpui_pre::accesskit::Role::Button)
                        .aria_label(format!("Move {label} up"))
                        .px(px(theme.spacing.xsmall))
                        .child("↑")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.reorder_id(up_id.clone(), -1, cx)
                        })),
                )
                .child(
                    div()
                        .id(format!("layer-move-down-{id}"))
                        .role(gpui_pre::accesskit::Role::Button)
                        .aria_label(format!("Move {label} down"))
                        .px(px(theme.spacing.xsmall))
                        .child("↓")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.reorder_id(down_id.clone(), 1, cx)
                        })),
                );
            let click_id = id.clone();
            row = row
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.select(click_id.clone(), cx);
                    if group {
                        this.group_for(click_id.clone(), !expanded, cx);
                    }
                }))
                .on_drag(drag.clone(), |drag, position, _, cx| {
                    cx.new(|_| LayerDragPreview { label: format!("Move {}", drag.label), position })
                })
                .can_drop({
                    let drop_parent = drag.parent_id.clone();
                    let drop_id = drag.id.clone();
                    move |value, _, _| {
                        value.downcast_ref::<LayerDrag>().is_some_and(|dragged| {
                            dragged.id != drop_id && dragged.parent_id == drop_parent
                        })
                    }
                })
                .on_drop(cx.listener(move |this, dragged: &LayerDrag, _, cx| {
                    this.drop_before(dragged, &id, cx)
                }));
            root = root.child(row);
        }
        root
    }
}

fn flatten(nodes: &[LayerNode], level: usize, out: &mut Vec<(String, String, usize, bool)>) {
    for node in nodes {
        let group = node.is_group();
        out.push((node.id.clone(), node.label.clone(), level, group));
        if let LayerKind::Group { expanded: true, children } = &node.kind {
            flatten(children, level + 1, out);
        }
    }
}
fn find<'a>(nodes: &'a [LayerNode], id: &str) -> Option<&'a LayerNode> {
    for n in nodes {
        if n.id == id {
            return Some(n);
        }
        if let LayerKind::Group { children, .. } = &n.kind
            && let Some(found) = find(children, id)
        {
            return Some(found);
        }
    }
    None
}
fn find_mut<'a>(nodes: &'a mut [LayerNode], id: &str) -> Option<&'a mut LayerNode> {
    for n in nodes {
        if n.id == id {
            return Some(n);
        }
        if let LayerKind::Group { children, .. } = &mut n.kind
            && let Some(found) = find_mut(children, id)
        {
            return Some(found);
        }
    }
    None
}
fn siblings_mut<'a>(
    nodes: &'a mut Vec<LayerNode>,
    parent: Option<&str>,
) -> Option<&'a mut Vec<LayerNode>> {
    match parent {
        None => Some(nodes),
        Some(id) => {
            let node = find_mut(nodes, id)?;
            match &mut node.kind {
                LayerKind::Group { children, .. } => Some(children),
                _ => None,
            }
        }
    }
}
fn locate(
    nodes: &[LayerNode],
    id: &str,
    parent: Option<String>,
) -> Option<(Option<String>, usize, usize)> {
    for (i, node) in nodes.iter().enumerate() {
        if node.id == id {
            return Some((parent, i, nodes.len()));
        }
        if let LayerKind::Group { children, .. } = &node.kind
            && let Some(found) = locate(children, id, Some(node.id.clone()))
        {
            return Some(found);
        }
    }
    None
}

fn reorder_before(
    layers: &[LayerNode],
    dragged_id: &str,
    target_id: &str,
) -> Option<ReorderRequested> {
    if dragged_id == target_id {
        return None;
    }
    let (source_parent, source_index, _) = locate(layers, dragged_id, None)?;
    let (target_parent, target_index, _) = locate(layers, target_id, None)?;
    if source_parent != target_parent {
        return None;
    }
    let index = if source_index < target_index { target_index - 1 } else { target_index };
    if index == source_index {
        return None;
    }
    Some(ReorderRequested { id: dragged_id.into(), parent_id: source_parent, index })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Modifiers, TestAppContext};
    fn fixture() -> Vec<LayerNode> {
        vec![
            LayerNode::group(
                "g",
                "Group",
                vec![
                    LayerNode::layer("a", "Alpha").with_swatch(gpui_pre::rgb(0xff0000)),
                    LayerNode::layer("c", "Gamma"),
                ],
            ),
            LayerNode::layer("b", "Beta"),
        ]
    }
    #[test]
    fn flatten_respects_collapsed_groups() {
        let mut layers = fixture();
        if let LayerKind::Group { expanded, .. } = &mut layers[0].kind {
            *expanded = false;
        }
        let mut rows = vec![];
        flatten(&layers, 1, &mut rows);
        assert_eq!(rows.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(), vec!["g", "b"]);
    }
    #[test]
    fn locate_returns_sibling_index_and_parent() {
        assert_eq!(locate(&fixture(), "a", None), Some((Some("g".into()), 0, 2)));
    }
    #[test]
    fn reorder_before_accepts_only_same_parent_targets() {
        let layers = fixture();
        assert_eq!(
            reorder_before(&layers, "c", "a"),
            Some(ReorderRequested { id: "c".into(), parent_id: Some("g".into()), index: 0 })
        );
        assert_eq!(reorder_before(&layers, "a", "b"), None);
        assert_eq!(reorder_before(&layers, "a", "a"), None);
    }
    #[gpui_pre::test]
    fn pointer_toggles_and_keyboard_reorders(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (panel, visual) = cx.add_window_view(|_, _| LayerPanel::new("Layers", fixture()));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| panel.focus_handle(cx).focus(window, cx));
        let visibility = visual.debug_bounds("layer-visible-a").expect("visibility toggle");
        visual.simulate_click(visibility.center(), Modifiers::default());
        assert!(matches!(
            find(&panel.read_with(visual, |view, _| view.layers.clone()), "a").unwrap().kind,
            LayerKind::Layer { visible: false, .. }
        ));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let group = visual.debug_bounds("layer-row-g").expect("group row");
        visual.simulate_click(group.center(), Modifiers::default());
        visual.simulate_keystrokes("alt-down");
        assert_eq!(panel.read_with(visual, |view, _| view.layers[0].id.clone()), "b");
    }

    #[gpui_pre::test]
    fn pointer_drag_reorders_within_nested_siblings(cx: &mut TestAppContext) {
        use gpui_pre::MouseButton;
        cx.update(mkit_core::theme::set_light_theme);
        let (panel, visual) = cx.add_window_view(|_, _| LayerPanel::new("Layers", fixture()));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let start = visual.debug_bounds("layer-row-c").expect("drag source row").center();
        let end = visual.debug_bounds("layer-row-a").expect("same-parent drop row").center();
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        panel.read_with(visual, |panel, _| {
            let LayerKind::Group { children, .. } = &panel.layers[0].kind else { panic!("group") };
            assert_eq!(
                children.iter().map(|node| node.id.as_str()).collect::<Vec<_>>(),
                vec!["c", "a"]
            );
        });
    }

    #[gpui_pre::test]
    fn controlled_pointer_drag_emits_same_parent_request_without_mutating(cx: &mut TestAppContext) {
        use gpui_pre::MouseButton;
        use std::{cell::RefCell, rc::Rc};
        cx.update(mkit_core::theme::set_light_theme);
        let initial = fixture();
        let (panel, visual) =
            cx.add_window_view(|_, _| LayerPanel::controlled("Layers", initial, Some("c".into())));
        let requests = Rc::new(RefCell::new(Vec::new()));
        let requests_log = requests.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&panel, move |_, event: &ReorderRequested, _| {
                requests_log.borrow_mut().push(event.clone());
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let start = visual.debug_bounds("layer-row-c").expect("drag source row").center();
        let end = visual.debug_bounds("layer-row-a").expect("same-parent drop row").center();
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        assert_eq!(
            requests.borrow().as_slice(),
            &[ReorderRequested { id: "c".into(), parent_id: Some("g".into()), index: 0 }]
        );
        panel.read_with(visual, |panel, _| {
            let LayerKind::Group { children, .. } = &panel.layers[0].kind else { panic!("group") };
            assert_eq!(
                children.iter().map(|node| node.id.as_str()).collect::<Vec<_>>(),
                vec!["a", "c"]
            );
        });
    }

    #[gpui_pre::test]
    fn pointer_drag_to_another_parent_is_a_no_op(cx: &mut TestAppContext) {
        use gpui_pre::MouseButton;
        use std::{cell::RefCell, rc::Rc};
        cx.update(mkit_core::theme::set_light_theme);
        let (panel, visual) = cx.add_window_view(|_, _| LayerPanel::new("Layers", fixture()));
        let requests = Rc::new(RefCell::new(Vec::<ReorderRequested>::new()));
        let log = requests.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&panel, move |_, event: &ReorderRequested, _| {
                log.borrow_mut().push(event.clone());
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let start = visual.debug_bounds("layer-row-a").expect("nested source").center();
        let end = visual.debug_bounds("layer-row-b").expect("root target").center();
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        assert!(requests.borrow().is_empty());
        panel.read_with(visual, |panel, _| {
            assert_eq!(panel.layers[1].id, "b");
            let LayerKind::Group { children, .. } = &panel.layers[0].kind else { panic!("group") };
            assert_eq!(
                children.iter().map(|node| node.id.as_str()).collect::<Vec<_>>(),
                vec!["a", "c"]
            );
        });
    }
}
