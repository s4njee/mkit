//! A themed pan-and-zoom surface for app-drawn content.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, BorderStyle, Bounds, ClickEvent, Context, EventEmitter, FocusHandle, Focusable,
    FontWeight, IntoElement, KeyBinding, KeyDownEvent, KeyUpEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PinchEvent, Pixels, Point, Render, Rgba, ScrollDelta,
    ScrollWheelEvent, TextAlign, Window, actions, canvas, div, fill, point, prelude::*, px, quad,
    size,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
use std::{cell::Cell, rc::Rc};

pub const KEY_CONTEXT: &str = "MkitViewport";
actions!(viewport, [PanLeft, PanRight, PanUp, PanDown, ZoomIn, ZoomOut, Fit, ActualSize]);

pub fn default_key_bindings() -> [KeyBinding; 8] {
    [
        KeyBinding::new("left", PanLeft, Some(KEY_CONTEXT)),
        KeyBinding::new("right", PanRight, Some(KEY_CONTEXT)),
        KeyBinding::new("up", PanUp, Some(KEY_CONTEXT)),
        KeyBinding::new("down", PanDown, Some(KEY_CONTEXT)),
        KeyBinding::new("=", ZoomIn, Some(KEY_CONTEXT)),
        KeyBinding::new("-", ZoomOut, Some(KEY_CONTEXT)),
        KeyBinding::new("f", Fit, Some(KEY_CONTEXT)),
        KeyBinding::new("1", ActualSize, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewTransform {
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for ViewTransform {
    fn default() -> Self {
        Self { scale: 1.0, offset_x: 0.0, offset_y: 0.0 }
    }
}

impl ViewTransform {
    pub fn valid(self) -> bool {
        self.scale.is_finite()
            && self.scale > 0.0
            && self.scale <= 16.0
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
        let scale = (self.scale * factor).clamp(f32::MIN_POSITIVE, 16.0);
        Self { scale, offset_x: screen.x - world.x * scale, offset_y: screen.y - world.y * scale }
    }

    pub fn fitted(content: Point<f32>, viewport: Point<f32>, padding: f32) -> Option<Self> {
        if !content.x.is_finite()
            || !content.y.is_finite()
            || content.x <= 0.0
            || content.y <= 0.0
            || viewport.x <= 0.0
            || viewport.y <= 0.0
        {
            return None;
        }
        let width = (viewport.x - 2.0 * padding).max(1.0);
        let height = (viewport.y - 2.0 * padding).max(1.0);
        let scale = (width / content.x).min(height / content.y).min(16.0);
        Some(Self {
            scale,
            offset_x: (viewport.x - content.x * scale) / 2.0,
            offset_y: (viewport.y - content.y * scale) / 2.0,
        })
    }

    pub fn actual_size(content: Point<f32>, viewport: Point<f32>) -> Self {
        Self {
            scale: 1.0,
            offset_x: (viewport.x - content.x) / 2.0,
            offset_y: (viewport.y - content.y) / 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformChanged(pub ViewTransform);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Guide {
    Horizontal(f32),
    Vertical(f32),
}

type Painter = Rc<dyn Fn(Bounds<Pixels>, ViewTransform, &mut Window)>;

/// Colours derived from theme tokens; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    high_contrast: bool,
    /// shadcn "muted": ruler strips and control hover fill.
    muted: Rgba,
    /// Frame border, toolbar and ruler dividers, and ruler ticks.
    divider: Rgba,
    /// Opaque outline-button border.
    control_border: Rgba,
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
            ring: c.focus,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    Look {
        high_contrast: false,
        muted: mix(c.text, c.background, if dark { 0.12 } else { 0.04 }),
        divider: if dark { c.text.opacity(0.1) } else { c.border },
        control_border: if dark { mix(c.text, c.background, 0.1) } else { c.border },
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
/// shadcn/ui focus ring width, drawn outside the focused element.
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

/// A small outline button (the restyled Button's `outline` variant) at the dense
/// `controls.xsmall` toolbar height.
fn toolbar_button(
    id: &'static str,
    label: &'static str,
    theme: Theme,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
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
            move |s| if look.high_contrast { s.border_color(c.accent) } else { s.bg(look.muted) },
        )
        .focus_visible(move |s| {
            s.border_color(c.focus).bg(c.background).shadow(vec![focus_ring(look.ring)])
        })
        .on_click(on_click)
        .child(label)
}

/// Entity-backed viewport. The supplied painter owns the scene, while this view owns its transform.
pub struct Viewport {
    label: String,
    content: Point<f32>,
    painter: Painter,
    transform: ViewTransform,
    controlled: bool,
    drag_pan_enabled: bool,
    show_rulers: bool,
    guides: Vec<Guide>,
    space_held: bool,
    pointer_down: bool,
    last_pointer: Option<Point<f32>>,
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    focus: Option<FocusHandle>,
}

impl EventEmitter<TransformChanged> for Viewport {}

impl Viewport {
    pub fn new(
        label: impl Into<String>,
        content: Point<f32>,
        painter: impl Fn(Bounds<Pixels>, ViewTransform, &mut Window) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            content,
            painter: Rc::new(painter),
            transform: ViewTransform::default(),
            controlled: false,
            drag_pan_enabled: true,
            show_rulers: false,
            guides: Vec::new(),
            space_held: false,
            pointer_down: false,
            last_pointer: None,
            bounds: Rc::new(Cell::new(None)),
            focus: None,
        }
    }

    pub fn controlled(mut self, transform: ViewTransform) -> Self {
        if transform.valid() {
            self.transform = transform;
            self.controlled = true;
        }
        self
    }

    pub fn drag_pan_enabled(mut self, enabled: bool) -> Self {
        self.drag_pan_enabled = enabled;
        self
    }

    pub fn show_rulers(mut self, show: bool) -> Self {
        self.show_rulers = show;
        self
    }

    pub fn guides(mut self, guides: Vec<Guide>) -> Self {
        self.guides = guides;
        self
    }

    pub fn transform(&self) -> ViewTransform {
        self.transform
    }

    pub fn world_to_screen(&self, world: Point<f32>) -> Point<f32> {
        self.transform.world_to_screen(world)
    }

    pub fn screen_to_world(&self, screen: Point<f32>) -> Point<f32> {
        self.transform.screen_to_world(screen)
    }

    pub fn set_transform(&mut self, transform: ViewTransform, cx: &mut Context<Self>) {
        if transform.valid() && self.transform != transform {
            self.transform = transform;
            cx.notify();
        }
    }

    fn request(&mut self, next: ViewTransform, cx: &mut Context<Self>) {
        if !next.valid() || next == self.transform {
            return;
        }
        if !self.controlled {
            self.transform = next;
        }
        cx.emit(TransformChanged(next));
        cx.notify();
    }

    fn viewport_size(&self) -> Option<Point<f32>> {
        self.bounds
            .get()
            .map(|bounds| point(bounds.size.width.as_f32(), bounds.size.height.as_f32()))
    }

    fn center(&self) -> Option<Point<f32>> {
        self.viewport_size().map(|size| point(size.x / 2.0, size.y / 2.0))
    }

    fn pointer_local(&self, pointer: Point<Pixels>) -> Option<Point<f32>> {
        self.bounds.get().map(|bounds| {
            point(
                pointer.x.as_f32() - bounds.origin.x.as_f32(),
                pointer.y.as_f32() - bounds.origin.y.as_f32(),
            )
        })
    }

    fn pan(&mut self, dx: f32, dy: f32, cx: &mut Context<Self>) {
        self.request(self.transform.panned(dx, dy), cx);
    }

    fn zoom(&mut self, factor: f32, point: Point<f32>, cx: &mut Context<Self>) {
        self.request(self.transform.zoomed_at(point, factor), cx);
    }

    fn fit(&mut self, theme: Theme, cx: &mut Context<Self>) {
        if let Some(size) = self.viewport_size()
            && let Some(next) = ViewTransform::fitted(self.content, size, theme.spacing.medium)
        {
            self.request(next, cx);
        }
    }

    fn actual_size(&mut self, cx: &mut Context<Self>) {
        if let Some(size) = self.viewport_size() {
            self.request(ViewTransform::actual_size(self.content, size), cx);
        }
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, _: &mut Window, _: &mut Context<Self>) {
        if event.button == MouseButton::Left && (self.drag_pan_enabled || self.space_held) {
            self.pointer_down = true;
            self.last_pointer = self.pointer_local(event.position);
        }
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.pointer_down || !event.dragging() {
            return;
        }
        let current = self.pointer_local(event.position);
        if let (Some(previous), Some(current)) = (self.last_pointer, current) {
            self.pan(current.x - previous.x, current.y - previous.y, cx);
        }
        self.last_pointer = current;
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.pointer_down = false;
        self.last_pointer = None;
    }

    fn on_wheel(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        match event.delta {
            ScrollDelta::Lines(delta) => {
                if let Some(pointer) = self.pointer_local(event.position) {
                    self.zoom(1.12_f32.powf(delta.y), pointer, cx);
                }
            }
            ScrollDelta::Pixels(delta) => {
                self.pan(delta.x.as_f32(), delta.y.as_f32(), cx);
            }
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
}

impl Focusable for Viewport {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("Viewport focus handle initialized during render")
    }
}

impl Render for Viewport {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        let painter = self.painter.clone();
        let bounds_cell = self.bounds.clone();
        let transform = self.transform;
        let show_rulers = self.show_rulers;
        let guides = self.guides.clone();
        let look = look(&theme);
        let pan_step = theme.spacing.large * 2.0;
        let zoom_in = cx.listener(|this, _: &ZoomIn, _, cx| {
            if let Some(center) = this.center() {
                this.zoom(1.2, center, cx);
            }
        });
        let zoom_out = cx.listener(|this, _: &ZoomOut, _, cx| {
            if let Some(center) = this.center() {
                this.zoom(1.0 / 1.2, center, cx);
            }
        });
        div()
            .id("mkit-viewport")
            .debug_selector(|| "mkit-viewport".into())
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .tab_index(0)
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.label.clone())
            .aria_description("Arrow keys pan; plus and minus zoom; F fits; 1 shows actual size")
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .bg(theme.colors.background)
            .border(px(theme.borders.hairline))
            .border_color(look.divider)
            .rounded(px(theme.radii.large))
            .focus_visible(move |el| {
                el.border_color(theme.colors.focus).shadow(vec![focus_ring(look.ring)])
            })
            .on_action(cx.listener(move |this, _: &PanLeft, _, cx| this.pan(pan_step, 0.0, cx)))
            .on_action(cx.listener(move |this, _: &PanRight, _, cx| this.pan(-pan_step, 0.0, cx)))
            .on_action(cx.listener(move |this, _: &PanUp, _, cx| this.pan(0.0, pan_step, cx)))
            .on_action(cx.listener(move |this, _: &PanDown, _, cx| this.pan(0.0, -pan_step, cx)))
            .on_action(zoom_in)
            .on_action(zoom_out)
            .on_action(cx.listener(move |this, _: &Fit, _, cx| this.fit(theme, cx)))
            .on_action(cx.listener(|this, _: &ActualSize, _, cx| this.actual_size(cx)))
            .on_key_down(cx.listener(Self::on_key_down))
            .on_key_up(cx.listener(Self::on_key_up))
            .child(
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
                    .child(format!("{:.0}%", self.transform.scale * 100.0))
                    .child(toolbar_button(
                        "mkit-viewport-fit",
                        "Fit",
                        theme,
                        cx.listener(move |this, _, _, cx| this.fit(theme, cx)),
                    ))
                    .child(toolbar_button(
                        "mkit-viewport-actual",
                        "100%",
                        theme,
                        cx.listener(|this, _, _, cx| this.actual_size(cx)),
                    ))
                    .child(toolbar_button(
                        "mkit-viewport-zoom-out",
                        "Zoom out",
                        theme,
                        cx.listener(|this, _, _, cx| {
                            if let Some(center) = this.center() {
                                this.zoom(1.0 / 1.2, center, cx);
                            }
                        }),
                    ))
                    .child(toolbar_button(
                        "mkit-viewport-zoom-in",
                        "Zoom in",
                        theme,
                        cx.listener(|this, _, _, cx| {
                            if let Some(center) = this.center() {
                                this.zoom(1.2, center, cx);
                            }
                        }),
                    )),
            )
            .child(
                div()
                    .id("mkit-viewport-canvas")
                    .debug_selector(|| "mkit-viewport-canvas".into())
                    .flex_1()
                    .overflow_hidden()
                    .text_color(theme.colors.text_muted)
                    .text_size(px(theme.typography.caption))
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
                    .on_mouse_move(cx.listener(Self::on_mouse_move))
                    .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
                    .on_scroll_wheel(cx.listener(Self::on_wheel))
                    .on_pinch(cx.listener(Self::on_pinch))
                    .child(
                        canvas(
                            move |bounds, _, _| {
                                bounds_cell.set(Some(bounds));
                                (bounds, transform)
                            },
                            move |_, (bounds, transform), window, cx| {
                                painter(bounds, transform, window);
                                paint_overlays(
                                    bounds,
                                    transform,
                                    &guides,
                                    show_rulers,
                                    theme,
                                    window,
                                    cx,
                                );
                            },
                        )
                        .size_full(),
                    ),
            )
    }
}

fn paint_overlays(
    bounds: Bounds<Pixels>,
    transform: ViewTransform,
    guides: &[Guide],
    show_rulers: bool,
    theme: Theme,
    window: &mut Window,
    cx: &mut App,
) {
    let look = look(&theme);
    let hairline = px(theme.borders.hairline);
    let strip = px(theme.controls.xsmall);
    // Guide handles sit against the canvas edge of the ruler strips, clear of most tick labels.
    let handle = px(theme.spacing.medium);
    let inner_edge = strip - handle / 2.0 - hairline;
    let mut handles = Vec::new();
    for guide in guides {
        match *guide {
            Guide::Vertical(world_x) if world_x.is_finite() => {
                let x = bounds.left() + px(transform.world_to_screen(point(world_x, 0.0)).x);
                if x >= bounds.left() && x <= bounds.right() {
                    window.paint_quad(fill(
                        Bounds::new(point(x, bounds.top()), size(hairline, bounds.size.height)),
                        theme.colors.focus,
                    ));
                    if x >= bounds.left() + strip {
                        handles.push(point(x + hairline / 2.0, bounds.top() + inner_edge));
                    }
                }
            }
            Guide::Horizontal(world_y) if world_y.is_finite() => {
                let y = bounds.top() + px(transform.world_to_screen(point(0.0, world_y)).y);
                if y >= bounds.top() && y <= bounds.bottom() {
                    window.paint_quad(fill(
                        Bounds::new(point(bounds.left(), y), size(bounds.size.width, hairline)),
                        theme.colors.focus,
                    ));
                    if y >= bounds.top() + strip {
                        handles.push(point(bounds.left() + inner_edge, y + hairline / 2.0));
                    }
                }
            }
            _ => {}
        }
    }
    if !show_rulers {
        return;
    }
    // Ruler strips: muted fill with a hairline divider along the canvas edge.
    window.paint_quad(fill(Bounds::new(bounds.origin, size(bounds.size.width, strip)), look.muted));
    window
        .paint_quad(fill(Bounds::new(bounds.origin, size(strip, bounds.size.height)), look.muted));
    window.paint_quad(fill(
        Bounds::new(
            point(bounds.left(), bounds.top() + strip - hairline),
            size(bounds.size.width, hairline),
        ),
        look.divider,
    ));
    window.paint_quad(fill(
        Bounds::new(
            point(bounds.left() + strip - hairline, bounds.top()),
            size(hairline, bounds.size.height),
        ),
        look.divider,
    ));
    if let Some(step) = ruler_step(transform.scale) {
        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32();
        let start_x = (transform.screen_to_world(point(0.0, 0.0)).x / step).floor();
        let start_y = (transform.screen_to_world(point(0.0, 0.0)).y / step).floor();
        for index in 0..256 {
            let world_x = (start_x + index as f32) * step;
            let screen_x = transform.world_to_screen(point(world_x, 0.0)).x;
            if screen_x > width {
                break;
            }
            if screen_x >= strip.as_f32() && screen_x.is_finite() {
                let x = bounds.left() + px(screen_x);
                window.paint_quad(fill(
                    Bounds::new(point(x, bounds.top() + strip * 0.7), size(hairline, strip * 0.3)),
                    look.divider,
                ));
                if screen_x <= width - theme.controls.medium {
                    paint_ruler_label(
                        format_ruler_value(world_x, step),
                        point(x + px(theme.spacing.xsmall), bounds.top()),
                        window,
                        cx,
                    );
                }
            }
        }
        for index in 0..256 {
            let world_y = (start_y + index as f32) * step;
            let screen_y = transform.world_to_screen(point(0.0, world_y)).y;
            if screen_y > height {
                break;
            }
            if screen_y >= strip.as_f32() && screen_y.is_finite() {
                let y = bounds.top() + px(screen_y);
                window.paint_quad(fill(
                    Bounds::new(point(bounds.left() + strip * 0.7, y), size(strip * 0.3, hairline)),
                    look.divider,
                ));
                if screen_y <= height - theme.controls.medium {
                    paint_ruler_label(
                        format_ruler_value(world_y, step),
                        point(bounds.left() + px(theme.spacing.xsmall), y),
                        window,
                        cx,
                    );
                }
            }
        }
    }
    // Corner square where the strips meet.
    window.paint_quad(quad(
        Bounds::new(bounds.origin, size(strip, strip)),
        px(0.),
        look.muted,
        gpui_pre::Edges { right: hairline, bottom: hairline, ..Default::default() },
        look.divider,
        BorderStyle::Solid,
    ));
    // Guide handles: the restyled Slider thumb look (opaque `background` fill, a hairline border
    // in the guide colour, and the small shadow) drawn as a `spacing.medium` circle.
    for centre in handles {
        let handle_bounds =
            Bounds::new(centre - point(handle / 2.0, handle / 2.0), size(handle, handle));
        let radius = gpui_pre::Corners::all(handle / 2.0);
        window.paint_drop_shadows(handle_bounds, radius, &[box_shadow(theme.shadows.small)]);
        window.paint_quad(quad(
            handle_bounds,
            radius,
            theme.colors.background,
            hairline,
            theme.colors.focus,
            BorderStyle::Solid,
        ));
    }
}

fn ruler_step(scale: f32) -> Option<f32> {
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let target = 50.0 / scale;
    if !target.is_finite() || target <= 0.0 {
        return None;
    }
    let decade = 10.0_f32.powf(target.log10().floor());
    [1.0, 2.0, 5.0, 10.0].into_iter().map(|multiple| multiple * decade).find(|step| *step >= target)
}

fn format_ruler_value(value: f32, step: f32) -> String {
    if step >= 1.0 { format!("{value:.0}") } else { format!("{value:.2}") }
}

fn paint_ruler_label(label: String, origin: Point<Pixels>, window: &mut Window, cx: &mut App) {
    let style = window.text_style();
    let shaped = window.text_system().shape_line(
        label.clone().into(),
        style.font_size.to_pixels(window.rem_size()),
        &[style.to_run(label.len())],
        None,
    );
    let _ = shaped.paint(origin, window.line_height(), TextAlign::Left, None, window, cx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{KeyUpEvent, Keystroke, Modifiers, TestAppContext};

    #[test]
    fn transform_round_trip_and_cursor_anchor() {
        let transform = ViewTransform { scale: 2.0, offset_x: 30.0, offset_y: -8.0 };
        let world = point(12.0, 20.0);
        assert_eq!(transform.screen_to_world(transform.world_to_screen(world)), world);
        let cursor = point(200.0, 130.0);
        let anchored = transform.screen_to_world(cursor);
        let zoomed = transform.zoomed_at(cursor, 1.25);
        assert_eq!(zoomed.world_to_screen(anchored), cursor);
    }

    #[test]
    fn fit_centers_positive_content() {
        let fit = ViewTransform::fitted(point(400.0, 200.0), point(800.0, 600.0), 12.0).unwrap();
        assert!(fit.scale > 1.0);
        assert!((fit.offset_x - 12.0).abs() < 0.01);
        assert!(fit.offset_y > 100.0);
        assert!(ViewTransform::fitted(point(0.0, 200.0), point(800.0, 600.0), 12.0).is_none());
        let very_large =
            ViewTransform::fitted(point(100_000.0, 100_000.0), point(800.0, 600.0), 12.0).unwrap();
        assert!(very_large.valid() && very_large.scale < 0.1);
        assert!(100_000.0 * very_large.scale <= 600.0 - 24.0);
    }

    #[test]
    fn ruler_tick_step_tracks_zoom_without_overcrowding() {
        assert_eq!(ruler_step(1.0), Some(50.0));
        assert_eq!(ruler_step(2.0), Some(50.0));
        assert_eq!(ruler_step(0.5), Some(100.0));
        assert_eq!(ruler_step(0.0), None);
        assert_eq!(ruler_step(f32::NAN), None);
    }

    #[gpui_pre::test]
    fn pointer_wheel_and_keyboard_update_the_same_transform(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (viewport, visual) = cx.add_window_view(|_, _| {
            Viewport::new("Canvas viewport", point(400.0, 200.0), |_, _, _| {})
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let bounds = visual.debug_bounds("mkit-viewport-canvas").unwrap();
        let start = bounds.origin + point(px(100.0), px(80.0));
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(
            start + point(px(30.0), px(15.0)),
            Some(MouseButton::Left),
            Modifiers::default(),
        );
        visual.simulate_mouse_up(
            start + point(px(30.0), px(15.0)),
            MouseButton::Left,
            Modifiers::default(),
        );
        let panned = viewport.read_with(visual, |v, _| v.transform());
        assert_eq!((panned.offset_x, panned.offset_y), (30.0, 15.0));

        let cursor = start + point(px(30.0), px(15.0));
        visual.simulate_event(ScrollWheelEvent {
            position: cursor,
            delta: ScrollDelta::Lines(point(0.0, 1.0)),
            ..Default::default()
        });
        let zoomed = viewport.read_with(visual, |v, _| v.transform());
        assert!(zoomed.scale > 1.0);
        let local = point(
            cursor.x.as_f32() - bounds.origin.x.as_f32(),
            cursor.y.as_f32() - bounds.origin.y.as_f32(),
        );
        let fixed_world = panned.screen_to_world(local);
        let after = zoomed.world_to_screen(fixed_world);
        assert!((after.x - local.x).abs() < 0.01 && (after.y - local.y).abs() < 0.01);

        visual.update(|window, cx| viewport.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("f");
        let fitted = viewport.read_with(visual, |v, _| v.transform());
        assert!(fitted.scale > 1.0);
        visual.simulate_keystrokes("1");
        assert_eq!(viewport.read_with(visual, |v, _| v.transform().scale), 1.0);
    }

    #[gpui_pre::test]
    fn space_drag_and_toolbar_fit_work_without_ordinary_drag_pan(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let (viewport, visual) = cx.add_window_view(|_, _| {
            Viewport::new("Canvas viewport", point(400.0, 200.0), |_, _, _| {})
                .drag_pan_enabled(false)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| viewport.focus_handle(cx).focus(window, cx));
        let start = visual.debug_bounds("mkit-viewport-canvas").unwrap().center();
        let end = start + point(px(25.0), px(10.0));
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        assert_eq!(viewport.read_with(visual, |v, _| v.transform()), ViewTransform::default());

        let space = Keystroke::parse("space").unwrap();
        visual.simulate_event(KeyDownEvent {
            keystroke: space.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        visual.simulate_event(KeyUpEvent { keystroke: space });
        let panned = viewport.read_with(visual, |v, _| v.transform());
        assert_eq!((panned.offset_x, panned.offset_y), (25.0, 10.0));

        visual.update(|window, cx| window.draw(cx).clear(cx));
        let fit = visual.debug_bounds("mkit-viewport-fit").unwrap().center();
        visual.simulate_mouse_down(fit, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_up(fit, MouseButton::Left, Modifiers::default());
        assert!(viewport.read_with(visual, |v, _| v.transform().scale) > 1.0);
    }

    #[gpui_pre::test]
    fn pixel_scroll_pans_and_pinch_zooms_at_the_gesture_center(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let (viewport, visual) = cx.add_window_view(|_, _| {
            Viewport::new("Canvas viewport", point(400.0, 200.0), |_, _, _| {})
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let bounds = visual.debug_bounds("mkit-viewport-canvas").unwrap();
        let center = bounds.origin + point(px(120.0), px(90.0));
        visual.simulate_event(ScrollWheelEvent {
            position: center,
            delta: ScrollDelta::Pixels(point(px(18.0), px(-12.0))),
            ..Default::default()
        });
        let panned = viewport.read_with(visual, |v, _| v.transform());
        assert_eq!((panned.offset_x, panned.offset_y, panned.scale), (18.0, -12.0, 1.0));

        let local = point(
            center.x.as_f32() - bounds.origin.x.as_f32(),
            center.y.as_f32() - bounds.origin.y.as_f32(),
        );
        let anchored_world = panned.screen_to_world(local);
        visual.simulate_event(PinchEvent { position: center, delta: 0.25, ..Default::default() });
        let zoomed = viewport.read_with(visual, |v, _| v.transform());
        assert!((zoomed.scale - 1.25).abs() < 0.0001);
        let fixed = zoomed.world_to_screen(anchored_world);
        assert!((fixed.x - local.x).abs() < 0.01 && (fixed.y - local.y).abs() < 0.01);
    }
}
