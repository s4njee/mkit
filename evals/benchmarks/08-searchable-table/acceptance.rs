//! Acceptance tests for benchmark 08 (searchable table). See spec.md.

mod support;

use gpui_pre::TestAppContext;
use serde_json::{Value, json};
use support::{Bench, array_field, open, str_field, u64_field};

#[derive(Clone)]
struct Row {
    id: u64,
    name: String,
    city: String,
    amount: u64,
}

fn rows(app: &Bench<'_>) -> Vec<Row> {
    std::fs::read_to_string(app.fixture.join("rows.csv"))
        .unwrap()
        .lines()
        .skip(1)
        .map(|line| {
            let f: Vec<_> = line.split(',').collect();
            Row {
                id: f[0].parse().unwrap(),
                name: f[1].into(),
                city: f[2].into(),
                amount: f[3].parse().unwrap(),
            }
        })
        .collect()
}

fn ids(rows: &[Row]) -> Vec<u64> {
    rows.iter().take(20).map(|row| row.id).collect()
}

fn top_ids(snapshot: &Value) -> Vec<u64> {
    array_field(snapshot, "top_ids").iter().map(|id| id.as_u64().expect("id")).collect()
}

fn rendered(snapshot: &Value) -> Vec<u64> {
    array_field(snapshot, "rendered_ids").iter().map(|id| id.as_u64().expect("id")).collect()
}

#[gpui_pre::test]
fn ac1_loads_rows_virtualized(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let snap = app.snapshot();
    assert_eq!(u64_field(&snap, "match_count"), 500);
    assert_eq!(top_ids(&snap), (1..=20).collect::<Vec<_>>());
    assert_eq!(snap["sort"], json!({"column": "id", "direction": "asc"}));
    assert_eq!(snap["selected_id"], Value::Null);
    assert_eq!(str_field(&snap, "focus"), "search");
    let count = rendered(&snap).len();
    assert!((1..=100).contains(&count), "rendered rows: {count}");
}

#[gpui_pre::test]
fn ac2_search_filters_case_insensitively(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let expected: Vec<Row> = rows(&app)
        .into_iter()
        .filter(|r| r.name.to_lowercase().contains("osl") || r.city.to_lowercase().contains("osl"))
        .collect();
    assert!(!expected.is_empty());
    app.type_text("osl");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "query"), "osl");
    assert_eq!(u64_field(&snap, "match_count"), expected.len() as u64);
    assert_eq!(top_ids(&snap), ids(&expected));
    app.press("backspace backspace backspace");
    assert_eq!(u64_field(&app.snapshot(), "match_count"), 500);
}

#[gpui_pre::test]
fn ac3_header_click_sorts(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let all = rows(&app);
    let mut by_amount = all.clone();
    by_amount.sort_by_key(|r| (r.amount, r.id));
    app.click("header-amount");
    let snap = app.snapshot();
    assert_eq!(snap["sort"], json!({"column": "amount", "direction": "asc"}));
    assert_eq!(top_ids(&snap), ids(&by_amount));
    let mut desc = all.clone();
    desc.sort_by(|a, b| b.amount.cmp(&a.amount).then(a.id.cmp(&b.id)));
    app.click("header-amount");
    let snap = app.snapshot();
    assert_eq!(snap["sort"], json!({"column": "amount", "direction": "desc"}));
    assert_eq!(top_ids(&snap), ids(&desc));
    let mut by_name = all;
    by_name.sort_by_key(|r| (r.name.to_lowercase(), r.id));
    app.click("header-name");
    assert_eq!(top_ids(&app.snapshot()), ids(&by_name));
}

#[gpui_pre::test]
fn ac4_keyboard_selection_and_details(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let third = rows(&app)[2].clone();
    app.press("down");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "focus"), "table");
    assert_eq!(snap["selected_id"], json!(1));
    app.press("down down");
    let snap = app.snapshot();
    assert_eq!(snap["selected_id"], json!(3));
    assert_eq!(
        snap["details"],
        json!({"id": third.id, "name": third.name, "city": third.city, "amount": third.amount})
    );
    app.click("row-5");
    assert_eq!(app.snapshot()["selected_id"], json!(5));
}

#[gpui_pre::test]
fn ac5_escape_clears_and_focus_shortcut(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.type_text("lim");
    app.press("down escape");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "focus"), "search");
    assert_eq!(str_field(&snap, "query"), "lim");
    app.press("escape");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "query"), "");
    assert_eq!(u64_field(&snap, "match_count"), 500);
    app.press("down");
    assert_eq!(str_field(&app.snapshot(), "focus"), "table");
    app.press("secondary-f");
    assert_eq!(str_field(&app.snapshot(), "focus"), "search");
}

#[gpui_pre::test]
fn ac6_scrolling_keeps_rendering_bounded(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("down end");
    let snap = app.snapshot();
    assert_eq!(snap["selected_id"], json!(500));
    let built = rendered(&snap);
    assert!(built.contains(&500), "selected row 500 is rendered: {built:?}");
    assert!(built.len() <= 100, "at most 100 rows built, got {}", built.len());
    app.press("home");
    assert_eq!(app.snapshot()["selected_id"], json!(1));
    app.press("pagedown");
    assert_eq!(app.snapshot()["selected_id"], json!(21));
}
