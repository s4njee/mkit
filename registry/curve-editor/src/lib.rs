//! Stateful normalized tone-curve editor with channel selection and point editing.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Bounds, Context, DispatchPhase, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyBinding, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PathBuilder, Pixels,
    Render, SharedString, Subscription, Window, actions, canvas, div, point, prelude::*, px,
    relative,
};
use mkit_core::theme::Theme;

pub const KEY_CONTEXT: &str = "CurveEditor";
pub const MAX_POINTS: usize = 64;
const MIN_X_GAP: f64 = 0.001;

actions!(
    curve_editor,
    [
        NudgeLeft,
        NudgeRight,
        NudgeUp,
        NudgeDown,
        FineLeft,
        FineRight,
        FineUp,
        FineDown,
        DeletePoint,
        NextChannel,
        PreviousChannel,
        LinearMode,
        SmoothMode
    ]
);

pub fn default_key_bindings() -> [KeyBinding; 13] {
    [
        KeyBinding::new("left", NudgeLeft, Some(KEY_CONTEXT)),
        KeyBinding::new("right", NudgeRight, Some(KEY_CONTEXT)),
        KeyBinding::new("up", NudgeUp, Some(KEY_CONTEXT)),
        KeyBinding::new("down", NudgeDown, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-left", FineLeft, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-right", FineRight, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-up", FineUp, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-down", FineDown, Some(KEY_CONTEXT)),
        KeyBinding::new("delete", DeletePoint, Some(KEY_CONTEXT)),
        KeyBinding::new("tab", NextChannel, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-tab", PreviousChannel, Some(KEY_CONTEXT)),
        KeyBinding::new("l", LinearMode, Some(KEY_CONTEXT)),
        KeyBinding::new("s", SmoothMode, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Interpolation {
    #[default]
    Linear,
    Smooth,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CurvePoint {
    pub x: f64,
    pub y: f64,
}

impl CurvePoint {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurveChannel {
    pub name: String,
    pub points: Vec<CurvePoint>,
}

impl CurveChannel {
    pub fn new(name: impl Into<String>, points: Vec<CurvePoint>) -> Self {
        Self { name: name.into(), points: normalize_points(points) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurveChanged {
    pub channel: usize,
    pub points: Vec<CurvePoint>,
}
impl EventEmitter<CurveChanged> for CurveEditor {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelChanged(pub usize);
impl EventEmitter<ChannelChanged> for CurveEditor {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InterpolationChanged(pub Interpolation);
impl EventEmitter<InterpolationChanged> for CurveEditor {}

pub struct CurveEditor {
    label: String,
    channels: Vec<CurveChannel>,
    active_channel: usize,
    interpolation: Interpolation,
    controlled: bool,
    disabled: bool,
    selected_point: Option<usize>,
    dragging: bool,
    bounds: Option<Bounds<Pixels>>,
    focus: Option<FocusHandle>,
    focus_subscription: Option<Subscription>,
}

impl CurveEditor {
    pub fn new(label: impl Into<String>, channels: Vec<CurveChannel>) -> Self {
        let mut channels = channels;
        if channels.is_empty() {
            channels.push(CurveChannel::new("Master", Vec::new()));
        }
        for channel in &mut channels {
            channel.points = normalize_points(std::mem::take(&mut channel.points));
        }
        Self {
            label: label.into(),
            channels,
            active_channel: 0,
            interpolation: Interpolation::Linear,
            controlled: false,
            disabled: false,
            selected_point: None,
            dragging: false,
            bounds: None,
            focus: None,
            focus_subscription: None,
        }
    }

    pub fn controlled(label: impl Into<String>, channels: Vec<CurveChannel>) -> Self {
        let mut editor = Self::new(label, channels);
        editor.controlled = true;
        editor
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn interpolation(mut self, mode: Interpolation) -> Self {
        self.interpolation = mode;
        self
    }
    pub fn active_channel(&self) -> usize {
        self.active_channel
    }
    pub fn interpolation_mode(&self) -> Interpolation {
        self.interpolation
    }
    pub fn channels(&self) -> &[CurveChannel] {
        &self.channels
    }
    pub fn selected_point(&self) -> Option<usize> {
        self.selected_point
    }
    pub fn points(&self) -> &[CurvePoint] {
        &self.channels[self.active_channel].points
    }

    pub fn set_channel_points(
        &mut self,
        channel: usize,
        points: Vec<CurvePoint>,
        cx: &mut Context<Self>,
    ) {
        if let Some(target) = self.channels.get_mut(channel) {
            target.points = normalize_points(points);
            self.selected_point =
                self.selected_point.map(|i| i.min(target.points.len().saturating_sub(1)));
            cx.notify();
        }
    }
    pub fn select_channel(&mut self, channel: usize, cx: &mut Context<Self>) {
        if channel >= self.channels.len() || channel == self.active_channel {
            return;
        }
        self.active_channel = channel;
        self.selected_point = None;
        cx.notify();
        cx.emit(ChannelChanged(channel));
    }
    pub fn set_interpolation(&mut self, mode: Interpolation, cx: &mut Context<Self>) {
        if self.interpolation == mode {
            return;
        }
        self.interpolation = mode;
        cx.notify();
        cx.emit(InterpolationChanged(mode));
    }

    fn request_points(&mut self, points: Vec<CurvePoint>, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let points = normalize_points(points);
        if self.channels[self.active_channel].points == points {
            return;
        }
        if !self.controlled {
            self.channels[self.active_channel].points = points.clone();
            cx.notify();
        }
        cx.emit(CurveChanged { channel: self.active_channel, points });
    }
    fn set_mode(&mut self, mode: Interpolation, cx: &mut Context<Self>) {
        if self.disabled || self.interpolation == mode {
            return;
        }
        self.set_interpolation(mode, cx);
    }
    fn nudge(&mut self, dx: f64, dy: f64, cx: &mut Context<Self>) {
        let Some(index) = self.selected_point else { return };
        if self.disabled || index == 0 || index + 1 >= self.points().len() {
            return;
        }
        let mut points = self.points().to_vec();
        points[index].x = (points[index].x + dx)
            .clamp(points[index - 1].x + MIN_X_GAP, points[index + 1].x - MIN_X_GAP);
        points[index].y = (points[index].y + dy).clamp(0.0, 1.0);
        self.request_points(points, cx);
    }
    fn delete_selected(&mut self, _: &DeletePoint, _: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.selected_point else { return };
        if self.disabled || index == 0 || index + 1 >= self.points().len() {
            return;
        }
        let mut points = self.points().to_vec();
        points.remove(index);
        self.selected_point = None;
        self.request_points(points, cx);
    }
    fn on_nudge_left(&mut self, _: &NudgeLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge(-0.01, 0.0, cx);
    }
    fn on_nudge_right(&mut self, _: &NudgeRight, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge(0.01, 0.0, cx);
    }
    fn on_nudge_up(&mut self, _: &NudgeUp, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge(0.0, 0.01, cx);
    }
    fn on_nudge_down(&mut self, _: &NudgeDown, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge(0.0, -0.01, cx);
    }
    fn on_fine_left(&mut self, _: &FineLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge(-0.001, 0.0, cx);
    }
    fn on_fine_right(&mut self, _: &FineRight, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge(0.001, 0.0, cx);
    }
    fn on_fine_up(&mut self, _: &FineUp, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge(0.0, 0.001, cx);
    }
    fn on_fine_down(&mut self, _: &FineDown, _: &mut Window, cx: &mut Context<Self>) {
        self.nudge(0.0, -0.001, cx);
    }
    fn on_next_channel(&mut self, _: &NextChannel, _: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled {
            self.select_channel((self.active_channel + 1) % self.channels.len(), cx);
        }
    }
    fn on_previous_channel(&mut self, _: &PreviousChannel, _: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled {
            self.select_channel(
                (self.active_channel + self.channels.len() - 1) % self.channels.len(),
                cx,
            );
        }
    }
    fn on_linear(&mut self, _: &LinearMode, _: &mut Window, cx: &mut Context<Self>) {
        self.set_mode(Interpolation::Linear, cx);
    }
    fn on_smooth(&mut self, _: &SmoothMode, _: &mut Window, cx: &mut Context<Self>) {
        self.set_mode(Interpolation::Smooth, cx);
    }

    fn pointer_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled
            || (event.button != MouseButton::Left && event.button != MouseButton::Right)
        {
            return;
        }
        let Some(bounds) = self.bounds else { return };
        let local_x = (f32::from(event.position.x - bounds.origin.x) / f32::from(bounds.size.width))
            .clamp(0.0, 1.0) as f64;
        let local_y = 1.0
            - (f32::from(event.position.y - bounds.origin.y) / f32::from(bounds.size.height))
                .clamp(0.0, 1.0) as f64;
        let points = self.points().to_vec();
        let nearest = points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                point_distance(**a, local_x, local_y)
                    .total_cmp(&point_distance(**b, local_x, local_y))
            })
            .map(|(i, p)| (i, point_distance(*p, local_x, local_y)));
        let hit_radius = 0.035_f64;
        if event.button == MouseButton::Right {
            if let Some((index, distance)) = nearest
                && distance <= hit_radius
                && index > 0
                && index + 1 < points.len()
            {
                let mut next = points.clone();
                next.remove(index);
                self.selected_point = None;
                self.request_points(next, cx);
            }
            return;
        }
        if let Some((index, distance)) = nearest
            && distance <= hit_radius
        {
            self.selected_point = Some(index);
            if index > 0 && index + 1 < points.len() {
                self.dragging = true;
            }
        } else if points.len() < MAX_POINTS {
            let mut next = points.clone();
            let index = next.partition_point(|p| p.x < local_x);
            let lower = next[index.saturating_sub(1)].x + MIN_X_GAP;
            let upper = next[index].x - MIN_X_GAP;
            if lower > upper {
                return;
            }
            let x = local_x.clamp(lower, upper);
            next.insert(index, CurvePoint::new(x, local_y));
            self.selected_point = Some(index);
            self.dragging = true;
            self.request_points(next, cx);
        }
        if let Some(focus) = self.focus.as_ref() {
            window.focus(focus, cx);
        }
        cx.notify();
    }
    fn pointer_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if !self.dragging || !event.dragging() {
            return;
        }
        let Some(bounds) = self.bounds else { return };
        let Some(index) = self.selected_point else { return };
        if index == 0 || index + 1 >= self.points().len() {
            return;
        }
        let x = (f32::from(event.position.x - bounds.origin.x) / f32::from(bounds.size.width))
            .clamp(0.0, 1.0) as f64;
        let y = 1.0
            - (f32::from(event.position.y - bounds.origin.y) / f32::from(bounds.size.height))
                .clamp(0.0, 1.0) as f64;
        let mut next = self.points().to_vec();
        next[index].x = x.clamp(next[index - 1].x + MIN_X_GAP, next[index + 1].x - MIN_X_GAP);
        next[index].y = y;
        self.request_points(next, cx);
    }
    fn pointer_up(&mut self, event: &MouseUpEvent) {
        if event.button == MouseButton::Left {
            self.dragging = false;
        }
    }
}

impl Focusable for CurveEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone().expect("curve editor focus initialized during render")
    }
}

impl Render for CurveEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self
            .focus
            .get_or_insert_with(|| {
                let focus = cx.focus_handle().tab_stop(!self.disabled);
                self.focus_subscription = Some(cx.on_focus(&focus, window, |_, _, _| {}));
                focus
            })
            .clone();
        let theme = *cx.global::<Theme>();
        let active = self.active_channel;
        let channel_points = self.points().to_vec();
        let interpolation = self.interpolation;
        let disabled = self.disabled;
        let selected = self.selected_point;
        let entity = cx.entity();
        let mut tabs = div().flex().gap(px(theme.spacing.xsmall));
        for (index, channel) in self.channels.iter().enumerate() {
            let label = channel.name.clone();
            tabs = tabs.child(
                div()
                    .id(SharedString::from(format!("curve-channel-{index}")))
                    .px(px(theme.spacing.small))
                    .py(px(theme.spacing.xsmall))
                    .rounded(px(theme.radii.small))
                    .bg(if index == active {
                        theme.colors.surface
                    } else {
                        theme.colors.background
                    })
                    .text_color(if index == active {
                        theme.colors.text
                    } else {
                        theme.colors.text_muted
                    })
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label(label.clone())
                    .aria_description(if index == active { "Selected" } else { "Not selected" })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(
                            move |this: &mut Self,
                                  _: &MouseDownEvent,
                                  _: &mut Window,
                                  cx: &mut Context<Self>| {
                                if !this.disabled {
                                    this.select_channel(index, cx);
                                }
                            },
                        ),
                    )
                    .child(label),
            );
        }
        let mut mode = div().flex().gap(px(theme.spacing.xsmall));
        for (label, value) in [("Linear", Interpolation::Linear), ("Smooth", Interpolation::Smooth)]
        {
            mode = mode.child(
                div()
                    .id(SharedString::from(format!("curve-mode-{}", label.to_lowercase())))
                    .px(px(theme.spacing.small))
                    .py(px(theme.spacing.xsmall))
                    .rounded(px(theme.radii.small))
                    .bg(if value == interpolation {
                        theme.colors.surface
                    } else {
                        theme.colors.background
                    })
                    .text_color(if value == interpolation {
                        theme.colors.text
                    } else {
                        theme.colors.text_muted
                    })
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label(label)
                    .aria_description(if value == interpolation {
                        "Selected"
                    } else {
                        "Not selected"
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(
                            move |this: &mut Self,
                                  _: &MouseDownEvent,
                                  _: &mut Window,
                                  cx: &mut Context<Self>| {
                                this.set_mode(value, cx)
                            },
                        ),
                    )
                    .child(label),
            );
        }
        let graph_h = theme.controls.large * 6.0;
        let graph_entity = entity.clone();
        let graph = div()
            .id("curve-editor-graph")
            .debug_selector(|| "mkit-curve-editor-graph".into())
            .relative()
            .w_full()
            .h(px(graph_h))
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .bg(theme.colors.background)
            .overflow_hidden()
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(format!("{} curve graph", self.channels[active].name))
            .aria_description(
                "Click to add a point; drag to move; right click or Delete to remove a point. Arrow keys nudge the selected point.",
            )
            .when(disabled, |el| el.a11y_synthetic_children(|b| b.parent_node().set_disabled()));
        let mut graph = graph;
        for (i, p) in channel_points.iter().enumerate() {
            let size = theme.controls.xsmall * 0.5;
            graph = graph.child(
                div()
                    .id(SharedString::from(format!("curve-point-{i}")))
                    .absolute()
                    .left(relative(p.x as f32))
                    .top(relative((1.0 - p.y) as f32))
                    .ml(px(-size / 2.0))
                    .mt(px(-size / 2.0))
                    .size(px(size))
                    .rounded(px(theme.radii.pill))
                    .bg(if disabled { theme.colors.disabled } else { theme.colors.surface })
                    .border(px(if selected == Some(i) {
                        theme.borders.strong
                    } else {
                        theme.borders.hairline
                    }))
                    .border_color(if selected == Some(i) {
                        theme.colors.focus
                    } else {
                        theme.colors.accent
                    })
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label(format!(
                        "{} point {}, input {:.3}, output {:.3}",
                        self.channels[active].name, i, p.x, p.y
                    )),
            );
        }
        graph = graph.child(
            canvas(
                move |_, _, _| (),
                move |bounds, (), window, cx| {
                    graph_entity.update(cx, |editor, _| editor.bounds = Some(bounds));
                    let origin = bounds.origin;
                    let width = bounds.size.width;
                    let height = bounds.size.height;
                    for i in 1..4 {
                        let x = origin.x + width * (i as f32 / 4.0);
                        let y = origin.y + height * (i as f32 / 4.0);
                        let mut vertical = PathBuilder::stroke(px(theme.borders.hairline));
                        vertical.move_to(point(x, origin.y));
                        vertical.line_to(point(x, origin.y + height));
                        if let Ok(path) = vertical.build() {
                            window.paint_path(path, theme.colors.border);
                        }
                        let mut horizontal = PathBuilder::stroke(px(theme.borders.hairline));
                        horizontal.move_to(point(origin.x, y));
                        horizontal.line_to(point(origin.x + width, y));
                        if let Ok(path) = horizontal.build() {
                            window.paint_path(path, theme.colors.border);
                        }
                    }
                    let mut reference = PathBuilder::stroke(px(theme.borders.hairline));
                    reference.move_to(origin + point(px(0.0), height));
                    reference.line_to(origin + point(width, px(0.0)));
                    if let Ok(path) = reference.build() {
                        window.paint_path(path, theme.colors.text_muted);
                    }
                    let mut curve = PathBuilder::stroke(px(theme.borders.strong));
                    let samples = 96;
                    for i in 0..=samples {
                        let t = i as f64 / samples as f64;
                        let y = sample_curve(&channel_points, interpolation, t);
                        let at = point(
                            origin.x + width * t as f32,
                            origin.y + height * (1.0 - y) as f32,
                        );
                        if i == 0 {
                            curve.move_to(at);
                        } else {
                            curve.line_to(at);
                        }
                    }
                    if let Ok(path) = curve.build() {
                        window.paint_path(
                            path,
                            if disabled { theme.colors.disabled } else { theme.colors.accent },
                        );
                    }
                    let down = graph_entity.clone();
                    window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                        if phase == DispatchPhase::Capture && bounds.contains(&event.position) {
                            down.update(cx, |editor, cx| {
                                editor.bounds = Some(bounds);
                                editor.pointer_down(event, window, cx);
                            });
                        }
                    });
                    let moved = graph_entity.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture {
                            moved.update(cx, |editor, cx| editor.pointer_move(event, cx));
                        }
                    });
                    let up = graph_entity.clone();
                    window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture {
                            up.update(cx, |editor, _| editor.pointer_up(event));
                        }
                    });
                },
            )
            .absolute()
            .inset_0(),
        );
        div()
            .id("curve-editor")
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.small))
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.label.clone())
            .aria_description("Select a channel, then use arrow keys to adjust the selected point. Press Delete to remove an interior point.")
            .when(disabled, |el| el.aria_description("Unavailable"))
            .on_action(cx.listener(Self::on_nudge_left))
            .on_action(cx.listener(Self::on_nudge_right))
            .on_action(cx.listener(Self::on_nudge_up))
            .on_action(cx.listener(Self::on_nudge_down))
            .on_action(cx.listener(Self::on_fine_left))
            .on_action(cx.listener(Self::on_fine_right))
            .on_action(cx.listener(Self::on_fine_up))
            .on_action(cx.listener(Self::on_fine_down))
            .on_action(cx.listener(Self::delete_selected))
            .on_action(cx.listener(Self::on_next_channel))
            .on_action(cx.listener(Self::on_previous_channel))
            .on_action(cx.listener(Self::on_linear))
            .on_action(cx.listener(Self::on_smooth))
            .child(div().flex().items_center().justify_between().child(tabs).child(mode))
            .child(graph)
    }
}

