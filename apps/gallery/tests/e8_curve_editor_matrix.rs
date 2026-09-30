use gpui_pre::{
    App, Bounds, Context, Entity, Focusable, InputEvent, IntoElement, Keystroke, MouseButton,
    MouseDownEvent, Pixels, Render, Window, div, point, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    curve_editor::{CurveChannel, CurveEditor, CurvePoint, Interpolation},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{cell::Cell, fs, path::PathBuf, rc::Rc};

const SIZE: (f32, f32) = (720.0, 430.0);

fn master() -> CurveChannel {
    CurveChannel::new(
        "Master",
        vec![CurvePoint::new(0.23, 0.14), CurvePoint::new(0.55, 0.72), CurvePoint::new(0.82, 0.91)],
    )
}

struct CurveFixture {
    state: &'static str,
    curve: Option<Entity<CurveEditor>>,
    /// Laid-out bounds of the editor root, recorded so pointer input can target a point.
    editor_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl Render for CurveFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let curve = self.curve.get_or_insert_with(|| {
            cx.new(|_| {
                let channels = if state == "multiple-channels" {
                    vec![
                        master(),
                        CurveChannel::new("Red", vec![CurvePoint::new(0.32, 0.73)]),
                        CurveChannel::new("Blue", vec![CurvePoint::new(0.68, 0.28)]),
                    ]
                } else {
                    vec![master()]
                };
                let editor = CurveEditor::new("Tone curve", channels);
                let editor = if state == "smooth" {
                    editor.interpolation(Interpolation::Smooth)
                } else {
                    editor
                };
                if state == "disabled" { editor.disabled(true) } else { editor }
            })
        });
        let editor_bounds = self.editor_bounds.clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading)).child("Tone curve"))
            .child(
                div()
                    .w(px(560.0))
                    .p(px(theme.spacing.medium))
                    .on_children_prepainted(move |bounds, _, _| {
                        editor_bounds.set(bounds.first().copied())
                    })
                    .child(curve.clone()),
            )
    }
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        CurveFixture { state, curve: None, editor_bounds: Rc::default() },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
        },
    )?;
    if state == "multiple-channels" {
        session.update(|root, _, cx| {
            let curve = root.read(cx).curve.as_ref().expect("curve initialized").clone();
            curve.update(cx, |curve, cx| curve.select_channel(1, cx));
            assert_eq!(curve.read(cx).active_channel(), 1);
        })?;
    }
    if state == "focused" {
        // The editor is the fixture's only tab stop. An unbound Tab keystroke records keyboard
        // input, then focus moves to the editor so its focus-visible look is captured.
        session.update(|root, window, cx| {
            let curve = root.read(cx).curve.as_ref().expect("curve initialized").clone();
            window.dispatch_keystroke(Keystroke::parse("tab").expect("Tab key"), cx);
            window.focus_next(cx);
            assert!(curve.read(cx).focus_handle(cx).is_focused(window), "Tab focuses the editor");
            assert!(window.last_input_was_keyboard());
            assert_eq!(curve.read(cx).selected_point(), None);
        })?;
    }
    if state == "selected-point" {
        session.update(|root, window, cx| {
            let curve = root.read(cx).curve.as_ref().expect("curve initialized").clone();
            // The graph is the editor's last child and is six `controls.large` tall; its plot
            // area is inset by the border. Target the interior point at (0.55, 0.72).
            let theme = *cx.global::<Theme>();
            let editor = root.read(cx).editor_bounds.get().expect("editor laid out");
            let border = px(theme.borders.hairline);
            let graph_height = px(theme.controls.large * 6.0);
            let top = editor.origin.y + editor.size.height - graph_height + border;
            let width = editor.size.width - border * 2.0;
            let height = graph_height - border * 2.0;
            let position =
                point(editor.origin.x + border + width * 0.55, top + height * (1.0 - 0.72));
            window.dispatch_event(
                MouseDownEvent {
                    position,
                    button: MouseButton::Left,
                    modifiers: Default::default(),
                    click_count: 1,
                    first_mouse: false,
                }
                .to_platform_input(),
                cx,
            );
            assert_eq!(curve.read(cx).selected_point(), Some(2));
            assert_eq!(curve.read(cx).points().len(), 5, "the click selects, not adds, a point");
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/curve-editor/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated CurveEditor screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing CurveEditor baseline {}: {error}", path.display()))
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
        diff_image.save(&diff).expect("write CurveEditor diff");
        panic!("CurveEditor screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched CurveEditor screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/curve-editor/tests/conformance.json"))
            .expect("CurveEditor manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 36, "six states × three themes × two scales");
    for case in cases {
        let (state, fixture) = match case["state"].as_str().expect("state") {
            "linear" => ("linear", "linear_fixture"),
            "smooth" => ("smooth", "smooth_fixture"),
            "selected-point" => ("selected-point", "selected_point_fixture"),
            "multiple-channels" => ("multiple-channels", "multiple_channels_fixture"),
            "disabled" => ("disabled", "disabled_fixture"),
            "focused" => ("focused", "focused_fixture"),
            other => panic!("unmapped CurveEditor state: {other}"),
        };
        assert_eq!(case["load_fixture"].as_str(), Some(fixture));
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped CurveEditor theme: {other}"),
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
            println!("Skipping CurveEditor screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("CurveEditor screenshot matrix failed: {error}"),
    }
}
