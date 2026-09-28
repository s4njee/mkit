//! Acceptance tests for benchmark 04 (file browser). See spec.md.

mod support;

use gpui_pre::TestAppContext;
use serde_json::{Value, json};
use support::{array_field, bool_field, open, str_field};

fn names(snapshot: &Value) -> Vec<String> {
    array_field(snapshot, "entries")
        .iter()
        .map(|entry| entry["name"].as_str().expect("entry name").to_owned())
        .collect()
}

#[gpui_pre::test]
fn ac1_lists_root_sorted(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let snap = app.snapshot();
    assert_eq!(names(&snap), vec!["docs", "src", "Zeta", "Cargo.toml", "notes.txt", "README.md"]);
    for entry in array_field(&snap, "entries") {
        let name = entry["name"].as_str().unwrap();
        let metadata = std::fs::metadata(app.fixture.join(name)).unwrap();
        if metadata.is_dir() {
            assert_eq!(entry["kind"], json!("dir"), "{entry}");
            assert_eq!(entry["size"], Value::Null, "{entry}");
        } else {
            assert_eq!(entry["kind"], json!("file"), "{entry}");
            assert_eq!(entry["size"], json!(metadata.len()), "{entry}");
        }
    }
    assert_eq!(snap["selected"], json!(0));
    assert_eq!(str_field(&snap, "path"), "");
    assert_eq!(snap["opened"], Value::Null);
}

#[gpui_pre::test]
fn ac2_enter_opens_directory_and_backspace_returns(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("enter");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "path"), "docs");
    assert_eq!(names(&snap), vec!["guide", "readme.md"]);
    app.press("enter");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "path"), "docs/guide");
    assert_eq!(names(&snap), vec!["intro.md"]);
    app.press("backspace");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "path"), "docs");
    assert_eq!(snap["selected"], json!(0));
    app.press("backspace down enter");
    assert_eq!(str_field(&app.snapshot(), "path"), "src");
    app.press("backspace");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "path"), "");
    assert_eq!(snap["selected"], json!(1), "the directory just left (src) is selected");
}

#[gpui_pre::test]
fn ac3_root_is_a_boundary(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("backspace");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "path"), "");
    assert_eq!(names(&snap).len(), 6);
    app.press("up");
    assert_eq!(app.snapshot()["selected"], json!(0));
    app.press("down down down down down down down down");
    assert_eq!(app.snapshot()["selected"], json!(5));
}

#[gpui_pre::test]
fn ac4_hidden_entries_toggle(cx: &mut TestAppContext) {
    let mut app = open(cx);
    assert!(!names(&app.snapshot()).contains(&".hidden".to_owned()));
    app.press("secondary-shift-.");
    let snap = app.snapshot();
    assert!(bool_field(&snap, "show_hidden"));
    assert_eq!(names(&snap)[3], ".hidden", "first file, before Cargo.toml: {snap}");
    app.press("secondary-shift-.");
    let snap = app.snapshot();
    assert!(!bool_field(&snap, "show_hidden"));
    assert!(!names(&snap).contains(&".hidden".to_owned()));
}

#[gpui_pre::test]
fn ac5_open_file_and_click_select(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.click("entry-notes.txt");
    assert_eq!(app.snapshot()["selected"], json!(4));
    app.press("enter");
    let snap = app.snapshot();
    assert_eq!(snap["opened"], json!("notes.txt"));
    assert_eq!(str_field(&snap, "path"), "");
    app.click("entry-src");
    app.press("enter");
    app.click("entry-main.rs");
    app.press("enter");
    assert_eq!(app.snapshot()["opened"], json!("src/main.rs"));
}

#[gpui_pre::test]
fn ac6_refresh_reads_disk_again(cx: &mut TestAppContext) {
    let mut app = open(cx);
    std::fs::write(app.fixture.join("new.txt"), "fresh\n").unwrap();
    app.press("secondary-r");
    let snap = app.snapshot();
    assert!(names(&snap).contains(&"new.txt".to_owned()), "{snap}");
    assert_eq!(str_field(&snap, "path"), "");
}

#[gpui_pre::test]
fn ac8_accessibility_lists_entries(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.check_a11y("ac8_accessibility_lists_entries", &["docs", "notes.txt", "README.md"]);
}
