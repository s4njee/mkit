//! Numeric slider supporting stepped single values and two-thumb ranges.
extern crate gpui_pre as gpui;

use gpui_pre::{
    Bounds, Context, DispatchPhase, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Render, Subscription,
    Window, actions, canvas, div, prelude::*, px, relative,
};
use mkit_core::theme::Theme;

pub const KEY_CONTEXT: &str = "Slider";
actions!(
    slider,
    [
        Increase,
        Decrease,
        PageIncrease,
        PageDecrease,
        Minimum,
        Maximum,
        FineIncrease,
        FineDecrease,
        MicroIncrease,
        MicroDecrease,
        CancelGesture
    ]
);

pub fn default_key_bindings() -> [KeyBinding; 11] {
    [
        KeyBinding::new("right", Increase, Some(KEY_CONTEXT)),
        KeyBinding::new("left", Decrease, Some(KEY_CONTEXT)),
        KeyBinding::new("pageup", PageIncrease, Some(KEY_CONTEXT)),
        KeyBinding::new("pagedown", PageDecrease, Some(KEY_CONTEXT)),
        KeyBinding::new("home", Minimum, Some(KEY_CONTEXT)),
        KeyBinding::new("end", Maximum, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-right", FineIncrease, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-left", FineDecrease, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-right", MicroIncrease, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-left", MicroDecrease, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", CancelGesture, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChangeRequested(pub Vec<f64>);
impl EventEmitter<ChangeRequested> for Slider {}

#[derive(Clone, Debug, PartialEq)]
pub struct InteractionStarted(pub Vec<f64>);
impl EventEmitter<InteractionStarted> for Slider {}

#[derive(Clone, Debug, PartialEq)]
pub struct InteractionEnded(pub Vec<f64>);
impl EventEmitter<InteractionEnded> for Slider {}

#[derive(Clone, Debug, PartialEq)]
pub struct InteractionCancelled(pub Vec<f64>);
impl EventEmitter<InteractionCancelled> for Slider {}

pub struct Slider {
    label: String,
    values: Vec<f64>,
    controlled: bool,
    min: f64,
    max: f64,
    step: f64,
    disabled: bool,
    active: usize,
    focus: Vec<FocusHandle>,
    focus_subscriptions: Vec<Subscription>,
    last_bounds: Option<Bounds<Pixels>>,
    dragging: bool,
    gesture_start: Option<Vec<f64>>,
    gesture_latest: Option<Vec<f64>>,
}

impl Slider {
    pub fn new(label: impl Into<String>, value: f64, min: f64, max: f64, step: f64) -> Self {
        let (min, max, step) = valid_domain(min, max, step);
        Self {
            label: label.into(),
            values: vec![snap(value, min, max, step)],
            controlled: false,
            min,
            max,
            step,
            disabled: false,
            active: 0,
            focus: Vec::new(),
            focus_subscriptions: Vec::new(),
            last_bounds: None,
            dragging: false,
            gesture_start: None,
            gesture_latest: None,
        }
    }

    pub fn range(
        label: impl Into<String>,
        low: f64,
        high: f64,
        min: f64,
        max: f64,
        step: f64,
    ) -> Self {
        let mut slider = Self::new(label, low, min, max, step);
        let lower = snap(low.min(high), slider.min, slider.max, slider.step);
        let upper = snap(high.max(low), slider.min, slider.max, slider.step);
        slider.values = vec![lower.min(upper), lower.max(upper)];
        slider
    }

    /// Create a controlled slider with either one value or two ordered range values.
    pub fn controlled(
        label: impl Into<String>,
        values: Vec<f64>,
        min: f64,
        max: f64,
        step: f64,
    ) -> Self {
        let initial = values.first().copied().unwrap_or(min);
        let mut slider = Self::new(label, initial, min, max, step);
        slider.values = normalize_values(values, slider.min, slider.max, slider.step);
        slider.controlled = true;
        slider
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Apply values supplied by the owner after a controlled change request.
    pub fn set_value(&mut self, values: Vec<f64>, cx: &mut Context<Self>) {
        self.values = normalize_values(values, self.min, self.max, self.step / 100.0);
        self.active = self.active.min(self.values.len().saturating_sub(1));
        cx.notify();
    }

    pub fn values(&self) -> &[f64] {
        &self.values
    }

    fn request_value(&mut self, index: usize, requested: f64, cx: &mut Context<Self>) -> bool {
        if self.disabled || index >= self.values.len() {
            return false;
        }
        let Some(next_values) =
            requested_values(&self.values, index, requested, self.min, self.max, self.step / 100.0)
        else {
            return false;
        };
        if !self.controlled {
            self.values = next_values.clone();
            cx.notify();
        }
        if self.gesture_start.is_some() {
            self.gesture_latest = Some(next_values.clone());
        }
        cx.emit(ChangeRequested(next_values));
        true
    }

    fn adjust(&mut self, delta: f64, cx: &mut Context<Self>) {
        if self.values.is_empty() || self.disabled {
            return;
        }
        let index = self.active.min(self.values.len() - 1);
        let old = self.values[index];
        let requested = if delta == f64::MAX {
            if index + 1 < self.values.len() { self.values[index + 1] } else { self.max }
        } else if delta == f64::MIN {
            if index > 0 { self.values[index - 1] } else { self.min }
        } else {
            old + delta
        };
        let initial = self.values.clone();
        let Some(next_values) =
            requested_values(&initial, index, requested, self.min, self.max, self.step / 100.0)
        else {
            return;
        };
        self.gesture_start = Some(initial.clone());
        self.gesture_latest = Some(initial.clone());
        cx.emit(InteractionStarted(initial));
        let changed = self.request_value(index, requested, cx);
        if changed {
            cx.emit(InteractionEnded(self.gesture_latest.take().unwrap_or(next_values)));
        } else {
            self.gesture_latest = None;
        }
        self.gesture_start = None;
    }

    fn set_from_pointer(&mut self, x: f32, cx: &mut Context<Self>) {
        let Some(bounds) = self.last_bounds else { return };
        let index = self.active.min(self.values.len().saturating_sub(1));
        let value = pointer_value(
            x,
            f32::from(bounds.origin.x),
            f32::from(bounds.size.width),
            self.min,
            self.max,
            self.step,
        );
        let (Some(value), true) = (value, !self.values.is_empty() && !self.disabled) else {
            return;
        };
        self.request_value(index, value, cx);
    }

    fn pointer_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || event.button != MouseButton::Left || self.values.is_empty() {
            return;
        }
        let Some(bounds) = self.last_bounds else { return };
        let Some(target) = pointer_value(
            f32::from(event.position.x),
            f32::from(bounds.origin.x),
            f32::from(bounds.size.width),
            self.min,
            self.max,
            self.step,
        ) else {
            return;
        };
        self.active = nearest_thumb(&self.values, target);
        self.dragging = true;
        self.gesture_start = Some(self.values.clone());
        self.gesture_latest = Some(self.values.clone());
        cx.emit(InteractionStarted(self.values.clone()));
        if let Some(focus) = self.focus.get(self.active) {
            window.focus(focus, cx);
        }
        self.set_from_pointer(f32::from(event.position.x), cx);
    }

    fn pointer_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.dragging && event.dragging() {
            self.set_from_pointer(f32::from(event.position.x), cx);
        }
    }

    fn pointer_up(&mut self, event: &MouseUpEvent, cx: &mut Context<Self>) {
        if event.button == MouseButton::Left {
            self.dragging = false;
            if self.gesture_start.take().is_some() {
                cx.emit(InteractionEnded(
                    self.gesture_latest.take().unwrap_or_else(|| self.values.clone()),
                ));
            }
        }
    }

    fn cancel_gesture(&mut self, _: &CancelGesture, _: &mut Window, cx: &mut Context<Self>) {
        if !self.dragging {
            return;
        }
        self.dragging = false;
        if let Some(original) = self.gesture_start.take() {
            self.gesture_latest = None;
            if !self.controlled {
                self.values = original.clone();
                cx.notify();
            }
            cx.emit(InteractionCancelled(original));
        }
    }

    fn focus_thumb(&mut self, index: usize, _: &mut Window, cx: &mut Context<Self>) {
        self.active = index.min(self.values.len().saturating_sub(1));
        cx.notify();
    }

    fn inc(&mut self, _: &Increase, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(self.step, cx)
    }
    fn dec(&mut self, _: &Decrease, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(-self.step, cx)
    }
    fn pinc(&mut self, _: &PageIncrease, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(self.step * 10.0, cx)
    }
    fn pdec(&mut self, _: &PageDecrease, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(-self.step * 10.0, cx)
    }
    fn fine_inc(&mut self, _: &FineIncrease, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(self.step / 10.0, cx)
    }
    fn fine_dec(&mut self, _: &FineDecrease, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(-self.step / 10.0, cx)
    }
    fn micro_inc(&mut self, _: &MicroIncrease, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(self.step / 100.0, cx)
    }
    fn micro_dec(&mut self, _: &MicroDecrease, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(-self.step / 100.0, cx)
    }
    fn home(&mut self, _: &Minimum, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(f64::MIN, cx)
    }
    fn end(&mut self, _: &Maximum, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(f64::MAX, cx)
    }
}

impl Focusable for Slider {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.first().cloned().expect("slider focus initialized during render")
    }
}

impl Render for Slider {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        while self.focus.len() < self.values.len() {
            let index = self.focus.len();
            let handle = cx.focus_handle().tab_index(index as isize).tab_stop(!self.disabled);
            let subscription = cx.on_focus(&handle, window, move |slider, window, cx| {
                slider.focus_thumb(index, window, cx)
            });
            self.focus.push(handle);
            self.focus_subscriptions.push(subscription);
        }
        self.focus.truncate(self.values.len());
        self.focus_subscriptions.truncate(self.values.len());
        let theme = *cx.global::<Theme>();
        let entity = cx.entity();
        let min = self.min;
        let max = self.max;
        let disabled = self.disabled;
        let values = self.values.clone();
        let label = self.label.clone();
        let focus = self.focus.clone();
        let pct = |value: f64| {
            if max > min { ((value - min) / (max - min)).clamp(0.0, 1.0) as f32 } else { 0.0 }
        };
        let start = if values.len() == 1 { min } else { values.first().copied().unwrap_or(min) };
        let end = values.last().copied().unwrap_or(max);
        let track_height = theme.borders.strong;
        let thumb_size = theme.controls.xsmall * 0.6;

        let mut track = div()
            .id("slider-track")
            .debug_selector(|| "mkit-slider-track".into())
            .relative()
            .w_full()
            .h(px(theme.controls.small))
            .flex()
            .items_center()
            .child(
                div()
                    .w_full()
                    .h(px(track_height))
                    .rounded(px(theme.radii.pill))
                    .bg(theme.colors.border)
                    .child(
                        div()
                            .h_full()
                            .ml(relative(pct(start)))
                            .w(relative((pct(end) - pct(start)).max(0.0)))
                            .rounded(px(theme.radii.pill))
                            .bg(if disabled { theme.colors.disabled } else { theme.colors.accent }),
                    ),
            );

        for (index, value) in values.iter().copied().enumerate() {
            let value_name = if values.len() == 1 {
                label.clone()
            } else if index == 0 {
                format!("{label} minimum")
            } else {
                format!("{label} maximum")
            };
            let focused = self.active == index;
            let thumb = div()
                .id(if index == 0 { "slider-thumb-0" } else { "slider-thumb-1" })
                .absolute()
                .left(relative(pct(value)))
                .ml(px(-thumb_size / 2.0))
                .top(px((theme.controls.small - thumb_size) / 2.0))
                .size(px(thumb_size))
                .rounded(px(theme.radii.pill))
                .border(px(if focused { theme.borders.regular } else { theme.borders.hairline }))
                .border_color(if focused { theme.colors.focus } else { theme.colors.text_muted })
                .bg(if disabled { theme.colors.disabled } else { theme.colors.surface })
                .role(gpui_pre::accesskit::Role::Slider)
                .aria_label(value_name)
                .aria_numeric_value(value)
                .aria_min_numeric_value(min)
                .aria_max_numeric_value(max)
                .when(disabled, |el| el.aria_description("Unavailable"))
                .when(disabled, |el| {
                    el.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
                })
                .when(!disabled, |el| el.track_focus(&focus[index]))
                .key_context(KEY_CONTEXT)
                .on_action(cx.listener(Self::inc))
                .on_action(cx.listener(Self::dec))
                .on_action(cx.listener(Self::pinc))
                .on_action(cx.listener(Self::pdec))
                .on_action(cx.listener(Self::home))
                .on_action(cx.listener(Self::end))
                .on_action(cx.listener(Self::fine_inc))
                .on_action(cx.listener(Self::fine_dec))
                .on_action(cx.listener(Self::micro_inc))
                .on_action(cx.listener(Self::micro_dec))
                .on_action(cx.listener(Self::cancel_gesture));
            track = track.child(thumb);
        }

        let entity = entity.clone();
        track.child(
            canvas(
                |_, _, _| (),
                move |bounds, (), window, cx| {
                    let down_target = entity.clone();
                    window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                        if phase == DispatchPhase::Capture
                            && event.button == MouseButton::Left
                            && bounds.contains(&event.position)
                        {
                            down_target.update(cx, |slider, cx| {
                                slider.last_bounds = Some(bounds);
                                slider.pointer_down(event, window, cx);
                            });
                        }
                    });
                    let move_target = entity.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                        if phase == DispatchPhase::Capture && event.dragging() {
                            move_target
                                .update(cx, |slider, cx| slider.pointer_move(event, window, cx));
                        }
                    });
                    let up_target = entity.clone();
                    window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture {
                            up_target.update(cx, |slider, cx| slider.pointer_up(event, cx));
                        }
                    });
                    entity.update(cx, |slider, _| slider.last_bounds = Some(bounds));
                },
            )
            .absolute()
            .inset_0(),
        )
    }
}

