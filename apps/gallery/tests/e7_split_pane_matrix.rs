use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Keystroke, Render, Window, div, prelude::*, px,
    size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    split_pane::{self, Orientation, SplitPane},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (520.0, 280.0);

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Centered,
    Resized,
    VerticalCentered,
    Focused,
    Disabled,
}

struct Panel {
    title: &'static str,
    elevated: bool,
}

impl Render for Panel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        div()
            .size_full()
            .bg(if self.elevated { theme.colors.elevated_surface } else { theme.colors.surface })
            .p(px(theme.spacing.medium))
            .text_size(px(theme.typography.body))
            .text_color(theme.colors.text)
            .child(self.title)
    }
}

struct SplitPaneFixture {
    state: State,
    pane: Option<Entity<SplitPane>>,
}

impl Render for SplitPaneFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        if self.pane.is_none() {
            let leading = cx.new(|_| Panel { title: "Explorer", elevated: false });
            let trailing = cx.new(|_| Panel { title: "Editor", elevated: true });
            let ratio = if self.state == State::Resized { 0.68 } else { 0.5 };
            let state = self.state;
            self.pane = Some(cx.new(|_| {
                SplitPane::new(leading, trailing, ratio)
                    .orientation(if state == State::VerticalCentered {
                        Orientation::Vertical
                    } else {
                        Orientation::Horizontal
                    })
                    .disabled(state == State::Disabled)
            }));
        }
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
                    .child("Workspace panes"),
            )
            .child(
                div()
                    .w(px(440.0))
                    .h(px(190.0))
                    .border(px(theme.borders.hairline))
                    .border_color(theme.colors.border)
                    .rounded(px(theme.radii.small))
                    .overflow_hidden()
                    .child(self.pane.as_ref().expect("SplitPane initialized").clone()),
            )
    }
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        SplitPaneFixture { state, pane: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(split_pane::default_key_bindings());
        },
    )?;
    if state == State::Focused {
        session.update(|root, window, cx| {
            window.dispatch_keystroke(Keystroke::parse("tab").expect("Tab key"), cx);
            window.focus_next(cx);
            let pane = root.read(cx).pane.as_ref().expect("SplitPane initialized").clone();
            assert!(pane.read(cx).focus_handle(cx).is_focused(window));
            assert!(window.last_input_was_keyboard());
        })?;
    } else if state == State::Disabled {
        session.update(|root, window, cx| {
            window.dispatch_keystroke(Keystroke::parse("tab").expect("Tab key"), cx);
            window.focus_next(cx);
            let pane = root.read(cx).pane.as_ref().expect("SplitPane initialized").clone();
            assert!(!pane.read(cx).focus_handle(cx).is_focused(window));
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/split-pane/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated SplitPane screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing SplitPane baseline {}: {error}", path.display()))
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
        diff_image.save(&diff).expect("write SplitPane diff");
        panic!("SplitPane screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched SplitPane screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/split-pane/tests/conformance.json"))
            .expect("SplitPane manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 30, "five states × three themes × two scales");
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "centered" => State::Centered,
            "resized" => State::Resized,
            "vertical-centered" => State::VerticalCentered,
            "focused" => State::Focused,
            "disabled" => State::Disabled,
            other => panic!("unmapped SplitPane state: {other}"),
        };
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped SplitPane theme: {other}"),
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
            println!("Skipping SplitPane screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("SplitPane screenshot matrix failed: {error}"),
    }
}
