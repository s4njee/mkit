use gpui_pre::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    slider::Slider,
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (440.0, 165.0);

struct SliderFixture {
    state: &'static str,
    slider: Option<Entity<Slider>>,
}

impl Render for SliderFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let slider = self.slider.get_or_insert_with(|| {
            cx.new(|_| match state {
                "minimum" => Slider::new("Volume", 0.0, 0.0, 100.0, 1.0),
                "middle" => Slider::new("Volume", 50.0, 0.0, 100.0, 1.0),
                "maximum" => Slider::new("Volume", 100.0, 0.0, 100.0, 1.0),
                "range" => Slider::range("Volume", 25.0, 75.0, 0.0, 100.0, 1.0),
                "disabled" => Slider::new("Volume", 50.0, 0.0, 100.0, 1.0).disabled(true),
                other => panic!("unmapped Slider state: {other}"),
            })
        });
        let value = match state {
            "minimum" => "0%",
            "middle" => "50%",
            "maximum" => "100%",
            "range" => "25–75%",
            "disabled" => "50% · unavailable",
            other => panic!("unmapped Slider state: {other}"),
        };
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
                    .child("Volume"),
            )
            .child(
                div()
                    .w(px(320.0))
                    .flex()
                    .flex_col()
                    .gap(px(theme.spacing.small))
                    .child(
                        div()
                            .text_color(theme.colors.text_muted)
                            .text_size(px(theme.typography.body))
                            .child(value),
                    )
                    .child(slider.clone()),
            )
    }
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        SliderFixture { state, slider: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
        },
    )?;
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/slider/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated Slider screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing Slider baseline {}: {error}", path.display()))
        .to_rgba8();
    if !(PixelTolerance { channel_delta: 2, max_different_pixels: 32 }).matches(&expected, actual) {
        let diff = path
            .with_file_name(format!("{}-diff.png", path.file_stem().unwrap().to_string_lossy()));
        let mut image = RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let left = expected.get_pixel(x, y);
            let right = actual.get_pixel(x, y);
            *pixel = image::Rgba([
                left[0].abs_diff(right[0]).saturating_mul(4),
                left[1].abs_diff(right[1]).saturating_mul(4),
                left[2].abs_diff(right[2]).saturating_mul(4),
                255,
            ]);
        }
        image.save(&diff).expect("write diff");
        panic!("Slider screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched Slider screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let update_state = std::env::var("UPDATE_SNAPSHOT_STATE").ok();
    let updating =
        std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1"));
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/slider/tests/conformance.json"))
            .expect("Slider manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 30);
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "minimum" => "minimum",
            "middle" => "middle",
            "maximum" => "maximum",
            "range" => "range",
            "disabled" => "disabled",
            other => panic!("unmapped Slider state: {other}"),
        };
        if updating && update_state.as_deref().is_some_and(|selected| selected != state) {
            continue;
        }
        let theme = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped Slider theme: {other}"),
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
            println!("Skipping Slider screenshot matrix: macOS Metal is required.")
        }
        Err(error) => panic!("Slider screenshot matrix failed: {error}"),
    }
}
