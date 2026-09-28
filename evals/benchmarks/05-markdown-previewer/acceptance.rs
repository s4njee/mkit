//! Acceptance tests for benchmark 05 (markdown previewer). See spec.md.

mod support;

use gpui_pre::TestAppContext;
use serde_json::{Value, json};
use support::{array_field, f64_field, open, str_field};

fn kinds(snapshot: &Value) -> Vec<String> {
    array_field(snapshot, "blocks")
        .iter()
        .map(|block| block["kind"].as_str().expect("block kind").to_owned())
        .collect()
}

fn has_span(block: &Value, text: &str, style: &str) -> bool {
    block["spans"].as_array().is_some_and(|spans| {
        spans.iter().any(|span| span["text"] == json!(text) && span[style] == json!(true))
    })
}

#[gpui_pre::test]
fn ac1_parses_blocks(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let snap = app.snapshot();
    let blocks = array_field(&snap, "blocks");
    let mut expected = vec!["heading", "paragraph", "heading", "list_item", "list_item"];
    expected.extend(["list_item", "heading", "code_block", "heading"]);
    expected.extend(std::iter::repeat_n("paragraph", 60));
    expected.extend(["heading", "paragraph"]);
    assert_eq!(kinds(&snap), expected);
    assert_eq!(blocks[0]["level"], json!(1));
    assert_eq!(blocks[0]["text"], json!("Field notes"));
    assert!(str_field(&blocks[1], "text").starts_with("GPUI renders retained state"));
    assert_eq!(blocks[2]["level"], json!(2));
    assert_eq!(blocks[3]["text"], json!("first item"));
    assert_eq!(blocks[7]["lang"], json!("rust"));
    assert_eq!(str_field(&blocks[7], "text").trim_end().lines().count(), 3);
    assert_eq!(blocks[blocks.len() - 2]["text"], json!("Last heading"));
    assert_eq!(blocks[blocks.len() - 1]["text"], json!("Final paragraph."));
}

#[gpui_pre::test]
fn ac2_inline_styles(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let snap = app.snapshot();
    let blocks = array_field(&snap, "blocks");
    let first = &blocks[1];
    assert!(has_span(first, "retained state", "bold"), "{first}");
    assert!(has_span(first, "immediate-style", "italic"), "{first}");
    assert!(has_span(first, "cx.notify()", "code"), "{first}");
    let text = str_field(first, "text");
    assert!(!text.contains('*') && !text.contains('`'), "markers removed: {text}");
    let joined: String = first["spans"]
        .as_array()
        .unwrap()
        .iter()
        .map(|span| span["text"].as_str().unwrap())
        .collect();
    assert_eq!(joined, text);
    assert!(has_span(&blocks[4], "bold", "bold"), "{}", blocks[4]);
}

#[gpui_pre::test]
fn ac3_source_mode_toggle(cx: &mut TestAppContext) {
    let mut app = open(cx);
    assert_eq!(str_field(&app.snapshot(), "mode"), "preview");
    app.press("secondary-e");
    assert_eq!(str_field(&app.snapshot(), "mode"), "source");
    app.press("secondary-e");
    assert_eq!(str_field(&app.snapshot(), "mode"), "preview");
}

#[gpui_pre::test]
fn ac4_reload_from_disk(cx: &mut TestAppContext) {
    let mut app = open(cx);
    std::fs::write(app.fixture.join("doc.md"), "# Changed\n\nNew body.\n").unwrap();
    app.press("secondary-r");
    let snap = app.snapshot();
    let blocks = array_field(&snap, "blocks");
    assert_eq!(kinds(&snap), vec!["heading", "paragraph"]);
    assert_eq!(blocks[0]["text"], json!("Changed"));
    assert_eq!(blocks[1]["text"], json!("New body."));
}

#[gpui_pre::test]
fn ac5_heading_navigation_scrolls(cx: &mut TestAppContext) {
    let mut app = open(cx);
    assert_eq!(app.snapshot()["current_heading"], Value::Null);
    app.press("ctrl-down ctrl-down ctrl-down ctrl-down ctrl-down");
    let snap = app.snapshot();
    assert_eq!(snap["current_heading"], json!(4));
    assert!(f64_field(&snap, "scroll_top") > 0.0, "last heading scrolled into view: {snap}");
    app.press("ctrl-up");
    assert_eq!(app.snapshot()["current_heading"], json!(3));
    app.press("home");
    let snap = app.snapshot();
    assert_eq!(snap["current_heading"], Value::Null);
    assert!(f64_field(&snap, "scroll_top").abs() < 0.5, "{snap}");
}