fn point_distance(point: CurvePoint, x: f64, y: f64) -> f64 {
    ((point.x - x).powi(2) + (point.y - y).powi(2)).sqrt()
}

pub fn normalize_points(mut points: Vec<CurvePoint>) -> Vec<CurvePoint> {
    points.retain(|p| p.x.is_finite() && p.y.is_finite());
    for p in &mut points {
        p.x = p.x.clamp(0.0, 1.0);
        p.y = p.y.clamp(0.0, 1.0);
    }
    points.sort_by(|a, b| a.x.total_cmp(&b.x));
    let mut normalized = vec![CurvePoint::new(0.0, 0.0)];
    for p in points {
        if p.x <= MIN_X_GAP || p.x >= 1.0 - MIN_X_GAP {
            continue;
        }
        if p.x - normalized.last().unwrap().x < MIN_X_GAP {
            continue;
        }
        normalized.push(p);
    }
    normalized.push(CurvePoint::new(1.0, 1.0));
    normalized.truncate(MAX_POINTS - 1);
    if normalized.last().is_some_and(|p| p.x < 1.0) {
        normalized.push(CurvePoint::new(1.0, 1.0));
    }
    normalized
}

pub fn sample_curve(points: &[CurvePoint], mode: Interpolation, x: f64) -> f64 {
    if points.len() < 2 {
        return x.clamp(0.0, 1.0);
    }
    let x = x.clamp(0.0, 1.0);
    let index = points.partition_point(|p| p.x < x).saturating_sub(1).min(points.len() - 2);
    let a = points[index];
    let b = points[index + 1];
    let width = (b.x - a.x).max(MIN_X_GAP);
    let t = ((x - a.x) / width).clamp(0.0, 1.0);
    match mode {
        Interpolation::Linear => a.y + (b.y - a.y) * t,
        Interpolation::Smooth => {
            let before = if index == 0 { a } else { points[index - 1] };
            let after = if index + 2 >= points.len() { b } else { points[index + 2] };
            let slope = (b.y - a.y) / width;
            let m0 = if index == 0 {
                slope
            } else {
                monotone_tangent((a.y - before.y) / (a.x - before.x), slope)
            };
            let m1 = if index + 2 >= points.len() {
                slope
            } else {
                monotone_tangent(slope, (after.y - b.y) / (after.x - b.x))
            };
            let t2 = t * t;
            let t3 = t2 * t;
            ((2.0 * t3 - 3.0 * t2 + 1.0) * a.y
                + (t3 - 2.0 * t2 + t) * width * m0
                + (-2.0 * t3 + 3.0 * t2) * b.y
                + (t3 - t2) * width * m1)
                .clamp(a.y.min(b.y), a.y.max(b.y))
        }
    }
}
fn monotone_tangent(left: f64, right: f64) -> f64 {
    if left.signum() != right.signum() || left == 0.0 || right == 0.0 {
        0.0
    } else {
        2.0 * left * right / (left + right)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ChannelChanged, CurveChanged, CurveChannel, CurveEditor, CurvePoint, Interpolation,
        InterpolationChanged, default_key_bindings, normalize_points, sample_curve,
    };
    use gpui_pre::{Focusable, Modifiers, MouseButton, TestAppContext, point, px};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn normalized_points_are_sorted_bounded_and_keep_anchors() {
        let points = normalize_points(vec![
            CurvePoint::new(0.7, 2.0),
            CurvePoint::new(0.3, -1.0),
            CurvePoint::new(0.3, 0.5),
        ]);
        assert_eq!(points.first(), Some(&CurvePoint::new(0.0, 0.0)));
        assert_eq!(points.last(), Some(&CurvePoint::new(1.0, 1.0)));
        assert_eq!(points[1], CurvePoint::new(0.3, 0.0));
        assert_eq!(points[2], CurvePoint::new(0.7, 1.0));
    }

    #[test]
    fn interpolation_is_bounded_and_modes_differ_between_points() {
        let points = normalize_points(vec![CurvePoint::new(0.3, 0.9), CurvePoint::new(0.7, 0.4)]);
        let linear = sample_curve(&points, Interpolation::Linear, 0.15);
        let smooth = sample_curve(&points, Interpolation::Smooth, 0.15);
        assert!((0.0..=1.0).contains(&smooth));
        assert_ne!(linear, smooth);
    }

    #[gpui_pre::test]
    fn keyboard_nudge_delete_and_pointer_add_move_delete_work(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let channels = vec![
            CurveChannel::new("Master", vec![CurvePoint::new(0.5, 0.5)]),
            CurveChannel::new("Red", vec![]),
        ];
        let (editor, visual) = cx.add_window_view(|_, _| CurveEditor::new("Tone curve", channels));
        let changes = Rc::new(RefCell::new(Vec::new()));
        let change_log = changes.clone();
        let _sub = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &CurveChanged, _| {
                change_log.borrow_mut().push(event.clone())
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| editor.focus_handle(cx).focus(window, cx));
        editor.update(visual, |editor, _| editor.selected_point = Some(1));
        visual.simulate_keystrokes("right");
        assert!((editor.read_with(visual, |e, _| e.points()[1].x) - 0.51).abs() < 1e-9);
        assert_eq!(changes.borrow().len(), 1);
        visual.simulate_keystrokes("delete");
        assert_eq!(editor.read_with(visual, |e, _| e.points().len()), 2);

        let bounds = visual.debug_bounds("mkit-curve-editor-graph").unwrap();
        let new_point = point(
            bounds.origin.x + px(0.75 * f32::from(bounds.size.width)),
            bounds.origin.y + px(0.25 * f32::from(bounds.size.height)),
        );
        visual.simulate_mouse_down(new_point, MouseButton::Left, Modifiers::default());
        assert_eq!(editor.read_with(visual, |e, _| e.points().len()), 3);
        let within_graph = point(
            bounds.origin.x + px(0.85 * f32::from(bounds.size.width)),
            bounds.origin.y + px(0.3 * f32::from(bounds.size.height)),
        );
        visual.simulate_mouse_move(within_graph, Some(MouseButton::Left), Modifiers::default());
        let moved = point(
            bounds.origin.x + px(1.2 * f32::from(bounds.size.width)),
            bounds.origin.y + px(0.3 * f32::from(bounds.size.height)),
        );
        visual.simulate_mouse_move(moved, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(moved, MouseButton::Left, Modifiers::default());
        let moved_point = editor.read_with(visual, |e, _| e.points()[1]);
        assert!(moved_point.x > 0.9);
        let near_moved_point = point(
            bounds.origin.x + px(moved_point.x as f32 * f32::from(bounds.size.width)),
            bounds.origin.y + px((1.0 - moved_point.y) as f32 * f32::from(bounds.size.height)),
        );
        visual.simulate_mouse_down(near_moved_point, MouseButton::Right, Modifiers::default());
        assert_eq!(editor.read_with(visual, |e, _| e.points().len()), 2);
        editor.update(visual, |e, cx| e.select_channel(1, cx));
        assert_eq!(editor.read_with(visual, |e, _| e.active_channel()), 1);
    }

    #[gpui_pre::test]
    fn controlled_keyboard_edits_wait_for_owner_and_emit_normalized_proposals(
        cx: &mut TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let channel = CurveChannel::new("Master", vec![CurvePoint::new(0.5, 0.5)]);
        let (editor, visual) =
            cx.add_window_view(|_, _| CurveEditor::controlled("Tone curve", vec![channel]));
        let changes = Rc::new(RefCell::new(Vec::new()));
        let change_log = changes.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &CurveChanged, _| {
                change_log.borrow_mut().push(event.clone());
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| editor.focus_handle(cx).focus(window, cx));
        editor.update(visual, |editor, _| editor.selected_point = Some(1));
        let original = editor.read_with(visual, |editor, _| editor.points().to_vec());

        visual.simulate_keystrokes("right");
        assert_eq!(editor.read_with(visual, |editor, _| editor.points().to_vec()), original);
        let proposal = changes.borrow()[0].clone();
        assert_eq!(proposal.channel, 0);
        assert_eq!(proposal.points.len(), 3);
        assert!((proposal.points[1].x - 0.51).abs() < 1e-9);
        assert_eq!(proposal.points.first(), Some(&CurvePoint::new(0.0, 0.0)));
        assert_eq!(proposal.points.last(), Some(&CurvePoint::new(1.0, 1.0)));

        editor
            .update(visual, |editor, cx| editor.set_channel_points(0, proposal.points.clone(), cx));
        assert_eq!(editor.read_with(visual, |editor, _| editor.points().to_vec()), proposal.points);
        editor
            .update(visual, |editor, cx| editor.set_channel_points(0, proposal.points.clone(), cx));
        assert_eq!(changes.borrow().len(), 1, "owner echo must not emit a user change");

        visual.simulate_keystrokes("delete");
        assert_eq!(editor.read_with(visual, |editor, _| editor.points().len()), 3);
        assert_eq!(changes.borrow().len(), 2);
        assert_eq!(changes.borrow()[1].channel, 0);
        assert_eq!(
            changes.borrow()[1].points,
            vec![CurvePoint::new(0.0, 0.0), CurvePoint::new(1.0, 1.0)]
        );
    }

    #[gpui_pre::test]
    fn keyboard_channel_and_interpolation_actions_emit_only_on_changes(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let channels = vec![
            CurveChannel::new("Master", vec![]),
            CurveChannel::new("Red", vec![CurvePoint::new(0.4, 0.6)]),
        ];
        let (editor, visual) = cx.add_window_view(|_, _| CurveEditor::new("Tone curve", channels));
        let selected = Rc::new(RefCell::new(Vec::new()));
        let modes = Rc::new(RefCell::new(Vec::new()));
        let selected_log = selected.clone();
        let mode_log = modes.clone();
        let _subscriptions = visual.update(|_, app| {
            (
                app.subscribe(&editor, move |_, event: &ChannelChanged, _| {
                    selected_log.borrow_mut().push(event.0);
                }),
                app.subscribe(&editor, move |_, event: &InterpolationChanged, _| {
                    mode_log.borrow_mut().push(event.0);
                }),
            )
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| editor.focus_handle(cx).focus(window, cx));

        visual.simulate_keystrokes("tab s s shift-tab l l");
        assert_eq!(editor.read_with(visual, |editor, _| editor.active_channel()), 0);
        assert_eq!(
            editor.read_with(visual, |editor, _| editor.interpolation),
            Interpolation::Linear
        );
        assert_eq!(*selected.borrow(), vec![1, 0]);
        assert_eq!(*modes.borrow(), vec![Interpolation::Smooth, Interpolation::Linear]);
    }
}
