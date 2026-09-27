use gpui_pre::{
    App, Bounds, Context, Entity, Focusable, IntoElement, Keystroke, Pixels, Render, Window, div,
    fill, point, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    viewport::{self, Guide, ViewTransform, Viewport},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (500.0, 430.0);

struct ViewportFixture {
    viewport: Option<Entity<Viewport>>,
}

impl Render for ViewportFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let viewport = self.viewport.get_or_insert_with(|| {
            cx.new(|_| {
                Viewport::new(
                    "Artwork canvas",
                    point(500.0, 320.0),
                    move |bounds, transform, window| {
                        paint_artwork(bounds, transform, window, theme);
                    },
                )
                .show_rulers(true)
                .guides(vec![Guide::Vertical(135.0), Guide::Horizontal(72.0)])
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
                    .child("Artwork viewport"),
            )
            .child(div().w(px(440.0)).h(px(320.0)).child(viewport.clone()))
    }
}

fn paint_artwork(
    bounds: Bounds<Pixels>,
    transform: ViewTransform,
    window: &mut Window,
    theme: Theme,
) {
    window.paint_quad(fill(bounds, theme.colors.background));
    let origin = transform.world_to_screen(point(0.0, 0.0));
    let left = bounds.origin.x.as_f32() + origin.x;
    let top = bounds.origin.y.as_f32() + origin.y;
    let width = 500.0 * transform.scale;
    let height = 320.0 * transform.scale;
    window.paint_quad(fill(
        Bounds::new(point(px(left), px(top)), size(px(width), px(height))),
        theme.colors.surface,
    ));
    for index in 1..8 {
        let x = left + index as f32 * 50.0 * transform.scale;
        window.paint_quad(fill(
            Bounds::new(point(px(x), px(top)), size(px(theme.borders.hairline), px(height))),
            theme.colors.border,
        ));
    }
    for index in 1..6 {
        let y = top + index as f32 * 43.0 * transform.scale;
        window.paint_quad(fill(
            Bounds::new(point(px(left), px(y)), size(px(width), px(theme.borders.hairline))),
            theme.colors.border,
        ));
    }
    let accent = transform.world_to_screen(point(135.0, 72.0));
    window.paint_quad(
        fill(
            Bounds::new(
                point(
                    px(bounds.origin.x.as_f32() + accent.x),
                    px(bounds.origin.y.as_f32() + accent.y),
                ),
                size(px(130.0 * transform.scale), px(105.0 * transform.scale)),
            ),
            theme.colors.accent,
        )
        .corner_radii(px(theme.radii.medium)),
    );
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        ViewportFixture { viewport: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(viewport::default_key_bindings());
        },
    )?;
    session.update(|root, window, cx| {
        let viewport = root.read(cx).viewport.as_ref().expect("viewport initialized").clone();
        viewport.update(cx, |viewport, cx| viewport.focus_handle(cx).focus(window, cx));
        window.dispatch_keystroke(Keystroke::parse("1").expect("actual-size key"), cx);
        let base = viewport.read(cx).transform();
        let keys: &[&str] = match state {
            "actual-size" => &[],
            "fitted" => &["f"],
            "panned" => &["right"],
            "zoomed" => &["="],
            other => panic!("unmapped Viewport state: {other}"),
        };
        for key in keys {
            window.dispatch_keystroke(Keystroke::parse(key).expect("valid key"), cx);
        }
        let transform = viewport.read(cx).transform();
        match state {
            "actual-size" => assert_eq!(transform.scale, 1.0),
            "fitted" => assert_ne!(transform, base, "Fit must change the actual-size transform"),
            "panned" => assert!(transform.offset_x < base.offset_x),
            "zoomed" => assert!(transform.scale > base.scale),
            _ => unreachable!(),
        }
    })?;
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/viewport/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated Viewport screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing Viewport baseline {}: {error}", path.display()))
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
        diff_image.save(&diff).expect("write Viewport diff");
        panic!("Viewport screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched Viewport screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/viewport/tests/conformance.json"))
            .expect("Viewport manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 24, "four states × three themes × two scales");
    for case in cases {
        let (state, fixture) = match case["state"].as_str().expect("state") {
            "actual-size" => ("actual-size", "viewport_actual_size"),
            "fitted" => ("fitted", "viewport_fitted"),
            "panned" => ("panned", "viewport_panned"),
            "zoomed" => ("zoomed", "viewport_zoomed"),
            other => panic!("unmapped Viewport state: {other}"),
        };
        assert_eq!(case["load_fixture"].as_str(), Some(fixture));
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped Viewport theme: {other}"),
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
            println!("Skipping Viewport screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("Viewport screenshot matrix failed: {error}"),
    }
}
