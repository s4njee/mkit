use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    tag_input::{self, Suggestion, Tag, TagInput},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (560.0, 280.0);

struct TagInputFixture {
    state: &'static str,
    input: Option<Entity<TagInput>>,
}

impl Render for TagInputFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let input = self.input.get_or_insert_with(|| {
            cx.new(move |_| {
                let tags = || vec![Tag::new("Rust"), Tag::new("GPUI")];
                match state {
                    "empty" => TagInput::new("Labels").placeholder("Add a label…"),
                    "tags" => TagInput::new("Labels").default_tags(tags()),
                    "query" => TagInput::new("Labels").default_tags(tags()).default_query("new"),
                    "selected_tag" => {
                        TagInput::new("Labels").default_tags(tags()).with_fixture_selected_tag(0)
                    }
                    "validation_error" => TagInput::new("Labels")
                        .default_tags(vec![Tag::new("typo").validation_message("Unknown label.")]),
                    "max_reached" => TagInput::new("Labels").default_tags(tags()).max_tags(2),
                    "suggestions" => TagInput::new("Labels")
                        .default_query("ru")
                        .suggestions(vec![
                            Suggestion::new("Ruby").disabled(true),
                            Suggestion::new("Rust"),
                            Suggestion::new("Rustacean"),
                        ])
                        .with_fixture_active_suggestion(1),
                    "disabled" => TagInput::new("Labels").default_tags(tags()).disabled(true),
                    "focused" => TagInput::new("Labels").default_tags(tags()),
                    other => panic!("unmapped TagInput state: {other}"),
                }
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
                    .text_size(px(theme.typography.heading_small))
                    .text_color(theme.colors.text)
                    .child("TagInput"),
            )
            .child(input.clone())
    }
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        TagInputFixture { state, input: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            theme::set_theme(cx, theme_value);
            cx.bind_keys(tag_input::default_key_bindings());
        },
    )?;
    if state == "focused" {
        session.update(|root, window, cx| {
            let input = root.read(cx).input.as_ref().expect("TagInput initialized").clone();
            let focus = input.read(cx).focus_handle(cx);
            focus.focus(window, cx);
            assert!(focus.is_focused(window), "the focused screenshot must focus the editor");
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let relative = baseline.strip_prefix("tag-input/").expect("component prefix");
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/tag-input/tests/baselines")
        .join(relative);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent"))
            .expect("create baseline directory");
        actual.save(&path).expect("write baseline");
        println!("Updated TagInput screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}", path.display()))
        .to_rgba8();
    assert!(
        PixelTolerance { channel_delta: 2, max_different_pixels: 32 }.matches(&expected, actual),
        "screenshot differs: {}",
        path.display()
    );
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/tag-input/tests/conformance.json"))
            .expect("TagInput manifest");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    let states = [
        "empty",
        "tags",
        "query",
        "selected_tag",
        "validation_error",
        "max_reached",
        "suggestions",
        "disabled",
        "focused",
    ];
    assert_eq!(cases.len(), states.len() * 3 * 2, "state/theme/scale matrix is complete");
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "empty" => "empty",
            "tags" => "tags",
            "query" => "query",
            "selected_tag" => "selected_tag",
            "validation_error" => "validation_error",
            "max_reached" => "max_reached",
            "suggestions" => "suggestions",
            "disabled" => "disabled",
            "focused" => "focused",
            other => panic!("unexpected TagInput state: {other}"),
        };
        assert!(states.contains(&state), "unexpected TagInput state: {state}");
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("scale");
        let actual = capture(state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, case["baseline"].as_str().expect("baseline"));
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => println!("TagInput screenshot matrix passed."),
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping TagInput screenshots: macOS Metal is required.")
        }
        Err(error) => panic!("TagInput screenshot matrix failed: {error}"),
    }
}
