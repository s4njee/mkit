use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Render, Window, div, point, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    scrubbable_number_field::{self, ScrubbableNumberField},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf};

const SIZE: (f32, f32) = (420.0, 160.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Idle,
    Focused,
    Editing,
    InvalidEdit,
    Scrubbing,
    Disabled,
}
impl State {
    fn from_id(value: &str) -> Self {
        match value {
            "idle" => Self::Idle,
            "focused" => Self::Focused,
            "editing" => Self::Editing,
            "invalid_edit" => Self::InvalidEdit,
            "scrubbing" => Self::Scrubbing,
            "disabled" => Self::Disabled,
            other => panic!("unmapped ScrubbableNumberField screenshot state: {other}"),
        }
    }
}

struct Fixture {
    state: State,
    field: Option<Entity<ScrubbableNumberField>>,
}
impl Fixture {
    fn new(state: State) -> Self {
        Self { state, field: None }
    }
}
impl Render for Fixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let field = self
            .field
            .get_or_insert_with(|| {
                cx.new(|_| {
                    ScrubbableNumberField::new(12.5)
                        .bounds(Some(0.0), Some(100.0))
                        .step(1.0)
                        .label("Opacity")
                        .description("Layer opacity")
                        .unit("%")
                        .disabled(state == State::Disabled)
                })
            })
            .clone();
        div()
            .id("number-field-matrix")
            .size_full()
            .bg(theme.colors.background)
            .flex()
            .items_center()
            .justify_center()
            .child(div().w(px(220.0)).child(field))
    }
}

fn assert_value(
    session: &mut HeadlessSession<Fixture>,
    expected: f64,
) -> Result<(), ScreenshotError> {
    session.update(|root, _, cx| {
        let field = root.read(cx).field.as_ref().expect("field initialized").clone();
        let actual = field.read(cx).value();
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    })?;
    Ok(())
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        Fixture::new(state),
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(scrubbable_number_field::default_key_bindings());
        },
    )?;
    let origin = point(px(SIZE.0 / 2.0), px(SIZE.1 / 2.0));
    match state {
        State::Idle => assert_value(&mut session, 12.5)?,
        State::Focused => {
            session.update(|root, window, cx| {
                let field = root.read(cx).field.as_ref().expect("field initialized").clone();
                field.focus_handle(cx).focus(window, cx);
            })?;
            assert_value(&mut session, 12.5)?;
        }
        State::Editing | State::InvalidEdit => {
            session.update(|root, window, cx| {
                let field = root.read(cx).field.as_ref().expect("field initialized").clone();
                field.focus_handle(cx).focus(window, cx);
                field.update(cx, |field, _| field.select_all());
            })?;
            session.simulate_input(if state == State::Editing { "13.5" } else { "-" })?;
            session.update(|root, _, cx| {
                let field = root.read(cx).field.as_ref().expect("field initialized").clone();
                let field = field.read(cx);
                assert_eq!(field.value(), 12.5, "text draft is uncommitted");
                assert_eq!(field.draft_text(), if state == State::Editing { "13.5" } else { "-" });
            })?;
        }
        State::Scrubbing => {
            session.simulate_pointer_down(origin)?;
            session.simulate_pointer_drag_to(origin + point(px(8.0), px(0.0)))?;
            assert_value(&mut session, 14.5)?;
        }
        State::Disabled => {
            session.simulate_keystrokes("up")?;
            session.simulate_pointer_down(origin)?;
            session.simulate_pointer_drag_to(origin + point(px(8.0), px(0.0)))?;
            session.simulate_pointer_up(origin + point(px(8.0), px(0.0)))?;
            assert_value(&mut session, 12.5)?;
        }
    }
    let image = session.capture()?;
    if state == State::Scrubbing {
        session.simulate_pointer_up(origin + point(px(8.0), px(0.0)))?;
    }
    Ok(image)
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/scrubbable-number-field/tests/baselines")
        .join(baseline);
    if let Some(candidate_root) = std::env::var_os("E8_NUMBER_FIELD_CANDIDATE_DIR") {
        let candidate = PathBuf::from(candidate_root).join(baseline);
        fs::create_dir_all(candidate.parent().expect("candidate parent"))
            .expect("create candidate dir");
        actual.save(&candidate).expect("write candidate");
        println!("Captured number field candidate: {}", candidate.display());
        return;
    }
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated number field baseline: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing number field baseline {}: {error}", path.display()))
        .to_rgba8();
    assert!(
        PixelTolerance { channel_delta: 2, max_different_pixels: 32 }.matches(&expected, actual),
        "number field screenshot differs: {}",
        path.display()
    );
    println!("Matched number field baseline: {}", path.display());
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../registry/scrubbable-number-field/tests/conformance.json"
    ))
    .expect("number field conformance manifest");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 6 * 3 * 2, "six states × three themes × two scales");
    let requested_id = std::env::var("MKIT_NUMBER_FIELD_CASE_ID").ok();
    let mut seen = HashSet::new();
    let selected = cases
        .iter()
        .filter(|case| requested_id.as_ref().is_none_or(|id| case["id"] == *id))
        .collect::<Vec<_>>();
    if let Some(id) = requested_id {
        assert_eq!(selected.len(), 1, "unknown screenshot case id: {id}");
    }
    for case in selected {
        let state_name = case["state"].as_str().expect("state");
        let state = State::from_id(state_name);
        let theme_name = case["theme"].as_str().expect("theme");
        let theme_value = match theme_name {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped number field theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        assert!(matches!(scale, 1 | 2));
        assert_eq!(case["load_fixture"], format!("number_{state_name}"));
        assert_eq!(case["platform"], "macos");
        assert!(seen.insert((state_name, theme_name, scale)), "duplicate case");
        let actual = capture(state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, case["baseline"].as_str().expect("baseline"));
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping number field matrix: GPUI headless capture requires macOS Metal.")
        }
        Err(error) => panic!("number field screenshot matrix failed: {error}"),
    }
}
