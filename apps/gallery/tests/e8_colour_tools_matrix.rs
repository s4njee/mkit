use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Keystroke, Render, Window, div, prelude::*, px,
    size,
};
use image::RgbaImage;
use mkit::{
    colour_tools::{Colour, ColourTools, GradeOffset, Grading, Srgb},
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (760.0, 540.0);

struct ColourToolsFixture {
    state: &'static str,
    tools: Option<Entity<ColourTools>>,
}

impl Render for ColourToolsFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let tools = self.tools.get_or_insert_with(|| {
            cx.new(|_| {
                let colour = match state {
                    "selected" | "focused" => Srgb { red: 71, green: 118, blue: 208 },
                    "eyedropper-unavailable" => Srgb { red: 221, green: 119, blue: 66 },
                    "grading" => Srgb { red: 137, green: 128, blue: 153 },
                    other => panic!("unmapped ColourTools state: {other}"),
                };
                ColourTools::new("Artwork colour", Colour::from_srgb(colour))
            })
        });
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading)).child("Colour tools"))
            .child(div().w(px(610.0)).child(tools.clone()))
    }
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        ColourToolsFixture { state, tools: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
        },
    )?;
    if state == "grading" {
        session.update(|root, _, cx| {
            let tools = root.read(cx).tools.as_ref().expect("tools initialized").clone();
            let grading = Grading {
                shadows: GradeOffset { hue: 220.0, saturation: 0.55 },
                midtones: GradeOffset { hue: 35.0, saturation: 0.38 },
                highlights: GradeOffset { hue: 48.0, saturation: 0.75 },
            };
            tools.update(cx, |tools, cx| tools.set_grading(grading, cx));
            assert_eq!(tools.read(cx).grading(), grading);
        })?;
    }
    if state == "focused" {
        // An unbound Tab keystroke records keyboard input; the main wheel then takes focus
        // through the public focus handle so its focus-visible handle ring is captured.
        session.update(|root, window, cx| {
            let tools = root.read(cx).tools.as_ref().expect("tools initialized").clone();
            window.dispatch_keystroke(Keystroke::parse("tab").expect("Tab key"), cx);
            let handle = tools.read(cx).focus_handle(cx);
            window.focus(&handle, cx);
            assert!(handle.is_focused(window), "the hue-saturation wheel has focus");
            assert!(window.last_input_was_keyboard());
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/colour-tools/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated ColourTools screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing ColourTools baseline {}: {error}", path.display()))
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
        diff_image.save(&diff).expect("write ColourTools diff");
        panic!("ColourTools screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched ColourTools screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/colour-tools/tests/conformance.json"))
            .expect("ColourTools manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 24, "four states × three themes × two scales");
    for case in cases {
        let (state, fixture) = match case["state"].as_str().expect("state") {
            "selected" => ("selected", "selected_fixture"),
            "eyedropper-unavailable" => {
                ("eyedropper-unavailable", "eyedropper_unavailable_fixture")
            }
            "grading" => ("grading", "grading_fixture"),
            "focused" => ("focused", "focused_fixture"),
            other => panic!("unmapped ColourTools state: {other}"),
        };
        assert_eq!(case["load_fixture"].as_str(), Some(fixture));
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped ColourTools theme: {other}"),
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
            println!("Skipping ColourTools screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("ColourTools screenshot matrix failed: {error}"),
    }
}
