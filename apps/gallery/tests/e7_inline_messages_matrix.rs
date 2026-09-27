use gpui_pre::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    empty_state::EmptyState,
    inline_alert::{self, InlineAlert, Severity},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (500.0, 280.0);

struct InlineMessagesFixture {
    component: &'static str,
    state: &'static str,
    alert: Option<Entity<InlineAlert>>,
}

impl Render for InlineMessagesFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let element = match self.component {
            "inline-alert" => {
                let state = self.state;
                self.alert
                    .get_or_insert_with(|| {
                        cx.new(|_| {
                            let severity = match state {
                                "info" | "dismissed" => Severity::Info,
                                "success" => Severity::Success,
                                "warning" => Severity::Warning,
                                "error" => Severity::Error,
                                other => panic!("unmapped InlineAlert state: {other}"),
                            };
                            let message = match state {
                                "info" => "Review the import settings before continuing.",
                                "success" => "Your changes are saved.",
                                "warning" => "Review the pending changes before publishing.",
                                "error" => "We could not save your changes. Try again.",
                                "dismissed" => "Your changes are saved.",
                                _ => unreachable!(),
                            };
                            if state == "dismissed" {
                                InlineAlert::controlled(severity, message, true)
                            } else {
                                InlineAlert::new(severity, message)
                                    .title(match state {
                                        "info" => "Information",
                                        "success" => "Saved",
                                        "warning" => "Review required",
                                        "error" => "Could not save",
                                        _ => unreachable!(),
                                    })
                                    .icon(|| div().child("●"))
                                    .action("View details")
                                    .dismissible(true)
                            }
                        })
                    })
                    .clone()
                    .into_any_element()
            }
            "empty-state" => {
                let empty = match self.state {
                    "basic" => EmptyState::new("No projects")
                        .description("Projects you create will appear here."),
                    "illustrated" => EmptyState::new("No projects")
                        .description("Projects you create will appear here.")
                        .icon(div().text_size(px(32.0)).child("◇")),
                    "actionable" => EmptyState::new("No projects")
                        .description("Create a project to get started.")
                        .icon(div().text_size(px(32.0)).child("◇"))
                        .action(
                            div()
                                .id("empty-state-create-action")
                                .role(gpui_pre::accesskit::Role::Button)
                                .aria_label("Create project")
                                .h(px(theme.controls.small))
                                .px(px(theme.spacing.medium))
                                .flex()
                                .items_center()
                                .rounded(px(theme.radii.small))
                                .bg(theme.colors.accent)
                                .text_color(theme.colors.accent_text)
                                .child("Create project"),
                        ),
                    other => panic!("unmapped EmptyState state: {other}"),
                };
                empty.into_any_element()
            }
            other => panic!("unmapped component: {other}"),
        };
        div()
            .size_full()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.large))
            .child(
                div()
                    .text_size(px(theme.typography.heading_small))
                    .text_color(theme.colors.text)
                    .child(if self.component == "inline-alert" {
                        "Inline alert"
                    } else {
                        "Empty state"
                    }),
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
        InlineMessagesFixture { component, state, alert: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            theme::set_theme(cx, theme_value);
            cx.bind_keys(inline_alert::default_key_bindings());
        },
    )?;
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, component: &str, baseline: &str) {
    let relative =
        baseline.strip_prefix(&format!("{component}/")).expect("baseline component prefix");
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry")
        .join(component)
        .join("tests/baselines")
        .join(relative);
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
        "inline-alert" => "inline-alert",
        "empty-state" => "empty-state",
        other => panic!("unmapped component: {other}"),
    };
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    let states: &[&str] = if component == "inline-alert" {
        &["info", "success", "warning", "error", "dismissed"]
    } else {
        &["basic", "illustrated", "actionable"]
    };
    assert_eq!(cases.len(), states.len() * 3 * 2, "state/theme/scale matrix is complete");
    for case in cases {
        let state = case["state"].as_str().expect("state");
        assert!(states.contains(&state), "unexpected {component} state: {state}");
        let state_id = match state {
            "info" => "info",
            "success" => "success",
            "warning" => "warning",
            "error" => "error",
            "dismissed" => "dismissed",
            "basic" => "basic",
            "illustrated" => "illustrated",
            "actionable" => "actionable",
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
    let inline_alert: Value =
        serde_json::from_str(include_str!("../../../registry/inline-alert/tests/conformance.json"))
            .expect("InlineAlert manifest");
    let empty_state: Value =
        serde_json::from_str(include_str!("../../../registry/empty-state/tests/conformance.json"))
            .expect("EmptyState manifest");
    run_component("inline-alert", &inline_alert)?;
    run_component("empty-state", &empty_state)
}

fn main() {
    match run() {
        Ok(()) => println!("InlineAlert and EmptyState screenshot matrices passed."),
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping InlineAlert and EmptyState screenshots: macOS Metal is required.")
        }
        Err(error) => panic!("InlineAlert and EmptyState screenshot matrix failed: {error}"),
    }
}
