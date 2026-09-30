use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Keystroke, Render, Window, div, prelude::*, px,
    size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    file_drop_zone::{self, FileDropZone, SelectedFile as DroppedFile},
    file_field::{self, FileField, SelectedFile as FieldFile},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (760.0, 360.0);
const DROP_STATES: [&str; 4] = ["idle", "focused", "selected", "disabled"];
const FIELD_STATES: [&str; 5] = ["empty", "focused", "selected", "multiple", "disabled"];

struct FileInputsFixture {
    component: &'static str,
    state: &'static str,
    zone: Option<Entity<FileDropZone>>,
    field: Option<Entity<FileField>>,
}

impl Render for FileInputsFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let component = self.component;
        let state = self.state;
        if component == "file-drop-zone" {
            let zone = self.zone.get_or_insert_with(|| {
                cx.new(|_| {
                    let base = FileDropZone::new("Upload attachments")
                        .description("PDF and text files only")
                        .accept([".pdf", ".txt"])
                        .multiple(true);
                    match state {
                        "selected" => base.files(vec![
                            DroppedFile::new("/tmp/brief.pdf"),
                            DroppedFile::new("/tmp/notes.txt"),
                        ]),
                        "disabled" => base.disabled(true),
                        "idle" | "focused" => base,
                        other => panic!("unmapped FileDropZone screenshot state: {other}"),
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
                        .child("FileDropZone"),
                )
                .child(zone.clone())
        } else {
            let field = self.field.get_or_insert_with(|| {
                cx.new(|_| {
                    let base = FileField::new("Source files")
                        .description("Choose PDF or text files")
                        .accept([".pdf", ".txt"])
                        .multiple(true);
                    match state {
                        "selected" => {
                            base.files(vec![FieldFile::new("/tmp/brief.pdf").size_bytes(18_432)])
                        }
                        "multiple" => base.files(vec![
                            FieldFile::new("/tmp/brief.pdf").size_bytes(18_432),
                            FieldFile::new("/tmp/notes.txt").size_bytes(2_048),
                            FieldFile::new("/tmp/references.pdf").size_bytes(258_048),
                        ]),
                        "disabled" => {
                            base.files(vec![FieldFile::new("/tmp/archived.pdf")]).disabled(true)
                        }
                        "empty" | "focused" => base,
                        other => panic!("unmapped FileField screenshot state: {other}"),
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
                        .child("FileField"),
                )
                .child(field.clone())
        }
    }
}

fn theme(name: &str) -> Theme {
    match name {
        "light" => SHADCN_LIGHT,
        "dark" => SHADCN_DARK,
        "high-contrast" => HIGH_CONTRAST,
        other => panic!("unmapped file input theme: {other}"),
    }
}

fn capture(
    component: &'static str,
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        FileInputsFixture { component, state, zone: None, field: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(file_drop_zone::default_key_bindings());
            cx.bind_keys(file_field::default_key_bindings());
        },
    )?;
    if state == "focused" {
        // A Tab keystroke marks the last input as keyboard, so `:focus-visible` styling applies;
        // focus then moves to the Browse button, the first tab stop.
        let focused = session.update(|root, window, cx| {
            window.dispatch_keystroke(Keystroke::parse("tab").expect("Tab key"), cx);
            let fixture = root.read(cx);
            let handle = match component {
                "file-drop-zone" => fixture.zone.as_ref().expect("zone rendered").focus_handle(cx),
                _ => fixture.field.as_ref().expect("field rendered").focus_handle(cx),
            };
            handle.focus(window, cx);
            handle
        })?;
        assert!(
            session.update(|_, window, _| focused.is_focused(window))?,
            "{component} Browse button holds keyboard focus"
        );
    }
    session.capture()
}

fn baseline_path(component: &str, baseline: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry")
        .join(component)
        .join("tests/baselines");
    root.join(component)
        .join(baseline.strip_prefix(&format!("{component}/")).expect("baseline component prefix"))
}

fn compare_or_update(actual: &RgbaImage, component: &str, baseline: &str) {
    let path = std::env::var_os("SNAPSHOT_CANDIDATE_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(baseline))
        .unwrap_or_else(|| baseline_path(component, baseline));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent"))
            .expect("create baseline directory");
        actual.save(&path).expect("write file input screenshot");
        println!("Updated file input screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}", path.display()))
        .to_rgba8();
    assert!(
        PixelTolerance { channel_delta: 2, max_different_pixels: 32 }.matches(&expected, actual),
        "file input screenshot differs: {}",
        path.display()
    );
    println!("Matched file input screenshot: {baseline}");
}

fn run_component(
    component: &'static str,
    manifest: &Value,
    states: &[&'static str],
) -> Result<(), ScreenshotError> {
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    let declared: std::collections::BTreeSet<_> =
        cases.iter().map(|case| case["state"].as_str().expect("state")).collect();
    assert_eq!(cases.len(), declared.len() * 6, "declared states × three themes × two scales");
    for case in cases {
        let state = case["state"].as_str().expect("state");
        if case["status"].as_str() == Some("unsupported_headless_capture") {
            println!(
                "Skipped {component}/{state}: {}",
                case["limitation"].as_str().unwrap_or("unsupported in headless run")
            );
            continue;
        }
        let state =
            states.iter().copied().find(|candidate| *candidate == state).unwrap_or_else(|| {
                panic!("no visual fixture for declared screenshot state {state}")
            });
        let selected_state = std::env::var("UPDATE_SNAPSHOT_STATE").ok();
        if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1"))
            && selected_state.as_deref().is_some_and(|selected| selected != state)
        {
            continue;
        }
        let theme_name = case["theme"].as_str().expect("theme");
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(component, state, theme(theme_name), scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, component, baseline);
    }
    Ok(())
}

fn run() -> Result<(), ScreenshotError> {
    let drop_manifest: Value = serde_json::from_str(include_str!(
        "../../../registry/file-drop-zone/tests/conformance.json"
    ))
    .expect("FileDropZone manifest");
    let field_manifest: Value =
        serde_json::from_str(include_str!("../../../registry/file-field/tests/conformance.json"))
            .expect("FileField manifest");
    run_component("file-drop-zone", &drop_manifest, &DROP_STATES)?;
    run_component("file-field", &field_manifest, &FIELD_STATES)?;
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping file input screenshot matrix: macOS Metal is required.")
        }
        Err(error) => panic!("file input screenshot matrix failed: {error}"),
    }
}
