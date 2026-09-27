//! Single-value precision slider with reset and bipolar presentation.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Bounds, Context, DispatchPhase, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyBinding, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Render,
    Subscription, Window, actions, canvas, div, prelude::*, px, relative,
};
use mkit_core::theme::Theme;

pub const KEY_CONTEXT: &str = "PrecisionSlider";
actions!(
    precision_slider,
    [Increase, Decrease, Minimum, Maximum, FineIncrease, FineDecrease, Reset]
);

pub fn default_key_bindings() -> [KeyBinding; 7] {
    [
        KeyBinding::new("right", Increase, Some(KEY_CONTEXT)),
        KeyBinding::new("left", Decrease, Some(KEY_CONTEXT)),
        KeyBinding::new("home", Minimum, Some(KEY_CONTEXT)),
        KeyBinding::new("end", Maximum, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-right", FineIncrease, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-left", FineDecrease, Some(KEY_CONTEXT)),
        KeyBinding::new("r", Reset, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChangeRequested(pub f64);
impl EventEmitter<ChangeRequested> for PrecisionSlider {}

pub struct PrecisionSlider {
    label: String,
    value: f64,
    reset_value: f64,
    controlled: bool,
    min: f64,
    max: f64,
    step: f64,
    bipolar: bool,
    disabled: bool,
    dragging: bool,
    fine_drag: bool,
    drag_origin_x: f32,
    drag_origin_value: f64,
    bounds: Option<Bounds<Pixels>>,
    focus: Option<FocusHandle>,
    focus_subscription: Option<Subscription>,
}

impl PrecisionSlider {
    pub fn new(label: impl Into<String>, value: f64, min: f64, max: f64, step: f64) -> Self {
        let (min, max, step) = valid_domain(min, max, step);
        Self {
            label: label.into(),
            value: snap(value, min, max, step),
            reset_value: snap(value, min, max, step),
            controlled: false,
            min,
            max,
            step,
            bipolar: false,
            disabled: false,
            dragging: false,
            fine_drag: false,
            drag_origin_x: 0.0,
            drag_origin_value: 0.0,
            bounds: None,
            focus: None,
            focus_subscription: None,
        }
    }

    pub fn controlled(label: impl Into<String>, value: f64, min: f64, max: f64, step: f64) -> Self {
        let mut slider = Self::new(label, value, min, max, step);
        slider.controlled = true;
        slider
    }

    pub fn reset_value(mut self, value: f64) -> Self {
        self.reset_value = snap(value, self.min, self.max, self.step);
        self
    }
    pub fn bipolar(mut self, enabled: bool) -> Self {
        self.bipolar = enabled;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Apply a value accepted by the owner in controlled mode.
    pub fn set_value(&mut self, value: f64, cx: &mut Context<Self>) {
        self.value = snap(value, self.min, self.max, self.step / 10.0);
        cx.notify();
    }

    fn request(&mut self, value: f64, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let next = snap(value, self.min, self.max, self.step / 10.0);
        if (next - self.value).abs() <= f64::EPSILON {
            return;
        }
        if !self.controlled {
            self.value = next;
            cx.notify();
        }
        cx.emit(ChangeRequested(next));
    }
    fn reset(&mut self, _: &Reset, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.reset_value, cx);
    }
    fn inc(&mut self, _: &Increase, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.value + self.step, cx);
    }
    fn dec(&mut self, _: &Decrease, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.value - self.step, cx);
    }
    fn fine_inc(&mut self, _: &FineIncrease, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.value + self.step / 10.0, cx);
    }
    fn fine_dec(&mut self, _: &FineDecrease, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.value - self.step / 10.0, cx);
    }
    fn min_value(&mut self, _: &Minimum, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.min, cx);
    }
    fn max_value(&mut self, _: &Maximum, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.max, cx);
    }

    fn pointer_value(&self, x: f32, fine: bool) -> Option<f64> {
        let b = self.bounds?;
        let span = f32::from(b.size.width);
        if span <= 0.0 {
            return None;
        }
        let fraction = ((x - f32::from(b.origin.x)) / span).clamp(0.0, 1.0) as f64;
        let raw = if fine {
            self.drag_origin_value
                + (x - self.drag_origin_x) as f64 / span as f64 * (self.max - self.min) / 10.0
        } else {
            self.min + fraction * (self.max - self.min)
        };
        Some(snap(raw, self.min, self.max, if fine { self.step / 10.0 } else { self.step }))
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
        let Some(bounds) = self.bounds else { return };
        if event.click_count >= 2 {
            self.request(self.reset_value, cx);
            self.dragging = false;
            return;
        }
        self.dragging = true;
        self.fine_drag = event.modifiers.shift;
        self.drag_origin_x = f32::from(event.position.x);
        self.drag_origin_value = self.value;
        if let Some(focus) = self.focus.as_ref() {
            window.focus(focus, cx);
        }
        if let Some(value) = self.pointer_value(f32::from(event.position.x), self.fine_drag) {
            self.request(value, cx);
        }
        self.bounds = Some(bounds);
    }
    fn pointer_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if !self.dragging || !event.dragging() {
            return;
        }
        if let Some(value) = self.pointer_value(f32::from(event.position.x), self.fine_drag) {
            self.request(value, cx);
        }
    }
    fn pointer_up(&mut self, event: &MouseUpEvent) {
        if event.button == MouseButton::Left {
            self.dragging = false;
            self.fine_drag = false;
        }
    }
}

