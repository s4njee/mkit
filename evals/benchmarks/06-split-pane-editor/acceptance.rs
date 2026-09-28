//! Acceptance tests for benchmark 06 (split-pane editor). See spec.md.

mod support;

use gpui_pre::{TestAppContext, point, px};
use serde_json::{Value, json};
use support::{Bench, approx, array_field, f64_field, open, str_field};

fn lines(snapshot: &Value) -> Vec<String> {
    array_field(snapshot, "lines")
        .iter()
        .map(|line| line.as_str().expect("line text").to_owned())
        .collect()
}

fn cursor(snapshot: &Value) -> (u64, u64) {
    let cursor = &snapshot["cursor"];
    (
        cursor["line"].as_u64().expect("cursor.line"),
        cursor["column"].as_u64().expect("cursor.column"),
    )
}

fn dirty(snapshot: &Value, name: &str) -> bool {
    array_field(snapshot, "files")
        .iter()
        .find(|file| file["name"] == json!(name))
        .and_then(|file| file["dirty"].as_bool())
        .unwrap_or_else(|| panic!("file `{name}` with a dirty flag: {snapshot}"))
}

fn open_a(app: &mut Bench<'_>) {
    app.press("enter");
    assert_eq!(app.snapshot()["open"], json!("a.txt"));
}

#[gpui_pre::test]
fn ac1_lists_files_and_opens_first(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let snap = app.snapshot();
    let names: Vec<_> = array_field(&snap, "files").iter().map(|f| f["name"].clone()).collect();
    assert_eq!(names, vec![json!("a.txt"), json!("b.txt")]);
    assert_eq!(snap["open"], Value::Null);
    assert_eq!(str_field(&snap, "focus"), "list");
    open_a(&mut app);
    let snap = app.snapshot();
    assert_eq!(lines(&snap), vec!["alpha", "beta"]);
    assert_eq!(cursor(&snap), (0, 0));
    assert_eq!(str_field(&snap, "focus"), "editor");
}

#[gpui_pre::test]
fn ac2_typing_edits_the_buffer(cx: &mut TestAppContext) {
    let mut app = open(cx);
    open_a(&mut app);
    app.type_text("x");
    let snap = app.snapshot();
    assert_eq!(lines(&snap)[0], "xalpha");
    assert_eq!(cursor(&snap), (0, 1));
    assert!(dirty(&snap, "a.txt"));
    app.press("end");
    app.type_text("!");
    assert_eq!(lines(&app.snapshot())[0], "xalpha!");
}

#[gpui_pre::test]
fn ac3_enter_backspace_and_arrows(cx: &mut TestAppContext) {
    let mut app = open(cx);
    open_a(&mut app);
    app.press("left");
    assert_eq!(cursor(&app.snapshot()), (0, 0));
    app.press("end enter");
    let snap = app.snapshot();
    assert_eq!(lines(&snap), vec!["alpha", "", "beta"]);
    assert_eq!(cursor(&snap), (1, 0));
    app.type_text("mid");
    assert_eq!(lines(&app.snapshot()), vec!["alpha", "mid", "beta"]);
    app.press("home backspace");
    let snap = app.snapshot();
    assert_eq!(lines(&snap), vec!["alphamid", "beta"]);
    assert_eq!(cursor(&snap), (0, 5));
    app.press("down");
    assert_eq!(cursor(&app.snapshot()), (1, 4));
    app.press("left left up");
    assert_eq!(cursor(&app.snapshot()), (0, 2));
}

#[gpui_pre::test]
fn ac4_save_writes_file(cx: &mut TestAppContext) {
    let mut app = open(cx);
    open_a(&mut app);
    app.type_text("z");
    app.press("secondary-s");
    let on_disk = std::fs::read_to_string(app.fixture.join("a.txt")).unwrap();
    assert_eq!(on_disk, "zalpha\nbeta\n");
    assert!(!dirty(&app.snapshot(), "a.txt"));
}

#[gpui_pre::test]
fn ac5_buffers_survive_switching(cx: &mut TestAppContext) {
    let mut app = open(cx);
    open_a(&mut app);
    app.type_text("q");
    app.press("escape down enter");
    let snap = app.snapshot();
    assert_eq!(snap["open"], json!("b.txt"));
    assert_eq!(lines(&snap), vec!["gamma"]);
    assert!(dirty(&snap, "a.txt"));
    app.click("file-a.txt");
    let snap = app.snapshot();
    assert_eq!(snap["open"], json!("a.txt"));
    assert_eq!(lines(&snap)[0], "qalpha");
}

#[gpui_pre::test]
fn ac6_divider_drag_and_keyboard_resize(cx: &mut TestAppContext) {
    let mut app = open(cx);
    assert!(approx(f64_field(&app.snapshot(), "sidebar_width"), 200.0, 0.5));
    let start = app.target("divider").center();
    app.drag(start, point(start.x + px(80.), start.y), 4);
    let width = f64_field(&app.snapshot(), "sidebar_width");
    assert!(approx(width, 280.0, 2.0), "width after an 80px drag: {width}");
    let start = app.target("divider").center();
    app.drag(start, point(start.x + px(600.), start.y), 4);
    assert!(approx(f64_field(&app.snapshot(), "sidebar_width"), 400.0, 0.5));
    for _ in 0..20 {
        app.press("alt-left");
    }
    assert!(approx(f64_field(&app.snapshot(), "sidebar_width"), 120.0, 0.5));
    app.press("alt-right");
    assert!(approx(f64_field(&app.snapshot(), "sidebar_width"), 140.0, 0.5));
}
