//! A small, theme-aware editor for normalized one-dimensional sRGB gradients.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Bounds, Context, DispatchPhase, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyBinding, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PathBuilder, Pixels,
    Render, Rgba, Window, actions, canvas, div, linear_color_stop, linear_gradient, point,
    prelude::*, px, relative,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};

/// Resolved card, handle, field and button colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    high_contrast: bool,
    /// Card and preview border (the shadcn "border" role).
    line: Rgba,
    muted: Rgba,
    text: Rgba,
    label: Rgba,
    accent: Rgba,
    handle_fill: Rgba,
    /// `shadows.small`, halved when disabled; transparent in high contrast.
    shadow: ShadowToken,
    ring: Rgba,
    field_fill: Rgba,
    field_border: Rgba,
    button_border: Rgba,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight, ..foreground }, Rgba { a: 1.0, ..base })
}
/// Disabled parts render at 50% opacity as one layer, like the web look's `opacity: .5`. GPUI
/// applies element opacity to each painted part separately, so each colour is composited opaque
/// over `background` and then mixed 50% with it instead.
fn dim(color: Rgba, background: Rgba) -> Rgba {
    if color.a == 0. { color } else { mix(composite(color, background), background, 0.5) }
}
fn look(t: &Theme, disabled: bool) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        let (text, accent, line) = if disabled {
            (c.disabled, c.disabled, c.disabled)
        } else {
            (c.text, c.accent, c.border)
        };
        return Look {
            high_contrast: true,
            line,
            muted: c.background,
            text,
            label: if disabled { c.disabled } else { c.text_muted },
            accent,
            handle_fill: c.background,
            shadow: t.shadows.small,
            ring: c.focus,
            field_fill: c.background,
            field_border: line,
            button_border: line,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let field_fill = if dark { mix(c.text, c.background, 0.15 * 0.3) } else { c.background };
    let look = Look {
        high_contrast: false,
        line: if dark { mix(c.text, c.background, 0.1) } else { c.border },
        muted: mix(c.text, c.background, if dark { 0.12 } else { 0.04 }),
        text: c.text,
        label: c.text_muted,
        accent: c.accent,
        handle_fill: c.background,
        shadow: t.shadows.small,
        // The ring sits over gradient data, so it is composited over `background` to read the
        // same as the rings on theme surfaces.
        ring: composite(c.focus.opacity(0.5), c.background),
        field_fill,
        field_border: if dark { composite(c.text.opacity(0.15), field_fill) } else { c.border },
        button_border: if dark { mix(c.text, c.background, 0.1) } else { c.border },
    };
    if !disabled { look } else { look.dimmed(c.background) }
}
impl Look {
    fn dimmed(self, bg: Rgba) -> Self {
        if self.high_contrast {
            return self;
        }
        Look {
            line: dim(self.line, bg),
            muted: dim(self.muted, bg),
            text: dim(self.text, bg),
            label: dim(self.label, bg),
            accent: dim(self.accent, bg),
            handle_fill: dim(self.handle_fill, bg),
            shadow: ShadowToken {
                color: self.shadow.color.opacity(self.shadow.color.a * 0.5),
                ..self.shadow
            },
            field_fill: dim(self.field_fill, bg),
            field_border: dim(self.field_border, bg),
            button_border: dim(self.button_border, bg),
            ..self
        }
    }
    /// A single unavailable control (Remove at an endpoint) in an otherwise enabled editor.
    fn unavailable(self, t: &Theme) -> Self {
        if self.high_contrast {
            let d = t.colors.disabled;
            Look { text: d, label: d, accent: d, line: d, button_border: d, ..self }
        } else {
            self.dimmed(t.colors.background)
        }
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
/// shadcn/ui focus ring width, drawn outside the preview or a selected handle.
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
/// Decorative Lucide icon drawn as a vector stroke so it stays crisp at every scale. Each
/// polyline is a list of points on a 24-unit grid; the stroke is 2 units, Lucide's default.
fn icon(size: f32, lines: &'static [&'static [(f32, f32)]], color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let origin = bounds.origin;
            let mut path = PathBuilder::stroke(unit * 2.0);
            for line in lines {
                for (i, (x, y)) in line.iter().enumerate() {
                    let at = origin + point(unit * *x, unit * *y);
                    if i == 0 { path.move_to(at) } else { path.line_to(at) }
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
/// Lucide `minus`.
const MINUS: &[&[(f32, f32)]] = &[&[(5., 12.), (19., 12.)]];
/// Lucide `plus`.
const PLUS: &[&[(f32, f32)]] = &[&[(5., 12.), (19., 12.)], &[(12., 5.), (12., 19.)]];

pub const KEY_CONTEXT: &str = "MkitGradientEditor";
const MIN_GAP: f64 = 0.01;
pub const MAX_STOPS: usize = 16;
actions!(
    gradient_editor,
    [
        PreviousStop,
        NextStop,
        MoveLeft,
        MoveRight,
        FineLeft,
        FineRight,
        FirstPosition,
        LastPosition,
        DeleteStop,
        RedUp,
        RedDown,
        GreenUp,
        GreenDown,
        BlueUp,
        BlueDown
    ]
);

pub fn default_key_bindings() -> [KeyBinding; 15] {
    [
        KeyBinding::new("shift-tab", PreviousStop, Some(KEY_CONTEXT)),
        KeyBinding::new("tab", NextStop, Some(KEY_CONTEXT)),
        KeyBinding::new("left", MoveLeft, Some(KEY_CONTEXT)),
        KeyBinding::new("right", MoveRight, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-left", FineLeft, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-right", FineRight, Some(KEY_CONTEXT)),
        KeyBinding::new("home", FirstPosition, Some(KEY_CONTEXT)),
        KeyBinding::new("end", LastPosition, Some(KEY_CONTEXT)),
        KeyBinding::new("delete", DeleteStop, Some(KEY_CONTEXT)),
        KeyBinding::new("r", RedUp, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-r", RedDown, Some(KEY_CONTEXT)),
        KeyBinding::new("g", GreenUp, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-g", GreenDown, Some(KEY_CONTEXT)),
        KeyBinding::new("b", BlueUp, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-b", BlueDown, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Colour {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}
impl Colour {
    pub const fn new(red: f64, green: f64, blue: f64) -> Self {
        Self { red, green, blue }
    }
    fn sanitized(self) -> Self {
        Self { red: unit(self.red), green: unit(self.green), blue: unit(self.blue) }
    }
    fn rgba(self) -> Rgba {
        let c = self.sanitized();
        Rgba { r: c.red as f32, g: c.green as f32, b: c.blue as f32, a: 1.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub position: f64,
    pub colour: Colour,
}
impl GradientStop {
    pub fn new(position: f64, colour: Colour) -> Self {
        Self { position, colour }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub stops: Vec<GradientStop>,
}
impl Default for Gradient {
    fn default() -> Self {
        Self {
            stops: vec![
                GradientStop::new(0.0, Colour::new(0.10, 0.38, 0.86)),
                GradientStop::new(1.0, Colour::new(0.65, 0.25, 0.72)),
            ],
        }
    }
}
impl Gradient {
    pub fn new(stops: Vec<GradientStop>) -> Self {
        Self { stops: normalize_stops(stops) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GradientChanged(pub Gradient);
impl EventEmitter<GradientChanged> for GradientEditor {}
#[derive(Clone, Debug, PartialEq)]
pub struct StopSelected(pub usize);
impl EventEmitter<StopSelected> for GradientEditor {}

/// Stateful gradient editor. Its value type is local so the registry crate needs no colour-tools dependency.
pub struct GradientEditor {
    label: String,
    gradient: Gradient,
    selected: usize,
    controlled: bool,
    disabled: bool,
    focus: Option<FocusHandle>,
    preview_bounds: Option<Bounds<Pixels>>,
    dragging: bool,
}
impl GradientEditor {
    pub fn new(label: impl Into<String>, gradient: Gradient) -> Self {
        Self::base(label, gradient, false)
    }
    pub fn controlled(label: impl Into<String>, gradient: Gradient) -> Self {
        Self::base(label, gradient, true)
    }
    fn base(label: impl Into<String>, gradient: Gradient, controlled: bool) -> Self {
        Self {
            label: label.into(),
            gradient: Gradient::new(gradient.stops),
            selected: 0,
            controlled,
            disabled: false,
            focus: None,
            preview_bounds: None,
            dragging: false,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn gradient(&self) -> &Gradient {
        &self.gradient
    }
    pub fn selected_stop(&self) -> usize {
        self.selected
    }
    /// Owner applies an accepted proposal in controlled mode.
    pub fn set_gradient(&mut self, gradient: Gradient, cx: &mut Context<Self>) {
        self.gradient = Gradient::new(gradient.stops);
        self.selected = self.selected.min(self.gradient.stops.len() - 1);
        cx.notify();
    }
    pub fn select_stop(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.disabled && index < self.gradient.stops.len() && self.selected != index {
            self.selected = index;
            cx.emit(StopSelected(index));
            cx.notify();
        }
    }
    pub fn add_stop(&mut self, cx: &mut Context<Self>) {
        if self.disabled || self.gradient.stops.len() >= MAX_STOPS {
            return;
        }
        let (index, position) = widest_gap(&self.gradient.stops);
        let mut next = self.gradient.clone();
        next.stops
            .insert(index, GradientStop::new(position, sample_gradient(&self.gradient, position)));
        self.selected = index;
        self.propose(next, cx);
        cx.emit(StopSelected(index));
    }
    pub fn delete_stop(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.disabled || index == 0 || index + 1 >= self.gradient.stops.len() {
            return;
        }
        let mut next = self.gradient.clone();
        next.stops.remove(index);
        self.selected = self.selected.min(next.stops.len() - 1);
        self.propose(next, cx);
    }
    pub fn move_stop(&mut self, index: usize, position: f64, cx: &mut Context<Self>) {
        if self.disabled || index == 0 || index + 1 >= self.gradient.stops.len() {
            return;
        }
        let mut next = self.gradient.clone();
        let lo = next.stops[index - 1].position + MIN_GAP;
        let hi = next.stops[index + 1].position - MIN_GAP;
        next.stops[index].position = finite(position).clamp(lo, hi);
        self.propose(next, cx);
    }
    pub fn set_stop_colour(&mut self, index: usize, colour: Colour, cx: &mut Context<Self>) {
        if self.disabled || index >= self.gradient.stops.len() {
            return;
        }
        let mut next = self.gradient.clone();
        next.stops[index].colour = colour.sanitized();
        self.propose(next, cx);
    }
    fn propose(&mut self, next: Gradient, cx: &mut Context<Self>) {
        if next == self.gradient {
            return;
        }
        if !self.controlled {
            self.gradient = next.clone();
            cx.notify();
        }
        cx.emit(GradientChanged(next));
    }
    fn move_selected(&mut self, delta: f64, cx: &mut Context<Self>) {
        let i = self.selected;
        self.move_stop(i, self.gradient.stops[i].position + delta, cx);
    }
    fn channel(&mut self, which: u8, delta: f64, cx: &mut Context<Self>) {
        let i = self.selected;
        let mut c = self.gradient.stops[i].colour;
        match which {
            0 => c.red += delta,
            1 => c.green += delta,
            _ => c.blue += delta,
        };
        self.set_stop_colour(i, c, cx);
    }
    fn adjust_field(&mut self, field: u8, direction: f64, cx: &mut Context<Self>) {
        if field == 0 {
            self.move_selected(direction * 0.01, cx);
        } else {
            self.channel(field - 1, direction / 255.0, cx);
        }
    }
    fn choose(&mut self, d: isize, cx: &mut Context<Self>) {
        let n = self.gradient.stops.len();
        self.select_stop((self.selected as isize + d).rem_euclid(n as isize) as usize, cx);
    }
    fn on_previous(&mut self, _: &PreviousStop, _: &mut Window, cx: &mut Context<Self>) {
        self.choose(-1, cx);
    }
    fn on_next(&mut self, _: &NextStop, _: &mut Window, cx: &mut Context<Self>) {
        self.choose(1, cx);
    }
    fn on_left(&mut self, _: &MoveLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selected(-0.01, cx);
    }
    fn on_right(&mut self, _: &MoveRight, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selected(0.01, cx);
    }
    fn on_fine_left(&mut self, _: &FineLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selected(-0.001, cx);
    }
    fn on_fine_right(&mut self, _: &FineRight, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selected(0.001, cx);
    }
    fn on_first(&mut self, _: &FirstPosition, _: &mut Window, cx: &mut Context<Self>) {
        self.move_stop(self.selected, self.gradient.stops[self.selected].position - 10.0, cx);
    }
    fn on_last(&mut self, _: &LastPosition, _: &mut Window, cx: &mut Context<Self>) {
        self.move_stop(self.selected, self.gradient.stops[self.selected].position + 10.0, cx);
    }
    fn on_delete(&mut self, _: &DeleteStop, _: &mut Window, cx: &mut Context<Self>) {
        self.delete_stop(self.selected, cx);
    }
    fn on_red_up(&mut self, _: &RedUp, _: &mut Window, cx: &mut Context<Self>) {
        self.channel(0, 1.0 / 255.0, cx);
    }
    fn on_red_down(&mut self, _: &RedDown, _: &mut Window, cx: &mut Context<Self>) {
        self.channel(0, -1.0 / 255.0, cx);
    }
    fn on_green_up(&mut self, _: &GreenUp, _: &mut Window, cx: &mut Context<Self>) {
        self.channel(1, 1.0 / 255.0, cx);
    }
    fn on_green_down(&mut self, _: &GreenDown, _: &mut Window, cx: &mut Context<Self>) {
        self.channel(1, -1.0 / 255.0, cx);
    }
    fn on_blue_up(&mut self, _: &BlueUp, _: &mut Window, cx: &mut Context<Self>) {
        self.channel(2, 1.0 / 255.0, cx);
    }
    fn on_blue_down(&mut self, _: &BlueDown, _: &mut Window, cx: &mut Context<Self>) {
        self.channel(2, -1.0 / 255.0, cx);
    }
    fn pointer_position(&self, x: f32) -> Option<f64> {
        let b = self.preview_bounds?;
        let width = f32::from(b.size.width);
        if width <= 0.0 {
            return None;
        }
        Some(((x - f32::from(b.origin.x)) / width).clamp(0.0, 1.0) as f64)
    }
    fn pointer_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || event.button != MouseButton::Left {
            return;
        }
        let Some(bounds) = self.preview_bounds else { return };
        let Some(pos) = self.pointer_position(f32::from(event.position.x)) else { return };
        self.dragging = true;
        if let Some(f) = self.focus.as_ref() {
            window.focus(f, cx);
        }
        let nearest =
            self.gradient.stops.iter().enumerate().min_by(|(_, a), (_, b)| {
                (a.position - pos).abs().total_cmp(&(b.position - pos).abs())
            });
        if let Some((i, _stop)) = nearest.filter(|(_, s)| (s.position - pos).abs() < 0.035) {
            self.select_stop(i, cx);
            self.move_stop(i, pos, cx);
        } else {
            self.add_stop(cx);
            self.move_stop(self.selected, pos, cx);
        }
        self.preview_bounds = Some(bounds);
    }
    fn pointer_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if self.dragging
            && event.dragging()
            && let Some(pos) = self.pointer_position(f32::from(event.position.x))
        {
            self.move_stop(self.selected, pos, cx);
        }
    }
    fn pointer_up(&mut self, event: &MouseUpEvent) {
        if event.button == MouseButton::Left {
            self.dragging = false;
        }
    }
}
impl Focusable for GradientEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone().expect("focus initialized during render")
    }
}
impl Render for GradientEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_stop(true)).clone();
        let theme = *cx.global::<Theme>();
        let disabled = self.disabled;
        let look = look(&theme, disabled);
        let focus_visible =
            !disabled && focus.is_focused(window) && window.last_input_was_keyboard();
        let transparent = theme.colors.background.opacity(0.);
        let mut strip = div()
            .id("mkit-gradient-preview")
            .debug_selector(|| "mkit-gradient-preview".into())
            .relative()
            .flex()
            .size_full()
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.hairline))
            .border_color(if focus_visible { theme.colors.focus } else { look.line })
            .when(focus_visible, |el| el.shadow(vec![focus_ring(look.ring)]))
            .overflow_hidden()
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label("Gradient preview");
        // GPUI's current Background gradient accepts two stops, so render a sampled strip for any stop count.
        const SEGMENTS: usize = 64;
        for n in 0..SEGMENTS {
            let a = n as f64 / SEGMENTS as f64;
            let b = (n + 1) as f64 / SEGMENTS as f64;
            strip = strip.child(div().h_full().flex_1().bg(linear_gradient(
                90.0,
                linear_color_stop(sample_gradient(&self.gradient, a).rgba(), 0.0),
                linear_color_stop(sample_gradient(&self.gradient, b).rgba(), 1.0),
            )));
        }
        let handle_entity = cx.entity();
        let stop_count = self.gradient.stops.len();
        let selected_index = self.selected;
        let handles = self.gradient.stops.iter().enumerate().map(|(i, stop)| {
            let select_entity = handle_entity.clone();
            let label = format!("Stop {} at {:.0}%", i + 1, stop.position * 100.0);
            let selected = i == selected_index;
            let color = stop.colour.rgba();
            div()
                .id(format!("gradient-stop-{i}"))
                .debug_selector(move || format!("gradient-stop-{i}"))
                .flex_none()
                .w(px(theme.controls.large * 3.6))
                .h(px(theme.controls.small))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(theme.spacing.xsmall))
                .px(px(theme.spacing.xsmall))
                .text_color(look.text)
                .text_size(px(theme.typography.caption))
                .whitespace_nowrap()
                .bg(if selected { look.muted } else { theme.colors.background })
                .border(px(theme.borders.regular))
                .border_color(if selected { look.accent } else { look.button_border })
                .rounded(px(theme.radii.medium))
                .shadow(vec![box_shadow(look.shadow)])
                .when(!disabled && !selected, |el| {
                    el.hover(move |style| {
                        if look.high_contrast {
                            style.border_color(look.accent)
                        } else {
                            style.bg(look.muted)
                        }
                    })
                })
                .role(gpui_pre::accesskit::Role::Button)
                .aria_label(label)
                .aria_selected(selected)
                .aria_description(if i == 0 || i + 1 == stop_count {
                    "Endpoint stop; its position is fixed."
                } else {
                    "Select this stop, then use left and right arrows to move it."
                })
                .on_click(move |_, _, cx| {
                    select_entity.update(cx, |this, cx| this.select_stop(i, cx));
                })
                .child(
                    div()
                        .flex_none()
                        .w(px(theme.controls.xsmall))
                        .h(px(theme.controls.xsmall))
                        .rounded(px(theme.radii.small))
                        .border(px(theme.borders.hairline))
                        .border_color(look.line)
                        .bg(color),
                )
                .child(format!("Stop {} · {:.0}%", i + 1, stop.position * 100.0))
        });
        let i = self.selected;
        let stop = self.gradient.stops[i];
        let can_delete = i > 0 && i + 1 < self.gradient.stops.len() && !self.disabled;
        let add_entity = cx.entity();
        let delete_entity = cx.entity();
        let button = |id: &'static str, label: &'static str, look: Look, enabled: bool| {
            div()
                .id(id)
                .debug_selector(move || id.into())
                .role(gpui_pre::accesskit::Role::Button)
                .h(px(theme.controls.small))
                .px(px(theme.spacing.medium))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(theme.radii.medium))
                .border(px(theme.borders.regular))
                .border_color(look.button_border)
                .bg(theme.colors.background)
                .shadow(vec![box_shadow(look.shadow)])
                .text_color(look.text)
                .text_size(px(theme.typography.body))
                .font_weight(gpui_pre::FontWeight::MEDIUM)
                .whitespace_nowrap()
                .when(enabled, |el| {
                    el.hover(move |style| {
                        if look.high_contrast {
                            style.border_color(look.accent)
                        } else {
                            style.bg(look.muted)
                        }
                    })
                })
                .child(label)
        };
        let add = button("gradient-add-stop", "Add stop", look, !disabled)
            .aria_label("Add gradient stop")
            .on_click(move |_, _, cx| add_entity.update(cx, |this, cx| this.add_stop(cx)));
        let remove_look = if can_delete { look } else { look.unavailable(&theme) };
        let remove = button("gradient-remove-stop", "Remove", remove_look, can_delete)
            .aria_label("Remove selected gradient stop")
            .aria_description(if can_delete {
                "Removes the selected interior stop."
            } else {
                "Unavailable for endpoint stops or when editing is disabled."
            })
            .on_click(move |_, _, cx| {
                delete_entity.update(cx, |this, cx| this.delete_stop(this.selected, cx))
            });
        let field = |label: &'static str,
                     value: String,
                     channel: u8,
                     dec_id: &'static str,
                     inc_id: &'static str,
                     dec_label: &'static str,
                     inc_label: &'static str,
                     cx: &mut Context<Self>| {
            let decrease = cx.entity();
            let increase = cx.entity();
            let step =
                |id: &'static str, name: &'static str, glyph: &'static [&'static [(f32, f32)]]| {
                    div()
                        .id(id)
                        .role(gpui_pre::accesskit::Role::Button)
                        .aria_label(name)
                        .w(px(theme.controls.small))
                        .h(px(theme.controls.small))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(theme.radii.medium))
                        .border(px(theme.borders.hairline))
                        .border_color(transparent)
                        .when(!disabled, |el| {
                            el.hover(move |style| {
                                if look.high_contrast {
                                    style.border_color(look.accent)
                                } else {
                                    style.bg(look.muted)
                                }
                            })
                        })
                        .child(icon(theme.spacing.large, glyph, look.text))
                };
            div()
                .flex()
                .items_center()
                .gap(px(theme.spacing.xsmall))
                .p(px(theme.spacing.xsmall))
                .rounded(px(theme.radii.medium))
                .border(px(theme.borders.regular))
                .border_color(look.field_border)
                .bg(look.field_fill)
                .shadow(vec![box_shadow(look.shadow)])
                .child(
                    div()
                        .w(px(theme.controls.large * 1.2))
                        .whitespace_nowrap()
                        .text_size(px(theme.typography.caption))
                        .text_color(look.label)
                        .child(label),
                )
                .child(step(dec_id, dec_label, MINUS).on_click(move |_, _, cx| {
                    decrease.update(cx, |this, cx| this.adjust_field(channel, -1.0, cx))
                }))
                .child(
                    div()
                        .w(px(theme.controls.large))
                        .flex()
                        .justify_center()
                        .text_size(px(theme.typography.body))
                        .text_color(look.text)
                        .whitespace_nowrap()
                        .child(value),
                )
                .child(step(inc_id, inc_label, PLUS).on_click(move |_, _, cx| {
                    increase.update(cx, |this, cx| this.adjust_field(channel, 1.0, cx))
                }))
        };
        let controls = div()
            .flex()
            .flex_wrap()
            .gap(px(theme.spacing.small))
            .child(field(
                "Position",
                format!("{:.1}%", stop.position * 100.0),
                0,
                "gradient-position-dec",
                "gradient-position-inc",
                "Move selected stop left by one percent",
                "Move selected stop right by one percent",
                cx,
            ))
            .child(field(
                "Red",
                format!("{:02X}", byte(stop.colour.red)),
                1,
                "gradient-red-down",
                "gradient-red-up",
                "Decrease red channel by one",
                "Increase red channel by one",
                cx,
            ))
            .child(field(
                "Green",
                format!("{:02X}", byte(stop.colour.green)),
                2,
                "gradient-green-down",
                "gradient-green-up",
                "Decrease green channel by one",
                "Increase green channel by one",
                cx,
            ))
            .child(field(
                "Blue",
                format!("{:02X}", byte(stop.colour.blue)),
                3,
                "gradient-blue-down",
                "gradient-blue-up",
                "Decrease blue channel by one",
                "Increase blue channel by one",
                cx,
            ));
        let preview_entity = cx.entity();
        strip = strip.child(
            canvas(
                |_bounds, _window, _cx| {},
                move |bounds, _, window, _cx| {
                    let down = preview_entity.clone();
                    window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                        if phase == DispatchPhase::Capture && bounds.contains(&event.position) {
                            down.update(cx, |this, cx| {
                                this.preview_bounds = Some(bounds);
                                this.pointer_down(event, window, cx);
                            });
                        }
                    });
                    let moved = preview_entity.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture {
                            moved.update(cx, |this, cx| this.pointer_move(event, cx));
                        }
                    });
                    let up = preview_entity.clone();
                    window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture {
                            up.update(cx, |this, _| this.pointer_up(event));
                        }
                    });
                },
            )
            .absolute()
            .inset_0(),
        );
        // Stop handles sit above the strip so endpoint handles overhang its ends instead of
        // being clipped. GPUI keeps an element's corner radius when spreading a ring shadow,
        // so the selected ring is its own circle behind the handle.
        let handle_size = theme.spacing.large;
        let ring_size = handle_size + FOCUS_RING_WIDTH * 2.0;
        let preview_height = theme.controls.large * 1.8;
        let mut preview = div().relative().w_full().h(px(preview_height)).child(strip);
        for (index, stop) in self.gradient.stops.iter().enumerate() {
            let ringed = index == self.selected && !disabled;
            preview = preview
                .child(
                    div()
                        .absolute()
                        .left(relative(stop.position as f32))
                        .ml(px(-theme.borders.hairline / 2.0))
                        .top(px(theme.borders.hairline))
                        .bottom(px(theme.borders.hairline))
                        .w(px(theme.borders.hairline))
                        .bg(look.accent),
                )
                .child(
                    div()
                        .absolute()
                        .left(relative(stop.position as f32))
                        .ml(px(-ring_size / 2.0))
                        .top(px((preview_height - ring_size) / 2.0))
                        .size(px(ring_size))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(theme.radii.pill))
                        .when(ringed, |el| el.bg(look.ring))
                        .child(
                            div()
                                .flex_none()
                                .size(px(handle_size))
                                .rounded(px(theme.radii.pill))
                                .bg(look.handle_fill)
                                .border(px(theme.borders.hairline))
                                .border_color(look.accent)
                                .when(!ringed, |el| el.shadow(vec![box_shadow(look.shadow)])),
                        ),
                );
        }
        div()
            .id("mkit-gradient-editor")
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.small))
            .p(px(theme.spacing.medium))
            .rounded(px(theme.radii.large))
            .border(px(theme.borders.hairline))
            .border_color(look.line)
            .bg(theme.colors.background)
            .shadow(vec![box_shadow(look.shadow)])
            .text_color(look.text)
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.label.clone())
            .aria_description("Gradient preview with editable color stops. Select a stop, adjust its position and color, add or remove stops.")
            .on_action(cx.listener(Self::on_previous))
            .on_action(cx.listener(Self::on_next))
            .on_action(cx.listener(Self::on_left))
            .on_action(cx.listener(Self::on_right))
            .on_action(cx.listener(Self::on_fine_left))
            .on_action(cx.listener(Self::on_fine_right))
            .on_action(cx.listener(Self::on_first))
            .on_action(cx.listener(Self::on_last))
            .on_action(cx.listener(Self::on_delete))
            .on_action(cx.listener(Self::on_red_up))
            .on_action(cx.listener(Self::on_red_down))
            .on_action(cx.listener(Self::on_green_up))
            .on_action(cx.listener(Self::on_green_down))
            .on_action(cx.listener(Self::on_blue_up))
            .on_action(cx.listener(Self::on_blue_down))
            .child(preview)
            .child(
                div()
                    .id("gradient-stop-list")
                    .flex()
                    .flex_wrap()
                    .gap(px(theme.spacing.xsmall))
                    .children(handles),
            )
            .child(controls)
            .child(div().flex().gap(px(theme.spacing.small)).child(add).child(remove))
    }
}

