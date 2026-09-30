//! Segmented time entry with spinbutton semantics.
extern crate gpui_pre as gpui;

use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, KeyDownEvent, Render,
    Rgba, Window, actions, div, point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::Theme,
};

/// Resolved segment colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    fill: Rgba,
    border: Rgba,
    text: Rgba,
    separator: Rgba,
    /// Pointer-hover fill; `None` in high contrast.
    hover_bg: Option<Rgba>,
    /// Pointer-hover border in high contrast.
    hover_border: Option<Rgba>,
    focus_fill: Rgba,
    focus_border: Rgba,
    focus_ring: Rgba,
    invalid_border: Rgba,
    /// Ring on the focused segment while invalid; `None` in high contrast, where focus keeps its ring.
    invalid_ring: Option<Rgba>,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}
/// The web convention's `opacity: .5` applied as one layer: composite over `base`, then mix 50%.
fn dim(color: Rgba, base: Rgba) -> Rgba {
    mix(composite(color, base), base, 0.5)
}
fn look(t: &Theme, disabled: bool) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        let (text, border) = if disabled { (c.disabled, c.disabled) } else { (c.text, c.border) };
        return Look {
            fill: c.background,
            border,
            text,
            separator: if disabled { c.disabled } else { c.text_muted },
            hover_bg: None,
            hover_border: Some(c.accent),
            focus_fill: c.background,
            focus_border: c.focus,
            focus_ring: c.focus,
            invalid_border: c.danger,
            invalid_ring: None,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    let fill = c.background;
    let look = Look {
        fill,
        border: if dark { mix(c.text, fill, 0.1) } else { c.border },
        text: c.text,
        separator: c.text_muted,
        hover_bg: Some(muted),
        hover_border: None,
        focus_fill: muted,
        focus_border: c.focus,
        focus_ring: c.focus.opacity(0.5),
        invalid_border: c.danger,
        invalid_ring: Some(c.danger.opacity(if dark { 0.4 } else { 0.2 })),
    };
    if !disabled {
        return look;
    }
    let bg = c.background;
    Look {
        fill: dim(look.fill, bg),
        border: dim(look.border, bg),
        text: dim(look.text, bg),
        separator: dim(look.separator, bg),
        hover_bg: None,
        ..look
    }
}
/// shadcn/ui focus ring width, drawn outside the focused segment.
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

pub const KEY_CONTEXT: &str = "TimeField";
actions!(
    time_field,
    [
        Increment,
        Decrement,
        LargeIncrement,
        LargeDecrement,
        NextSegment,
        PreviousSegment,
        Commit,
        Minimum,
        Maximum
    ]
);

