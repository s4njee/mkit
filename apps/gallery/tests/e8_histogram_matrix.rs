use gpui_pre::{App, Context, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    histogram::Histogram,
};
use mkit_harness::{PixelTolerance, ScreenshotError, screenshot};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (480.0, 220.0);

#[derive(Clone, Copy)]
enum State {
    Luminance,
    Rgb,
    Clipping,
    Empty,
}

struct HistogramFixture(State);

impl Render for HistogramFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let histogram = match self.0 {
            State::Luminance => Histogram::new(
                "Luminance histogram",
                "Luminance values peak in the midtones; no clipping.",
            )
            .luminance(vec![0.08, 0.2, 0.48, 0.85, 1.0, 0.76, 0.38, 0.17]),
            State::Rgb => Histogram::new(
                "RGB histogram",
                "Red peaks in shadows, green in midtones, blue in highlights.",
            )
            .rgb(
                vec![0.95, 0.8, 0.54, 0.33, 0.18, 0.12, 0.06, 0.03],
                vec![0.05, 0.2, 0.58, 1.0, 0.82, 0.43, 0.18, 0.08],
                vec![0.02, 0.06, 0.15, 0.26, 0.48, 0.79, 1.0, 0.86],
            ),
            State::Clipping => Histogram::new(
                "Clipped luminance histogram",
                "Luminance values with both shadows and highlights clipped.",
            )
            .luminance(vec![1.0, 0.7, 0.39, 0.21, 0.3, 0.48, 0.76, 1.0])
            .clipping(true, true),
            State::Empty => Histogram::new(
                "Empty histogram",
                "No image values are available; the plot is empty.",
            ),
        };
        div()
            .size_full()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .justify_center()
            .child(histogram)
    }
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    screenshot(
        HistogramFixture(state),
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
        },
    )
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/histogram/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated Histogram screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing Histogram baseline {}: {error}", path.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 32 };
    if !tolerance.matches(&expected, actual) {
        let diff = path.with_file_name(format!(
            "{}-diff.png",
            path.file_stem().expect("baseline stem").to_string_lossy()
        ));
        let mut diff_image = RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in diff_image.enumerate_pixels_mut() {
            let left = expected.get_pixel(x, y);
            let right = actual.get_pixel(x, y);
            *pixel = image::Rgba([
                left[0].abs_diff(right[0]).saturating_mul(4),
                left[1].abs_diff(right[1]).saturating_mul(4),
                left[2].abs_diff(right[2]).saturating_mul(4),
                255,
            ]);
        }
        diff_image.save(&diff).expect("write Histogram diff");
        panic!("Histogram screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched Histogram screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/histogram/tests/conformance.json"))
            .expect("Histogram manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 24, "four states × three themes × two scales");
    for case in cases {
        let (state, fixture) = match case["state"].as_str().expect("state") {
            "luminance" => (State::Luminance, "luminance_fixture"),
            "rgb" => (State::Rgb, "rgb_fixture"),
            "clipping" => (State::Clipping, "clipping_fixture"),
            "empty" => (State::Empty, "empty_fixture"),
            other => panic!("unmapped Histogram state: {other}"),
        };
        assert_eq!(case["load_fixture"].as_str(), Some(fixture));
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped Histogram theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, baseline);
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping Histogram screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("Histogram screenshot matrix failed: {error}"),
    }
}
