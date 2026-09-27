use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    stepper::{Step, Stepper},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (760.0, 340.0);
const STATES: [&str; 4] = ["first", "middle", "final", "validation-error"];

struct StepperFixture {
    state: &'static str,
    stepper: Option<Entity<Stepper>>,
}
impl Render for StepperFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let stepper = self.stepper.get_or_insert_with(|| {
            cx.new(|_| {
                let steps = vec![
                    Step::new("account", "Account details"),
                    Step::new("preferences", "Preferences"),
                    Step::new("review", "Review and finish"),
                ];
                match state {
                    "middle" => Stepper::new("Import setup", steps).default_step(1),
                    "final" => Stepper::new("Import setup", steps).default_step(2),
                    "validation-error" => Stepper::new("Import setup", steps)
                        .validate_with(|_| Err("Choose a destination before continuing".into())),
                    _ => Stepper::new("Import setup", steps),
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
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.heading_small))
                    .child("Stepper"),
            )
            .child(div().text_color(theme.colors.text_muted).child(state))
            .child(stepper.clone())
    }
}

fn theme(name: &str) -> Theme {
    match name {
        "light" => SHADCN_LIGHT,
        "dark" => SHADCN_DARK,
        "high-contrast" => HIGH_CONTRAST,
        other => panic!("unknown theme {other}"),
    }
}
fn baseline_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/stepper/tests/baselines")
        .join(name)
}
fn capture(state: &'static str, t: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        StepperFixture { state, stepper: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, t);
            cx.bind_keys(mkit::stepper::default_key_bindings());
        },
    )?;
    if state == "validation-error" {
        session.update(|root, window, cx| {
            let stepper = root.read(cx).stepper.as_ref().unwrap().clone();
            stepper.read(cx).focus_handle(cx).focus(window, cx);
        })?;
        session.simulate_keystrokes("enter")?;
    }
    session.capture()
}
fn compare(actual: &RgbaImage, name: &str) {
    let path = std::env::var_os("SNAPSHOT_CANDIDATE_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(name))
        .unwrap_or_else(|| baseline_path(name));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().unwrap()).expect("create baseline directory");
        actual.save(&path).expect("write Stepper screenshot");
        println!("Updated Stepper screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|e| panic!("missing Stepper baseline {}: {e}", path.display()))
        .to_rgba8();
    assert!(
        PixelTolerance { channel_delta: 2, max_different_pixels: 32 }.matches(&expected, actual),
        "Stepper screenshot differs: {}",
        path.display()
    );
    println!("Matched Stepper screenshot: {name}");
}
fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/stepper/tests/conformance.json"))
            .unwrap();
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), STATES.len() * 6);
    for case in cases {
        let state = case["state"].as_str().expect("state");
        let theme_name = case["theme"].as_str().expect("theme");
        let scale = case["scale"].as_u64().expect("scale") as u32;
        let baseline = case["baseline"].as_str().expect("baseline");
        assert!(STATES.contains(&state));
        compare(
            &capture(
                STATES[STATES.iter().position(|s| *s == state).unwrap()],
                theme(theme_name),
                scale,
            )?,
            baseline,
        );
    }
    Ok(())
}
fn main() {
    run().expect("Stepper screenshot matrix");
}
