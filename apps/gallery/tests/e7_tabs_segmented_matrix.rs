use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke,
    PlatformInput, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    segmented_control::{self, SegmentedControl},
    tabs::{self, Tabs},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf};

const SIZE: (f32, f32) = (520.0, 260.0);
const THEMES: usize = 3;
const SCALES: usize = 2;
const TABS_STATES: [&str; 6] =
    ["selection", "disabled", "controlled", "vertical", "focus_pending", "selected_three"];
const TABS_FIXTURES: [&str; 6] = [
    "tabs_selection",
    "tabs_disabled",
    "tabs_controlled",
    "tabs_vertical",
    "tabs_focus_pending",
    "tabs_selected_three",
];
const SEGMENTED_STATES: [&str; 6] =
    ["selection", "disabled", "controlled", "vertical", "empty", "selected_three"];
const SEGMENTED_FIXTURES: [&str; 6] = [
    "segmented_control_selection",
    "segmented_control_disabled",
    "segmented_control_controlled",
    "segmented_control_vertical",
    "segmented_control_empty",
    "segmented_control_selected_three",
];

#[derive(Clone, Copy)]
enum Component {
    Tabs,
    Segmented,
}

struct TabsFixture {
    state: &'static str,
    tabs: Option<Entity<Tabs>>,
}

impl Render for TabsFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let control = self.tabs.get_or_insert_with(|| {
            let items = tabs_items();
            cx.new(|_| match state {
                "selection" | "focus_pending" => Tabs::new("Views", items, Some("one".into())),
                "disabled" => Tabs::new("Views", items, Some("one".into())).disabled(true),
                "controlled" => Tabs::controlled("Views", items, Some("one".into())),
                "vertical" => Tabs::new("Views", items, Some("one".into()))
                    .orientation(tabs::Orientation::Vertical),
                "selected_three" => Tabs::new("Views", items, Some("three".into())),
                other => panic!("unmapped Tabs state: {other}"),
            })
        });
        let (caption, note) = tabs_copy(state);
        fixture_frame(theme, "Tabs", caption, note, control.clone())
    }
}

struct SegmentedFixture {
    state: &'static str,
    control: Option<Entity<SegmentedControl>>,
}

impl Render for SegmentedFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let control = self.control.get_or_insert_with(|| {
            let items = segmented_items();
            cx.new(|_| match state {
                "selection" => SegmentedControl::new("Views", items, Some("one".into())),
                "disabled" => {
                    SegmentedControl::new("Views", items, Some("one".into())).disabled(true)
                }
                "controlled" => SegmentedControl::controlled("Views", items, Some("one".into())),
                "vertical" => SegmentedControl::new("Views", items, Some("one".into()))
                    .orientation(segmented_control::Orientation::Vertical),
                "empty" => SegmentedControl::new("Views", items, None),
                "selected_three" => SegmentedControl::new("Views", items, Some("three".into())),
                other => panic!("unmapped SegmentedControl state: {other}"),
            })
        });
        let (caption, note) = segmented_copy(state);
        fixture_frame(theme, "Segmented control", caption, note, control.clone())
    }
}

fn tabs_items() -> Vec<tabs::Item> {
    vec![
        tabs::Item::new("one", "One"),
        tabs::Item::new("skip", "Skip").disabled(true),
        tabs::Item::new("three", "Three"),
    ]
}

fn segmented_items() -> Vec<segmented_control::Item> {
    vec![
        segmented_control::Item::new("one", "One"),
        segmented_control::Item::new("skip", "Skip").disabled(true),
        segmented_control::Item::new("three", "Three"),
    ]
}

fn fixture_frame(
    theme: Theme,
    heading: &'static str,
    caption: &'static str,
    note: &'static str,
    control: impl IntoElement,
) -> impl IntoElement {
    div()
        .size_full()
        .bg(theme.colors.background)
        .p(px(theme.spacing.large))
        .flex()
        .flex_col()
        .items_start()
        .gap(px(theme.spacing.medium))
        .child(
            div()
                .text_color(theme.colors.text)
                .text_size(px(theme.typography.heading_small))
                .child(heading),
        )
        .child(div().text_color(theme.colors.text_muted).child(caption))
        .child(control)
        .child(div().text_color(theme.colors.text_muted).child(note))
}

fn tabs_copy(state: &str) -> (&'static str, &'static str) {
    match state {
        "selection" => ("Selection", "Manual activation keeps focus separate from selection"),
        "disabled" => ("Disabled", "All tabs are unavailable"),
        "controlled" => ("Controlled", "Owner retains the selected tab"),
        "vertical" => ("Vertical", "Arrow keys follow the vertical axis"),
        "focus_pending" => ("Focus pending", "Three is focused; One remains selected"),
        "selected_three" => ("Selected three", "Three is selected"),
        other => panic!("unmapped Tabs state: {other}"),
    }
}

fn segmented_copy(state: &str) -> (&'static str, &'static str) {
    match state {
        "selection" => ("Selection", "One is selected"),
        "disabled" => ("Disabled", "All options are unavailable"),
        "controlled" => ("Controlled", "Owner retains the selected option"),
        "vertical" => ("Vertical", "Arrow keys follow the vertical axis"),
        "empty" => ("Empty", "No option is selected"),
        "selected_three" => ("Selected three", "Three is selected"),
        other => panic!("unmapped SegmentedControl state: {other}"),
    }
}

