use gpui_pre::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    toggle_button::ToggleButton,
    toggle_group::{Item, ToggleGroup},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (440.0, 170.0);

#[derive(Clone, Copy)]
enum ComponentState {
    ButtonOff,
    ButtonOn,
    ButtonDisabled,
    GroupSelection,
    GroupDisabled,
}

struct ToggleFixture(ComponentState, Option<Entity<ToggleButton>>, Option<Entity<ToggleGroup>>);

impl Render for ToggleFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        match self.0 {
            ComponentState::ButtonOff
            | ComponentState::ButtonOn
            | ComponentState::ButtonDisabled => {
                let state = self.0;
                let button = self.1.get_or_insert_with(|| {
                    cx.new(|_| match state {
                        ComponentState::ButtonOff => ToggleButton::new("Italic", false),
                        ComponentState::ButtonOn => ToggleButton::new("Bold", true),
                        ComponentState::ButtonDisabled => {
                            ToggleButton::new("Underline", true).disabled(true)
                        }
                        _ => unreachable!(),
                    })
                });
                div()
                    .size_full()
                    .bg(theme.colors.background)
                    .p(px(theme.spacing.large))
                    .flex()
                    .items_center()
                    .child(button.clone())
            }
            ComponentState::GroupSelection | ComponentState::GroupDisabled => {
                let state = self.0;
                let group = self.2.get_or_insert_with(|| {
                    cx.new(|_| {
                        ToggleGroup::new(
                            "Text alignment",
                            vec![
                                Item::new("left", "Left"),
                                Item::new("center", "Center"),
                                Item::new("right", "Right"),
                            ],
                            Some("center".into()),
                        )
                        .disabled(matches!(state, ComponentState::GroupDisabled))
                    })
                });
                div()
                    .size_full()
                    .bg(theme.colors.background)
                    .p(px(theme.spacing.large))
                    .flex()
                    .items_center()
                    .child(group.clone())
            }
        }
    }
}

fn capture(
    state: ComponentState,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        ToggleFixture(state, None, None),
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
        },
    )?;
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, component: &str, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry")
        .join(component)
        .join("tests/baselines")
        .join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated toggle screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing toggle baseline {}: {error}", path.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 32 };
    if !tolerance.matches(&expected, actual) {
        panic!("toggle screenshot differs: {}", path.display());
    }
    println!("Matched toggle screenshot: {}", path.display());
}

fn run_component(component: &str, manifest: &Value) -> Result<(), ScreenshotError> {
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    let expected_count = if component == "toggle-button" { 18 } else { 12 };
    assert_eq!(cases.len(), expected_count, "all state/theme/scale combinations must be declared");
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        let state = match (component, state_name) {
            ("toggle-button", "off") => ComponentState::ButtonOff,
            ("toggle-button", "on") => ComponentState::ButtonOn,
            ("toggle-button", "disabled") => ComponentState::ButtonDisabled,
            ("toggle-group", "selection") => ComponentState::GroupSelection,
            ("toggle-group", "disabled") => ComponentState::GroupDisabled,
            (_, other) => panic!("unmapped {component} state: {other}"),
        };
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let actual = capture(state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, component, case["baseline"].as_str().expect("baseline"));
    }
    Ok(())
}

fn run() -> Result<(), ScreenshotError> {
    let button: Value = serde_json::from_str(include_str!(
        "../../../registry/toggle-button/tests/conformance.json"
    ))
    .expect("ToggleButton manifest JSON");
    let group: Value =
        serde_json::from_str(include_str!("../../../registry/toggle-group/tests/conformance.json"))
            .expect("ToggleGroup manifest JSON");
    run_component("toggle-button", &button)?;
    run_component("toggle-group", &group)
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping toggle screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("toggle screenshot matrix failed: {error}"),
    }
}
