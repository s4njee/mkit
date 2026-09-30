use gpui_pre::{
    App, Context, IntoElement, Keystroke, Render, ScrollHandle, Window, div, point, prelude::*, px,
    size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    scroll_area::ScrollArea,
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (480.0, 260.0);

struct ScrollAreaFixture {
    handle: ScrollHandle,
}

impl Render for ScrollAreaFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let rows = (1..=8).map(|index| {
            div()
                .h(px(theme.controls.medium))
                .flex()
                .items_center()
                .px(px(theme.spacing.small))
                .text_color(theme.colors.text)
                .text_size(px(theme.typography.body))
                .child(format!("Activity {index:02}"))
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
                    .child("Recent activity"),
            )
            .child(
                ScrollArea::new(
                    theme.controls.medium * 4.0,
                    div().flex().flex_col().children(rows),
                )
                .id(7101)
                .label("Recent activity")
                .width(320.0)
                .handle(self.handle.clone()),
            )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Scrolled,
    Focused,
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        ScrollAreaFixture { handle: ScrollHandle::new() },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::scroll_area::default_key_bindings());
        },
    )?;
    if state == State::Focused {
        // The viewport is the fixture's only tab stop; Tab gives it keyboard focus.
        session.update(|_, window, cx| {
            window.dispatch_keystroke(Keystroke::parse("tab").expect("Tab key"), cx);
            window.focus_next(cx);
            assert!(window.focused(cx).is_some(), "Tab focuses the scroll area");
            assert!(window.last_input_was_keyboard());
        })?;
    }
    if state == State::Scrolled {
        session.update(|root, window, cx| {
            let handle = root.read(cx).handle.clone();
            let line = cx.global::<Theme>().spacing.large * 2.0;
            let maximum = f32::from(handle.max_offset().y);
            assert!(maximum >= line, "fixture must overflow by at least one line");
            handle.set_offset(point(px(0.0), px(-line)));
            assert_eq!(f32::from(handle.offset().y), -line);
            window.refresh();
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/scroll-area/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated ScrollArea screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing ScrollArea baseline {}: {error}", path.display()))
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
        diff_image.save(&diff).expect("write ScrollArea diff");
        panic!("ScrollArea screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched ScrollArea screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/scroll-area/tests/conformance.json"))
            .expect("ScrollArea manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 18, "three states × three themes × two scales");
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "idle" => State::Idle,
            "scrolled" => State::Scrolled,
            "focused" => State::Focused,
            other => panic!("unmapped ScrollArea state: {other}"),
        };
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped ScrollArea theme: {other}"),
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
            println!("Skipping ScrollArea screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("ScrollArea screenshot matrix failed: {error}"),
    }
}
