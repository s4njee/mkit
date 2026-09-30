use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Keystroke, Render, Window, div, prelude::*, px,
    size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    gradient_editor::{Colour, Gradient, GradientEditor, GradientStop},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (700.0, 450.0);

fn three_stops() -> Gradient {
    Gradient::new(vec![
        GradientStop::new(0.0, Colour::new(0.07, 0.19, 0.48)),
        GradientStop::new(0.42, Colour::new(0.96, 0.65, 0.23)),
        GradientStop::new(1.0, Colour::new(0.68, 0.25, 0.52)),
    ])
}

fn many_stops() -> Gradient {
    Gradient::new(vec![
        GradientStop::new(0.0, Colour::new(0.08, 0.18, 0.40)),
        GradientStop::new(0.22, Colour::new(0.22, 0.56, 0.82)),
        GradientStop::new(0.48, Colour::new(0.32, 0.74, 0.54)),
        GradientStop::new(0.74, Colour::new(0.96, 0.73, 0.22)),
        GradientStop::new(1.0, Colour::new(0.86, 0.24, 0.38)),
    ])
}

struct GradientFixture {
    state: &'static str,
    editor: Option<Entity<GradientEditor>>,
}

impl Render for GradientFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let editor = self.editor.get_or_insert_with(|| {
            cx.new(|_| match state {
                "default" => GradientEditor::new("Fill gradient", Gradient::default()),
                "selected-stop" => GradientEditor::new("Fill gradient", three_stops()),
                "many-stops" => GradientEditor::new("Fill gradient", many_stops()),
                "controlled" => GradientEditor::controlled("Fill gradient", three_stops()),
                "disabled" => {
                    GradientEditor::new("Fill gradient", Gradient::default()).disabled(true)
                }
                "focused" => GradientEditor::new("Fill gradient", three_stops()),
                other => panic!("unmapped GradientEditor state: {other}"),
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
            .child(div().text_size(px(theme.typography.heading)).child("Gradient editor"))
            .child(div().w(px(620.0)).child(editor.clone()))
    }
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        GradientFixture { state, editor: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
        },
    )?;
    if state == "selected-stop" || state == "controlled" {
        session.update(|root, _, cx| {
            let editor = root.read(cx).editor.as_ref().expect("editor initialized").clone();
            editor.update(cx, |editor, cx| editor.select_stop(1, cx));
            assert_eq!(editor.read(cx).selected_stop(), 1);
        })?;
    }
    if state == "focused" {
        // The editor is the fixture's only tab stop. An unbound Tab keystroke records keyboard
        // input, then focus moves to the editor so its focus-visible look is captured.
        session.update(|root, window, cx| {
            let editor = root.read(cx).editor.as_ref().expect("editor initialized").clone();
            window.dispatch_keystroke(Keystroke::parse("tab").expect("Tab key"), cx);
            window.focus_next(cx);
            assert!(editor.read(cx).focus_handle(cx).is_focused(window), "Tab focuses the editor");
            assert!(window.last_input_was_keyboard());
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/gradient-editor/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated GradientEditor screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| {
            panic!("missing GradientEditor baseline {}: {error}", path.display())
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
        diff_image.save(&diff).expect("write GradientEditor diff");
        panic!("GradientEditor screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched GradientEditor screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../registry/gradient-editor/tests/conformance.json"
    ))
    .expect("GradientEditor manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 36, "six states × three themes × two scales");
    for case in cases {
        let (state, fixture) = match case["state"].as_str().expect("state") {
            "default" => ("default", "default_fixture"),
            "selected-stop" => ("selected-stop", "selected_stop_fixture"),
            "many-stops" => ("many-stops", "many_stops_fixture"),
            "controlled" => ("controlled", "controlled_fixture"),
            "disabled" => ("disabled", "disabled_fixture"),
            "focused" => ("focused", "focused_fixture"),
            other => panic!("unmapped GradientEditor state: {other}"),
        };
        assert_eq!(case["load_fixture"].as_str(), Some(fixture));
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped GradientEditor theme: {other}"),
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
            println!("Skipping GradientEditor screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("GradientEditor screenshot matrix failed: {error}"),
    }
}
