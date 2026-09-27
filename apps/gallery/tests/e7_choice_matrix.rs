use gpui_pre::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    checkbox::{self, Checkbox},
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    switch::{self, Switch},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf};

const SIZE: (f32, f32) = (420.0, 150.0);

struct ChoiceFixture {
    component: &'static str,
    state: &'static str,
    checkbox: Option<Entity<Checkbox>>,
    switch: Option<Entity<Switch>>,
}

impl ChoiceFixture {
    fn new(component: &'static str, state: &'static str) -> Self {
        Self { component, state, checkbox: None, switch: None }
    }
}

impl Render for ChoiceFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let control = match self.component {
            "checkbox" => {
                let state = self.state;
                let control = self.checkbox.get_or_insert_with(|| {
                    cx.new(|_| match state {
                        "unchecked" => Checkbox::new("Receive updates", false),
                        "checked" => Checkbox::new("Receive updates", true),
                        "indeterminate" => {
                            Checkbox::new("Receive updates", false).indeterminate(true)
                        }
                        "disabled" => Checkbox::new("Receive updates", false).disabled(true),
                        other => panic!("unmapped Checkbox state: {other}"),
                    })
                });
                control.clone().into_any_element()
            }
            "switch" => {
                let state = self.state;
                let control = self.switch.get_or_insert_with(|| {
                    cx.new(|_| match state {
                        "off" => Switch::new("Enable notifications", false),
                        "on" => Switch::new("Enable notifications", true),
                        // The manifest declares the disabled-off fixture. Disabled preserves
                        // the value supplied to the component, so this fixture starts false.
                        "disabled" => Switch::new("Enable notifications", false).disabled(true),
                        other => panic!("unmapped Switch state: {other}"),
                    })
                });
                control.clone().into_any_element()
            }
            other => panic!("unmapped choice component: {other}"),
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
                    .text_size(px(theme.typography.heading_small))
                    .text_color(theme.colors.text)
                    .child(if self.component == "checkbox" { "Checkbox" } else { "Switch" }),
            )
            .child(control)
    }
}

fn capture(
    component: &'static str,
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        ChoiceFixture::new(component, state),
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(checkbox::default_key_bindings());
            cx.bind_keys(switch::default_key_bindings());
        },
    )?;
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, component: &str, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry")
        .join(component)
        .join("tests/baselines")
        .join(baseline.strip_prefix(&format!("{component}/")).expect("component baseline prefix"));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated {component} screenshot: {}", path.display());
        return;
    }

    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing {component} baseline {}: {error}", path.display()))
        .to_rgba8();
    if !(PixelTolerance { channel_delta: 2, max_different_pixels: 32 }).matches(&expected, actual) {
        let diff = path.with_file_name(format!(
            "{}-diff.png",
            path.file_stem().expect("baseline stem").to_string_lossy()
        ));
        let mut diff_image = RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in diff_image.enumerate_pixels_mut() {
            let before = expected.get_pixel(x, y);
            let after = actual.get_pixel(x, y);
            *pixel = image::Rgba([
                before[0].abs_diff(after[0]).saturating_mul(4),
                before[1].abs_diff(after[1]).saturating_mul(4),
                before[2].abs_diff(after[2]).saturating_mul(4),
                255,
            ]);
        }
        diff_image.save(&diff).expect("write screenshot diff");
        panic!("{component} screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched {component} screenshot: {}", path.display());
}

fn run_component(component: &str, manifest_text: &str) -> Result<(), ScreenshotError> {
    let component_name = match component {
        "checkbox" => "checkbox",
        "switch" => "switch",
        _ => panic!("unmapped choice component: {component}"),
    };
    let manifest: Value = serde_json::from_str(manifest_text).expect("choice manifest JSON");
    assert_eq!(manifest["component"].as_str(), Some(component));
    let declared_states = manifest["screenshot_cases"]
        .as_array()
        .expect("screenshot cases")
        .iter()
        .map(|case| case["state"].as_str().expect("state"))
        .collect::<HashSet<_>>();
    let (states, fixture_names): (&[&str], &[&str]) = match component {
        "checkbox" => (
            &["unchecked", "checked", "indeterminate", "disabled"],
            &["unchecked_fixture", "checked_fixture", "indeterminate_fixture", "disabled_fixture"],
        ),
        "switch" => {
            (&["off", "on", "disabled"], &["off_fixture", "on_fixture", "disabled_fixture"])
        }
        _ => panic!("unmapped choice component: {component}"),
    };
    assert_eq!(declared_states.len(), states.len(), "manifest declares every state exactly");
    assert!(
        states.iter().all(|state| declared_states.contains(state)),
        "manifest state set is complete"
    );

    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), states.len() * 3 * 2, "every state/theme/scale case must be declared");
    let mut seen = HashSet::new();
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        let state_index = states
            .iter()
            .position(|candidate| *candidate == state_name)
            .unwrap_or_else(|| panic!("unmapped {component} state: {state_name}"));
        let state = match (component, state_name) {
            ("checkbox", "unchecked") => "unchecked",
            ("checkbox", "checked") => "checked",
            ("checkbox", "indeterminate") => "indeterminate",
            ("checkbox", "disabled") => "disabled",
            ("switch", "off") => "off",
            ("switch", "on") => "on",
            ("switch", "disabled") => "disabled",
            _ => unreachable!("state list validated above"),
        };
        assert_eq!(case["load_fixture"].as_str(), Some(fixture_names[state_index]));
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped theme: {other}"),
        };
        let theme_name = case["theme"].as_str().expect("theme");
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        assert!(matches!(scale, 1 | 2), "unsupported declared scale: {scale}");
        assert!(seen.insert((state_name, theme_name, scale)), "duplicate screenshot case");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(component_name, state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, component, baseline);
    }
    assert_eq!(seen.len(), states.len() * 3 * 2, "manifest matrix has no missing combination");
    Ok(())
}

fn run() -> Result<(), ScreenshotError> {
    run_component("checkbox", include_str!("../../../registry/checkbox/tests/conformance.json"))?;
    run_component("switch", include_str!("../../../registry/switch/tests/conformance.json"))
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping checkbox and switch screenshot matrices: macOS Metal is required.");
        }
        Err(error) => panic!("choice screenshot matrix failed: {error}"),
    }
}
