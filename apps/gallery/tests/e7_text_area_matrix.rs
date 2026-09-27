use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    text_area::TextArea,
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, ops::Range, path::PathBuf};

const SIZE: (f32, f32) = (760.0, 430.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Empty,
    Filled,
    Focused,
    Composing,
    Selected,
    Invalid,
    Disabled,
}

impl State {
    fn fixture_name(self) -> &'static str {
        match self {
            Self::Empty => "empty_fixture",
            Self::Filled => "filled_fixture",
            Self::Focused => "focused_fixture",
            Self::Composing => "composing_fixture",
            Self::Selected => "selected_fixture",
            Self::Invalid => "invalid_fixture",
            Self::Disabled => "disabled_fixture",
        }
    }

    fn value(self) -> &'static str {
        match self {
            Self::Empty => "",
            Self::Filled => {
                "Ship the first draft\nReview with the design team\nPublish the release notes"
            }
            Self::Focused => "The quick brown fox\njumps over the quiet dog.",
            Self::Composing => "Cafe\u{301} notes\nDraft in progress",
            Self::Selected => "Selected text",
            Self::Invalid => "This note is missing a title.",
            Self::Disabled => {
                "Archived notes are read-only.\nContact the project owner to make changes."
            }
        }
    }

    fn selection(self) -> Range<usize> {
        match self {
            Self::Empty => 0..0,
            Self::Filled => 74..74,
            Self::Focused => 10..10,
            Self::Composing => 5..5,
            Self::Selected => 0..13,
            Self::Invalid => 29..29,
            Self::Disabled => 72..72,
        }
    }

    fn marked(self) -> Option<Range<usize>> {
        (self == Self::Composing).then_some(3..5)
    }

    fn is_focused(self) -> bool {
        matches!(self, Self::Focused | Self::Composing)
    }
}

struct TextAreaFixture {
    state: State,
    editor: Option<Entity<TextArea>>,
}

impl Render for TextAreaFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        if self.editor.is_none() {
            let state = self.state;
            let editor = cx.new(|cx| {
                let mut editor =
                    TextArea::new(cx).with_label("Release note").with_placeholder("Write a note…");
                match state {
                    State::Empty => editor = editor.with_description("A short project note."),
                    State::Filled => {
                        editor = editor.with_description("Three-line release checklist.")
                    }
                    State::Focused => editor = editor.with_description("Editing the opening line."),
                    State::Composing => {
                        editor = editor.with_description("Composing a project note.")
                    }
                    State::Selected => {
                        editor = editor.with_description("Selected text in the note.")
                    }
                    State::Invalid => {
                        editor = editor.with_validation_message("Add a title before saving.")
                    }
                    State::Disabled => {
                        editor = editor
                            .with_description("This archived note cannot be edited.")
                            .disabled(true)
                    }
                }
                editor.set_fixture_state(state.value(), state.selection(), state.marked());
                editor
            });
            self.editor = Some(editor);
        }
        let editor = self.editor.as_ref().expect("TextArea initialized").clone();
        let state_name = self.state.fixture_name().trim_end_matches("_fixture");
        div()
            .size_full()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .justify_center()
            .child(
                div()
                    .w(px(696.0))
                    .flex()
                    .flex_col()
                    .gap(px(theme.spacing.medium))
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(px(theme.typography.heading_small))
                            .child("Release note"),
                    )
                    .child(editor)
                    .child(
                        div()
                            .text_color(theme.colors.text_muted)
                            .text_size(px(theme.typography.caption))
                            .child(format!("Text area · {state_name} state")),
                    ),
            )
    }
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        TextAreaFixture { state, editor: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::text_area::default_key_bindings());
        },
    )?;
    if state.is_focused() {
        session.update(|root, window, cx| {
            let editor = root.read(cx).editor.as_ref().expect("TextArea initialized").clone();
            let focus = editor.read(cx).focus_handle(cx);
            window.focus(&focus, cx);
            assert!(editor.read(cx).focus_handle(cx).is_focused(window));
        })?;
    }
    session.capture()
}

fn assert_second_line_starts_at_content_edge(actual: &RgbaImage, state: State, scale: u32) {
    if !state.is_focused() {
        return;
    }
    let reference = actual.get_pixel(700 * scale, 110 * scale);
    let has_ink_at_left_edge = (44 * scale..76 * scale).any(|x| {
        (90 * scale..120 * scale).any(|y| {
            let pixel = actual.get_pixel(x, y);
            (0..3).any(|channel| pixel[channel].abs_diff(reference[channel]) > 70)
        })
    });
    assert!(
        has_ink_at_left_edge,
        "{state:?} second explicit line must begin at the text area's content edge"
    );
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/text-area/tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated TextArea screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing TextArea baseline {}: {error}", path.display()))
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
        panic!("TextArea screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched TextArea screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/text-area/tests/conformance.json"))
            .expect("TextArea manifest JSON");
    assert_eq!(manifest["component"], "text-area");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 42, "seven states × three themes × two scales");
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "empty" => State::Empty,
            "filled" => State::Filled,
            "focused" => State::Focused,
            "composing" => State::Composing,
            "selected" => State::Selected,
            "invalid" => State::Invalid,
            "disabled" => State::Disabled,
            other => panic!("unmapped TextArea state: {other}"),
        };
        assert_eq!(case["load_fixture"], state.fixture_name());
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped TextArea theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        assert_second_line_starts_at_content_edge(&actual, state, scale);
        compare_or_update(&actual, baseline);
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping TextArea screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("TextArea screenshot matrix failed: {error}"),
    }
}
