use gpui_pre::{App, Context, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    separator::{Orientation, Separator},
};
use mkit_harness::{PixelTolerance, ScreenshotError, screenshot};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (480.0, 180.0);

#[derive(Clone, Copy)]
enum State {
    Horizontal,
    Vertical,
}

struct SeparatorFixture(State);

impl Render for SeparatorFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        match self.0 {
            State::Horizontal => div()
                .size_full()
                .bg(t.colors.background)
                .p(px(t.spacing.large))
                .flex()
                .flex_col()
                .justify_center()
                .gap(px(t.spacing.medium))
                .child(
                    div()
                        .text_color(t.colors.text)
                        .text_size(px(t.typography.heading_small))
                        .child("Workspace settings"),
                )
                .child(
                    Separator::new()
                        .orientation(Orientation::Horizontal)
                        .decorative(false)
                        .label("Workspace settings divider"),
                )
                .child(
                    div()
                        .text_color(t.colors.text_muted)
                        .text_size(px(t.typography.body))
                        .child("Members and access controls"),
                ),
            State::Vertical => div()
                .size_full()
                .bg(t.colors.background)
                .p(px(t.spacing.large))
                .flex()
                .items_center()
                .gap(px(t.spacing.large))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(t.spacing.small))
                        .child(
                            div()
                                .text_color(t.colors.text)
                                .text_size(px(t.typography.heading_small))
                                .child("General"),
                        )
                        .child(
                            div()
                                .text_color(t.colors.text_muted)
                                .text_size(px(t.typography.body))
                                .child("Name and description"),
                        ),
                )
                .child(
                    Separator::new()
                        .orientation(Orientation::Vertical)
                        .decorative(false)
                        .label("Settings panel divider"),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(t.spacing.small))
                        .child(
                            div()
                                .text_color(t.colors.text)
                                .text_size(px(t.typography.heading_small))
                                .child("Access"),
                        )
                        .child(
                            div()
                                .text_color(t.colors.text_muted)
                                .text_size(px(t.typography.body))
                                .child("Members and permissions"),
                        ),
                ),
        }
    }
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    screenshot(
        SeparatorFixture(state),
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
        .join("../../registry/separator/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated Separator screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing Separator baseline {}: {error}", path.display()))
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
        diff_image.save(&diff).expect("write Separator diff");
        panic!("Separator screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched Separator screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/separator/tests/conformance.json"))
            .expect("Separator manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 12, "two orientations × three themes × two scales");
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "horizontal" => State::Horizontal,
            "vertical" => State::Vertical,
            other => panic!("unmapped Separator state: {other}"),
        };
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped Separator theme: {other}"),
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
            println!("Skipping Separator screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("Separator screenshot matrix failed: {error}"),
    }
}
