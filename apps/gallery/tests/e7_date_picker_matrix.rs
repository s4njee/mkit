use gpui_pre::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    core::{
        CivilDate,
        theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    },
    date_picker::{CalendarLabels, DatePicker, DatePickerLabels, DateRange},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (640.0, 590.0);
const ACTIVE: CivilDate = CivilDate::new(2024, 4, 15).unwrap();

struct PickerFixture {
    state: &'static str,
    picker: Option<Entity<DatePicker>>,
}

impl Render for PickerFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let picker = self.picker.get_or_insert_with(|| {
            cx.new(|_| {
                let calendar = calendar_labels();
                let labels = date_picker_labels();
                let parse = parse_date;
                let format = format_date;
                let picker = if state == "range_start" || state == "range_end" {
                    DatePicker::range(
                        "Appointment dates",
                        (state == "range_end")
                            .then(|| DateRange::new(ACTIVE, ACTIVE.add_days(4).unwrap())),
                        ACTIVE,
                        calendar,
                        labels,
                        parse,
                        format,
                    )
                } else {
                    let value = (state == "closed_value").then_some(ACTIVE);
                    DatePicker::new(
                        "Appointment date",
                        value,
                        ACTIVE,
                        calendar,
                        labels,
                        parse,
                        format,
                    )
                };
                match state {
                    "disabled" => picker.disabled(true),
                    "no_available_dates" => {
                        picker.bounds(Some(ACTIVE), Some(ACTIVE)).disabled_dates(|_| true)
                    }
                    _ => picker,
                }
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
                    .child("DatePicker"),
            )
            .child(div().text_color(theme.colors.text_muted).child(state))
            .child(picker.clone())
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

fn date_picker_labels() -> DatePickerLabels {
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
        PickerFixture { state, picker: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::date_picker::default_key_bindings());
        },
    )?;

    if matches!(
        state,
        "editing_valid"
            | "editing_invalid"
            | "open_single"
            | "range_start"
            | "range_end"
            | "no_available_dates"
    ) {
        // The input is rendered below the heading and state caption; click its text area.
        let x = if state == "range_end" { 160.0 } else { 120.0 };
        session.simulate_pointer_down(gpui_pre::point(px(x), px(118.0)))?;
        session.simulate_pointer_up(gpui_pre::point(px(x), px(118.0)))?;
    }
    match state {
        "editing_valid" => {
            session.simulate_input("2024-04-18")?;
        }
        "editing_invalid" => {
            session.simulate_input("2024-99-99")?;
            session.simulate_keystrokes("enter")?;
        }
        "open_single" | "no_available_dates" => {
            session.simulate_keystrokes("alt-down")?;
        }
        "range_start" => {
            session.simulate_keystrokes("alt-down enter")?;
        }
        "range_end" => session.simulate_keystrokes("alt-down")?,
        _ => {}
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/date-picker/tests/baselines")
        .join(baseline.strip_prefix("date-picker/").expect("DatePicker baseline prefix"));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated DatePicker screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing DatePicker baseline {}: {error}", path.display()))
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
        panic!("DatePicker screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched DatePicker screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/date-picker/tests/conformance.json"))
            .expect("DatePicker manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 54);
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "closed_empty" => "closed_empty",
            "closed_value" => "closed_value",
            "editing_valid" => "editing_valid",
            "editing_invalid" => "editing_invalid",
            "open_single" => "open_single",
            "range_start" => "range_start",
            "range_end" => "range_end",
            "disabled" => "disabled",
            "no_available_dates" => "no_available_dates",
            other => panic!("unmapped DatePicker state: {other}"),
        };
        let theme = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped DatePicker theme: {other}"),
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
            println!("Skipping DatePicker screenshot matrix: macOS Metal is required.")
        }
        Err(error) => panic!("DatePicker screenshot matrix failed: {error}"),
    }
}
