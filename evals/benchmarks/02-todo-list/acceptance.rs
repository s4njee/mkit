//! Acceptance tests for benchmark 02 (todo list). See spec.md.

mod support;

use gpui_pre::TestAppContext;
use serde_json::{Value, json};
use support::{array_field, open, str_field};

fn items(snapshot: &Value) -> Vec<(String, bool)> {
    array_field(snapshot, "items")
        .iter()
        .map(|item| {
            (
                item["text"].as_str().expect("item text").to_owned(),
                item["done"].as_bool().expect("item done"),
            )
        })
        .collect()
}

fn texts(snapshot: &Value) -> Vec<String> {
    items(snapshot).into_iter().map(|(text, _)| text).collect()
}

#[gpui_pre::test]
fn ac1_initial_state(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let snap = app.snapshot();
    assert_eq!(
        items(&snap),
        vec![
            ("buy milk".into(), false),
            ("write chapter".into(), false),
            ("review pr".into(), false)
        ]
    );
    assert_eq!(snap["selected"], json!(0));
    assert_eq!(str_field(&snap, "draft"), "");
    assert_eq!(str_field(&snap, "focus"), "draft");
}

#[gpui_pre::test]
fn ac2_typing_and_enter_adds_an_item(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.type_text("stretch");
    assert_eq!(str_field(&app.snapshot(), "draft"), "stretch");
    app.press("enter");
    let snap = app.snapshot();
    assert_eq!(texts(&snap).last().map(String::as_str), Some("stretch"));
    assert_eq!(texts(&snap).len(), 4);
    assert_eq!(snap["selected"], json!(3));
    assert_eq!(str_field(&snap, "draft"), "");
    app.type_text("a b");
    assert_eq!(str_field(&app.snapshot(), "draft"), "a b");
    app.press("backspace");
    assert_eq!(str_field(&app.snapshot(), "draft"), "a ");
}

#[gpui_pre::test]
fn ac3_empty_draft_is_ignored(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("enter");
    assert_eq!(texts(&app.snapshot()).len(), 3);
    app.press("space space enter");
    assert_eq!(texts(&app.snapshot()).len(), 3);
}

#[gpui_pre::test]
fn ac4_list_navigation_and_toggle(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("tab");
    assert_eq!(str_field(&app.snapshot(), "focus"), "list");
    app.press("down space");
    let snap = app.snapshot();
    assert_eq!(snap["selected"], json!(1));
    assert_eq!(items(&snap)[1], ("write chapter".into(), true));
    assert_eq!(str_field(&snap, "draft"), "", "list keys must not reach the draft");
    app.press("up up");
    assert_eq!(app.snapshot()["selected"], json!(0));
    app.press("shift-tab");
    assert_eq!(str_field(&app.snapshot(), "focus"), "draft");
}

#[gpui_pre::test]
fn ac5_delete_and_reorder(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("tab down backspace");
    let snap = app.snapshot();
    assert_eq!(texts(&snap), vec!["buy milk", "review pr"]);
    assert_eq!(snap["selected"], json!(1));
    app.press("alt-up");
    let snap = app.snapshot();
    assert_eq!(texts(&snap), vec!["review pr", "buy milk"]);
    assert_eq!(snap["selected"], json!(0));
    app.press("alt-down");
    let snap = app.snapshot();
    assert_eq!(texts(&snap), vec!["buy milk", "review pr"]);
    assert_eq!(snap["selected"], json!(1));
}

#[gpui_pre::test]
fn ac6_pointer_selects_and_toggles(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.click("todo-2");
    let snap = app.snapshot();
    assert_eq!(snap["selected"], json!(2));
    assert_eq!(str_field(&snap, "focus"), "list");
    app.click("toggle-0");
    let snap = app.snapshot();
    assert!(items(&snap)[0].1, "toggle-0 marks the first item done");
    assert_eq!(snap["selected"], json!(2));
}

#[gpui_pre::test]
fn ac8_accessibility_lists_items(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.check_a11y("ac8_accessibility_lists_items", &["buy milk", "write chapter", "review pr"]);
}
