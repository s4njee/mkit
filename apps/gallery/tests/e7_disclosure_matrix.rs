use gpui_pre::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    accordion::{self, Accordion, Item, Mode},
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    disclosure::{self, Disclosure},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (440.0, 240.0);

struct DisclosureMatrixFixture {
    component: &'static str,
    state: &'static str,
    disclosure: Option<Entity<Disclosure>>,
    accordion: Option<Entity<Accordion>>,
}

impl Render for DisclosureMatrixFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let element = match self.component {
            "disclosure" => {
                let state = self.state;
                let entity = self.disclosure.get_or_insert_with(|| {
                    cx.new(|_| {
                        let disclosure =
                            Disclosure::new("gallery-disclosure", "Advanced settings", || {
                                div()
                                    .id("disclosure-details")
                                    .child("Additional controls are available.")
                            });
                        match state {
                            "collapsed" => disclosure,
                            "expanded" => disclosure.expanded(true),
                            "disabled" => disclosure.expanded(true).disabled(true),
                            other => panic!("unmapped Disclosure state: {other}"),
                        }
                    })
                });
                entity.clone().into_any_element()
            }
            "accordion" => {
                let state = self.state;
                let entity = self.accordion.get_or_insert_with(|| {
                    cx.new(|_| {
                        let items = match state {
                            "none_open" => vec![
                                Item::new("transform", "Transform", || {
                                    div().child("Position and rotation")
                                }),
                                Item::new("appearance", "Appearance", || {
                                    div().child("Opacity and color")
                                }),
                            ],
                            "one_open" => vec![
                                Item::new("transform", "Transform", || {
                                    div().child("Position and rotation")
                                })
                                .expanded(true),
                                Item::new("appearance", "Appearance", || {
                                    div().child("Opacity and color")
                                }),
                            ],
                            "multiple_open" => vec![
                                Item::new("transform", "Transform", || {
                                    div().child("Position and rotation")
                                })
                                .expanded(true),
                                Item::new("appearance", "Appearance", || {
                                    div().child("Opacity and color")
                                })
                                .expanded(true),
                            ],
                            "disabled_item" => vec![
                                Item::new("transform", "Transform", || {
                                    div().child("Position and rotation")
                                }),
                                Item::new("locked", "Locked settings", || {
                                    div().child("Managed by your organization")
                                })
                                .expanded(true)
                                .disabled(true),
                            ],
                            other => panic!("unmapped Accordion state: {other}"),
                        };
                        Accordion::new(
                            items,
                            if state == "multiple_open" { Mode::Multiple } else { Mode::Single },
                        )
                    })
                });
                entity.clone().into_any_element()
            }
            other => panic!("unmapped component: {other}"),
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
                    .child(if self.component == "disclosure" { "Disclosure" } else { "Accordion" }),
            )
            .child(element)
    }
}

fn capture(
    component: &'static str,
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        DisclosureMatrixFixture { component, state, disclosure: None, accordion: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            theme::set_theme(cx, theme_value);
            cx.bind_keys(disclosure::default_key_bindings());
            cx.bind_keys(accordion::default_key_bindings());
        },
    )?;
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, component: &str, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry")
        .join(component)
        .join("tests/baselines")
        .join(baseline.strip_prefix(&format!("{component}/")).expect("baseline component prefix"));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent"))
            .expect("create baseline directory");
        actual.save(&path).expect("write baseline");
        println!("Updated {component} screenshot: {}", path.display());
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

fn run_component(component: &str, manifest: &Value) -> Result<(), ScreenshotError> {
    let component_id = match component {
        "disclosure" => "disclosure",
        "accordion" => "accordion",
        other => panic!("unmapped component: {other}"),
    };
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    let states: &[&str] = if component == "disclosure" {
        &["collapsed", "expanded", "disabled"]
    } else {
        &["none_open", "one_open", "multiple_open", "disabled_item"]
    };
    assert_eq!(cases.len(), states.len() * 3 * 2, "state/theme/scale matrix is complete");
    for case in cases {
        let state = case["state"].as_str().expect("state");
        assert!(states.contains(&state), "unexpected {component} state: {state}");
        let state_id = match state {
            "collapsed" => "collapsed",
            "expanded" => "expanded",
            "disabled" => "disabled",
            "none_open" => "none_open",
            "one_open" => "one_open",
            "multiple_open" => "multiple_open",
            "disabled_item" => "disabled_item",
            other => panic!("unmapped state: {other}"),
        };
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let actual = capture(component_id, state_id, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, component, case["baseline"].as_str().expect("baseline"));
    }
    Ok(())
}

fn run() -> Result<(), ScreenshotError> {
    let disclosure: Value =
        serde_json::from_str(include_str!("../../../registry/disclosure/tests/conformance.json"))
            .expect("Disclosure manifest");
    let accordion: Value =
        serde_json::from_str(include_str!("../../../registry/accordion/tests/conformance.json"))
            .expect("Accordion manifest");
    run_component("disclosure", &disclosure)?;
    run_component("accordion", &accordion)
}

fn main() {
    match run() {
        Ok(()) => println!("Disclosure and Accordion screenshot matrices passed."),
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping Disclosure and Accordion screenshots: macOS Metal is required.")
        }
        Err(error) => panic!("Disclosure and Accordion screenshot matrix failed: {error}"),
    }
}