pub fn default_key_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("up", Increment, Some(KEY_CONTEXT)),
        KeyBinding::new("down", Decrement, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-up", LargeIncrement, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-down", LargeDecrement, Some(KEY_CONTEXT)),
        KeyBinding::new("right", NextSegment, Some(KEY_CONTEXT)),
        KeyBinding::new("left", PreviousSegment, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Commit, Some(KEY_CONTEXT)),
        KeyBinding::new("home", Minimum, Some(KEY_CONTEXT)),
        KeyBinding::new("end", Maximum, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimeValue {
    hour: u8,
    minute: u8,
    second: u8,
}

impl TimeValue {
    pub const MIDNIGHT: Self = Self { hour: 0, minute: 0, second: 0 };

    pub fn new(hour: u8, minute: u8, second: u8) -> Option<Self> {
        (hour < 24 && minute < 60 && second < 60).then_some(Self { hour, minute, second })
    }

    pub fn hour(self) -> u8 {
        self.hour
    }

    pub fn minute(self) -> u8 {
        self.minute
    }

    pub fn second(self) -> u8 {
        self.second
    }

    fn seconds_since_midnight(self) -> i32 {
        i32::from(self.hour) * 3600 + i32::from(self.minute) * 60 + i32::from(self.second)
    }

    fn from_seconds(seconds: i32) -> Self {
        let seconds = seconds.rem_euclid(24 * 60 * 60);
        Self {
            hour: (seconds / 3600) as u8,
            minute: (seconds / 60 % 60) as u8,
            second: (seconds % 60) as u8,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HourCycle {
    #[default]
    H24,
    H12,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Segment {
    Hour,
    Minute,
    Second,
    Period,
}

impl Segment {
    fn index(self) -> usize {
        match self {
            Self::Hour => 0,
            Self::Minute => 1,
            Self::Second => 2,
            Self::Period => 3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TimeFieldLabels {
    pub hour: String,
    pub minute: String,
    pub second: String,
    pub period: String,
    pub am: String,
    pub pm: String,
    pub separator: String,
    pub outside_bounds: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeChangeRequested(pub Option<TimeValue>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeChanged(pub Option<TimeValue>);

pub struct TimeField {
    label: String,
    labels: TimeFieldLabels,
    value: Option<TimeValue>,
    draft: Option<TimeValue>,
    controlled: bool,
    cycle: HourCycle,
    seconds: bool,
    step: u8,
    min: Option<TimeValue>,
    max: Option<TimeValue>,
    disabled: bool,
    validation: Option<String>,
    active: Segment,
    digit_buffer: String,
    focus: [Option<FocusHandle>; 4],
}

impl EventEmitter<TimeChangeRequested> for TimeField {}
impl EventEmitter<TimeChanged> for TimeField {}

impl TimeField {
    pub fn new(
        label: impl Into<String>,
        value: Option<TimeValue>,
        labels: TimeFieldLabels,
    ) -> Self {
        Self {
            label: label.into(),
            labels,
            value,
            draft: value,
            controlled: false,
            cycle: HourCycle::H24,
            seconds: false,
            step: 1,
            min: None,
            max: None,
            disabled: false,
            validation: None,
            active: Segment::Hour,
            digit_buffer: String::new(),
            focus: [None, None, None, None],
        }
    }

    pub fn controlled(mut self) -> Self {
        self.controlled = true;
        self
    }

    pub fn hour_cycle(mut self, cycle: HourCycle) -> Self {
        self.cycle = cycle;
        self
    }

    pub fn seconds(mut self, visible: bool) -> Self {
        self.seconds = visible;
        self
    }

    pub fn step(mut self, minutes: u8) -> Self {
        self.step = minutes.max(1);
        self
    }

    pub fn bounds(mut self, min: Option<TimeValue>, max: Option<TimeValue>) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn value(&self) -> Option<TimeValue> {
        self.value
    }

    pub fn validation_message(&self) -> Option<&str> {
        self.validation.as_deref()
    }

    pub fn set_value(&mut self, value: Option<TimeValue>, cx: &mut Context<Self>) {
        self.value = value;
        self.draft = value;
        self.digit_buffer.clear();
        self.validation = None;
        cx.notify();
    }

    fn visible_segments(&self) -> Vec<Segment> {
        let mut segments = vec![Segment::Hour, Segment::Minute];
        if self.seconds {
            segments.push(Segment::Second);
        }
        if self.cycle == HourCycle::H12 {
            segments.push(Segment::Period);
        }
        segments
    }

    fn move_segment(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let segments = self.visible_segments();
        let at = segments.iter().position(|s| *s == self.active).unwrap_or(0);
        let next = (at as isize + delta).clamp(0, segments.len() as isize - 1) as usize;
        self.active = segments[next];
        if let Some(handle) = self.focus[self.active.index()].as_ref() {
            handle.focus(window, cx);
        }
        cx.notify();
    }

    fn increment(&mut self, _: &Increment, window: &mut Window, cx: &mut Context<Self>) {
        self.adjust(1, window, cx);
    }

    fn decrement(&mut self, _: &Decrement, window: &mut Window, cx: &mut Context<Self>) {
        self.adjust(-1, window, cx);
    }

    fn large_increment(&mut self, _: &LargeIncrement, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust_by(1, 5, cx);
    }

    fn large_decrement(&mut self, _: &LargeDecrement, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust_by(-1, 5, cx);
    }

    fn adjust(&mut self, direction: i32, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust_by(direction, 1, cx);
    }

    fn adjust_by(&mut self, direction: i32, multiplier: i32, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let current = self.draft.or(self.value).unwrap_or(TimeValue::MIDNIGHT);
        let delta = match self.active {
            Segment::Hour => direction * multiplier * 3600,
            Segment::Minute => direction * multiplier * i32::from(self.step) * 60,
            Segment::Second => direction * multiplier,
            Segment::Period => direction * 12 * 3600,
        };
        let candidate =
            self.constrain(TimeValue::from_seconds(current.seconds_since_midnight() + delta));
        self.commit(candidate, cx);
    }

    fn home(&mut self, _: &Minimum, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let current = self.draft.or(self.value).unwrap_or(TimeValue::MIDNIGHT);
        let next = match self.active {
            Segment::Hour => {
                let hour = match self.cycle {
                    HourCycle::H24 => 0,
                    HourCycle::H12 if current.hour >= 12 => 12,
                    HourCycle::H12 => 0,
                };
                TimeValue::new(hour, current.minute, current.second).unwrap()
            }
            Segment::Minute => TimeValue::new(current.hour, 0, current.second).unwrap(),
            Segment::Second => TimeValue::new(current.hour, current.minute, 0).unwrap(),
            Segment::Period => {
                TimeValue::new(current.hour % 12, current.minute, current.second).unwrap()
            }
        };
        let next = self.constrain(next);
        self.commit(next, cx);
        let _ = window;
    }

    fn end(&mut self, _: &Maximum, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let current = self.draft.or(self.value).unwrap_or(TimeValue::MIDNIGHT);
        let next = match self.active {
            Segment::Hour => {
                let hour = match self.cycle {
                    HourCycle::H24 => 23,
                    HourCycle::H12 if current.hour >= 12 => 12,
                    HourCycle::H12 => 0,
                };
                TimeValue::new(hour, current.minute, current.second).unwrap()
            }
            Segment::Minute => TimeValue::new(current.hour, 59, current.second).unwrap(),
            Segment::Second => TimeValue::new(current.hour, current.minute, 59).unwrap(),
            Segment::Period => {
                TimeValue::new(current.hour % 12 + 12, current.minute, current.second).unwrap()
            }
        };
        let next = self.constrain(next);
        self.commit(next, cx);
        let _ = window;
    }

    fn next_segment(&mut self, _: &NextSegment, window: &mut Window, cx: &mut Context<Self>) {
        self.move_segment(1, window, cx);
    }

    fn previous_segment(
        &mut self,
        _: &PreviousSegment,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_segment(-1, window, cx);
    }

    fn commit_action(&mut self, _: &Commit, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if !self.digit_buffer.is_empty() {
            self.accept_digits(cx);
        } else if let Some(candidate) = self.draft {
            self.commit(candidate, cx);
        }
    }

    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let key = event.keystroke.key.as_str();
        if self.cycle == HourCycle::H12
            && self.active == Segment::Period
            && (key == "a" || key == "p" || key == "space")
        {
            let current = self.draft.or(self.value).unwrap_or(TimeValue::MIDNIGHT);
            let to_pm = key == "p" || (key == "space" && current.hour < 12);
            let hour = (current.hour % 12) + if to_pm { 12 } else { 0 };
            self.commit(TimeValue::new(hour, current.minute, current.second).unwrap(), cx);
        }
        if key.len() == 1 && key.as_bytes()[0].is_ascii_digit() && self.active != Segment::Period {
            if self.digit_buffer.len() >= 2 {
                self.digit_buffer.clear();
            }
            self.digit_buffer.push_str(key);
            if self.digit_buffer.len() == 2 {
                self.accept_digits(cx);
            } else {
                cx.notify();
            }
            return;
        }
        let _ = window;
    }

    fn accept_digits(&mut self, cx: &mut Context<Self>) {
        let Ok(input) = self.digit_buffer.parse::<u8>() else {
            self.validation = Some(self.labels.outside_bounds.clone());
            self.digit_buffer.clear();
            cx.notify();
            return;
        };
        let current = self.draft.or(self.value).unwrap_or(TimeValue::MIDNIGHT);
        let candidate = match self.active {
            Segment::Hour => {
                let display_hour = match self.cycle {
                    HourCycle::H24 if input < 24 => input,
                    HourCycle::H12 if (1..=12).contains(&input) => {
                        (input % 12) + if current.hour >= 12 { 12 } else { 0 }
                    }
                    _ => {
                        self.validation = Some(self.labels.outside_bounds.clone());
                        self.digit_buffer.clear();
                        cx.notify();
                        return;
                    }
                };
                TimeValue::new(display_hour, current.minute, current.second).unwrap()
            }
            Segment::Minute if input < 60 => {
                TimeValue::new(current.hour, input, current.second).unwrap()
            }
            Segment::Second if input < 60 => {
                TimeValue::new(current.hour, current.minute, input).unwrap()
            }
            _ => {
                self.validation = Some(self.labels.outside_bounds.clone());
                self.digit_buffer.clear();
                cx.notify();
                return;
            }
        };
        self.digit_buffer.clear();
        if self.within_bounds(candidate) {
            self.commit(candidate, cx);
        } else {
            self.draft = Some(candidate);
            self.validation = Some(self.labels.outside_bounds.clone());
            cx.notify();
        }
    }

    fn within_bounds(&self, value: TimeValue) -> bool {
        !self.min.is_some_and(|min| value < min) && !self.max.is_some_and(|max| value > max)
    }

    fn constrain(&self, value: TimeValue) -> TimeValue {
        let min = self.min.unwrap_or(TimeValue::MIDNIGHT);
        let max = self.max.unwrap_or(TimeValue::new(23, 59, 59).unwrap());
        value.max(min).min(max)
    }

    fn commit(&mut self, value: TimeValue, cx: &mut Context<Self>) {
        self.validation = None;
        self.draft = Some(value);
        if self.controlled {
            cx.emit(TimeChangeRequested(Some(value)));
        } else {
            self.value = Some(value);
            cx.emit(TimeChanged(self.value));
        }
        cx.notify();
    }

    fn display_value(&self) -> TimeValue {
        self.draft.or(self.value).unwrap_or(TimeValue::MIDNIGHT)
    }

    fn display_hour(&self, value: TimeValue) -> u8 {
        match self.cycle {
            HourCycle::H24 => value.hour,
            HourCycle::H12 => match value.hour % 12 {
                0 => 12,
                hour => hour,
            },
        }
    }

    fn segment_element(
        &self,
        segment: Segment,
        value: TimeValue,
        theme: Theme,
        window: &Window,
        focus: &FocusHandle,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let text = match segment {
            Segment::Hour => format!("{:02}", self.display_hour(value)),
            Segment::Minute => format!("{:02}", value.minute),
            Segment::Second => format!("{:02}", value.second),
            Segment::Period => {
                if value.hour >= 12 {
                    self.labels.pm.clone()
                } else {
                    self.labels.am.clone()
                }
            }
        };
        let name = match segment {
            Segment::Hour => self.labels.hour.clone(),
            Segment::Minute => self.labels.minute.clone(),
            Segment::Second => self.labels.second.clone(),
            Segment::Period => self.labels.period.clone(),
        };
        let numeric = match segment {
            Segment::Hour => f64::from(self.display_hour(value)),
            Segment::Minute => f64::from(value.minute),
            Segment::Second => f64::from(value.second),
            Segment::Period => f64::from(value.hour >= 12),
        };
        let (min, max) = match segment {
            Segment::Hour => {
                if self.cycle == HourCycle::H12 {
                    (1.0, 12.0)
                } else {
                    (0.0, 23.0)
                }
            }
            Segment::Minute | Segment::Second => (0.0, 59.0),
            Segment::Period => (0.0, 1.0),
        };
        let focused = focus.is_focused(window) && !self.disabled;
        let look = look(&theme, self.disabled);
        let invalid = self.validation.is_some();
        let border = match (invalid, focused) {
            (true, _) => look.invalid_border,
            (false, true) => look.focus_border,
            (false, false) => look.border,
        };
        let ring = match look.invalid_ring {
            Some(ring) if invalid => ring,
            _ => look.focus_ring,
        };
        div()
            .id(match segment {
                Segment::Hour => "time-hour",
                Segment::Minute => "time-minute",
                Segment::Second => "time-second",
                Segment::Period => "time-period",
            })
            .debug_selector(|| format!("mkit-time-segment-{}", segment.index()))
            .role(gpui_pre::accesskit::Role::SpinButton)
            .aria_label(name)
            .aria_numeric_value(numeric)
            .aria_min_numeric_value(min)
            .aria_max_numeric_value(max)
            .aria_value(text.clone())
            .when_some(self.validation.clone(), |element, message| {
                element.aria_description(message).a11y_synthetic_children(|builder| {
                    builder.parent_node().set_invalid(gpui_pre::accesskit::Invalid::True)
                })
            })
            .when(self.disabled, |element| {
                element.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .when(!self.disabled, |element| {
                element.track_focus(focus).tab_stop(true).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.active = segment;
                        this.digit_buffer.clear();
                        cx.notify();
                    },
                ))
            })
            .when(self.disabled, |element| element.tab_stop(false))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(px(theme.controls.small))
            .min_w(px(theme.controls.small))
            .px(px(theme.spacing.xsmall))
            .rounded(px(theme.radii.small))
            .border(px(theme.borders.regular))
            .border_color(border)
            .bg(if focused { look.focus_fill } else { look.fill })
            .when(focused, |element| element.shadow(vec![focus_ring(ring)]))
            .when(!focused && !self.disabled, |element| {
                element.hover(move |style| {
                    let style = match look.hover_bg {
                        Some(color) => style.bg(color),
                        None => style,
                    };
                    match look.hover_border {
                        Some(color) if !invalid => style.border_color(color),
                        _ => style,
                    }
                })
            })
            .text_color(look.text)
            .text_size(px(theme.typography.body))
            .child(if self.value.is_none() && self.draft.is_none() {
                "--".to_owned()
            } else {
                text
            })
    }
}

impl Focusable for TimeField {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.focus[Segment::Hour.index()].clone().unwrap_or_else(|| cx.focus_handle())
    }
}

impl Render for TimeField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let segments = self.visible_segments();
        for segment in &segments {
            self.focus[segment.index()].get_or_insert_with(|| cx.focus_handle().tab_stop(true));
        }
        let value = self.display_value();
        let mut row = div()
            .id("time-field")
            .debug_selector(|| "mkit-time-field".into())
            .key_context(KEY_CONTEXT)
            .on_key_down(cx.listener(Self::key_down))
            .on_action(cx.listener(Self::increment))
            .on_action(cx.listener(Self::decrement))
            .on_action(cx.listener(Self::large_increment))
            .on_action(cx.listener(Self::large_decrement))
            .on_action(cx.listener(Self::next_segment))
            .on_action(cx.listener(Self::previous_segment))
            .on_action(cx.listener(Self::commit_action))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.label.clone())
            .flex()
            .items_center()
            .gap(px(theme.spacing.xsmall));

        let separator = look(&theme, self.disabled).separator;
        for (index, segment) in segments.into_iter().enumerate() {
            if index > 0 && segment != Segment::Period {
                row = row.child(
                    div()
                        .text_color(separator)
                        .text_size(px(theme.typography.body))
                        .child(self.labels.separator.clone()),
                );
            }
            row = row.child(self.segment_element(
                segment,
                value,
                theme,
                window,
                self.focus[segment.index()].as_ref().unwrap(),
                cx,
            ));
        }
        if let Some(message) = self.validation.clone() {
            row = row.child(
                div()
                    .ml(px(theme.spacing.xsmall))
                    .text_color(theme.colors.danger)
                    .text_size(px(theme.typography.body))
                    .child(message),
            );
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::TestAppContext;
    use std::{cell::RefCell, rc::Rc};

    fn labels() -> TimeFieldLabels {
        TimeFieldLabels {
            hour: "Hour".into(),
            minute: "Minute".into(),
            second: "Second".into(),
            period: "Period".into(),
            am: "AM".into(),
            pm: "PM".into(),
            separator: ":".into(),
            outside_bounds: "Outside allowed time".into(),
        }
    }

    #[gpui_pre::test]
    fn arrow_adjustment_uses_minute_step_and_emits_uncontrolled_commit(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = TimeValue::new(10, 0, 0).unwrap();
        let (field, window) =
            cx.add_window_view(|_, _| TimeField::new("Time", Some(initial), labels()).step(15));
        let events = Rc::new(RefCell::new(Vec::new()));
        let log = events.clone();
        let _subscription = window.update(|_, app| {
            app.subscribe(&field, move |_, event: &TimeChanged, _| log.borrow_mut().push(*event))
        });
        window.update(|win, cx| {
            field.update(cx, |field, _| field.active = Segment::Minute);
            win.draw(cx).clear(cx);
        });
        window.update(|win, cx| {
            let focus = field.read(cx).focus[Segment::Minute.index()].clone().unwrap();
            focus.focus(win, cx);
        });
        window.simulate_keystrokes("up");
        assert_eq!(
            field.read_with(window, |field, _| field.value),
            Some(TimeValue::new(10, 15, 0).unwrap())
        );
        assert_eq!(
            events.borrow().as_slice(),
            &[TimeChanged(Some(TimeValue::new(10, 15, 0).unwrap()))]
        );
    }

    #[gpui_pre::test]
    fn controlled_time_requests_wait_for_owner_update(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = TimeValue::new(10, 0, 0).unwrap();
        let (field, window) =
            cx.add_window_view(|_, _| TimeField::new("Time", Some(initial), labels()).controlled());
        let events = Rc::new(RefCell::new(Vec::new()));
        let log = events.clone();
        let _subscription = window.update(|_, app| {
            app.subscribe(&field, move |_, event: &TimeChangeRequested, _| {
                log.borrow_mut().push(*event)
            })
        });
        window.update(|win, cx| {
            field.update(cx, |field, _| field.active = Segment::Hour);
            win.draw(cx).clear(cx);
        });
        window.update(|win, cx| {
            let focus = field.read(cx).focus[Segment::Hour.index()].clone().unwrap();
            focus.focus(win, cx);
        });
        window.simulate_keystrokes("up");
        assert_eq!(field.read_with(window, |field, _| field.value), Some(initial));
        assert_eq!(
            events.borrow().as_slice(),
            &[TimeChangeRequested(Some(TimeValue::new(11, 0, 0).unwrap()))]
        );
    }

    #[gpui_pre::test]
    fn digit_entry_commits_a_valid_complete_segment(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (field, window) = cx.add_window_view(|_, _| {
            TimeField::new("Time", Some(TimeValue::new(10, 0, 0).unwrap()), labels())
        });
        window.update(|win, cx| {
            field.update(cx, |field, _| field.active = Segment::Minute);
            win.draw(cx).clear(cx);
        });
        window.update(|win, cx| {
            let focus = field.read(cx).focus[Segment::Minute.index()].clone().unwrap();
            focus.focus(win, cx);
        });
        window.simulate_keystrokes("4 5");
        assert_eq!(
            field.read_with(window, |field, _| field.value),
            Some(TimeValue::new(10, 45, 0).unwrap())
        );
    }

    #[gpui_pre::test]
    fn invalid_hour_is_rejected_and_disabled_fields_ignore_adjustment(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = TimeValue::new(10, 0, 0).unwrap();
        let (field, window) = cx.add_window_view(|_, _| {
            TimeField::new("Time", Some(initial), labels())
                .bounds(
                    Some(TimeValue::new(9, 0, 0).unwrap()),
                    Some(TimeValue::new(10, 30, 0).unwrap()),
                )
                .disabled(true)
        });
        window.update(|win, cx| {
            field.update(cx, |field, _| field.active = Segment::Hour);
            field.update(cx, |field, cx| field.adjust(1, win, cx));
        });
        assert_eq!(field.read_with(window, |field, _| field.value), Some(initial));
    }

    #[gpui_pre::test]
    fn twelve_hour_period_input_and_optional_seconds_are_supported(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let initial = TimeValue::new(0, 15, 7).unwrap();
        let (field, window) = cx.add_window_view(|_, _| {
            TimeField::new("Time", Some(initial), labels()).hour_cycle(HourCycle::H12).seconds(true)
        });
        assert_eq!(
            field.read_with(window, |field, _| field.visible_segments()),
            vec![Segment::Hour, Segment::Minute, Segment::Second, Segment::Period]
        );
        assert_eq!(field.read_with(window, |field, _| field.display_hour(initial)), 12);
        window.update(|win, cx| {
            field.update(cx, |field, _| field.active = Segment::Period);
            win.draw(cx).clear(cx);
        });
        window.update(|win, cx| {
            let focus = field.read(cx).focus[Segment::Period.index()].clone().unwrap();
            focus.focus(win, cx);
        });
        window.simulate_keystrokes("p");
        assert_eq!(
            field.read_with(window, |field, _| field.value),
            Some(TimeValue::new(12, 15, 7).unwrap())
        );
    }

    #[gpui_pre::test]
    fn typed_time_outside_bounds_keeps_draft_and_reports_validation(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let initial = TimeValue::new(10, 0, 0).unwrap();
        let (field, window) = cx.add_window_view(|_, _| {
            TimeField::new("Time", Some(initial), labels()).bounds(
                Some(TimeValue::new(9, 0, 0).unwrap()),
                Some(TimeValue::new(10, 30, 0).unwrap()),
            )
        });
        window.update(|win, cx| {
            field.update(cx, |field, _| field.active = Segment::Minute);
            win.draw(cx).clear(cx);
        });
        window.update(|win, cx| {
            let focus = field.read(cx).focus[Segment::Minute.index()].clone().unwrap();
            focus.focus(win, cx);
        });
        window.simulate_keystrokes("4 5");
        assert_eq!(field.read_with(window, |field, _| field.value), Some(initial));
        assert_eq!(
            field.read_with(window, |field, _| field.draft),
            Some(TimeValue::new(10, 45, 0).unwrap())
        );
        assert_eq!(
            field.read_with(window, |field, _| field.validation_message().map(str::to_owned)),
            Some("Outside allowed time".to_owned())
        );
    }

    #[gpui_pre::test]
    fn bounds_clamp_arrow_adjustment(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = TimeValue::new(10, 0, 0).unwrap();
        let (field, window) = cx.add_window_view(|_, _| {
            TimeField::new("Time", Some(initial), labels()).bounds(
                Some(TimeValue::new(9, 0, 0).unwrap()),
                Some(TimeValue::new(10, 30, 0).unwrap()),
            )
        });
        window.update(|win, cx| {
            field.update(cx, |field, _| field.active = Segment::Hour);
            win.draw(cx).clear(cx);
        });
        window.update(|win, cx| {
            let focus = field.read(cx).focus[Segment::Hour.index()].clone().unwrap();
            focus.focus(win, cx);
        });
        window.simulate_keystrokes("up");
        assert_eq!(
            field.read_with(window, |field, _| field.value),
            Some(TimeValue::new(10, 30, 0).unwrap())
        );
    }
}
