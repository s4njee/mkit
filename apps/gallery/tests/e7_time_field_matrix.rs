use gpui_pre::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    time_field::{HourCycle, TimeField, TimeFieldLabels, TimeValue},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (520.0, 260.0);
fn initial_time() -> TimeValue {
    TimeValue::new(9, 35, 12).unwrap()
}

struct TimeFieldFixture {
    state: &'static str,
    field: Option<Entity<TimeField>>,
}

impl Render for TimeFieldFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let field = self.field.get_or_insert_with(|| {
            cx.new(|_| {
                let mut field = TimeField::new(
                    "Departure time",
                    if state == "empty" { None } else { Some(initial_time()) },
                    labels(),
                )
                .step(15);
                match state {
                    "focused_second" => field = field.seconds(true),
                    "twelve_hour" => field = field.hour_cycle(HourCycle::H12).seconds(true),
                    "invalid_bounds" => {
                        field = field.bounds(
                            Some(TimeValue::new(9, 0, 0).unwrap()),
                            Some(TimeValue::new(9, 30, 59).unwrap()),
                        )
                    }
                    "disabled" => field = field.disabled(true),
                    _ => {}
                }
                field
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
                    .child("TimeField"),
            )
            .child(div().text_color(theme.colors.text_muted).child(state))
            .child(field.clone())
    }
}

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

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        TimeFieldFixture { state, field: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::time_field::default_key_bindings());
        },
    )?;

    // Segment centres: 32px segments from x=16, each followed by a 4px gap, separator, and gap.
    let x = match state {
        "focused_minute" | "invalid_bounds" => Some(82.0),
        "focused_second" => Some(126.0),
        "focused_hour" => Some(38.0),
        _ => None,
    };
    if let Some(x) = x {
        let point = gpui_pre::point(px(x), px(118.0));
        session.simulate_pointer_down(point)?;
        session.simulate_pointer_up(point)?;
    }
    if state == "invalid_bounds" {
        session.simulate_keystrokes("4 5")?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/time-field/tests/baselines")
        .join(baseline.strip_prefix("time-field/").expect("TimeField baseline prefix"));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated TimeField screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing TimeField baseline {}: {error}", path.display()))
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
        panic!("TimeField screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched TimeField screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/time-field/tests/conformance.json"))
            .expect("TimeField manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 42);
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "empty" => "empty",
            "focused_hour" => "focused_hour",
            "focused_minute" => "focused_minute",
            "focused_second" => "focused_second",
            "twelve_hour" => "twelve_hour",
            "invalid_bounds" => "invalid_bounds",
            "disabled" => "disabled",
            other => panic!("unmapped TimeField state: {other}"),
        };
        let theme = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped TimeField theme: {other}"),
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
            println!("Skipping TimeField screenshot matrix: macOS Metal is required.")
        }
        Err(error) => panic!("TimeField screenshot matrix failed: {error}"),
    }
}
