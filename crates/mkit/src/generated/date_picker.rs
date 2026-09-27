//! Text date pickers backed by the shared Calendar grid.
extern crate gpui_pre as gpui;

#[cfg(feature = "mkit-mirror")]
use crate::{
    calendar::{
        Calendar, Selection as CalendarSelection, SelectionChanged as CalendarSelectionChanged,
        default_key_bindings as calendar_key_bindings,
    },
    text_field::{InputChanged, TextField, default_key_bindings as text_field_key_bindings},
};
use gpui_pre::{
    AnchoredPositionMode, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyBinding, KeyDownEvent, Render, Window, actions, anchored, deferred, div, point, prelude::*,
    px,
};
use mkit_core::{CivilDate, theme::Theme};
#[cfg(not(feature = "mkit-mirror"))]
use mkit_registry_calendar::{
    Calendar, Selection as CalendarSelection, SelectionChanged as CalendarSelectionChanged,
    default_key_bindings as calendar_key_bindings,
};
#[cfg(not(feature = "mkit-mirror"))]
use mkit_registry_text_field::{
    InputChanged, TextField, default_key_bindings as text_field_key_bindings,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

pub const KEY_CONTEXT: &str = "DatePicker";
actions!(date_picker, [OpenCalendar, Dismiss, Commit]);
pub fn default_key_bindings() -> Vec<KeyBinding> {
    let mut bindings = vec![
        KeyBinding::new("alt-down", OpenCalendar, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", Dismiss, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Commit, Some(KEY_CONTEXT)),
    ];
    bindings.extend(calendar_key_bindings());
    bindings.extend(text_field_key_bindings());
    bindings
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DateRange {
    pub start: CivilDate,
    pub end: CivilDate,
}
impl DateRange {
    pub fn new(a: CivilDate, b: CivilDate) -> Self {
        if a <= b { Self { start: a, end: b } } else { Self { start: b, end: a } }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DateValue {
    Single(Option<CivilDate>),
    Range(Option<DateRange>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeRequested(pub DateValue);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Changed(pub DateValue);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RangeChangeRequested(pub Option<DateRange>);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RangeChanged(pub Option<DateRange>);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenChanged(pub bool);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cancelled;
impl EventEmitter<ChangeRequested> for DatePicker {}
impl EventEmitter<Changed> for DatePicker {}
impl EventEmitter<RangeChangeRequested> for DatePicker {}
impl EventEmitter<RangeChanged> for DatePicker {}
impl EventEmitter<OpenChanged> for DatePicker {}
impl EventEmitter<Cancelled> for DatePicker {}

type Parser = Arc<dyn Fn(&str) -> Result<CivilDate, String> + Send + Sync>;
type Formatter = Arc<dyn Fn(CivilDate) -> String + Send + Sync>;
type DisabledDate = Arc<dyn Fn(CivilDate) -> bool + Send + Sync>;

#[derive(Clone)]
pub struct CalendarLabels {
    pub weekdays_monday_first: [String; 7],
    pub months: [String; 12],
    pub first_weekday: u8,
}
#[derive(Clone)]
pub struct DatePickerLabels {
    pub single_placeholder: String,
    pub start_placeholder: String,
    pub end_placeholder: String,
    pub start_suffix: String,
    pub end_suffix: String,
    pub choose_date: String,
    pub choose_range: String,
    pub incomplete_range: String,
    pub outside_bounds: String,
    pub no_available_dates: String,
}

/// Shared implementation for the single and range picker. `DateRangePicker` is an alias so both
/// forms use the same keyboard, popover, parsing, and focus implementation.
pub struct DatePicker {
    label: String,
    range_mode: bool,
    value: DateValue,
    initial_value: DateValue,
    controlled: bool,
    open: bool,
    active_endpoint_end: bool,
    draft: [String; 2],
    validation: [Option<String>; 2],
    parser: Parser,
    formatter: Formatter,
    labels: CalendarLabels,
    text_labels: DatePickerLabels,
    initial_active: CivilDate,
    min: Option<CivilDate>,
    max: Option<CivilDate>,
    disabled_date: DisabledDate,
    disabled: bool,
    fields: [Option<Entity<TextField>>; 2],
    calendar: Option<Entity<Calendar>>,
    resolved_active: Option<CivilDate>,
    field_subscriptions: Vec<gpui_pre::Subscription>,
    calendar_subscription: Option<gpui_pre::Subscription>,
    field_bounds: Rc<RefCell<Option<gpui_pre::Bounds<gpui_pre::Pixels>>>>,
    focus: Option<FocusHandle>,
    last_open: bool,
    restore_focus_when_closed: bool,
}
pub type DateRangePicker = DatePicker;

impl DatePicker {
    pub fn new(
        label: impl Into<String>,
        value: Option<CivilDate>,
        initial_active: CivilDate,
        labels: CalendarLabels,
        text_labels: DatePickerLabels,
        parser: impl Fn(&str) -> Result<CivilDate, String> + Send + Sync + 'static,
        formatter: impl Fn(CivilDate) -> String + Send + Sync + 'static,
    ) -> Self {
        let value = DateValue::Single(value);
        Self::base(
            label,
            false,
            value,
            initial_active,
            labels,
            text_labels,
            Arc::new(parser),
            Arc::new(formatter),
        )
    }

    pub fn range(
        label: impl Into<String>,
        value: Option<DateRange>,
        initial_active: CivilDate,
        labels: CalendarLabels,
        text_labels: DatePickerLabels,
        parser: impl Fn(&str) -> Result<CivilDate, String> + Send + Sync + 'static,
        formatter: impl Fn(CivilDate) -> String + Send + Sync + 'static,
    ) -> Self {
        Self::base(
            label,
            true,
            DateValue::Range(value),
            initial_active,
            labels,
            text_labels,
            Arc::new(parser),
            Arc::new(formatter),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn base(
        label: impl Into<String>,
        range_mode: bool,
        value: DateValue,
        initial_active: CivilDate,
        labels: CalendarLabels,
        text_labels: DatePickerLabels,
        parser: Parser,
        formatter: Formatter,
    ) -> Self {
        let mut picker = Self {
            label: label.into(),
            range_mode,
            value,
            initial_value: value,
            controlled: false,
            open: false,
            active_endpoint_end: false,
            draft: [String::new(), String::new()],
            validation: [None, None],
            parser,
            formatter,
            labels,
            text_labels,
            initial_active,
            min: None,
            max: None,
            disabled_date: Arc::new(|_| false),
            disabled: false,
            fields: [None, None],
            calendar: None,
            resolved_active: None,
            field_subscriptions: Vec::new(),
            calendar_subscription: None,
            field_bounds: Rc::new(RefCell::new(None)),
            focus: None,
            last_open: false,
            restore_focus_when_closed: false,
        };
        picker.sync_draft();
        picker
    }

    pub fn controlled(mut self) -> Self {
        self.controlled = true;
        self
    }
    pub fn bounds(mut self, min: Option<CivilDate>, max: Option<CivilDate>) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn disabled_dates(
        mut self,
        predicate: impl Fn(CivilDate) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.disabled_date = Arc::new(predicate);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        if disabled {
            self.open = false;
        }
        self
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn value(&self) -> DateValue {
        self.value
    }
    pub fn validation_message(&self, end: bool) -> Option<&str> {
        self.validation[usize::from(end)].as_deref()
    }
    pub fn set_value(&mut self, value: DateValue, cx: &mut Context<Self>) {
        self.value = value;
        self.initial_value = value;
        self.resolved_active = None;
        self.sync_draft();
        self.sync_fields(cx);
        cx.notify();
    }

    fn sync_draft(&mut self) {
        self.draft = match self.value {
            DateValue::Single(date) => {
                [date.map(|d| (self.formatter)(d)).unwrap_or_default(), String::new()]
            }
            DateValue::Range(range) => range
                .map(|r| [(self.formatter)(r.start), (self.formatter)(r.end)])
                .unwrap_or_default(),
        };
        self.validation = [None, None];
    }
    fn sync_fields(&mut self, cx: &mut Context<Self>) {
        for index in 0..if self.range_mode { 2 } else { 1 } {
            if let Some(field) = self.fields[index].as_ref() {
                let value = self.draft[index].clone();
                let validation = self.validation[index].clone();
                field.update(cx, |field, cx| {
                    field.set_value(value, cx);
                    field.set_validation_message(validation);
                    cx.notify();
                });
            }
        }
    }
    fn ensure_children(&mut self, cx: &mut Context<Self>) {
        let labels = self.labels.clone();
        let preferred = match self.value {
            DateValue::Single(d) => d.unwrap_or(self.initial_active),
            DateValue::Range(r) => r.map(|v| v.start).unwrap_or(self.initial_active),
        };
        if self.resolved_active.is_none() {
            self.resolved_active = self.find_available_date(preferred);
        }
        let Some(start) = self.resolved_active else {
            self.create_fields(cx);
            return;
        };
        if self.calendar.is_none() {
            let disabled_date = self.disabled_date.clone();
            let disabled = self.disabled;
            let calendar = cx.new(|_| {
                let calendar = Calendar::new(
                    self.label.clone(),
                    start,
                    labels.weekdays_monday_first,
                    labels.months,
                )
                .first_weekday(labels.first_weekday)
                .bounds(self.min, self.max)
                .disabled_dates(move |date| disabled || disabled_date(date));
                if self.range_mode {
                    calendar.range_mode()
                } else {
                    calendar.default_selection(CalendarSelection::Single(match self.value {
                        DateValue::Single(v) => v,
                        _ => None,
                    }))
                }
            });
            let sub = cx.subscribe(&calendar, |this, _, event: &CalendarSelectionChanged, cx| {
                this.calendar_commit(event.0, cx)
            });
            self.calendar_subscription = Some(sub);
            self.calendar = Some(calendar);
        }
        self.create_fields(cx);
    }
    fn create_fields(&mut self, cx: &mut Context<Self>) {
        let field_count = if self.range_mode { 2 } else { 1 };
        for index in 0..field_count {
            if self.fields[index].is_none() {
                let label = if self.range_mode {
                    format!(
                        "{} {}",
                        self.label,
                        if index == 0 {
                            &self.text_labels.start_suffix
                        } else {
                            &self.text_labels.end_suffix
                        }
                    )
                } else {
                    self.label.clone()
                };
                let placeholder = if !self.range_mode {
                    self.text_labels.single_placeholder.clone()
                } else if index == 0 {
                    self.text_labels.start_placeholder.clone()
                } else {
                    self.text_labels.end_placeholder.clone()
                };
                let text = self.draft[index].clone();
                let validation = self.validation[index].clone();
                let field = cx.new(|cx| {
                    let mut field = TextField::new(cx)
                        .with_label(label)
                        .with_placeholder(placeholder)
                        .controlled(text);
                    field.set_disabled(self.disabled);
                    if let Some(message) = validation {
                        field.set_validation_message(Some(message));
                    }
                    field
                });
                let sub = cx.subscribe(&field, move |this, _, event: &InputChanged, cx| {
                    this.draft[index] = event.0.clone();
                    this.validation[index] = None;
                    this.sync_fields(cx);
                    cx.notify();
                });
                self.field_subscriptions.push(sub);
                self.fields[index] = Some(field);
            }
        }
    }
    fn find_available_date(&self, preferred: CivilDate) -> Option<CivilDate> {
        let start = match (self.min, self.max) {
            (Some(min), Some(max)) if min > max => return None,
            (Some(min), _) if preferred < min => min,
            (_, Some(max)) if preferred > max => max,
            _ => preferred,
        };
        if self.within_bounds(start) {
            return Some(start);
        }
        let mut next = start;
        while let Some(date) = next.add_days(1) {
            if self.max.is_some_and(|max| date > max) {
                break;
            }
            next = date;
            if self.within_bounds(date) {
                return Some(date);
            }
        }
        let mut previous = start;
        while let Some(date) = previous.add_days(-1) {
            if self.min.is_some_and(|min| date < min) {
                break;
            }
            previous = date;
            if self.within_bounds(date) {
                return Some(date);
            }
        }
        None
    }
    fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.open != open {
            self.open = open;
            cx.emit(OpenChanged(open));
            cx.notify();
        }
    }
    fn open_action(&mut self, _: &OpenCalendar, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if let Some(calendar) = &self.calendar {
            let selection = match self.value {
                DateValue::Single(v) => {
                    CalendarSelection::Single(v.filter(|date| self.within_bounds(*date)))
                }
                DateValue::Range(v) => {
                    let valid =
                        v.filter(|r| self.within_bounds(r.start) && self.within_bounds(r.end));
                    CalendarSelection::Range(valid.map(|r| r.start), valid.map(|r| r.end))
                }
            };
            calendar.update(cx, |calendar, cx| calendar.set_selection(selection, cx));
        }
        self.set_open(true, cx);
        if let Some(calendar) = &self.calendar {
            calendar.update(cx, |calendar, cx| calendar.request_focus(window, cx));
        }
    }
    fn dismiss(&mut self, _: &Dismiss, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open && self.draft == self.committed_strings() {
            return;
        }
        self.value = self.initial_value;
        self.sync_draft();
        self.sync_fields(cx);
        self.set_open(false, cx);
        cx.emit(Cancelled);
        self.restore_focus_when_closed = false;
        if let Some(field) = &self.fields[usize::from(self.active_endpoint_end)] {
            field.update(cx, |field, cx| field.focus_handle(cx).focus(window, cx));
        }
    }
    fn commit_action(&mut self, _: &Commit, _: &mut Window, cx: &mut Context<Self>) {
        self.commit_typed(cx);
    }
    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match (event.keystroke.key.as_str(), event.keystroke.modifiers.alt) {
            ("down", true) => self.open_action(&OpenCalendar, window, cx),
            ("enter", _) if !self.open => self.commit_typed(cx),
            ("escape", _) => self.dismiss(&Dismiss, window, cx),
            ("tab", _) => self.set_open(false, cx),
            _ => {}
        }
    }
    fn committed_strings(&self) -> [String; 2] {
        match self.value {
            DateValue::Single(value) => {
                [value.map(|d| (self.formatter)(d)).unwrap_or_default(), String::new()]
            }
            DateValue::Range(value) => value
                .map(|r| [(self.formatter)(r.start), (self.formatter)(r.end)])
                .unwrap_or_default(),
        }
    }
    fn within_bounds(&self, d: CivilDate) -> bool {
        !self.min.is_some_and(|m| d < m)
            && !self.max.is_some_and(|m| d > m)
            && !(self.disabled_date)(d)
    }
    fn commit_typed(&mut self, cx: &mut Context<Self>) {
        let i = usize::from(self.active_endpoint_end);
        if self.range_mode {
            if self.draft[0].trim().is_empty() || self.draft[1].trim().is_empty() {
                self.validation[i] = Some(self.text_labels.incomplete_range.clone());
                self.sync_fields(cx);
                cx.notify();
                return;
            }
            let start = (self.parser)(&self.draft[0]);
            let end = (self.parser)(&self.draft[1]);
            match (start, end) {
                (Ok(a), Ok(b)) if self.within_bounds(a) && self.within_bounds(b) => {
                    self.commit(DateValue::Range(Some(DateRange::new(a, b))), cx)
                }
                (Err(error), _) => {
                    self.validation[0] = Some(error);
                    self.validation[1] = None;
                    self.sync_fields(cx);
                    cx.notify();
                }
                (_, Err(error)) => {
                    self.validation[1] = Some(error);
                    self.validation[0] = None;
                    self.sync_fields(cx);
                    cx.notify();
                }
                (Ok(_), Ok(_)) => {
                    self.validation[i] = Some(self.text_labels.outside_bounds.clone());
                    self.sync_fields(cx);
                    cx.notify();
                }
            }
            return;
        }
        match (self.parser)(&self.draft[i]) {
            Ok(date) if self.within_bounds(date) => {
                let next = if self.range_mode {
                    let prior = match self.value {
                        DateValue::Range(v) => v,
                        _ => None,
                    };
                    let other =
                        prior.map(|r| if self.active_endpoint_end { r.start } else { r.end });
                    DateValue::Range(other.map(|o| {
                        DateRange::new(
                            if self.active_endpoint_end { o } else { date },
                            if self.active_endpoint_end { date } else { o },
                        )
                    }))
                } else {
                    DateValue::Single(Some(date))
                };
                self.commit(next, cx);
            }
            Ok(_) => {
                self.validation[i] = Some(self.text_labels.outside_bounds.clone());
                self.sync_fields(cx);
                cx.notify();
            }
            Err(error) => {
                self.validation[i] = Some(error);
                self.sync_fields(cx);
                cx.notify();
            }
        }
    }
    fn commit(&mut self, value: DateValue, cx: &mut Context<Self>) {
        match (self.controlled, value) {
            (true, DateValue::Single(value)) => cx.emit(ChangeRequested(DateValue::Single(value))),
            (true, DateValue::Range(value)) => cx.emit(RangeChangeRequested(value)),
            (false, DateValue::Single(value)) => {
                self.value = DateValue::Single(value);
                self.initial_value = self.value;
                self.sync_draft();
                self.sync_fields(cx);
                cx.emit(Changed(self.value));
            }
            (false, DateValue::Range(value)) => {
                self.value = DateValue::Range(value);
                self.initial_value = self.value;
                self.sync_draft();
                self.sync_fields(cx);
                cx.emit(RangeChanged(value));
            }
        }
        self.restore_focus_when_closed = true;
        self.set_open(false, cx);
    }
    fn calendar_commit(&mut self, selection: CalendarSelection, cx: &mut Context<Self>) {
        let date_value = match selection {
            CalendarSelection::Single(value) => DateValue::Single(value),
            CalendarSelection::Range(start, end) => {
                let range = start.zip(end).map(|(a, b)| DateRange::new(a, b));
                if self.range_mode {
                    DateValue::Range(range)
                } else {
                    return;
                }
            }
        };
        if !matches!(date_value, DateValue::Range(None)) {
            self.commit(date_value, cx);
        }
    }
    fn text_input_focus(&mut self, end: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.active_endpoint_end = end;
        if self.open
            && let Some(calendar) = &self.calendar
        {
            let selected = match self.value {
                DateValue::Single(v) => CalendarSelection::Single(v),
                DateValue::Range(v) => {
                    CalendarSelection::Range(v.map(|r| r.start), v.map(|r| r.end))
                }
            };
            calendar.update(cx, |calendar, cx| calendar.set_selection(selected, cx));
        }
        let _ = window;
    }
}

impl Focusable for DatePicker {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}
impl Render for DatePicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_children(cx);
        if self.last_open && !self.open && self.restore_focus_when_closed {
            if let Some(field) = &self.fields[usize::from(self.active_endpoint_end)] {
                field.update(cx, |field, cx| field.focus_handle(cx).focus(window, cx));
            }
            self.restore_focus_when_closed = false;
        }
        self.last_open = self.open;
        let theme = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_stop(true)).clone();
        let count = if self.range_mode { 2 } else { 1 };
        let mut fields = div().flex().items_center().gap(px(theme.spacing.small));
        for index in 0..count {
            let field = self.fields[index].as_ref().unwrap().clone();
            let end = index == 1;
            fields = fields.child(
                div()
                    .id(if end { "date-end-wrap" } else { "date-start-wrap" })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.text_input_focus(end, window, cx)
                    }))
                    .flex_1()
                    .child(field),
            );
        }
        let button = div()
            .id("date-picker-trigger")
            .debug_selector(|| "mkit-date-picker-trigger".into())
            .role(gpui_pre::accesskit::Role::Button)
            .aria_label(if self.range_mode {
                self.text_labels.choose_range.clone()
            } else {
                self.text_labels.choose_date.clone()
            })
            .aria_expanded(self.open)
            .track_focus(&focus)
            .tab_stop(!self.disabled)
            .px(px(theme.spacing.small))
            .py(px(theme.spacing.small))
            .rounded(px(theme.radii.small))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .text_color(theme.colors.text)
            .when(self.disabled, |button| button.opacity(0.55))
            .when(!self.disabled, |button| {
                button.on_click(
                    cx.listener(|this, _, window, cx| this.open_action(&OpenCalendar, window, cx)),
                )
            })
            .child(
                div()
                    .size(px(14.0))
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.hairline))
                    .border_color(theme.colors.text)
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(div().h(px(3.0)).w_full().bg(theme.colors.text))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .justify_around()
                            .px(px(2.0))
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .child(
                                        div().size(px(2.0)).rounded(px(1.0)).bg(theme.colors.text),
                                    )
                                    .child(
                                        div().size(px(2.0)).rounded(px(1.0)).bg(theme.colors.text),
                                    )
                                    .child(
                                        div().size(px(2.0)).rounded(px(1.0)).bg(theme.colors.text),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .child(
                                        div().size(px(2.0)).rounded(px(1.0)).bg(theme.colors.text),
                                    )
                                    .child(
                                        div().size(px(2.0)).rounded(px(1.0)).bg(theme.colors.text),
                                    )
                                    .child(
                                        div().size(px(2.0)).rounded(px(1.0)).bg(theme.colors.text),
                                    ),
                            ),
                    ),
            );
        fields = fields.child(button);
        let anchor_bounds = self.field_bounds.borrow().as_ref().copied();
        let field_bounds = Rc::clone(&self.field_bounds);
        let mut root = div()
            .on_children_prepainted(move |children, _, _| {
                if let Some(bounds) = children.first() {
                    *field_bounds.borrow_mut() = Some(*bounds);
                }
            })
            .id("date-picker")
            .debug_selector(|| "mkit-date-picker".into())
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::open_action))
            .on_action(cx.listener(Self::dismiss))
            .on_action(cx.listener(Self::commit_action))
            .on_key_down(cx.listener(Self::key_down))
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                if this.open {
                    this.set_open(false, cx);
                }
            }))
            .track_focus(&focus)
            .flex()
            .items_center()
            .gap(px(theme.spacing.small))
            .child(fields);
        if self.open {
            if let Some(calendar) = self.calendar.as_ref() {
                let panel = div()
                    .id("date-picker-calendar")
                    .debug_selector(|| "mkit-date-picker-calendar".into())
                    .p(px(theme.spacing.medium))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.elevated_surface)
                    .text_color(theme.colors.text)
                    .child(calendar.clone());
                if let Some(bounds) = anchor_bounds {
                    root = root.child(
                        deferred(
                            anchored()
                                .anchor(gpui_pre::Anchor::TopLeft)
                                .position_mode(AnchoredPositionMode::Window)
                                .position(point(
                                    bounds.left(),
                                    bounds.bottom() + px(theme.spacing.small),
                                ))
                                .snap_to_window_with_margin(px(theme.spacing.medium))
                                .child(panel),
                        )
                        .with_priority(mkit_core::overlay::layer::POPOVER),
                    );
                }
            } else if let Some(bounds) = anchor_bounds {
                let panel = div()
                    .id("date-picker-empty-calendar")
                    .p(px(theme.spacing.medium))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.elevated_surface)
                    .text_color(theme.colors.text)
                    .child(self.text_labels.no_available_dates.clone());
                root = root.child(
                    deferred(
                        anchored()
                            .anchor(gpui_pre::Anchor::TopLeft)
                            .position_mode(AnchoredPositionMode::Window)
                            .position(point(
                                bounds.left(),
                                bounds.bottom() + px(theme.spacing.small),
                            ))
                            .snap_to_window_with_margin(px(theme.spacing.medium))
                            .child(panel),
                    )
                    .with_priority(mkit_core::overlay::layer::POPOVER),
                );
            }
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Focusable, TestAppContext};
    use std::{cell::RefCell, rc::Rc};

    fn calendar_labels() -> CalendarLabels {
        CalendarLabels {
            weekdays_monday_first: ["M", "T", "W", "T", "F", "S", "S"].map(str::to_owned),
            months: [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ]
            .map(str::to_owned),
            first_weekday: 0,
        }
    }
    fn date_labels() -> DatePickerLabels {
        DatePickerLabels {
            single_placeholder: "YYYY-MM-DD".into(),
            start_placeholder: "Start".into(),
            end_placeholder: "End".into(),
            start_suffix: "start".into(),
            end_suffix: "end".into(),
            choose_date: "Choose date".into(),
            choose_range: "Choose range".into(),
            incomplete_range: "Enter both dates".into(),
            outside_bounds: "Unavailable date".into(),
            no_available_dates: "No dates available".into(),
        }
    }
    fn parse_date(s: &str) -> Result<CivilDate, String> {
        let parts = s.split('-').collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err("Use YYYY-MM-DD".into());
        }
        let year = parts[0].parse().map_err(|_| "Invalid year".to_string())?;
        let month = parts[1].parse().map_err(|_| "Invalid month".to_string())?;
        let day = parts[2].parse().map_err(|_| "Invalid day".to_string())?;
        CivilDate::new(year, month, day).ok_or_else(|| "Invalid date".into())
    }
    fn format_date(date: CivilDate) -> String {
        format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
    }

    #[gpui_pre::test]
    fn keyboard_opens_calendar_and_commits_active_date(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = CivilDate::new(2024, 4, 15).unwrap();
        let (picker, visual) = cx.add_window_view(|_, _| {
            DatePicker::new(
                "Date",
                None,
                initial,
                calendar_labels(),
                date_labels(),
                parse_date,
                format_date,
            )
        });
        let changes = Rc::new(RefCell::new(Vec::new()));
        let change_log = changes.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&picker, move |_, event: &Changed, _| {
                change_log.borrow_mut().push(event.0)
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field =
            picker.read_with(visual, |picker, _| picker.fields[0].as_ref().unwrap().clone());
        visual.update(|window, cx| field.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("alt-down enter");
        assert_eq!(
            picker.read_with(visual, |picker, _| picker.value()),
            DateValue::Single(Some(initial))
        );
        assert_eq!(changes.borrow().as_slice(), &[DateValue::Single(Some(initial))]);
    }

    #[gpui_pre::test]
    fn range_enter_requires_both_endpoints_without_emitting_a_change(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = CivilDate::new(2024, 4, 15).unwrap();
        let (picker, visual) = cx.add_window_view(|_, _| {
            DatePicker::range(
                "Dates",
                None,
                initial,
                calendar_labels(),
                date_labels(),
                parse_date,
                format_date,
            )
        });
        let changes = Rc::new(RefCell::new(0));
        let change_log = changes.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&picker, move |_, _: &RangeChanged, _| *change_log.borrow_mut() += 1)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field =
            picker.read_with(visual, |picker, _| picker.fields[0].as_ref().unwrap().clone());
        visual.update(|window, cx| field.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("enter");
        assert_eq!(picker.read_with(visual, |picker, _| picker.value()), DateValue::Range(None));
        assert_eq!(*changes.borrow(), 0);
        assert_eq!(
            picker
                .read_with(visual, |picker, _| picker.validation_message(false).map(str::to_owned)),
            Some("Enter both dates".to_owned())
        );
    }

    #[gpui_pre::test]
    fn typed_date_parses_and_commits_on_enter(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = CivilDate::new(2024, 4, 15).unwrap();
        let expected = CivilDate::new(2024, 4, 20).unwrap();
        let (picker, visual) = cx.add_window_view(|_, _| {
            DatePicker::new(
                "Date",
                None,
                initial,
                calendar_labels(),
                date_labels(),
                parse_date,
                format_date,
            )
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field =
            picker.read_with(visual, |picker, _| picker.fields[0].as_ref().unwrap().clone());
        visual.update(|window, cx| field.focus_handle(cx).focus(window, cx));
        visual.simulate_input("2024-04-20");
        visual.simulate_keystrokes("enter");
        assert_eq!(
            picker.read_with(visual, |picker, _| picker.value()),
            DateValue::Single(Some(expected))
        );
    }

    #[test]
    fn initial_calendar_date_moves_inside_bounds_and_skips_disabled_dates() {
        let preferred = CivilDate::new(2024, 4, 1).unwrap();
        let min = preferred.add_days(10).unwrap();
        let expected = min.add_days(1).unwrap();
        let picker = DatePicker::new(
            "Date",
            None,
            preferred,
            calendar_labels(),
            date_labels(),
            parse_date,
            format_date,
        )
        .bounds(Some(min), Some(min.add_days(5).unwrap()))
        .disabled_dates(move |date| date == min);
        assert_eq!(picker.find_available_date(preferred), Some(expected));

        let unavailable = DatePicker::new(
            "Date",
            None,
            preferred,
            calendar_labels(),
            date_labels(),
            parse_date,
            format_date,
        )
        .bounds(Some(min), Some(min))
        .disabled_dates(move |date| date == min);
        assert_eq!(unavailable.find_available_date(preferred), None);
    }

    #[gpui_pre::test]
    fn escape_restores_committed_text_and_controlled_commit_waits_for_owner(
        cx: &mut TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let old = CivilDate::new(2024, 4, 15).unwrap();
        let new = CivilDate::new(2024, 4, 20).unwrap();
        let (picker, visual) = cx.add_window_view(|_, _| {
            DatePicker::new(
                "Date",
                Some(old),
                old,
                calendar_labels(),
                date_labels(),
                parse_date,
                format_date,
            )
            .controlled()
        });
        let proposals = Rc::new(RefCell::new(Vec::new()));
        let proposal_log = proposals.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&picker, move |_, event: &ChangeRequested, _| {
                proposal_log.borrow_mut().push(event.0)
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field =
            picker.read_with(visual, |picker, _| picker.fields[0].as_ref().unwrap().clone());
        visual.update(|window, cx| field.focus_handle(cx).focus(window, cx));
        visual.simulate_input("bad draft");
        visual.simulate_keystrokes("escape");
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), format_date(old));
        visual.simulate_keystrokes("cmd-a");
        visual.simulate_input(&format_date(new));
        visual.simulate_keystrokes("enter");
        assert_eq!(
            picker.read_with(visual, |picker, _| picker.value()),
            DateValue::Single(Some(old))
        );
        assert_eq!(proposals.borrow().as_slice(), &[DateValue::Single(Some(new))]);
        picker.update(visual, |picker, cx| picker.set_value(DateValue::Single(Some(new)), cx));
        assert_eq!(
            picker.read_with(visual, |picker, _| picker.value()),
            DateValue::Single(Some(new))
        );
    }

    #[gpui_pre::test]
    fn typed_range_commits_only_after_both_fields_parse(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = CivilDate::new(2024, 4, 1).unwrap();
        let start = CivilDate::new(2024, 4, 12).unwrap();
        let end = CivilDate::new(2024, 4, 18).unwrap();
        let (picker, visual) = cx.add_window_view(|_, _| {
            DatePicker::range(
                "Dates",
                None,
                initial,
                calendar_labels(),
                date_labels(),
                parse_date,
                format_date,
            )
        });
        let _subscription =
            visual.update(|_, app| app.subscribe(&picker, |_, _: &RangeChanged, _| {}));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (start_field, end_field) = picker.read_with(visual, |picker, _| {
            (picker.fields[0].as_ref().unwrap().clone(), picker.fields[1].as_ref().unwrap().clone())
        });
        visual.update(|window, cx| start_field.focus_handle(cx).focus(window, cx));
        visual.simulate_input(&format_date(start));
        visual.update(|window, cx| end_field.focus_handle(cx).focus(window, cx));
        visual.simulate_input(&format_date(end));
        visual.simulate_keystrokes("enter");
        assert_eq!(
            picker.read_with(visual, |picker, _| picker.value()),
            DateValue::Range(Some(DateRange::new(start, end)))
        );
    }

    #[gpui_pre::test]
    fn disabled_picker_cannot_open(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let initial = CivilDate::new(2024, 4, 1).unwrap();
        let (picker, visual) = cx.add_window_view(|_, _| {
            DatePicker::new(
                "Date",
                None,
                initial,
                calendar_labels(),
                date_labels(),
                parse_date,
                format_date,
            )
            .disabled(true)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let trigger = visual.debug_bounds("mkit-date-picker-trigger").unwrap().center();
        visual.simulate_click(trigger, Default::default());
        assert!(!picker.read_with(visual, |picker, _| picker.is_open()));
    }
}