impl Focusable for PrecisionSlider {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone().expect("precision slider focus initialized during render")
    }
}

impl Render for PrecisionSlider {
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
        let fraction = |v: f64| ((v - self.min) / (self.max - self.min)).clamp(0.0, 1.0) as f32;
        let pos = fraction(self.value);
        let zero = if self.bipolar { fraction(0.0) } else { 0.0 };
        let fill_left = if self.bipolar { pos.min(zero) } else { 0.0 };
        let fill_width = if self.bipolar { (pos - zero).abs() } else { pos };
        let thumb_size = theme.controls.xsmall * 0.65;
        let active = !self.disabled;
        let show_tooltip = self.dragging;
        let display_value = format_value(self.value);
        let entity = cx.entity();
        let mut track = div()
            .id("precision-slider-track")
            .debug_selector(|| "mkit-precision-slider-track".into())
            .relative()
            .w_full()
            .h(px(theme.controls.small))
            .flex()
            .items_center()
            .child(
                div()
                    .w_full()
                    .h(px(theme.borders.strong))
                    .rounded(px(theme.radii.pill))
                    .bg(theme.colors.border)
                    .child(
                        div()
                            .h_full()
                            .ml(relative(fill_left))
                            .w(relative(fill_width))
                            .rounded(px(theme.radii.pill))
                            .bg(if active { theme.colors.accent } else { theme.colors.disabled }),
                    ),
            )
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .role(gpui_pre::accesskit::Role::Slider)
            .aria_label(self.label.clone())
            .aria_numeric_value(self.value)
            .aria_min_numeric_value(self.min)
            .aria_max_numeric_value(self.max)
            .when(self.disabled, |el| {
                el.aria_description("Unavailable")
                    .a11y_synthetic_children(|b| b.parent_node().set_disabled())
            })
            .when(!self.disabled, |el| el.focus_visible(|el| el.bg(theme.colors.focus)))
            .on_action(cx.listener(Self::inc))
            .on_action(cx.listener(Self::dec))
            .on_action(cx.listener(Self::fine_inc))
            .on_action(cx.listener(Self::fine_dec))
            .on_action(cx.listener(Self::min_value))
            .on_action(cx.listener(Self::max_value))
            .on_action(cx.listener(Self::reset));
        track = track.child(
            div()
                .id("precision-slider-thumb")
                .absolute()
                .left(relative(pos))
                .ml(px(-thumb_size / 2.0))
                .top(px((theme.controls.small - thumb_size) / 2.0))
                .size(px(thumb_size))
                .rounded(px(theme.radii.pill))
                .border(px(theme.borders.regular))
                .border_color(if self.disabled {
                    theme.colors.disabled
                } else {
                    theme.colors.border
                })
                .bg(theme.colors.surface),
        );
        if show_tooltip {
            track = track.child(
                div()
                    .id("precision-slider-tooltip")
                    .absolute()
                    .left(relative(pos))
                    .ml(px(-theme.controls.small * 0.75))
                    .bottom(px(theme.controls.small))
                    .px(px(theme.spacing.xsmall))
                    .py(px(theme.spacing.xsmall))
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.hairline))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.surface)
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.caption))
                    .child(display_value),
            );
        }
        let entity2 = entity.clone();
        track.child(
            canvas(
                |_, _, _| (),
                move |bounds, (), window, cx| {
                    entity2.update(cx, |slider, _| slider.bounds = Some(bounds));
                    let down = entity.clone();
                    window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                        if phase == DispatchPhase::Capture && bounds.contains(&event.position) {
                            down.update(cx, |slider, cx| {
                                slider.bounds = Some(bounds);
                                slider.pointer_down(event, window, cx);
                            });
                        }
                    });
                    let moved = entity.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture {
                            moved.update(cx, |slider, cx| slider.pointer_move(event, cx));
                        }
                    });
                    let up = entity.clone();
                    window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture {
                            up.update(cx, |slider, _| slider.pointer_up(event));
                        }
                    });
                },
            )
            .absolute()
            .inset_0(),
        )
    }
}

