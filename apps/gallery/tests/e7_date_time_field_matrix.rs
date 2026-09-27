use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    calendar::default_key_bindings as calendar_key_bindings,
    core::{
        CivilDate,
        theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    },
    date_picker::{
        CalendarLabels, DatePicker, DatePickerLabels, default_key_bindings as date_key_bindings,
    },
    date_time_field::DateTimeField,
    time_field::{
        TimeField, TimeFieldLabels, TimeValue, default_key_bindings as time_key_bindings,
    },
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (720.0, 690.0);
const DATE: CivilDate = CivilDate::new(2024, 4, 15).unwrap();
fn initial_time() -> TimeValue {
    TimeValue::new(14, 30, 45).unwrap()
}

struct DateTimeFixture {
    state: &'static str,
    date: Option<Entity<DatePicker>>,
    time: Option<Entity<TimeField>>,
}

impl Render for DateTimeFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let date = self.date.get_or_insert_with(|| {
            cx.new(|_| {
                let value = matches!(
                    state,
                    "date_value" | "complete" | "date_picker_open" | "time_segment_focused"
                )
                .then_some(DATE);
                let picker = DatePicker::new(
                    "Appointment date",
                    value,
                    DATE,
                    calendar_labels(),
                    date_labels(),
                    parse_date,
                    format_date,
                );
                if state == "disabled" { picker.disabled(true) } else { picker }
            })
        });
        let time = self.time.get_or_insert_with(|| {
            cx.new(|_| {
                let value = matches!(
                    state,
                    "time_value" | "complete" | "date_picker_open" | "time_segment_focused"
                )
                .then_some(initial_time());
                let field = TimeField::new("Appointment time", value, time_labels())
                    .hour_cycle(mkit::time_field::HourCycle::H12)
                    .seconds(true)
                    .step(15);
                if state == "disabled" { field.disabled(true) } else { field }
            })
        });
        div()
            .size_full()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.heading_small))
                    .child("DateTimeField"),
            )
            .child(div().text_color(theme.colors.text_muted).child(state))
            .child(DateTimeField::new(
                "Appointment date and time",
                date.clone(),
                time.clone(),
                "at",
            ))
    }
}

fn calendar_labels() -> CalendarLabels {
    CalendarLabels {
        weekdays_monday_first: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].map(str::to_owned),
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

fn date_labels() -> DatePickerLabels {
    DatePickerLabels {
        single_placeholder: "YYYY-MM-DD".into(),
        start_placeholder: "Start date".into(),
        end_placeholder: "End date".into(),
        start_suffix: "start".into(),
        end_suffix: "end".into(),
        choose_date: "Choose date".into(),
        choose_range: "Choose date range".into(),
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

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        DateTimeFixture { state, date: None, time: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            let mut bindings = date_key_bindings();
            bindings.extend(calendar_key_bindings());
            bindings.extend(time_key_bindings());
            cx.bind_keys(bindings);
        },
    )?;

    if matches!(state, "date_picker_open" | "time_segment_focused") {
        session.update(|root, window, cx| {
            let fixture = root.read(cx);
            let date = fixture.date.clone().unwrap();
            let time = fixture.time.clone().unwrap();
            if state == "date_picker_open" {
                date.focus_handle(cx).focus(window, cx);
            } else {
                time.focus_handle(cx).focus(window, cx);
            }
        })?;
    }
    if state == "date_picker_open" {
        session.simulate_keystrokes("alt-down")?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/date-time-field/tests/baselines")
        .join(baseline.strip_prefix("date-time-field/").expect("DateTimeField baseline prefix"));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated DateTimeField screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| {
            panic!("missing DateTimeField baseline {}: {error}", path.display())
        })
        .to_rgba8();
    if !(PixelTolerance { channel_delta: 2, max_different_pixels: 32 }).matches(&expected, actual) {
        let diff = path
            .with_file_name(format!("{}-diff.png", path.file_stem().unwrap().to_string_lossy()));
        let mut image = RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let before = expected.get_pixel(x, y);
            let after = actual.get_pixel(x, y);
            *pixel = image::Rgba([
                before[0].abs_diff(after[0]).saturating_mul(4),
                before[1].abs_diff(after[1]).saturating_mul(4),
                before[2].abs_diff(after[2]).saturating_mul(4),
                255,
            ]);
        }
        image.save(&diff).expect("write difference image");
        panic!("DateTimeField screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched DateTimeField screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../registry/date-time-field/tests/conformance.json"
    ))
    .expect("DateTimeField manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 42);
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "empty" => "empty",
            "date_value" => "date_value",
            "time_value" => "time_value",
            "complete" => "complete",
            "date_picker_open" => "date_picker_open",
            "time_segment_focused" => "time_segment_focused",
            "disabled" => "disabled",
            other => panic!("unmapped DateTimeField state: {other}"),
        };
        let theme = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped DateTimeField theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(state, theme, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, baseline);
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping DateTimeField screenshot matrix: macOS Metal is required.")
        }
        Err(error) => panic!("DateTimeField screenshot matrix failed: {error}"),
    }
}
