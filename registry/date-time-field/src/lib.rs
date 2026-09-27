//! Passive composition of DatePicker and TimeField.
extern crate gpui_pre as gpui;

#[cfg(feature = "mkit-mirror")]
use crate::{date_picker::DatePicker, time_field::TimeField};
use gpui_pre::{App, Entity, IntoElement, RenderOnce, Window, div, prelude::*, px};
use mkit_core::theme::Theme;
#[cfg(not(feature = "mkit-mirror"))]
use mkit_registry_date_picker::DatePicker;
#[cfg(not(feature = "mkit-mirror"))]
use mkit_registry_time_field::TimeField;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(IntoElement)]
pub struct DateTimeField {
    id: usize,
    label: String,
    date: Entity<DatePicker>,
    time: Entity<TimeField>,
    separator: String,
}

impl DateTimeField {
    /// Compose already-configured date and time fields. The caller retains each child entity and
    /// subscribes to its own controlled or uncontrolled events.
    pub fn new(
        label: impl Into<String>,
        date: Entity<DatePicker>,
        time: Entity<TimeField>,
        separator: impl Into<String>,
    ) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            label: label.into(),
            date,
            time,
            separator: separator.into(),
        }
    }
}

impl RenderOnce for DateTimeField {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        div()
            .id(("mkit-date-time-field", self.id))
            .debug_selector(|| "mkit-date-time-field".into())
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.label)
            .flex()
            .items_center()
            .gap(px(theme.spacing.small))
            .child(self.date)
            .child(
                div()
                    .text_color(theme.colors.text_muted)
                    .text_size(px(theme.typography.body))
                    .child(self.separator),
            )
            .child(self.time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "mkit-mirror")]
    use crate::{
        date_picker::{CalendarLabels, DatePickerLabels},
        time_field::{
            TimeChanged, TimeFieldLabels, TimeValue, default_key_bindings as time_key_bindings,
        },
    };
    use gpui_pre::{Context, Focusable, Render};
    use mkit_core::CivilDate;
    #[cfg(not(feature = "mkit-mirror"))]
    use mkit_registry_date_picker::{CalendarLabels, DatePickerLabels};
    #[cfg(not(feature = "mkit-mirror"))]
    use mkit_registry_time_field::{
        TimeChanged, TimeFieldLabels, TimeValue, default_key_bindings as time_key_bindings,
    };
    use std::{cell::RefCell, rc::Rc};

    struct CompositionFixture {
        date: Entity<DatePicker>,
        time: Entity<TimeField>,
    }

    impl Render for CompositionFixture {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().flex().child(DateTimeField::new(
                "Appointment",
                self.date.clone(),
                self.time.clone(),
                "at",
            ))
        }
    }

    fn calendar_labels() -> CalendarLabels {
        CalendarLabels {
            weekdays_monday_first: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
                .map(str::to_owned),
            months: [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ]
            .map(str::to_owned),
            first_weekday: 0,
        }
    }

    fn picker_labels() -> DatePickerLabels {
        DatePickerLabels {
            single_placeholder: "YYYY-MM-DD".into(),
            start_placeholder: "Start".into(),
            end_placeholder: "End".into(),
            start_suffix: "start".into(),
            end_suffix: "end".into(),
            choose_date: "Choose date".into(),
            choose_range: "Choose range".into(),
            incomplete_range: "Enter both dates".into(),
            outside_bounds: "Date unavailable".into(),
            no_available_dates: "No dates available".into(),
        }
    }

    fn time_labels() -> TimeFieldLabels {
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

    fn parse_date(text: &str) -> Result<CivilDate, String> {
        let parts = text.split('-').collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err("Use YYYY-MM-DD".into());
        }
        let year = parts[0].parse().map_err(|_| "Invalid year".to_owned())?;
        let month = parts[1].parse().map_err(|_| "Invalid month".to_owned())?;
        let day = parts[2].parse().map_err(|_| "Invalid day".to_owned())?;
        CivilDate::new(year, month, day).ok_or_else(|| "Invalid date".into())
    }

    fn format_date(date: CivilDate) -> String {
        format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
    }

    #[gpui_pre::test]
    fn composition_preserves_child_time_events_and_values(cx: &mut gpui_pre::TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(time_key_bindings()));
        let (fixture, window) = cx.add_window_view(|_, cx| {
            let date = cx.new(|_| {
                DatePicker::new(
                    "Date",
                    None,
                    CivilDate::new(2024, 4, 15).unwrap(),
                    calendar_labels(),
                    picker_labels(),
                    parse_date,
                    format_date,
                )
            });
            let time = cx.new(|_| {
                TimeField::new("Time", Some(TimeValue::new(10, 30, 0).unwrap()), time_labels())
            });
            CompositionFixture { date, time }
        });
        let time = fixture.read_with(window, |view, _| view.time.clone());
        let changes = Rc::new(RefCell::new(Vec::new()));
        let change_log = changes.clone();
        let _subscription = window.update(|_, app| {
            app.subscribe(&time, move |_, event: &TimeChanged, _| {
                change_log.borrow_mut().push(*event)
            })
        });
        window.update(|win, cx| {
            win.draw(cx).clear(cx);
            let focus = time.focus_handle(cx);
            focus.focus(win, cx);
        });
        window.simulate_keystrokes("up");
        let expected = Some(TimeValue::new(11, 30, 0).unwrap());
        assert_eq!(time.read_with(window, |time, _| time.value()), expected);
        assert_eq!(changes.borrow().as_slice(), &[TimeChanged(expected)]);
    }
}
