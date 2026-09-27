use gpui_pre::{App, Context, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    button::{Button, Variant},
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    status_bar::{CollapsePriority, StatusBar, StatusBarItem},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (960.0, 220.0);
const STATES: [&str; 5] = ["full", "compact", "narrow", "disabled", "updated"];

struct StatusBarFixture {
    state: &'static str,
}

impl Render for StatusBarFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let disabled = self.state == "disabled";
        let available_width = match self.state {
            "compact" => 500.0,
            "narrow" => 300.0,
            _ => SIZE.0,
        };
        let status_text = if self.state == "updated" { "Saved to cloud" } else { "Document ready" };
        let status_bar = StatusBar::new("Document status")
            .id("status-bar-matrix")
            .available_width(available_width)
            .leading(
                StatusBarItem::text("state", status_text)
                    .estimated_width(124.0)
                    .collapse_priority(CollapsePriority::Never),
            )
            .leading(
                StatusBarItem::text("sync", "All changes saved")
                    .estimated_width(136.0)
                    .collapse_priority(CollapsePriority::Low),
            )
            .leading(
                StatusBarItem::button(
                    "details",
                    Button::new("Details").variant(Variant::Ghost).id(31),
                )
                .estimated_width(84.0)
                .collapse_priority(CollapsePriority::Low),
            )
            .trailing(
                StatusBarItem::progress("upload", "Upload", 62.0)
                    .estimated_width(104.0)
                    .collapse_priority(CollapsePriority::Low),
            )
            .trailing(
                StatusBarItem::button("undo", Button::new("Undo").variant(Variant::Outline).id(32))
                    .estimated_width(76.0)
                    .collapse_priority(CollapsePriority::Normal),
            )
            .trailing(
                StatusBarItem::button(
                    "export",
                    Button::new("Export").variant(Variant::Secondary).disabled(disabled).id(33),
                )
                .estimated_width(88.0)
                .collapse_priority(CollapsePriority::High),
            );
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .flex()
            .flex_col()
            .child(
                div()
                    .p(px(theme.spacing.large))
                    .text_size(px(theme.typography.heading_small))
                    .child(format!("Status bar · {}", self.state)),
            )
            .child(div().flex_1())
            .child(status_bar)
    }
}

fn baseline_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/status-bar/tests/baselines")
        .join(name)
}

fn theme(name: &str) -> Theme {
    match name {
        "light" => SHADCN_LIGHT,
        "dark" => SHADCN_DARK,
        "high-contrast" => HIGH_CONTRAST,
        other => panic!("unmapped status bar theme: {other}"),
    }
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        StatusBarFixture { state },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::button::default_key_bindings());
        },
    )?;
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = std::env::var_os("SNAPSHOT_CANDIDATE_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(baseline))
        .unwrap_or_else(|| baseline_path(baseline));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write status bar screenshot");
        println!("Updated StatusBar screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing StatusBar baseline {}: {error}", path.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 32 };
    if !tolerance.matches(&expected, actual) {
        let diff = path.with_file_name(format!(
            "{}-diff.png",
            path.file_stem().expect("baseline stem").to_string_lossy()
        ));
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
        image.save(&diff).expect("write screenshot diff");
        panic!("StatusBar screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched StatusBar screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/status-bar/tests/conformance.json"))
            .expect("StatusBar manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), STATES.len() * 6, "five states × three themes × two scales");
    let selected_state = std::env::var("UPDATE_SNAPSHOT_STATE").ok();
    let updating =
        std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1"));
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        assert!(STATES.contains(&state_name), "unmapped status bar state {state_name}");
        let state = match state_name {
            "full" => "full",
            "compact" => "compact",
            "narrow" => "narrow",
            "disabled" => "disabled",
            "updated" => "updated",
            _ => unreachable!("state checked against STATES"),
        };
        if updating && selected_state.as_deref().is_some_and(|selected| selected != state) {
            continue;
        }
        assert_eq!(case["load_fixture"].as_str(), Some(format!("{state}_fixture").as_str()));
        let theme_name = case["theme"].as_str().expect("theme");
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(state, theme(theme_name), scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, baseline);
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping StatusBar screenshot matrix: macOS Metal is required.")
        }
        Err(error) => panic!("StatusBar screenshot matrix failed: {error}"),
    }
}
