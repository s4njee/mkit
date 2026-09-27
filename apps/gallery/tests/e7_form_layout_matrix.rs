use gpui_pre::{App, Context, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    form_layout::{FormLayout, FormRow},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (500.0, 330.0);

struct FormFixture {
    compact: bool,
}

impl Render for FormFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let field = |value: &str| {
            div()
                .h(px(36.0))
                .px(px(theme.spacing.medium))
                .flex()
                .items_center()
                .rounded(px(6.0))
                .border_1()
                .border_color(theme.colors.border)
                .bg(theme.colors.surface)
                .text_color(theme.colors.text)
                .text_size(px(theme.typography.body))
                .child(value.to_owned())
        };
        let form = FormLayout::new()
            .id(701)
            .label("Profile settings")
            .label_width(132.0)
            .compact(self.compact)
            .row(
                FormRow::new("Display name", field("Alex Morgan"))
                    .description("Shown to teammates across your workspace."),
            )
            .row(
                FormRow::new("Email address", field("alex@example.com"))
                    .description("Used for sign-in and account notices."),
            )
            .row(
                FormRow::new("Workspace URL", field("acme.example.com"))
                    .error("This address is already in use."),
            );

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
                    .child("Account details"),
            )
            .child(form)
    }
}

fn capture(compact: bool, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        FormFixture { compact },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
        },
    )?;
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/form-layout/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated FormLayout screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing FormLayout baseline {}: {error}", path.display()))
        .to_rgba8();
    if !(PixelTolerance { channel_delta: 2, max_different_pixels: 32 }).matches(&expected, actual) {
        let diff = path
            .with_file_name(format!("{}-diff.png", path.file_stem().unwrap().to_string_lossy()));
        let mut image = RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let left = expected.get_pixel(x, y);
            let right = actual.get_pixel(x, y);
            *pixel = image::Rgba([
                left[0].abs_diff(right[0]).saturating_mul(4),
                left[1].abs_diff(right[1]).saturating_mul(4),
                left[2].abs_diff(right[2]).saturating_mul(4),
                255,
            ]);
        }
        image.save(&diff).expect("write diff");
        panic!("FormLayout screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched FormLayout screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/form-layout/tests/conformance.json"))
            .expect("FormLayout manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 12);
    for case in cases {
        let state = case["state"].as_str().expect("state");
        let compact = match state {
            "default" => false,
            "compact" => true,
            other => panic!("unmapped FormLayout state: {other}"),
        };
        let theme = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped FormLayout theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(compact, theme, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, baseline);
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping FormLayout screenshot matrix: macOS Metal is required.")
        }
        Err(error) => panic!("FormLayout screenshot matrix failed: {error}"),
    }
}
