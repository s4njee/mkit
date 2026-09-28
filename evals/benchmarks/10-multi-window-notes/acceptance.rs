//! Acceptance tests for benchmark 10 (multi-window notes). See spec.md.

mod support;

use gpui_pre::{TestAppContext, VisualTestContext};
use serde_json::{Value, json};
use support::{Bench, array_field, bool_field, open};

fn note(snapshot: &Value, id: u64) -> Value {
    array_field(snapshot, "notes")
        .iter()
        .find(|note| note["id"] == json!(id))
        .cloned()
        .unwrap_or_else(|| panic!("note {id} in {snapshot}"))
}

fn only_note_window(app: &mut Bench<'_>) -> VisualTestContext {
    let others = app.other_windows();
    assert_eq!(others.len(), 1, "exactly one note window is open");
    app.window_cx(others[0])
}

fn type_in(window: &mut VisualTestContext, text: &str) {
    for ch in text.chars() {
        window.simulate_input(&ch.to_string());
    }
    window.update(|w, cx| w.draw(cx).clear(cx));
}

#[gpui_pre::test]
fn ac1_loads_store(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let snap = app.snapshot();
    assert_eq!(
        snap["notes"],
        json!([
            {"id": 1, "title": "Groceries", "body": "Groceries\nmilk, eggs"},
            {"id": 2, "title": "Ideas", "body": "Ideas\nwrite the book"}
        ])
    );
    assert_eq!(snap["selected"], json!(0));
    assert_eq!(snap["open_note_ids"], json!([]));
    assert!(!bool_field(&snap, "dirty"));
    assert!(app.other_windows().is_empty());
}

#[gpui_pre::test]
fn ac2_new_note_opens_window(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("secondary-n");
    let snap = app.snapshot();
    assert_eq!(note(&snap, 3), json!({"id": 3, "title": "Untitled", "body": ""}));
    assert_eq!(snap["selected"], json!(2));
    assert_eq!(snap["open_note_ids"], json!([3]));
    let mut window = only_note_window(&mut app);
    type_in(&mut window, "plan");
    let snap = app.snapshot();
    assert_eq!(note(&snap, 3), json!({"id": 3, "title": "plan", "body": "plan"}));
    assert_eq!(window.window_title().as_deref(), Some("plan"));
}

#[gpui_pre::test]
fn ac3_editing_updates_list_live(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("down enter");
    assert_eq!(app.snapshot()["open_note_ids"], json!([2]));
    let mut window = only_note_window(&mut app);
    type_in(&mut window, "!");
    let snap = app.snapshot();
    assert_eq!(note(&snap, 2)["body"], json!("Ideas\nwrite the book!"));
    assert!(bool_field(&snap, "dirty"));
}

#[gpui_pre::test]
fn ac4_save_persists_to_disk(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("secondary-n");
    let mut window = only_note_window(&mut app);
    type_in(&mut window, "saved");
    window.simulate_keystrokes("secondary-s");
    let stored: Value =
        serde_json::from_str(&std::fs::read_to_string(app.fixture.join("notes.json")).unwrap())
            .expect("notes.json stays valid JSON");
    let notes = array_field(&stored, "notes");
    assert!(notes.iter().any(|n| n["id"] == json!(3) && n["body"] == json!("saved")), "{stored}");
    assert!(!bool_field(&app.snapshot(), "dirty"));
}

#[gpui_pre::test]
fn ac5_open_existing_focuses_single_window(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("enter");
    app.press("enter");
    assert_eq!(app.other_windows().len(), 1);
    assert_eq!(app.snapshot()["open_note_ids"], json!([1]));
}

#[gpui_pre::test]
fn ac6_close_window(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("enter");
    let mut window = only_note_window(&mut app);
    window.simulate_keystrokes("secondary-w");
    assert!(app.other_windows().is_empty(), "note window closed");
    let snap = app.snapshot();
    assert_eq!(snap["open_note_ids"], json!([]));
    assert_eq!(array_field(&snap, "notes").len(), 2);
}
