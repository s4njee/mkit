use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    reorderable_list::{ListItem, ReorderableList},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (420.0, 260.0);
const STATES: [&str; 8] =
    ["default", "empty", "active", "dragging", "drop_target", "reordered", "controlled", "focused"];

struct Fixture {
    state: &'static str,
    list: Option<Entity<ReorderableList>>,
}
impl Render for Fixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let list = self.list.get_or_insert_with(|| {
            cx.new(|_| {
                let items = vec![
                    ListItem::new("a", "Draft agenda"),
                    ListItem::new("b", "Review notes"),
                    ListItem::new("c", "Send follow-up"),
                ];
                match state {
                    "empty" => {
                        ReorderableList::new("Meeting tasks", Vec::new()).empty_text("No tasks")
                    }
                    "controlled" => {
                        ReorderableList::controlled("Meeting tasks", items).active_id("b")
                    }
                    "reordered" => ReorderableList::new(
                        "Meeting tasks",
                        vec![
                            ListItem::new("b", "Review notes"),
                            ListItem::new("a", "Draft agenda"),
                            ListItem::new("c", "Send follow-up"),
                        ],
                    )
                    .active_id("a"),
                    "active" | "drop_target" | "dragging" | "focused" => {
                        ReorderableList::new("Meeting tasks", items).active_id("b")
                    }
                    _ => ReorderableList::new("Meeting tasks", items),
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
                    .child("ReorderableList"),
            )
            .child(div().text_color(theme.colors.text_muted).child(state))
            .child(list.clone())
    }
}
fn theme(name: &str) -> Theme {
    match name {
        "light" => SHADCN_LIGHT,
        "dark" => SHADCN_DARK,
        "high-contrast" => HIGH_CONTRAST,
        other => panic!("unmapped theme {other}"),
    }
}
fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        Fixture { state, list: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::reorderable_list::default_key_bindings());
        },
    )?;
    if state == "focused" {
        // The active row holds the list's only tab stop; Tab gives it keyboard focus.
        session.update(|root, window, cx| {
            window.dispatch_keystroke(gpui_pre::Keystroke::parse("tab").expect("Tab key"), cx);
            window.focus_next(cx);
            let list = root.read(cx).list.as_ref().expect("list initialized").clone();
            let focus = list.read(cx).focus_handle(cx);
            assert!(focus.contains_focused(window, cx), "Tab focuses the active row");
            assert!(window.last_input_was_keyboard());
        })?;
    }
    if state == "dragging" || state == "drop_target" {
        let from = gpui_pre::point(px(100.0), px(105.0));
        let to = gpui_pre::point(
            if state == "dragging" { px(260.0) } else { px(100.0) },
            if state == "dragging" { px(105.0) } else { px(170.0) },
        );
        session.simulate_pointer_down(from)?;
        // Cross the drag threshold near the initial grab point. Jumping directly
        // to the destination makes GPUI use that destination as the grab offset.
        session.simulate_pointer_drag_to(gpui_pre::point(from.x + px(10.0), from.y + px(2.0)))?;
        session.simulate_pointer_drag_to(to)?;
        session.simulate_pointer_drag_to(gpui_pre::point(to.x + px(2.0), to.y + px(2.0)))?;
    }
    session.capture()
}
fn compare(actual: &RgbaImage, baseline: &str) {
    let relative = baseline.strip_prefix("reorderable-list/").expect("baseline prefix");
    let path = std::env::var_os("SNAPSHOT_CANDIDATE_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(relative))
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../registry/reorderable-list/tests/baselines")
                .join(relative)
        });
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().unwrap()).expect("create baseline directory");
        actual.save(&path).expect("save baseline");
        println!("Updated ReorderableList screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|e| panic!("missing ReorderableList baseline {}: {e}", path.display()))
        .to_rgba8();
    assert!(
        PixelTolerance { channel_delta: 2, max_different_pixels: 32 }.matches(&expected, actual),
        "ReorderableList screenshot differs: {}",
        path.display()
    );
}
fn run() -> Result<(), ScreenshotError> {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../registry/reorderable-list/tests/conformance.json"
    ))
    .expect("manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), STATES.len() * 6);
    let updating =
        std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1"));
    let selected_state = std::env::var("UPDATE_SNAPSHOT_STATE").ok();
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        let state =
            STATES.iter().copied().find(|state| *state == state_name).expect("known fixture state");
        if updating
            && selected_state
                .as_deref()
                .is_some_and(|selected| !selected.split(',').any(|name| name == state))
        {
            continue;
        }
        let theme_value = theme(case["theme"].as_str().expect("theme"));
        let scale = case["scale"].as_u64().expect("scale") as u32;
        let actual = capture(state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare(&actual, case["baseline"].as_str().expect("baseline"));
    }
    Ok(())
}
fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping ReorderableList screenshot matrix: macOS Metal is required.")
        }
        Err(error) => panic!("ReorderableList matrix failed: {error}"),
    }
}