fn valid_domain(min: f64, max: f64, step: f64) -> (f64, f64, f64) {
    let min = if min.is_finite() { min } else { 0.0 };
    let max = if max.is_finite() { max.max(min) } else { min };
    let step = if step.is_finite() && step > 0.0 { step } else { 1.0 };
    (min, max, step)
}

fn normalize_values(values: Vec<f64>, min: f64, max: f64, step: f64) -> Vec<f64> {
    let mut values: Vec<_> = values.into_iter().take(2).map(|v| snap(v, min, max, step)).collect();
    if values.is_empty() {
        values.push(min);
    }
    if values.len() == 2 {
        vec![values[0].min(values[1]), values[0].max(values[1])]
    } else {
        values
    }
}

fn snap(value: f64, min: f64, max: f64, step: f64) -> f64 {
    quantize(value, min, max, step)
}

fn nearly_equal(a: f64, b: f64) -> bool {
    (a - b).abs() <= f64::EPSILON * a.abs().max(b.abs()).max(1.0) * 4.0
}

fn nearest_thumb(values: &[f64], target: f64) -> usize {
    values
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| (**a - target).abs().total_cmp(&(**b - target).abs()))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn requested_values(
    values: &[f64],
    index: usize,
    requested: f64,
    min: f64,
    max: f64,
    quantum: f64,
) -> Option<Vec<f64>> {
    let current = *values.get(index)?;
    let lower = if index > 0 { values[index - 1] } else { min };
    let upper = if index + 1 < values.len() { values[index + 1] } else { max };
    let next = quantize(requested, lower, upper, quantum);
    if nearly_equal(next, current) {
        return None;
    }
    let mut result = values.to_vec();
    result[index] = next;
    Some(result)
}