fn capture_tabs(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        TabsFixture { state, tabs: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(tabs::default_key_bindings());
        },
    )?;
    session.update(|root, window, cx| {
        let tabs = root.update(cx, |fixture, _| fixture.tabs.clone().expect("rendered Tabs"));
        if state == "focus_pending" {
            let initial_focus = tabs.read(cx).focus_handle(cx);
            window.focus(&initial_focus, cx);
            let key = Keystroke::parse("right").expect("valid Right keystroke");
            let _ = window.dispatch_event(
                PlatformInput::KeyDown(KeyDownEvent {
                    keystroke: key.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }),
                cx,
            );
            let _ = window.dispatch_event(PlatformInput::KeyUp(KeyUpEvent { keystroke: key }), cx);
        }
        let tab_view = tabs.read(cx);
        let expected_selected = if state == "selected_three" { Some("three") } else { Some("one") };
        let expected_focused =
            if state == "focus_pending" { Some("three") } else { expected_selected };
        assert_eq!(tab_view.value(), expected_selected, "Tabs selection state: {state}");
        assert_eq!(tab_view.focused_value(), expected_focused, "Tabs active tab: {state}");
        assert_eq!(
            tab_view.focus_handle(cx).is_focused(window),
            state == "focus_pending",
            "Tabs focus state: {state}"
        );
    })?;
    session.capture()
}

fn capture_segmented(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        SegmentedFixture { state, control: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(segmented_control::default_key_bindings());
        },
    )?;
    session.update(|root, window, cx| {
        let control = root
            .update(cx, |fixture, _| fixture.control.clone().expect("rendered SegmentedControl"));
        let view = control.read(cx);
        let expected_selected = match state {
            "empty" => None,
            "selected_three" => Some("three"),
            _ => Some("one"),
        };
        let expected_focused = match state {
            "empty" => Some("one"),
            "selected_three" => Some("three"),
            _ => Some("one"),
        };
        assert_eq!(view.value(), expected_selected, "SegmentedControl selection state: {state}");
        assert_eq!(
            view.focused_value(),
            expected_focused,
            "SegmentedControl active option: {state}"
        );
        assert!(
            !view.focus_handle(cx).is_focused(window),
            "SegmentedControl fixture remains unfocused: {state}"
        );
    })?;
    session.capture()
}

fn theme_for(name: &str) -> Theme {
    match name {
        "light" => SHADCN_LIGHT,
        "dark" => SHADCN_DARK,
        "high-contrast" => HIGH_CONTRAST,
        other => panic!("unmapped screenshot theme: {other}"),
    }
}

fn compare_or_update(component: Component, actual: &RgbaImage, baseline: &str) {
    let component_dir = match component {
        Component::Tabs => "tabs",
        Component::Segmented => "segmented-control",
    };
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry")
        .join(component_dir)
        .join("tests/baselines")
        .join(baseline.strip_prefix(&format!("{component_dir}/")).expect("baseline prefix"));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated {component_dir} screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}", path.display()))
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
        panic!("{component_dir} screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched {component_dir} screenshot: {baseline}");
}

fn run_component(
    component: Component,
    manifest_text: &str,
    states: &[&str],
    fixtures: &[&str],
) -> Result<(), ScreenshotError> {
    let component_name = match component {
        Component::Tabs => "tabs",
        Component::Segmented => "segmented-control",
    };
    let manifest: Value = serde_json::from_str(manifest_text).expect("component manifest JSON");
    assert_eq!(manifest["component"].as_str(), Some(component_name));
    assert_eq!(manifest["source_spec_version"].as_u64(), Some(1));
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    let declared_states =
        cases.iter().map(|case| case["state"].as_str().expect("state")).collect::<HashSet<_>>();
    assert_eq!(declared_states.len(), states.len(), "manifest contains every state");
    assert!(states.iter().all(|state| declared_states.contains(state)));
    assert_eq!(cases.len(), states.len() * THEMES * SCALES, "all theme/scale cases declared");
    let mut seen = HashSet::new();
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        let state_index = states
            .iter()
            .position(|candidate| *candidate == state_name)
            .unwrap_or_else(|| panic!("unmapped {component_name} state: {state_name}"));
        let state = match component {
            Component::Tabs => TABS_STATES[state_index],
            Component::Segmented => SEGMENTED_STATES[state_index],
        };
        assert_eq!(case["load_fixture"].as_str(), Some(fixtures[state_index]));
        let theme_name = case["theme"].as_str().expect("theme");
        let theme_value = theme_for(theme_name);
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        assert!(matches!(scale, 1 | 2), "unsupported scale: {scale}");
        assert!(seen.insert((state, theme_name, scale)), "duplicate matrix case");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = match component {
            Component::Tabs => capture_tabs(state, theme_value, scale)?,
            Component::Segmented => capture_segmented(state, theme_value, scale)?,
        };
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(component, &actual, baseline);
    }
    assert_eq!(seen.len(), states.len() * THEMES * SCALES, "matrix has no missing cases");
    Ok(())
}

fn run() -> Result<(), ScreenshotError> {
    run_component(
        Component::Tabs,
        include_str!("../../../registry/tabs/tests/conformance.json"),
        &TABS_STATES,
        &TABS_FIXTURES,
    )?;
    run_component(
        Component::Segmented,
        include_str!("../../../registry/segmented-control/tests/conformance.json"),
        &SEGMENTED_STATES,
        &SEGMENTED_FIXTURES,
    )
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => println!(
            "Skipping Tabs and SegmentedControl screenshot matrices: macOS Metal is required."
        ),
        Err(error) => panic!("Tabs and SegmentedControl screenshot matrices failed: {error}"),
    }
}
