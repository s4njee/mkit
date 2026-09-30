use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Keystroke, Render, Window, div, prelude::*, px,
    size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    toolbar::{Choice, Item, Orientation, OverflowPolicy, Toolbar},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

// Tall enough for the whole vertical toolbar.
const SIZE: (f32, f32) = (520.0, 320.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Horizontal,
    Vertical,
    Overflow,
    Disabled,
}

struct ToolbarFixture {
    state: State,
    toolbar: Option<Entity<Toolbar>>,
}

impl Render for ToolbarFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        if self.toolbar.is_none() {
            let disabled = self.state == State::Disabled;
            let items = vec![
                Item::button("undo", "Undo").disabled(disabled),
                Item::separator(),
                Item::toggle_group(
                    "alignment",
                    "Alignment",
                    vec![
                        Choice::new("left", "Left").disabled(disabled),
                        Choice::new("center", "Center").disabled(disabled),
                        Choice::new("right", "Right").disabled(disabled),
                    ],
                    Some("left".into()),
                )
                .overflow(OverflowPolicy::NeverOverflow),
                Item::toggle("bold", "Bold", true).disabled(disabled),
                Item::button("share", "Share"),
                Item::button("archive", "Archive").disabled(true),
            ];
            let orientation = if self.state == State::Vertical {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            };
            let capacity = if self.state == State::Overflow { 2 } else { usize::MAX };
            self.toolbar = Some(cx.new(|_| {
                Toolbar::new("Document actions", items)
                    .orientation(orientation)
                    .visible_capacity(capacity)
            }));
        }
        let label = match self.state {
            State::Horizontal => "Horizontal toolbar",
            State::Vertical => "Vertical toolbar",
            State::Overflow => "Toolbar with overflow",
            State::Disabled => "Disabled toolbar actions",
        };
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
                    .child(label),
            )
            .child(self.toolbar.as_ref().expect("toolbar initialized").clone())
    }
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        ToolbarFixture { state, toolbar: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::toolbar::default_key_bindings());
        },
    )?;
    if state == State::Overflow {
        session.update(|root, window, cx| {
            let toolbar = root.read(cx).toolbar.as_ref().expect("toolbar initialized").clone();
            toolbar.update(cx, |toolbar, cx| toolbar.focus_handle(cx).focus(window, cx));
            window.dispatch_keystroke(Keystroke::parse("end").expect("End key"), cx);
            window.dispatch_keystroke(Keystroke::parse("enter").expect("Enter key"), cx);
            assert!(
                toolbar.read(cx).is_overflow_open(),
                "overflow fixture opens through the real trigger"
            );
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/toolbar/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated Toolbar screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing Toolbar baseline {}: {error}", path.display()))
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
        diff_image.save(&diff).expect("write screenshot diff");
        panic!("Toolbar screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched Toolbar screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/toolbar/tests/conformance.json"))
            .expect("Toolbar conformance manifest");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 24, "four toolbar states × three themes × two scales");
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "horizontal" => State::Horizontal,
            "vertical" => State::Vertical,
            "overflow" => State::Overflow,
            "disabled" => State::Disabled,
            other => panic!("unmapped Toolbar state: {other}"),
        };
        let theme = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped screenshot theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("scale u32");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(state, theme, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, baseline);
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping Toolbar screenshot matrix: macOS Metal is required.")
        }
        Err(error) => panic!("Toolbar screenshot matrix failed: {error}"),
    }
}
