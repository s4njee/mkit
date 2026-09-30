use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    calendar::{Calendar, Selection},
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (400.0, 400.0);
const ACTIVE: mkit::core::CivilDate = mkit::core::CivilDate::new(2024, 4, 15).unwrap();

struct CalendarFixture {
    state: &'static str,
    calendar: Option<Entity<Calendar>>,
}
impl Render for CalendarFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let is_range = state == "range_start" || state == "range_complete";
        let disabled_day = state == "disabled_day";
        let calendar = self.calendar.get_or_insert_with(|| {
            cx.new(|_| {
                let weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].map(str::to_owned);
                let months = [
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
                .map(str::to_owned);
                let selection = match state {
                    "single" | "focused" => Selection::Single(Some(ACTIVE)),
                    "range_start" => Selection::Range(Some(ACTIVE), None),
                    "range_complete" => {
                        Selection::Range(Some(ACTIVE), Some(ACTIVE.add_days(4).unwrap()))
                    }
                    _ => Selection::Single(None),
                };
                let calendar = Calendar::new("Choose date", ACTIVE, weekdays, months);
                if is_range {
                    calendar.range_mode().default_selection(selection)
                } else {
                    calendar.default_selection(selection)
                }
                .disabled_dates(move |date| disabled_day && date == ACTIVE.add_days(1).unwrap())
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
                    .child("Calendar"),
            )
            .child(div().text_color(theme.colors.text_muted).child(state))
            .child(calendar.clone())
    }
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        CalendarFixture { state, calendar: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::calendar::default_key_bindings());
        },
    )?;
    if state == "focused" {
        // Keyboard focus on the active day, moved off the selected day by an arrow key so the
        // focus ring is drawn with keyboard modality on an unselected day.
        session.update(|root, window, cx| {
            let calendar = root.read(cx).calendar.clone().expect("Calendar initialized");
            calendar.update(cx, |calendar, cx| calendar.request_focus(window, cx));
        })?;
        session.simulate_keystrokes("right")?;
        session.update(|root, window, cx| {
            let calendar = root.read(cx).calendar.clone().expect("Calendar initialized");
            assert_eq!(calendar.read(cx).active_date(), ACTIVE.add_days(1).unwrap());
            assert!(calendar.focus_handle(cx).is_focused(window), "active day owns focus");
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/calendar/tests/baselines")
        .join(baseline.strip_prefix("calendar/").expect("Calendar baseline prefix"));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated Calendar screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing Calendar baseline {}: {error}", path.display()))
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
        panic!("Calendar screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched Calendar screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/calendar/tests/conformance.json"))
            .expect("Calendar manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 36);
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "single" => "single",
            "range_start" => "range_start",
            "range_complete" => "range_complete",
            "empty" => "empty",
            "disabled_day" => "disabled_day",
            "focused" => "focused",
            other => panic!("unmapped Calendar state: {other}"),
        };
        let theme = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped Calendar theme: {other}"),
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
            println!("Skipping Calendar screenshot matrix: macOS Metal is required.")
        }
        Err(error) => panic!("Calendar screenshot matrix failed: {error}"),
    }
}