fn valid_domain(min: f64, max: f64, step: f64) -> (f64, f64, f64) {
    let min = if min.is_finite() { min } else { 0.0 };
    let max = if max.is_finite() && max > min { max } else { min + 1.0 };
    let step = if step.is_finite() && step > 0.0 { step } else { 1.0 };
    (min, max, step)
}
fn snap(value: f64, min: f64, max: f64, step: f64) -> f64 {
    if !value.is_finite() {
        return min;
    }
    let quantum = if step.is_finite() && step > f64::EPSILON { step } else { f64::EPSILON };
    (((value - min) / quantum).round().mul_add(quantum, min)).clamp(min, max)
}
fn format_value(value: f64) -> String {
    if value.fract().abs() < 0.000001 { format!("{value:.0}") } else { format!("{value:.2}") }
}

#[cfg(test)]
mod tests {
    use super::{
        ChangeRequested, PrecisionSlider, default_key_bindings, format_value, snap, valid_domain,
    };
    use gpui_pre::{Focusable, Modifiers, MouseButton, MouseDownEvent, TestAppContext, point, px};
    use std::{cell::RefCell, rc::Rc};
    #[test]
    fn values_clamp_to_step_and_invalid_domains_are_repaired() {
        let (min, max, step) = valid_domain(0.0, 10.0, 1.0);
        assert_eq!(snap(-1.0, min, max, step), 0.0);
        assert_eq!(snap(4.6, min, max, step), 5.0);
        assert_eq!(valid_domain(3.0, 2.0, 0.0), (3.0, 4.0, 1.0));
    }
    #[test]
    fn tooltip_value_is_compact() {
        assert_eq!(format_value(2.0), "2");
        assert_eq!(format_value(2.25), "2.25");
    }

