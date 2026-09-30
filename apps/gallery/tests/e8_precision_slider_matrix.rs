use gpui_pre::{
    App, Context, Entity, Focusable, InputEvent, IntoElement, Keystroke, MouseButton,
    MouseDownEvent, MouseMoveEvent, Render, Window, div, point, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    precision_slider::PrecisionSlider,
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (480.0, 180.0);

struct PrecisionSliderFixture {
    state: &'static str,
    slider: Option<Entity<PrecisionSlider>>,
}

impl Render for PrecisionSliderFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let slider = self.slider.get_or_insert_with(|| {
            cx.new(|_| match state {
                "minimum" => PrecisionSlider::new("Exposure", -100.0, -100.0, 100.0, 1.0),
                "middle" => PrecisionSlider::new("Exposure", 0.0, -100.0, 100.0, 1.0).bipolar(true),
                "maximum" => PrecisionSlider::new("Exposure", 100.0, -100.0, 100.0, 1.0),
                "dragging" => PrecisionSlider::new("Exposure", 0.0, -100.0, 100.0, 1.0),
                "focused" => {
                    PrecisionSlider::new("Exposure", 40.0, -100.0, 100.0, 1.0).bipolar(true)
                }
                "disabled" => {
                    PrecisionSlider::new("Exposure", 35.0, -100.0, 100.0, 1.0).disabled(true)
                }
                other => panic!("unmapped PrecisionSlider state: {other}"),
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
                    .child("Exposure"),
            )
            .child(div().w(px(320.0)).child(slider.clone()))
    }
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        PrecisionSliderFixture { state, slider: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
        },
    )?;
    if state == "dragging" {
        session.update(|root, window, cx| {
            let slider = root.read(cx).slider.as_ref().expect("slider initialized").clone();
            let start = point(px(176.0), px(80.0));
            let end = point(px(260.0), px(80.0));
            window.dispatch_event(
                MouseDownEvent {
                    position: start,
                    button: MouseButton::Left,
                    modifiers: Default::default(),
                    click_count: 1,
                    first_mouse: false,
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseMoveEvent {
                    position: end,
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            assert!(slider.read(cx).value() > 0.0, "pointer drag must update slider value");
        })?;
    }
    if state == "focused" {
        // Focus the slider, then press an unbound key so GPUI records keyboard modality and the
        // thumb paints its keyboard focus ring.
        session.update(|root, window, cx| {
            let slider = root.read(cx).slider.as_ref().expect("slider initialized").clone();
            let focus = slider.focus_handle(cx);
            window.focus(&focus, cx);
            window.dispatch_keystroke(Keystroke::parse("escape").expect("Escape key"), cx);
            assert!(focus.is_focused(window), "slider keeps keyboard focus");
            assert!(window.last_input_was_keyboard(), "keyboard modality recorded");
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/precision-slider/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated PrecisionSlider screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| {
            panic!("missing PrecisionSlider baseline {}: {error}", path.display())
        })
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
        diff_image.save(&diff).expect("write PrecisionSlider diff");
        panic!("PrecisionSlider screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched PrecisionSlider screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../registry/precision-slider/tests/conformance.json"
    ))
    .expect("PrecisionSlider manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 36, "six states × three themes × two scales");
    let update_state = std::env::var("UPDATE_SNAPSHOT_STATE").ok();
    let updating =
        std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1"));
    for case in cases {
        let (state, fixture) = match case["state"].as_str().expect("state") {
            "minimum" => ("minimum", "minimum_fixture"),
            "middle" => ("middle", "middle_fixture"),
            "maximum" => ("maximum", "maximum_fixture"),
            "dragging" => ("dragging", "dragging_fixture"),
            "disabled" => ("disabled", "disabled_fixture"),
            "focused" => ("focused", "focused_fixture"),
            other => panic!("unmapped PrecisionSlider state: {other}"),
        };
        assert_eq!(case["load_fixture"].as_str(), Some(fixture));
        if updating && update_state.as_deref().is_some_and(|selected| selected != state) {
            continue;
        }
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped PrecisionSlider theme: {other}"),
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
            println!("Skipping PrecisionSlider screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("PrecisionSlider screenshot matrix failed: {error}"),
    }
}