fn finite(n: f64) -> f64 {
    if n.is_finite() { n } else { 0.0 }
}
fn unit(n: f64) -> f64 {
    finite(n).clamp(0.0, 1.0)
}
fn byte(n: f64) -> u8 {
    (unit(n) * 255.0).round() as u8
}
pub fn normalize_stops(stops: Vec<GradientStop>) -> Vec<GradientStop> {
    let start =
        stops.iter().find(|s| s.position == 0.0).map(|s| s.colour.sanitized()).unwrap_or_default();
    let end =
        stops.iter().find(|s| s.position == 1.0).map(|s| s.colour.sanitized()).unwrap_or(start);
    let mut interior: Vec<_> = stops
        .into_iter()
        .filter(|s| s.position.is_finite())
        .filter(|s| s.position > MIN_GAP && s.position < 1.0 - MIN_GAP)
        .map(|s| GradientStop::new(s.position.clamp(0.0, 1.0), s.colour.sanitized()))
        .collect();
    interior.sort_by(|a, b| a.position.total_cmp(&b.position));
    let mut result = vec![GradientStop::new(0.0, start)];
    for stop in interior {
        if stop.position - result.last().unwrap().position >= MIN_GAP
            && result.len() < MAX_STOPS - 1
        {
            result.push(stop);
        }
    }
    result.push(GradientStop::new(1.0, end));
    result
}
pub fn sample_gradient(gradient: &Gradient, position: f64) -> Colour {
    let stops = &gradient.stops;
    if stops.len() < 2 {
        return stops.first().map(|s| s.colour).unwrap_or_default();
    }
    let x = unit(position);
    let i = stops.partition_point(|s| s.position < x).saturating_sub(1).min(stops.len() - 2);
    let a = stops[i];
    let b = stops[i + 1];
    let t = ((x - a.position) / (b.position - a.position).max(f64::EPSILON)).clamp(0.0, 1.0);
    Colour::new(
        a.colour.red + (b.colour.red - a.colour.red) * t,
        a.colour.green + (b.colour.green - a.colour.green) * t,
        a.colour.blue + (b.colour.blue - a.colour.blue) * t,
    )
    .sanitized()
}
fn widest_gap(stops: &[GradientStop]) -> (usize, f64) {
    let mut best = (1, 0.5, 0.0);
    for i in 0..stops.len() - 1 {
        let gap = stops[i + 1].position - stops[i].position;
        if gap > best.2 {
            best = (i + 1, (stops[i].position + stops[i + 1].position) / 2.0, gap);
        }
    }
    (best.0, best.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Focusable, Modifiers, MouseButton, TestAppContext, point, px};
    use std::{cell::RefCell, rc::Rc};
    #[test]
    fn gradient_normalizes_endpoints_sorted_colors_and_finite_channels() {
        let g = Gradient::new(vec![
            GradientStop::new(0.0, Colour::new(0.05, 0.06, 0.07)),
            GradientStop::new(0.8, Colour::new(2.0, -1.0, f64::NAN)),
            GradientStop::new(0.2, Colour::new(0.2, 0.3, 0.4)),
            GradientStop::new(1.0, Colour::new(0.9, 0.8, 0.7)),
        ]);
        assert_eq!(g.stops.len(), 4);
        assert_eq!(g.stops[0].position, 0.0);
        assert_eq!(g.stops[0].colour, Colour::new(0.05, 0.06, 0.07));
        assert_eq!(g.stops[1].position, 0.2);
        assert_eq!(g.stops[1].colour, Colour::new(0.2, 0.3, 0.4));
        assert_eq!(g.stops[2].position, 0.8);
        assert_eq!(g.stops[2].colour, Colour::new(1.0, 0.0, 0.0));
        assert_eq!(g.stops[3].position, 1.0);
        assert_eq!(g.stops[3].colour, Colour::new(0.9, 0.8, 0.7));
    }
    #[test]
    fn sampling_is_piecewise_linear_and_add_uses_widest_interval() {
        let g = Gradient::new(vec![
            GradientStop::new(0.0, Colour::new(0.0, 0.0, 0.0)),
            GradientStop::new(1.0, Colour::new(1.0, 0.5, 0.25)),
        ]);
        assert_eq!(sample_gradient(&g, 0.5), Colour::new(0.5, 0.25, 0.125));
        assert_eq!(widest_gap(&g.stops), (1, 0.5));
    }
    #[gpui_pre::test]
    fn gpui_keyboard_and_add_button_interactions(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (editor, visual) =
            cx.add_window_view(|_, _| GradientEditor::new("Fill gradient", Gradient::default()));
        let changes = Rc::new(RefCell::new(Vec::new()));
        let log = changes.clone();
        let _sub = visual.update(|_, app| {
            app.subscribe(&editor, move |_, e: &GradientChanged, _| {
                log.borrow_mut().push(e.clone())
            })
        });
        visual.update(|w, cx| w.draw(cx).clear(cx));
        visual.update(|w, cx| editor.focus_handle(cx).focus(w, cx));
        editor.update(visual, |e, cx| e.add_stop(cx));
        assert_eq!(editor.read_with(visual, |e, _| e.gradient.stops.len()), 3);
        editor.update(visual, |e, cx| e.select_stop(1, cx));
        visual.simulate_keystrokes("right");
        assert!(editor.read_with(visual, |e, _| e.gradient.stops[1].position) > 0.5);
        visual.simulate_keystrokes("b");
        assert_eq!(changes.borrow().len(), 3);
        visual.update(|w, cx| w.draw(cx).clear(cx));
        let add = visual.debug_bounds("gradient-add-stop").expect("add button is rendered");
        visual.simulate_click(add.center(), Modifiers::default());
        assert_eq!(editor.read_with(visual, |e, _| e.gradient.stops.len()), 4);
        visual.update(|w, cx| w.draw(cx).clear(cx));
        let bounds = visual.debug_bounds("mkit-gradient-preview").expect("preview is rendered");
        let start = point(
            bounds.origin.x + px(0.25 * f32::from(bounds.size.width)),
            bounds.origin.y + px(0.5 * f32::from(bounds.size.height)),
        );
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        assert!(editor.read_with(visual, |e, _| e.dragging));
        let dragged = point(
            bounds.origin.x + px(0.30 * f32::from(bounds.size.width)),
            bounds.origin.y + px(0.5 * f32::from(bounds.size.height)),
        );
        visual.simulate_mouse_move(dragged, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(dragged, MouseButton::Left, Modifiers::default());
        assert_eq!(editor.read_with(visual, |e, _| e.gradient.stops.len()), 4);
        assert!(
            (editor.read_with(visual, |e, _| e.gradient.stops[1].position) - 0.30).abs() < 0.01
        );
    }

    #[gpui_pre::test]
    fn controlled_edits_emit_complete_proposals_without_applying_them(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let initial = Gradient::default();
        let (editor, visual) =
            cx.add_window_view(|_, _| GradientEditor::controlled("Fill gradient", initial.clone()));
        let proposals = Rc::new(RefCell::new(Vec::new()));
        let log = proposals.clone();
        let _sub = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &GradientChanged, _| {
                log.borrow_mut().push(event.0.clone())
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        editor.update(visual, |view, cx| view.add_stop(cx));
        assert_eq!(editor.read_with(visual, |view, _| view.gradient().stops.len()), 2);
        assert_eq!(proposals.borrow().len(), 1);
        assert_eq!(proposals.borrow()[0].stops.len(), 3);
    }

    #[gpui_pre::test]
    fn controlled_keyboard_navigation_proposes_and_owner_echo_and_noop_are_silent(
        cx: &mut TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = Gradient::new(vec![
            GradientStop::new(0.0, Colour::new(0.0, 0.0, 0.0)),
            GradientStop::new(0.5, Colour::new(0.5, 0.2, 0.8)),
            GradientStop::new(1.0, Colour::new(1.0, 1.0, 1.0)),
        ]);
        let (editor, visual) =
            cx.add_window_view(|_, _| GradientEditor::controlled("Fill gradient", initial.clone()));
        let proposals = Rc::new(RefCell::new(Vec::new()));
        let selections = Rc::new(RefCell::new(Vec::new()));
        let proposal_log = proposals.clone();
        let selection_log = selections.clone();
        let _gradient_sub = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &GradientChanged, _| {
                proposal_log.borrow_mut().push(event.0.clone());
            })
        });
        let _selection_sub = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &StopSelected, _| {
                selection_log.borrow_mut().push(event.0);
            })
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            editor.focus_handle(cx).focus(window, cx);
        });

        visual.simulate_keystrokes("tab right");
        assert_eq!(editor.read_with(visual, |view, _| view.selected_stop()), 1);
        assert_eq!(editor.read_with(visual, |view, _| view.gradient().clone()), initial);
        assert_eq!(selections.borrow().as_slice(), &[1]);
        assert_eq!(proposals.borrow().len(), 1);
        let proposed = proposals.borrow()[0].clone();
        assert!((proposed.stops[1].position - 0.51).abs() < 1e-9);

        editor.update(visual, |view, cx| view.set_gradient(proposed.clone(), cx));
        assert_eq!(editor.read_with(visual, |view, _| view.gradient().clone()), proposed);
        editor.update(visual, |view, cx| view.move_stop(1, 0.51, cx));
        assert_eq!(proposals.borrow().len(), 1, "owner echo and equal move emit nothing");

        visual.simulate_keystrokes("tab shift-tab");
        assert_eq!(selections.borrow().as_slice(), &[1, 2, 1]);
        assert_eq!(editor.read_with(visual, |view, _| view.selected_stop()), 1);
    }

    #[gpui_pre::test]
    fn disabled_editor_ignores_keyboard_and_stop_selection_events(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = Gradient::new(vec![
            GradientStop::new(0.0, Colour::new(0.0, 0.0, 0.0)),
            GradientStop::new(0.5, Colour::new(0.5, 0.2, 0.8)),
            GradientStop::new(1.0, Colour::new(1.0, 1.0, 1.0)),
        ]);
        let (editor, visual) = cx.add_window_view(|_, _| {
            GradientEditor::new("Fill gradient", initial.clone()).disabled(true)
        });
        let proposals = Rc::new(RefCell::new(Vec::new()));
        let selections = Rc::new(RefCell::new(Vec::new()));
        let proposal_log = proposals.clone();
        let selection_log = selections.clone();
        let _gradient_sub = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &GradientChanged, _| {
                proposal_log.borrow_mut().push(event.0.clone());
            })
        });
        let _selection_sub = visual.update(|_, app| {
            app.subscribe(&editor, move |_, event: &StopSelected, _| {
                selection_log.borrow_mut().push(event.0);
            })
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            editor.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("tab r right delete");
        let stop = visual.debug_bounds("gradient-stop-1").expect("interior stop is rendered");
        visual.simulate_click(stop.center(), Modifiers::default());

        assert_eq!(editor.read_with(visual, |view, _| view.selected_stop()), 0);
        assert_eq!(editor.read_with(visual, |view, _| view.gradient().clone()), initial);
        assert!(proposals.borrow().is_empty());
        assert!(selections.borrow().is_empty());

        // Owner-provided state remains authoritative even while editing is disabled.
        let replacement = Gradient::default();
        editor.update(visual, |view, cx| view.set_gradient(replacement.clone(), cx));
        assert_eq!(editor.read_with(visual, |view, _| view.gradient().clone()), replacement);
        assert!(proposals.borrow().is_empty());
        assert!(selections.borrow().is_empty());
    }
}
