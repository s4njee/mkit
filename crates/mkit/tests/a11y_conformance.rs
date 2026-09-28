//! Accessibility conformance cases for button, checkbox, and slider.
//!
//! Reads one generated `accessibility_cases` case from stdin, renders its
//! fixture in `mkit_harness::AccessibilitySession`, and prints adapter JSON.
//! The first top-level node is projected onto ARIA role/properties for the
//! generated expectations; the whole normalized tree is compared with
//! `registry/<component>/tests/baselines/a11y/<state>.txt`. Set
//! `MKIT_UPDATE_A11Y_BASELINES=1` to write missing or changed snapshots, then
//! review the diff before committing it.

use serde_json::{Value, json};
use std::io::{self, Read};

#[cfg(target_os = "macos")]
mod capture {
    use gpui_pre::{
        AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
        Window, div, px, size,
    };
    use mkit::{button::Button, checkbox::Checkbox, core::theme, slider::Slider};
    use mkit_harness::{AccessibilitySession, AccessibilityTree, AccessibilityValue};
    use serde_json::{Map, Value, json};
    use std::{fs, path::PathBuf};

    enum Fixture {
        Button { disabled: bool, loading: bool },
        Checkbox(Entity<Checkbox>),
        Slider(Entity<Slider>),
    }

    struct Host(Fixture);

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let host = div().w(px(320.));
            match &self.0 {
                // An activation handler makes the fixture a working button, so
                // the tree shows the actions an application would expose.
                Fixture::Button { disabled, loading } => host.child(
                    Button::new("Save")
                        .id(1)
                        .disabled(*disabled)
                        .loading(*loading)
                        .on_activate(|_, _| {}),
                ),
                Fixture::Checkbox(checkbox) => host.child(checkbox.clone()),
                Fixture::Slider(slider) => host.child(slider.clone()),
            }
        }
    }

    type BuildFixture = Box<dyn FnOnce(&mut gpui_pre::App) -> Fixture>;

    fn fixture(component: &str, name: &str) -> Result<BuildFixture, String> {
        let unknown = || format!("no {component} accessibility fixture named {name:?}");
        Ok(match component {
            "button" => {
                let (disabled, loading) = match name {
                    "button_idle" => (false, false),
                    "button_disabled" => (true, false),
                    "button_loading" => (false, true),
                    _ => return Err(unknown()),
                };
                Box::new(move |_| Fixture::Button { disabled, loading })
            }
            "checkbox" => {
                let checkbox = match name {
                    "unchecked_fixture" => Checkbox::new("Remember me", false),
                    "checked_fixture" => Checkbox::new("Remember me", true),
                    "indeterminate_fixture" => {
                        Checkbox::new("Remember me", false).indeterminate(true)
                    }
                    "disabled_fixture" => Checkbox::new("Remember me", false).disabled(true),
                    _ => return Err(unknown()),
                };
                Box::new(move |cx| Fixture::Checkbox(cx.new(|_| checkbox)))
            }
            // Values follow the slider spec's fixture note: 0-100, step 1.
            "slider" => {
                let slider = match name {
                    "minimum_fixture" => Slider::new("Volume", 0.0, 0.0, 100.0, 1.0),
                    "middle_fixture" => Slider::new("Volume", 50.0, 0.0, 100.0, 1.0),
                    "maximum_fixture" => Slider::new("Volume", 100.0, 0.0, 100.0, 1.0),
                    "range_fixture" => Slider::range("Volume", 25.0, 75.0, 0.0, 100.0, 1.0),
                    "disabled_fixture" => {
                        Slider::new("Volume", 50.0, 0.0, 100.0, 1.0).disabled(true)
                    }
                    _ => return Err(unknown()),
                };
                Box::new(move |cx| Fixture::Slider(cx.new(|_| slider)))
            }
            _ => return Err(format!("component {component:?} has no accessibility adapter")),
        })
    }

    pub fn capture(component: &str, fixture_name: &str) -> Result<AccessibilityTree, String> {
        let build = fixture(component, fixture_name)?;
        let session = AccessibilitySession::build(
            size(px(360.), px(120.)),
            theme::set_light_theme,
            |_, cx| {
                let fixture = build(cx);
                cx.new(|_| Host(fixture))
            },
        )
        .map_err(|error| error.to_string())?;
        session.tree().map_err(|error| error.to_string())
    }

    fn to_json(value: &AccessibilityValue) -> Value {
        match value {
            AccessibilityValue::Bool(value) => json!(value),
            // Whole numbers print as integers so JSON evidence matches specs.
            AccessibilityValue::Number(value) if value.fract() == 0.0 && value.abs() < 1e15 => {
                json!(*value as i64)
            }
            AccessibilityValue::Number(value) => json!(value),
            AccessibilityValue::Text(value) | AccessibilityValue::Token(value) => json!(value),
            AccessibilityValue::Nodes(nodes) => json!(nodes),
        }
    }

    pub fn evaluate(case: &Value) -> Result<Value, String> {
        let component = case["component"].as_str().ok_or("case has no component")?;
        let state = case["state"].as_str().ok_or("case has no state")?;
        let fixture_name = case["load_fixture"].as_str().ok_or("case has no load_fixture")?;
        let tree = capture(component, fixture_name)?;
        let snapshot = format!("{}\n", tree.as_text());

        let relative = format!("registry/{component}/tests/baselines/a11y/{state}.txt");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(&relative);
        let update = std::env::var_os("MKIT_UPDATE_A11Y_BASELINES").is_some();
        let expected = fs::read_to_string(&path).ok();
        let matched = if update && expected.as_deref() != Some(snapshot.as_str()) {
            fs::create_dir_all(path.parent().expect("baseline dir"))
                .and_then(|_| fs::write(&path, &snapshot))
                .map_err(|error| format!("write {relative}: {error}"))?;
            true
        } else {
            expected.as_deref() == Some(snapshot.as_str())
        };

        let node = tree.top_level().next();
        let properties: Map<String, Value> = node
            .map(|node| {
                node.aria_properties()
                    .iter()
                    .map(|(key, value)| (key.clone(), to_json(value)))
                    .collect()
            })
            .unwrap_or_default();
        let mut actual = json!({
            "role": node.map(|node| node.aria_role()),
            "properties": properties,
            "snapshot_baseline": relative,
            "snapshot_matched": matched,
            "snapshot": snapshot,
        });
        if expected.is_none() && !update {
            actual["snapshot_error"] =
                json!("missing baseline; rerun with MKIT_UPDATE_A11Y_BASELINES=1 and review it");
        }
        // `passed` reports a successful capture; run_conformance.py judges the
        // semantics and `snapshot_matched` evidence.
        Ok(json!({"passed": true, "actual": actual}))
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"component": "button", "kind": "accessibility_cases", "state": "idle", "load_fixture": "button_idle"})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "accessibility_cases", "only accessibility cases are handled here");

    #[cfg(target_os = "macos")]
    let report = match capture::evaluate(&case) {
        Ok(report) => report,
        Err(error) => json!({"passed": false, "actual": {"error": error}}),
    };
    #[cfg(not(target_os = "macos"))]
    let report = json!({
        "status": "unsupported",
        "reason": "headless accessibility capture currently requires macOS",
    });
    println!("{report}");
}
