//! Theme-driven accessible month calendar.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render, Window,
    actions, div, prelude::*, px,
};
use mkit_core::{CivilDate, theme::Theme};
use std::sync::Arc;

pub const KEY_CONTEXT: &str = "Calendar";
actions!(
    calendar,
    [
        DayForward,
        DayBack,
        WeekForward,
        WeekBack,
        MonthForward,
        MonthBack,
        YearForward,
        YearBack,
        WeekStart,
        WeekEnd,
        Select
    ]
);
pub fn default_key_bindings() -> [KeyBinding; 12] {
    [
        KeyBinding::new("right", DayForward, Some(KEY_CONTEXT)),
        KeyBinding::new("left", DayBack, Some(KEY_CONTEXT)),
        KeyBinding::new("down", WeekForward, Some(KEY_CONTEXT)),
        KeyBinding::new("up", WeekBack, Some(KEY_CONTEXT)),
        KeyBinding::new("pageup", MonthBack, Some(KEY_CONTEXT)),
        KeyBinding::new("pagedown", MonthForward, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-pageup", YearBack, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-pagedown", YearForward, Some(KEY_CONTEXT)),
        KeyBinding::new("home", WeekStart, Some(KEY_CONTEXT)),
        KeyBinding::new("end", WeekEnd, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Select, Some(KEY_CONTEXT)),
        KeyBinding::new("space", Select, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Selection {
    Single(Option<CivilDate>),
    Range(Option<CivilDate>, Option<CivilDate>),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectionChanged(pub Selection);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectionRequested(pub Selection);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveDateChanged(pub CivilDate);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ViewChanged(pub CivilDate);
impl EventEmitter<SelectionChanged> for Calendar {}
impl EventEmitter<SelectionRequested> for Calendar {}
impl EventEmitter<ActiveDateChanged> for Calendar {}
impl EventEmitter<ViewChanged> for Calendar {}

type DisabledDate = Arc<dyn Fn(CivilDate) -> bool + Send + Sync>;
pub struct Calendar {
    label: String,
    active: CivilDate,
    selection: Selection,
    controlled: bool,
    first_weekday: u8,
    weekdays: [String; 7],
    months: [String; 12],
    min: Option<CivilDate>,
    max: Option<CivilDate>,
    is_disabled: DisabledDate,
    focus: Option<FocusHandle>,
}
impl Calendar {
    pub fn new(
        label: impl Into<String>,
        active: CivilDate,
        weekdays: [String; 7],
        months: [String; 12],
    ) -> Self {
        Self {
            label: label.into(),
            active,
            selection: Selection::Single(None),
            controlled: false,
            first_weekday: 0,
            weekdays,
            months,
            min: None,
            max: None,
            is_disabled: Arc::new(|_| false),
            focus: None,
        }
    }
    pub fn controlled(
        label: impl Into<String>,
        active: CivilDate,
        selection: Selection,
        weekdays: [String; 7],
        months: [String; 12],
    ) -> Self {
        let mut calendar = Self::new(label, active, weekdays, months);
        calendar.selection = selection;
        calendar.controlled = true;
        calendar
    }
    pub fn first_weekday(mut self, monday_zero: u8) -> Self {
        self.first_weekday = monday_zero % 7;
        self
    }
    pub fn range_mode(mut self) -> Self {
        self.selection = Selection::Range(None, None);
        self
    }
    pub fn default_selection(mut self, selection: Selection) -> Self {
        self.selection = selection;
        self.controlled = false;
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
        self.is_disabled = Arc::new(predicate);
        self
    }
    pub fn is_disabled(&self, date: CivilDate) -> bool {
        self.min.is_some_and(|min| date < min)
            || self.max.is_some_and(|max| date > max)
            || (self.is_disabled)(date)
    }
    pub fn active_date(&self) -> CivilDate {
        self.active
    }
    /// Focus the active date cell, initializing its handle before the calendar is first rendered.
    pub fn request_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let handle = self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(0)).clone();
        handle.focus(window, cx);
    }
    pub fn selection(&self) -> Selection {
        self.selection
    }
    pub fn set_selection(&mut self, selection: Selection, cx: &mut Context<Self>) {
        self.selection = selection;
        cx.notify();
    }

    fn move_days(&mut self, amount: i32, cx: &mut Context<Self>) {
        if amount == 0 {
            return;
        }
        let Some(mut candidate) = self.active.add_days(amount) else {
            return;
        };
        loop {
            if self.min.is_some_and(|min| candidate < min)
                || self.max.is_some_and(|max| candidate > max)
            {
                break;
            }
            if !self.is_disabled(candidate) {
                self.active = candidate;
                cx.emit(ActiveDateChanged(candidate));
                cx.notify();
                break;
            }
            let Some(next) = candidate.add_days(amount.signum()) else {
                break;
            };
            candidate = next;
        }
    }
    fn move_months(&mut self, amount: i32, cx: &mut Context<Self>) {
        if let Some(next) = self.active.add_months(amount) {
            if self.min.is_some_and(|min| next < min) || self.max.is_some_and(|max| next > max) {
                return;
            }
            if !self.is_disabled(next) {
                self.active = next;
                cx.emit(ViewChanged(next));
                cx.emit(ActiveDateChanged(next));
                cx.notify();
            } else {
                self.active = next;
                cx.emit(ViewChanged(next));
                cx.notify();
                self.move_days(amount.signum(), cx);
            }
        }
    }
    fn day_forward(&mut self, _: &DayForward, _: &mut Window, cx: &mut Context<Self>) {
        self.move_days(1, cx);
    }
    fn day_back(&mut self, _: &DayBack, _: &mut Window, cx: &mut Context<Self>) {
        self.move_days(-1, cx);
    }
    fn week_forward(&mut self, _: &WeekForward, _: &mut Window, cx: &mut Context<Self>) {
        self.move_days(7, cx);
    }
    fn week_back(&mut self, _: &WeekBack, _: &mut Window, cx: &mut Context<Self>) {
        self.move_days(-7, cx);
    }
    fn month_forward(&mut self, _: &MonthForward, _: &mut Window, cx: &mut Context<Self>) {
        self.move_months(1, cx);
    }
    fn month_back(&mut self, _: &MonthBack, _: &mut Window, cx: &mut Context<Self>) {
        self.move_months(-1, cx);
    }
    fn year_forward(&mut self, _: &YearForward, _: &mut Window, cx: &mut Context<Self>) {
        self.move_months(12, cx);
    }
    fn year_back(&mut self, _: &YearBack, _: &mut Window, cx: &mut Context<Self>) {
        self.move_months(-12, cx);
    }
    fn week_start(&mut self, _: &WeekStart, _: &mut Window, cx: &mut Context<Self>) {
        let delta =
            (self.active.weekday_monday_zero() as i32 - self.first_weekday as i32).rem_euclid(7);
        self.move_days(-delta, cx);
    }
    fn week_end(&mut self, _: &WeekEnd, _: &mut Window, cx: &mut Context<Self>) {
        let delta = (self.first_weekday as i32 + 6 - self.active.weekday_monday_zero() as i32)
            .rem_euclid(7);
        self.move_days(delta, cx);
    }
    fn select(&mut self, _: &Select, _: &mut Window, cx: &mut Context<Self>) {
        self.select_date(self.active, cx);
    }
    fn select_date(&mut self, date: CivilDate, cx: &mut Context<Self>) {
        if self.is_disabled(date) {
            return;
        }
        let selection = match self.selection {
            Selection::Single(_) => Selection::Single(Some(date)),
            Selection::Range(None, _) | Selection::Range(_, Some(_)) => {
                Selection::Range(Some(date), None)
            }
            Selection::Range(Some(start), None) => {
                if date < start {
                    Selection::Range(Some(date), Some(start))
                } else {
                    Selection::Range(Some(start), Some(date))
                }
            }
        };
        if !self.controlled {
            self.selection = selection;
            cx.notify();
            cx.emit(SelectionChanged(selection));
        } else {
            cx.emit(SelectionRequested(selection));
        }
    }
}
impl Focusable for Calendar {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("calendar focus initialized")
    }
}
impl Render for Calendar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(0)).clone();
        let theme = *cx.global::<Theme>();
        let active = self.active;
        let selection = self.selection;
        let first_weekday = self.first_weekday;
        let start = CivilDate::new(active.year(), active.month(), 1).unwrap();
        let offset = (start.weekday_monday_zero() as i32 - first_weekday as i32).rem_euclid(7);
        let grid_start = start.add_days(-offset).unwrap_or(start);
        let mut root = div()
            .id("calendar")
            .debug_selector(|| "mkit-calendar".into())
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::day_forward))
            .on_action(cx.listener(Self::day_back))
            .on_action(cx.listener(Self::week_forward))
            .on_action(cx.listener(Self::week_back))
            .on_action(cx.listener(Self::month_forward))
            .on_action(cx.listener(Self::month_back))
            .on_action(cx.listener(Self::year_forward))
            .on_action(cx.listener(Self::year_back))
            .on_action(cx.listener(Self::week_start))
            .on_action(cx.listener(Self::week_end))
            .on_action(cx.listener(Self::select))
            .role(gpui_pre::accesskit::Role::Grid)
            .aria_label(self.label.clone())
            .flex()
            .flex_col()
            .gap(px(theme.spacing.xsmall))
            .text_color(theme.colors.text);
        root = root.child(div().flex().justify_between().child(format!(
            "{} {}",
            self.months[active.month() as usize - 1],
            active.year()
        )));
        let mut header = div().flex().justify_between();
        for col in 0..7 {
            header = header.child(
                div()
                    .w(px(theme.controls.small))
                    .text_size(px(theme.typography.caption))
                    .child(self.weekdays[(first_weekday as usize + col) % 7].clone()),
            );
        }
        root = root.child(header);
        for row in 0..6 {
            let mut week = div().flex().justify_between();
            for col in 0..7 {
                let date = grid_start.add_days(row * 7 + col).unwrap_or(start);
                let disabled = self.is_disabled(date);
                let selected = match selection {
                    Selection::Single(Some(d)) => d == date,
                    Selection::Range(Some(a), Some(b)) => date >= a && date <= b,
                    Selection::Range(Some(a), None) => a == date,
                    _ => false,
                };
                let active_day = date == active;
                let in_month = date.month() == active.month();
                let day = date.day().to_string();
                let bg = if selected { theme.colors.accent } else { theme.colors.surface };
                let fg = if selected {
                    theme.colors.accent_text
                } else if disabled || !in_month {
                    theme.colors.disabled
                } else {
                    theme.colors.text
                };
                let mut cell = div()
                    .id(format!("day-{}-{}-{}", date.year(), date.month(), date.day()))
                    .role(gpui_pre::accesskit::Role::GridCell)
                    .aria_label(format!(
                        "{} {}, {}",
                        self.months[date.month() as usize - 1],
                        date.day(),
                        date.year()
                    ))
                    .aria_selected(selected)
                    .when(disabled, |d| {
                        d.a11y_synthetic_children(|b| b.parent_node().set_disabled())
                    })
                    .w(px(theme.controls.small))
                    .h(px(theme.controls.small))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(theme.radii.small))
                    .text_size(px(theme.typography.body))
                    .text_color(fg)
                    .bg(bg)
                    .child(day);
                if active_day {
                    cell = cell.track_focus(&focus);
                }
                if !disabled {
                    cell = cell.on_click(cx.listener(move |this, _, _, cx| {
                        this.active = date;
                        this.select_date(date, cx);
                    }));
                }
                week = week.child(cell);
            }
            root = root.child(week);
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Focusable, TestAppContext};
    use std::{cell::RefCell, rc::Rc};
    #[test]
    fn range_endpoints_are_chronological() {
        let first = CivilDate::new(2025, 4, 9).unwrap();
        let second = CivilDate::new(2025, 4, 2).unwrap();
        let (start, end) = match (Selection::Range(Some(first), None), second) {
            (Selection::Range(Some(start), None), date) if date < start => (date, start),
            (Selection::Range(Some(start), None), date) => (start, date),
            _ => unreachable!(),
        };
        assert_eq!((start, end), (second, first));
    }

    fn labels() -> ([String; 7], [String; 12]) {
        (
            ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].map(str::to_owned),
            ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
                .map(str::to_owned),
        )
    }

    #[gpui_pre::test]
    fn keyboard_navigation_respects_bounds_and_selects_single_date(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let min = CivilDate::new(2024, 4, 18).unwrap();
        let initial = CivilDate::new(2024, 4, 17).unwrap();
        let (weekdays, months) = labels();
        let (calendar, visual) = cx.add_window_view(|_, _| {
            Calendar::new("Choose date", initial, weekdays, months).bounds(Some(min), None)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| calendar.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("right");
        assert_eq!(calendar.read_with(visual, |c, _| c.active_date()), min);
        visual.simulate_keystrokes("down");
        let after_week = min.add_days(7).unwrap();
        assert_eq!(calendar.read_with(visual, |c, _| c.active_date()), after_week);
        visual.simulate_keystrokes("enter");
        assert_eq!(
            calendar.read_with(visual, |c, _| c.selection()),
            Selection::Single(Some(after_week))
        );
    }

    #[gpui_pre::test]
    fn range_activation_emits_two_ordered_endpoints(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let later = CivilDate::new(2024, 4, 19).unwrap();
        let earlier = CivilDate::new(2024, 4, 17).unwrap();
        let (weekdays, months) = labels();
        let (calendar, visual) = cx.add_window_view(|_, _| {
            Calendar::new("Choose range", later, weekdays, months).range_mode()
        });
        let events = Rc::new(RefCell::new(Vec::new()));
        let event_log = events.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&calendar, move |_, event: &SelectionChanged, _| {
                event_log.borrow_mut().push(*event)
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| calendar.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("enter left left enter");
        let expected = Selection::Range(Some(earlier), Some(later));
        assert_eq!(calendar.read_with(visual, |c, _| c.selection()), expected);
        assert_eq!(events.borrow().len(), 2);
        assert_eq!(events.borrow()[1].0, expected);
    }

    #[gpui_pre::test]
    fn navigation_skips_long_app_disabled_span(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = CivilDate::new(2024, 1, 1).unwrap();
        let last_disabled = initial.add_days(500).unwrap();
        let (weekdays, months) = labels();
        let (calendar, visual) = cx.add_window_view(|_, _| {
            Calendar::new("Choose date", initial, weekdays, months)
                .disabled_dates(move |date| date > initial && date <= last_disabled)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| calendar.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("right");
        assert_eq!(
            calendar.read_with(visual, |c, _| c.active_date()),
            last_disabled.add_days(1).unwrap()
        );
    }
}
