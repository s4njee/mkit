//! Theme-driven accessible month calendar.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement, KeyBinding,
    PathBuilder, Render, Rgba, Window, actions, canvas, div, point, prelude::*, px,
};
use mkit_core::{
    CivilDate,
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
use std::sync::Arc;

/// Resolved calendar colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    card_fill: Rgba,
    card_border: Rgba,
    /// `shadows.small`; `None` in high contrast.
    card_shadow: Option<ShadowToken>,
    text: Rgba,
    muted_text: Rgba,
    /// Days outside the displayed month.
    outside_text: Rgba,
    disabled_text: Rgba,
    /// Pointer-hover fill for enabled days and chevrons; `None` in high contrast.
    hover_bg: Option<Rgba>,
    /// Pointer-hover border in high contrast.
    hover_border: Option<Rgba>,
    selected_bg: Rgba,
    selected_text: Rgba,
    /// Range-middle band fill; transparent in high contrast.
    band_bg: Rgba,
    /// Range-middle border in high contrast.
    band_border: Option<Rgba>,
    focus: Rgba,
    ring: Rgba,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    let transparent = c.background.opacity(0.);
    if t.name == "high-contrast" {
        return Look {
            card_fill: c.background,
            card_border: c.border,
            card_shadow: None,
            text: c.text,
            muted_text: c.text_muted,
            // `text_muted` is near-white here, so outside days use the lighter-weight `disabled`.
            outside_text: c.disabled,
            disabled_text: c.disabled,
            hover_bg: None,
            hover_border: Some(c.accent),
            selected_bg: c.accent,
            selected_text: c.accent_text,
            band_bg: transparent,
            band_border: Some(c.accent),
            focus: c.focus,
            ring: c.focus,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn "accent": text mixed into the background.
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    let card_fill = c.background;
    Look {
        card_fill,
        card_border: if dark { mix(c.text, card_fill, 0.1) } else { c.border },
        card_shadow: Some(t.shadows.small),
        text: c.text,
        muted_text: c.text_muted,
        outside_text: c.text_muted,
        // shadcn `text-muted-foreground opacity-50`, as one opaque colour over the card.
        disabled_text: mix(c.text_muted, card_fill, 0.5),
        hover_bg: Some(muted),
        hover_border: None,
        selected_bg: c.accent,
        selected_text: c.accent_text,
        band_bg: muted,
        band_border: None,
        focus: c.focus,
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
/// shadcn/ui focus ring width, drawn outside the focused day.
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
/// Lucide `chevron-left`.
const CHEVRON_LEFT: &[&[(f32, f32)]] = &[&[(15., 18.), (9., 12.), (15., 6.)]];
/// Lucide `chevron-right`.
const CHEVRON_RIGHT: &[&[(f32, f32)]] = &[&[(9., 18.), (15., 12.), (9., 6.)]];
/// Where a day sits in the selected range, which decides its fill and band.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RangePart {
    None,
    /// A single selected date, or a range with one endpoint so far.
    Selected,
    Start,
    End,
    Middle,
}
fn range_part(selection: Selection, date: CivilDate) -> RangePart {
    match selection {
        Selection::Single(Some(d)) | Selection::Range(Some(d), None) if d == date => {
            RangePart::Selected
        }
        Selection::Range(Some(a), Some(b)) => {
            let (start, end) = if a <= b { (a, b) } else { (b, a) };
            if date == start && date == end {
                RangePart::Selected
            } else if date == start {
                RangePart::Start
            } else if date == end {
                RangePart::End
            } else if date > start && date < end {
                RangePart::Middle
            } else {
                RangePart::None
            }
        }
        _ => RangePart::None,
    }
}

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
        let look = look(&theme);
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
            .flex_none()
            // Seven day columns plus padding and border, so the card never stretches with its parent.
            .w(px(7. * theme.controls.medium
                + 2. * theme.spacing.small
                + 2. * theme.borders.regular))
            .gap(px(theme.spacing.xsmall))
            .p(px(theme.spacing.small))
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.regular))
            .border_color(look.card_border)
            .bg(look.card_fill)
            .when_some(look.card_shadow, |root, shadow| root.shadow(vec![box_shadow(shadow)]))
            .text_color(look.text);
        let nav_button =
            |id: &'static str, lines: &'static [&'static [(f32, f32)]], months: i32| {
                div()
                    .id(id)
                    .size(px(theme.controls.small))
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(look.card_fill.opacity(0.))
                    .hover(move |style| {
                        let style = match look.hover_bg {
                            Some(color) => style.bg(color),
                            None => style,
                        };
                        match look.hover_border {
                            Some(color) => style.border_color(color),
                            None => style,
                        }
                    })
                    .on_click(cx.listener(move |this, _, _, cx| this.move_months(months, cx)))
                    .child(icon(theme.spacing.large, lines, look.text))
            };
        root = root.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(nav_button("calendar-previous-month", CHEVRON_LEFT, -1))
                .child(
                    div()
                        .text_size(px(theme.typography.body))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(format!(
                            "{} {}",
                            self.months[active.month() as usize - 1],
                            active.year()
                        )),
                )
                .child(nav_button("calendar-next-month", CHEVRON_RIGHT, 1)),
        );
        let mut header = div().flex();
        for col in 0..7 {
            header = header.child(
                div()
                    .w(px(theme.controls.medium))
                    .h(px(theme.controls.xsmall))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(theme.typography.caption))
                    .text_color(look.muted_text)
                    .child(self.weekdays[(first_weekday as usize + col) % 7].clone()),
            );
        }
        root = root.child(header);
        for row in 0..6 {
            let mut week = div().flex();
            for col in 0..7 {
                let date = grid_start.add_days(row * 7 + col).unwrap_or(start);
                let disabled = self.is_disabled(date);
                let selected = match selection {
                    Selection::Single(Some(d)) => d == date,
                    Selection::Range(Some(a), Some(b)) => date >= a && date <= b,
                    Selection::Range(Some(a), None) => a == date,
                    _ => false,
                };
                let part = range_part(selection, date);
                let filled =
                    matches!(part, RangePart::Selected | RangePart::Start | RangePart::End);
                let active_day = date == active;
                let in_month = date.month() == active.month();
                let day = date.day().to_string();
                let fg = if filled {
                    look.selected_text
                } else if disabled {
                    look.disabled_text
                } else if !in_month {
                    look.outside_text
                } else {
                    look.text
                };
                let band = part == RangePart::Middle;
                let transparent = look.card_fill.opacity(0.);
                let bg = if filled { look.selected_bg } else { transparent };
                let border = match look.band_border {
                    Some(color) if band => color,
                    _ => transparent,
                };
                // The fill under the focus ring is opaque; see the spec's "Focus" note.
                let focus_bg = if filled {
                    look.selected_bg
                } else if band && look.band_bg.a > 0. {
                    look.band_bg
                } else {
                    look.card_fill
                };
                let hoverable = !disabled && !filled;
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
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.regular))
                    .border_color(border)
                    .text_size(px(theme.typography.body))
                    .text_color(fg)
                    .bg(bg)
                    .when(hoverable, |cell| {
                        cell.hover(move |style| {
                            let style = match look.hover_bg {
                                Some(color) => style.bg(color),
                                None => style,
                            };
                            match look.hover_border {
                                Some(color) => style.border_color(color),
                                None => style,
                            }
                        })
                    })
                    .focus_visible(move |style| {
                        style
                            .border_color(look.focus)
                            .bg(focus_bg)
                            .shadow(vec![focus_ring(look.ring)])
                    })
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
                // The slot carries the range band: the whole slot for middle days and the inner
                // side of each endpoint, so the band runs behind the endpoint's rounded corners.
                let radius = px(theme.radii.small);
                let slot = div()
                    .w(px(theme.controls.medium))
                    .h(px(theme.controls.small))
                    .flex_none()
                    .map(|slot| match part {
                        RangePart::Middle => slot.bg(look.band_bg),
                        RangePart::Start => slot.bg(look.band_bg).rounded_l(radius),
                        RangePart::End => slot.bg(look.band_bg).rounded_r(radius),
                        _ => slot,
                    })
                    .child(cell);
                week = week.child(slot);
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