    #[gpui_pre::test]
    fn keyboard_steps_and_reset_emit_value_requests(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (slider, visual) = cx.add_window_view(|_, _| {
            PrecisionSlider::new("Exposure", 5.0, 0.0, 10.0, 1.0).reset_value(2.0)
        });
        let requests = Rc::new(RefCell::new(Vec::new()));
        let request_log = requests.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&slider, move |_, event: &ChangeRequested, _| {
                request_log.borrow_mut().push(event.0);
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| slider.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("right r");
        assert_eq!(slider.read_with(visual, |slider, _| slider.value()), 2.0);
        assert_eq!(*requests.borrow(), vec![6.0, 2.0]);
    }

    #[gpui_pre::test]
    fn pointer_drag_fine_modifier_and_double_click_reset(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let (slider, visual) = cx.add_window_view(|_, _| {
            PrecisionSlider::new("Exposure", 5.0, 0.0, 10.0, 1.0).reset_value(2.0)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let bounds = visual.debug_bounds("mkit-precision-slider-track").unwrap();
        let x_at = |fraction: f32| bounds.origin.x + px(f32::from(bounds.size.width) * fraction);
        let y = bounds.center().y;
        let start = point(x_at(0.7), y);
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(
            point(x_at(0.7), y),
            Some(MouseButton::Left),
            Modifiers::default(),
        );
        visual.simulate_mouse_move(
            point(x_at(0.8), y),
            Some(MouseButton::Left),
            Modifiers::default(),
        );
        visual.simulate_mouse_up(point(x_at(0.8), y), MouseButton::Left, Modifiers::default());
        assert_eq!(slider.read_with(visual, |slider, _| slider.value()), 8.0);

        let fine_start = point(x_at(0.8), y);
        visual.simulate_mouse_down(
            fine_start,
            MouseButton::Left,
            Modifiers { shift: true, ..Default::default() },
        );
        visual.simulate_mouse_move(
            point(x_at(0.9), y),
            Some(MouseButton::Left),
            Modifiers { shift: true, ..Default::default() },
        );
        visual.simulate_mouse_up(
            point(x_at(0.9), y),
            MouseButton::Left,
            Modifiers { shift: true, ..Default::default() },
        );
        assert!((slider.read_with(visual, |slider, _| slider.value()) - 8.1).abs() < 1e-9);

        visual.simulate_event(MouseDownEvent {
            position: point(x_at(0.9), y),
            modifiers: Modifiers::default(),
            button: MouseButton::Left,
            click_count: 2,
            first_mouse: false,
        });
        assert_eq!(slider.read_with(visual, |slider, _| slider.value()), 2.0);
    }

    #[gpui_pre::test]
    fn controlled_slider_emits_request_without_mutating_display(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (slider, visual) =
            cx.add_window_view(|_, _| PrecisionSlider::controlled("Exposure", 5.0, 0.0, 10.0, 1.0));
        let requests = Rc::new(RefCell::new(Vec::new()));
        let request_log = requests.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&slider, move |_, event: &ChangeRequested, _| {
                request_log.borrow_mut().push(event.0);
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| slider.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("right");
        assert_eq!(slider.read_with(visual, |slider, _| slider.value()), 5.0);
        assert_eq!(*requests.borrow(), vec![6.0]);
        slider.update(visual, |slider, cx| slider.set_value(6.0, cx));
        assert_eq!(slider.read_with(visual, |slider, _| slider.value()), 6.0);
    }

    #[gpui_pre::test]
    fn endpoints_and_disabled_input_obey_event_contract(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (slider, visual) =
            cx.add_window_view(|_, _| PrecisionSlider::new("Exposure", 5.0, 0.0, 10.0, 1.0));
        let requests = Rc::new(RefCell::new(Vec::new()));
        let request_log = requests.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&slider, move |_, event: &ChangeRequested, _| {
                request_log.borrow_mut().push(event.0);
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| slider.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("end home");
        assert_eq!(slider.read_with(visual, |slider, _| slider.value()), 0.0);
        assert_eq!(*requests.borrow(), vec![10.0, 0.0]);

        let (disabled, disabled_visual) = cx.add_window_view(|_, _| {
            PrecisionSlider::new("Exposure", 5.0, 0.0, 10.0, 1.0).disabled(true)
        });
        let disabled_requests = Rc::new(RefCell::new(Vec::new()));
        let disabled_log = disabled_requests.clone();
        let _disabled_subscription = disabled_visual.update(|_, app| {
            app.subscribe(&disabled, move |_, event: &ChangeRequested, _| {
                disabled_log.borrow_mut().push(event.0);
            })
        });
        disabled_visual.update(|window, cx| window.draw(cx).clear(cx));
        disabled_visual.update(|window, cx| disabled.focus_handle(cx).focus(window, cx));
        disabled_visual.simulate_keystrokes("right end r");
        let bounds = disabled_visual.debug_bounds("mkit-precision-slider-track").unwrap();
        disabled_visual.simulate_mouse_down(
            bounds.center(),
            MouseButton::Left,
            Modifiers::default(),
        );
        disabled_visual.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::default());
        assert_eq!(disabled.read_with(disabled_visual, |slider, _| slider.value()), 5.0);
        assert!(disabled_requests.borrow().is_empty());
    }
}
