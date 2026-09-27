use gpui_pre::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    text_field::TextField,
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf};

const SIZE: (f32, f32) = (480.0, 210.0);
const STATES: [&str; 6] = ["empty", "filled", "focused", "invalid", "disabled", "secure"];
const FIXTURES: [&str; 6] = [
    "empty_fixture",
    "filled_fixture",
    "focused_fixture",
    "invalid_fixture",
    "disabled_fixture",
    "secure_fixture",
];

struct TextFieldFixture {
    state: &'static str,
    field: Option<Entity<TextField>>,
}

impl TextFieldFixture {
    fn new(state: &'static str) -> Self {
        Self { state, field: None }
    }
}

impl Render for TextFieldFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let field = self.field.get_or_insert_with(|| {
            cx.new(|cx| {
                let mut field = TextField::new(cx).with_label("Workspace name");
                match state {
                    "empty" => {
                        field = field
                            .with_placeholder("Enter a workspace name")
                            .with_description("Used in invitations and settings.");
                        field.set_fixture_state("", 0..0, None);
                    }
                    "filled" | "focused" => {
                        field = field
                            .with_description("Used in invitations and settings.")
                            .controlled("Northstar Studio");
                        field.set_fixture_state("Northstar Studio", 9..9, None);
                    }
                    "invalid" => {
                        field = field
                            .controlled("northstar studio")
                            .with_validation_message("Use letters, numbers, and hyphens.");
                        field.set_fixture_state("northstar studio", 16..16, None);
                    }
                    "disabled" => {
                        field = field
                            .with_description("This workspace is managed by your organization.")
                            .controlled("Northstar Studio")
                            .disabled(true);
                        field.set_fixture_state("Northstar Studio", 16..16, None);
                    }
                    "secure" => {
                        field = field
                            .with_label("Secret token")
                            .secure(true)
                            .controlled("sample-token");
                        field.set_fixture_state("sample-token", 12..12, None);
                    }
                    other => panic!("unmapped TextField state: {other}"),
                }
                field
            })
        });
        let (state_title, helper) = match state {
            "empty" => ("Empty", "Placeholder with ordinary helper text"),
            "filled" => ("Filled", "Unfocused value with ordinary helper text"),
            "focused" => ("Focused", "Caret at the middle of the value"),
            "invalid" => ("Invalid", "Validation message and danger border"),
            "disabled" => ("Disabled", "Read-only value with organization note"),
            "secure" => ("Secure", "Value is masked"),
            other => panic!("unmapped TextField state: {other}"),
        };
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
                    .child("Text field"),
            )
            .child(div().text_color(theme.colors.text_muted).child(state_title))
            .child(field.clone())
            .child(div().text_color(theme.colors.text_muted).child(helper))
    }
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        TextFieldFixture::new(state),
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::text_field::default_key_bindings());
        },
    )?;
    session.update(|root, window, cx| {
        let field = root.update(cx, |fixture, _| fixture.field.clone().expect("rendered field"));
        let focus = field.read(cx).focus_handle_for_demo().expect("TextField focus handle");
        if state == "focused" {
            window.focus(&focus, cx);
        }
        assert_eq!(focus.is_focused(window), state == "focused", "fixture focus state: {state}");
    })?;
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    if let Some(output) = std::env::var_os("TEXT_FIELD_CAPTURE_DIR") {
        let path = PathBuf::from(output).join(baseline);
        fs::create_dir_all(path.parent().expect("capture parent")).expect("create capture dir");
        actual.save(&path).expect("write capture candidate");
        return;
    }
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/text-field/tests/baselines")
        .join(baseline.strip_prefix("text-field/").expect("text-field baseline prefix"));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated TextField screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing TextField baseline {}: {error}", path.display()))
        .to_rgba8();
    if !(PixelTolerance { channel_delta: 2, max_different_pixels: 32 }).matches(&expected, actual) {
        let diff = path
            .with_file_name(format!("{}-diff.png", path.file_stem().unwrap().to_string_lossy()));
        let mut image = RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let before = expected.get_pixel(x, y);
            let after = actual.get_pixel(x, y);
            *pixel = image::Rgba([
                before[0].abs_diff(after[0]).saturating_mul(4),
                before[1].abs_diff(after[1]).saturating_mul(4),
                before[2].abs_diff(after[2]).saturating_mul(4),
                255,
            ]);
        }
        image.save(&diff).expect("write screenshot diff");
        panic!("TextField screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched TextField screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/text-field/tests/conformance.json"))
            .expect("TextField manifest JSON");
    assert_eq!(manifest["component"].as_str(), Some("text-field"));
    assert_eq!(manifest["source_spec_version"].as_u64(), Some(1));
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    let declared_states =
        cases.iter().map(|case| case["state"].as_str().expect("state")).collect::<HashSet<_>>();
    assert_eq!(
        declared_states.len(),
        STATES.len(),
        "manifest declares every state once per matrix"
    );
    assert!(STATES.iter().all(|state| declared_states.contains(state)));
    assert_eq!(cases.len(), STATES.len() * 3 * 2, "every state/theme/scale is declared");
    let mut seen = HashSet::new();
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        let state_index = STATES
            .iter()
            .position(|candidate| *candidate == state_name)
            .unwrap_or_else(|| panic!("unmapped TextField state: {state_name}"));
        let state = STATES[state_index];
        assert_eq!(case["load_fixture"].as_str(), Some(FIXTURES[state_index]));
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped TextField theme: {other}"),
        };
        let theme_name = case["theme"].as_str().expect("theme");
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        assert!(matches!(scale, 1 | 2), "unsupported declared scale: {scale}");
        assert!(seen.insert((state, theme_name, scale)), "duplicate screenshot case");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, baseline);
    }
    assert_eq!(seen.len(), STATES.len() * 3 * 2, "manifest matrix has no missing combination");
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping TextField screenshot matrix: macOS Metal is required.")
        }
        Err(error) => panic!("TextField screenshot matrix failed: {error}"),
    }
}
