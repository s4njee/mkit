use gpui_pre::{
    App, Context, Entity, IntoElement, Render, Window, div, point, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    popover::Popover,
};
use mkit_harness::{PixelTolerance, ScreenshotError, screenshot};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (480.0, 260.0);

#[derive(Clone, Copy)]
enum State {
    Closed,
    Open,
    Busy,
}

struct PopoverFixture {
    state: State,
    popover: Option<Entity<Popover>>,
}

impl Render for PopoverFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.popover.is_none() {
            let state = self.state;
            self.popover = Some(cx.new(move |_| {
                Popover::controlled(
                    "Quick settings",
                    "Choose how this project appears.",
                    !matches!(state, State::Closed),
                )
                .trigger("Settings")
                .busy(matches!(state, State::Busy))
                .anchor_at(point(px(40.0), px(112.0)))
            }));
        }
        let theme = *cx.global::<Theme>();
        div()
            .size_full()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.large))
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.heading_small))
                    .child("Popover"),
            )
            .child(self.popover.as_ref().expect("popover initialized").clone())
    }
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    screenshot(
        PopoverFixture { state, popover: None },
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
        .join("../../registry/popover/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated Popover screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing Popover baseline {}: {error}", path.display()))
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
        diff_image.save(&diff).expect("write Popover diff");
        panic!("Popover screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched Popover screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/popover/tests/conformance.json"))
            .expect("Popover manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 18, "Popover states × three themes × two scales");
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "closed" => State::Closed,
            "open" => State::Open,
            "busy" => State::Busy,
            other => panic!("unmapped Popover state: {other}"),
        };
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped Popover theme: {other}"),
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
            println!("Skipping Popover screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("Popover screenshot matrix failed: {error}"),
    }
}
