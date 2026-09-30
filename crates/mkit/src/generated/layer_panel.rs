//! A compact, keyboard-navigable layer tree for editor applications.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, PathBuilder,
    Pixels, Point, Render, Rgba, Window, actions, canvas, div, img, point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};

/// Colours derived from theme tokens; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    card: Rgba,
    card_border: Rgba,
    active_bg: Rgba,
    active_text: Rgba,
    /// Pointer-hover fill for rows; `None` outlines the row instead (high contrast).
    hover_bg: Option<Rgba>,
    hover_border: Rgba,
    /// Weight of the shadcn "muted" mix, reused for ghost button hovers over a row fill.
    muted_weight: Option<f32>,
    icon: Rgba,
    quiet_icon: Rgba,
    swatch: Rgba,
    drop: Rgba,
    focus: Rgba,
    popover: Rgba,
    popover_shadow: ShadowToken,
    high_contrast: bool,
}

/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}

fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            card: c.surface,
            card_border: c.border,
            active_bg: c.accent,
            active_text: c.accent_text,
            hover_bg: None,
            hover_border: c.border,
            muted_weight: None,
            icon: c.text,
            quiet_icon: c.text,
            swatch: c.background,
            drop: c.accent,
            focus: c.focus,
            popover: c.background,
            popover_shadow: t.shadows.none,
            high_contrast: true,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let weight = if dark { 0.12 } else { 0.04 };
    // shadcn "accent"/"muted": text mixed into the background.
    let muted = mix(c.text, c.background, weight);
    let card_border = if dark { mix(c.text, c.surface, 0.1) } else { c.border };
    Look {
        card: c.surface,
        card_border,
        active_bg: muted,
        active_text: c.text,
        hover_bg: Some(muted),
        hover_border: c.background.opacity(0.),
        muted_weight: Some(weight),
        icon: c.text,
        quiet_icon: c.text_muted,
        swatch: muted,
        drop: c.accent,
        focus: c.focus,
        popover: c.surface,
        popover_shadow: t.shadows.medium,
        high_contrast: false,
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

/// One step of a Lucide icon outline on a 24-unit grid.
#[derive(Clone, Copy)]
enum Step {
    Move(f32, f32),
    Line(f32, f32),
    /// SVG elliptical arc with equal radii: radius, large-arc flag, sweep flag, end point.
    Arc(f32, bool, bool, f32, f32),
    Close,
}
use Step::{Arc, Close, Line, Move};

/// Decorative Lucide icon drawn as a vector stroke so it stays crisp at every scale; the stroke
/// is 2 units on the 24-unit grid, Lucide's default.
fn icon(size: f32, steps: &'static [Step], color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let at = |x: f32, y: f32| bounds.origin + point(unit * x, unit * y);
            let mut path = PathBuilder::stroke(unit * 2.0);
            for step in steps {
                match *step {
                    Move(x, y) => path.move_to(at(x, y)),
                    Line(x, y) => path.line_to(at(x, y)),
                    Arc(r, large, sweep, x, y) => {
                        path.arc_to(point(unit * r, unit * r), px(0.), large, sweep, at(x, y))
                    }
                    Close => path.close(),
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
}
/// Lucide `chevron-right`.
const CHEVRON_RIGHT: &[Step] = &[Move(9., 6.), Line(15., 12.), Line(9., 18.)];
/// Lucide `chevron-down`.
const CHEVRON_DOWN: &[Step] = &[Move(6., 9.), Line(12., 15.), Line(18., 9.)];
/// Lucide `eye`: the lens outline and the pupil.
const EYE: &[Step] = &[
    Move(2., 12.),
    Arc(10.75, false, true, 22., 12.),
    Arc(10.75, false, true, 2., 12.),
    Close,
    Move(15., 12.),
    Arc(3., true, true, 9., 12.),
    Arc(3., true, true, 15., 12.),
    Close,
];
/// Lucide `eye-off`: the broken lens, the pupil arc, and the slash.
const EYE_OFF: &[Step] = &[
    Move(10.73, 5.08),
    Arc(10.75, false, true, 20.49, 14.84),
    Move(14.08, 14.16),
    Arc(3., false, true, 9.84, 9.92),
    Move(17.48, 17.5),
    Arc(10.75, false, true, 2., 12.),
    Arc(10.75, false, true, 6.51, 6.51),
    Move(2., 2.),
    Line(22., 22.),
];
/// Lucide `lock`: the body and a closed shackle.
const LOCK: &[Step] = &[
    Move(5., 11.),
    Line(19., 11.),
    Arc(2., false, true, 21., 13.),
    Line(21., 20.),
    Arc(2., false, true, 19., 22.),
    Line(5., 22.),
    Arc(2., false, true, 3., 20.),
    Line(3., 13.),
    Arc(2., false, true, 5., 11.),
    Close,
    Move(7., 11.),
    Line(7., 7.),
    Arc(5., false, true, 17., 7.),
    Line(17., 11.),
];
/// Lucide `lock-open`: the body and an open shackle.
const LOCK_OPEN: &[Step] = &[
    Move(5., 11.),
    Line(19., 11.),
    Arc(2., false, true, 21., 13.),
    Line(21., 20.),
    Arc(2., false, true, 19., 22.),
    Line(5., 22.),
    Arc(2., false, true, 3., 20.),
    Line(3., 13.),
    Arc(2., false, true, 5., 11.),
    Close,
    Move(7., 11.),
    Line(7., 7.),
    Arc(5., false, true, 16.9, 6.),
];
/// Lucide `arrow-up`.
const ARROW_UP: &[Step] = &[Move(5., 12.), Line(12., 5.), Line(19., 12.), Move(12., 19.), Line(12., 5.)];
/// Lucide `arrow-down`.
const ARROW_DOWN: &[Step] =
    &[Move(12., 5.), Line(12., 19.), Move(19., 12.), Line(12., 19.), Line(5., 12.)];

/// Ghost icon button, like the restyled IconButton at the panel's compact size. The pointer
/// target is a `spacing.xlarge` square; the glyph is a `spacing.large` vector icon.
fn ghost_button(
    id: String,
    theme: &Theme,
    look: &Look,
    row_fill: Rgba,
    glyph: &'static [Step],
    color: Rgba,
    toggled: Option<bool>,
) -> gpui_pre::Stateful<gpui_pre::Div> {
    let hover_bg = look.muted_weight.map(|weight| mix(theme.colors.text, row_fill, weight));
    let hover_border = if look.high_contrast && row_fill == look.active_bg {
        look.active_text
    } else {
        theme.colors.accent
    };
    div()
        .id(id)
        .role(gpui_pre::accesskit::Role::Button)
        .when_some(toggled, |el, value| el.aria_toggled(gpui_pre::accesskit::Toggled::from(value)))
        .flex_none()
        .size(px(theme.spacing.xlarge))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme.radii.medium))
        .border(px(theme.borders.regular))
        .border_color(theme.colors.background.opacity(0.))
        .hover(move |style| match hover_bg {
            Some(fill) => style.bg(fill),
            None => style.border_color(hover_border),
        })
        .child(icon(theme.spacing.large, glyph, color))
}

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
        let look = look(&theme);
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
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.hairline))
            .border_color(look.card_border)
            .bg(look.popover)
            .shadow(vec![box_shadow(look.popover_shadow)])
            .text_color(theme.colors.text)
            .text_size(px(theme.typography.body))
            .child(div().truncate().child(self.label.clone()))
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let look = look(&theme);
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(0)).clone();
        // `:focus-visible`: the active row outline shows for keyboard focus on the panel or row.
        let focused = focus.contains_focused(window, cx) && window.last_input_was_keyboard();
        let hairline = px(theme.borders.hairline);
        let transparent = look.card.opacity(0.);
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
            .p(px(theme.spacing.xsmall))
            .rounded(px(theme.radii.large))
            .border(hairline)
            .border_color(look.card_border)
            .bg(look.card);
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
            let row_fill = if active { look.active_bg } else { look.card };
            // High contrast draws glyphs on the solid active fill in `accent_text`, as Tree does.
            let on_row =
                |color: Rgba| if active && look.high_contrast { look.active_text } else { color };
            let (hover_bg, hover_border) = (look.hover_bg, look.hover_border);
            let drop_border = if active && look.high_contrast { look.active_text } else { look.drop };
            let drop_fill = look.hover_bg;
            let chevron_color = on_row(if look.high_contrast { look.icon } else { look.quiet_icon });
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
                .flex_none()
                .h(px(theme.controls.small))
                // One `spacing.large` step per nesting level after the row's own padding.
                .pl(px(theme.spacing.small + theme.spacing.large * (level as f32 - 1.0)))
                .pr(px(theme.spacing.xsmall))
                .flex()
                .items_center()
                .gap(px(theme.spacing.small))
                .rounded(px(theme.radii.small))
                .border(hairline)
                .border_color(if focused && active { look.focus } else { transparent })
                .text_size(px(theme.typography.body))
                .text_color(on_row(theme.colors.text))
                .when(active, |el| el.bg(look.active_bg))
                .when(!active, |el| {
                    el.hover(move |style| match hover_bg {
                        Some(fill) => style.bg(fill),
                        None => style.border_color(hover_border),
                    })
                })
                .drag_over::<LayerDrag>(move |style, dragged, _, _| {
                    if dragged.id != drop_id && dragged.parent_id == drop_parent {
                        let style = style.border_color(drop_border);
                        match drop_fill {
                            Some(fill) if !active => style.bg(fill),
                            _ => style,
                        }
                    } else {
                        style
                    }
                })
                // The disclosure slot keeps labels aligned; only groups draw a chevron.
                .child(
                    div()
                        .flex_none()
                        .size(px(theme.spacing.large))
                        .when(group, |slot| {
                            slot.child(icon(
                                theme.spacing.large,
                                if expanded { CHEVRON_DOWN } else { CHEVRON_RIGHT },
                                chevron_color,
                            ))
                        }),
                );
            if let LayerKind::Layer { visible, locked, thumbnail, swatch } = node.kind {
                let tile = div()
                    .flex_none()
                    .size(px(theme.spacing.xlarge))
                    .rounded(px(theme.radii.small))
                    .border(hairline)
                    .border_color(on_row(look.card_border))
                    .overflow_hidden();
                let thumb = if let Some(source) = thumbnail {
                    tile.child(img(source).w_full().h_full())
                } else {
                    tile.bg(swatch.unwrap_or(look.swatch))
                };
                row = row.child(thumb);
                let visibility_label = if visible { "Hide layer" } else { "Show layer" };
                let lock_label = if locked { "Unlock layer" } else { "Lock layer" };
                let visibility_id = id.clone();
                let lock_id = id.clone();
                // The default state (visible, unlocked) is quieter than the exceptional one.
                let quiet = on_row(look.quiet_icon);
                let strong = on_row(look.icon);
                row = row
                    .child(div().flex_1().min_w(px(0.)).truncate().child(label.clone()))
                    .child(
                        ghost_button(
                            format!("layer-visible-{id}"),
                            &theme,
                            &look,
                            row_fill,
                            if visible { EYE } else { EYE_OFF },
                            if visible { quiet } else { strong },
                            Some(visible),
                        )
                        .aria_label(visibility_label)
                        .debug_selector({
                            let s = format!("layer-visible-{id}");
                            move || s.clone()
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.select(visibility_id.clone(), cx);
                            this.toggle_visibility_for(visibility_id.clone(), cx);
                        })),
                    )
                    .child(
                        ghost_button(
                            format!("layer-lock-{id}"),
                            &theme,
                            &look,
                            row_fill,
                            if locked { LOCK } else { LOCK_OPEN },
                            if locked { strong } else { quiet },
                            Some(locked),
                        )
                        .aria_label(lock_label)
                        .debug_selector({
                            let s = format!("layer-lock-{id}");
                            move || s.clone()
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.select(lock_id.clone(), cx);
                            this.toggle_lock_for(lock_id.clone(), cx);
                        })),
                    );
            } else {
                row = row.child(div().flex_1().min_w(px(0.)).truncate().child(label.clone()));
            }
            let up_id = id.clone();
            let down_id = id.clone();
            let arrow_color = on_row(look.icon);
            row = row
                .child(
                    ghost_button(
                        format!("layer-move-up-{id}"),
                        &theme,
                        &look,
                        row_fill,
                        ARROW_UP,
                        arrow_color,
                        None,
                    )
                    .aria_label(format!("Move {label} up"))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.reorder_id(up_id.clone(), -1, cx)
                    })),
                )
                .child(
                    ghost_button(
                        format!("layer-move-down-{id}"),
                        &theme,
                        &look,
                        row_fill,
                        ARROW_DOWN,
                        arrow_color,
                        None,
                    )
                    .aria_label(format!("Move {label} down"))
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