fn pointer_value(x: f32, origin: f32, width: f32, min: f64, max: f64, step: f64) -> Option<f64> {
    if width <= 0.0 || !width.is_finite() || max <= min {
        return None;
    }
    let fraction = ((x - origin) / width).clamp(0.0, 1.0) as f64;
    Some(snap(min + fraction * (max - min), min, max, step))
}

/// Clamp and snap a slider value to the nearest quantum relative to its minimum.
pub fn quantize(value: f64, min: f64, max: f64, quantum: f64) -> f64 {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() || max <= min {
        return min;
    }
    let q = if quantum.is_finite() && quantum.abs() > f64::EPSILON {
        quantum.abs()
    } else {
        f64::EPSILON
    };
    (((value - min) / q).round().mul_add(q, min)).clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::{
        ChangeRequested, InteractionCancelled, InteractionEnded, InteractionStarted, Slider,
        nearest_thumb, pointer_value, quantize, requested_values,
    };
    use gpui_pre::{Focusable, MouseButton, TestAppContext, point, px};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn quantize_clamps_and_supports_regular_fine_and_micro_steps() {
        assert_eq!(quantize(-2.0, 0.0, 10.0, 1.0), 0.0);
        assert_eq!(quantize(10.2, 0.0, 10.0, 0.1), 10.0);
        assert!((quantize(2.34, 0.0, 10.0, 0.01) - 2.34).abs() < 1e-9);
        assert!((quantize(2.1 + 0.1, 0.0, 10.0, 0.01) - 2.2).abs() < 1e-9);
    }

    #[test]
    fn range_pointer_selection_chooses_nearest_thumb_with_lower_tie_break() {
        let values = [25.0, 75.0];
        assert_eq!(nearest_thumb(&values, 24.0), 0);
        assert_eq!(nearest_thumb(&values, 76.0), 1);
        assert_eq!(nearest_thumb(&values, 50.0), 0);
    }

    #[test]
    fn requests_adjust_each_range_thumb_without_crossing_and_skip_noops() {
        let current = [20.0, 80.0];
        assert_eq!(requested_values(&current, 0, 35.0, 0.0, 100.0, 0.01), Some(vec![35.0, 80.0]));
        assert_eq!(requested_values(&current, 1, 65.0, 0.0, 100.0, 0.01), Some(vec![20.0, 65.0]));
        assert_eq!(requested_values(&current, 0, 100.0, 0.0, 100.0, 0.01), Some(vec![80.0, 80.0]));
        assert_eq!(requested_values(&current, 1, 0.0, 0.0, 100.0, 0.01), Some(vec![20.0, 20.0]));
        assert_eq!(requested_values(&current, 0, 20.0, 0.0, 100.0, 0.01), None);
        assert_eq!(current, [20.0, 80.0]);
    }

    #[test]
    fn drag_math_maps_track_and_clamps_outside_it() {
        assert_eq!(pointer_value(100.0, 0.0, 100.0, 0.0, 100.0, 10.0), Some(100.0));
        assert_eq!(pointer_value(-10.0, 0.0, 100.0, 0.0, 100.0, 10.0), Some(0.0));
        assert_eq!(pointer_value(37.0, 10.0, 100.0, 0.0, 100.0, 5.0), Some(25.0));
        assert_eq!(pointer_value(4.0, 0.0, 0.0, 0.0, 100.0, 1.0), None);
    }

    #[gpui_pre::test]
    fn keyboard_change_emits_start_change_end_and_controlled_value_waits_for_owner(
        cx: &mut TestAppContext,
    ) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(super::default_key_bindings());
        });
        let (slider, visual) =
            cx.add_window_view(|_, _| Slider::controlled("Exposure", vec![5.0], 0.0, 10.0, 1.0));
        let events = Rc::new(RefCell::new(Vec::new()));
        let start_events = events.clone();
        let _started = visual.update(|_, app| {
            app.subscribe(&slider, move |_, _: &InteractionStarted, _| {
                start_events.borrow_mut().push("start")
            })
        });
        let change_events = events.clone();
        let _changed = visual.update(|_, app| {
            app.subscribe(&slider, move |_, _: &ChangeRequested, _| {
                change_events.borrow_mut().push("change")
            })
        });
        let end_events = events.clone();
        let _ended = visual.update(|_, app| {
            app.subscribe(&slider, move |_, _: &InteractionEnded, _| {
                end_events.borrow_mut().push("end")
            })
        });
        visual.update(|window, app| {
            window.draw(app).clear(app);
            slider.focus_handle(app).focus(window, app);
        });
        visual.simulate_keystrokes("right");
        assert_eq!(*events.borrow(), ["start", "change", "end"]);
        assert_eq!(slider.read_with(visual, |slider, _| slider.values().to_vec()), [5.0]);
    }

    #[gpui_pre::test]
    fn pointer_gesture_ends_and_escape_restores_original_controlled_values(
        cx: &mut TestAppContext,
    ) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(super::default_key_bindings());
        });
        let (slider, visual) =
            cx.add_window_view(|_, _| Slider::controlled("Exposure", vec![5.0], 0.0, 10.0, 1.0));
        let events = Rc::new(RefCell::new(Vec::new()));
        let start_events = events.clone();
        let _started = visual.update(|_, app| {
            app.subscribe(&slider, move |_, _: &InteractionStarted, _| {
                start_events.borrow_mut().push("start")
            })
        });
        let change_events = events.clone();
        let _changed = visual.update(|_, app| {
            app.subscribe(&slider, move |_, _: &ChangeRequested, _| {
                change_events.borrow_mut().push("change")
            })
        });
        let end_events = events.clone();
        let _ended = visual.update(|_, app| {
            app.subscribe(&slider, move |_, _: &InteractionEnded, _| {
                end_events.borrow_mut().push("end")
            })
        });
        let cancel_events = events.clone();
        let cancelled_values = Rc::new(RefCell::new(Vec::new()));
        let cancelled_values_out = cancelled_values.clone();
        let _cancelled = visual.update(|_, app| {
            app.subscribe(&slider, move |_, event: &InteractionCancelled, _| {
                cancel_events.borrow_mut().push("cancel");
                *cancelled_values_out.borrow_mut() = event.0.clone();
            })
        });
        visual.update(|window, app| window.draw(app).clear(app));
        let bounds = visual.debug_bounds("mkit-slider-track").expect("slider track rendered");
        let start = point(bounds.left() + px(140.), bounds.center().y);
        visual.simulate_mouse_down(start, MouseButton::Left, Default::default());
        visual.simulate_mouse_move(
            start + point(px(30.), px(0.)),
            Some(MouseButton::Left),
            Default::default(),
        );
        visual.simulate_mouse_up(
            start + point(px(30.), px(0.)),
            MouseButton::Left,
            Default::default(),
        );
        assert_eq!(*events.borrow(), ["start", "change", "change", "end"]);

        events.borrow_mut().clear();
        visual.simulate_mouse_down(start, MouseButton::Left, Default::default());
        visual.simulate_mouse_move(
            start + point(px(20.), px(0.)),
            Some(MouseButton::Left),
            Default::default(),
        );
        visual.simulate_keystrokes("escape");
        assert_eq!(*events.borrow(), ["start", "change", "change", "cancel"]);
        assert_eq!(*cancelled_values.borrow(), [5.0]);
        assert_eq!(slider.read_with(visual, |slider, _| slider.values().to_vec()), [5.0]);
    }

    #[gpui_pre::test]
    fn uncontrolled_escape_restores_the_values_changed_during_pointer_gesture(
        cx: &mut TestAppContext,
    ) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(super::default_key_bindings());
        });
        let (slider, visual) =
            cx.add_window_view(|_, _| Slider::new("Exposure", 5.0, 0.0, 10.0, 1.0));
        visual.update(|window, app| window.draw(app).clear(app));
        let bounds = visual.debug_bounds("mkit-slider-track").expect("slider track rendered");
        let start = point(bounds.left() + px(80.), bounds.center().y);
        let moved = point(bounds.left() + px(240.), bounds.center().y);
        visual.simulate_mouse_down(start, MouseButton::Left, Default::default());
        visual.simulate_mouse_move(moved, Some(MouseButton::Left), Default::default());
        assert_ne!(slider.read_with(visual, |slider, _| slider.values().to_vec()), [5.0]);
        visual.simulate_keystrokes("escape");
        assert_eq!(slider.read_with(visual, |slider, _| slider.values().to_vec()), [5.0]);
    }
}
