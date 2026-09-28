//! Acceptance tests for benchmark 09 (canvas drawing). See spec.md.

mod support;

use gpui_pre::{Pixels, Point, TestAppContext, point, px};
use serde_json::{Value, json};
use support::{Bench, approx, array_field, bool_field, open, str_field};

fn at(app: &mut Bench<'_>, x: f32, y: f32) -> Point<Pixels> {
    let origin = app.target("canvas").origin;
    point(origin.x + px(x), origin.y + px(y))
}

fn draw(app: &mut Bench<'_>, points: &[(f32, f32)]) {
    let path: Vec<_> = points.iter().map(|(x, y)| at(app, *x, *y)).collect();
    app.drag_path(&path);
}

fn shapes(snapshot: &Value) -> &Vec<Value> {
    array_field(snapshot, "shapes")
}

fn bounds_match(shape: &Value, expected: (f64, f64, f64, f64)) -> bool {
    let b = &shape["bounds"];
    let get = |key: &str| b[key].as_f64().unwrap_or(f64::NAN);
    approx(get("x"), expected.0, 2.0)
        && approx(get("y"), expected.1, 2.0)
        && approx(get("width"), expected.2, 2.0)
        && approx(get("height"), expected.3, 2.0)
}

#[gpui_pre::test]
fn ac1_initial_state(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "tool"), "pen");
    assert_eq!(str_field(&snap, "color"), "black");
    assert!(shapes(&snap).is_empty());
    assert!(!bool_field(&snap, "can_undo"));
    assert!(!bool_field(&snap, "can_redo"));
}

#[gpui_pre::test]
fn ac2_drag_draws_a_stroke(cx: &mut TestAppContext) {
    let mut app = open(cx);
    draw(&mut app, &[(50., 50.), (100., 60.), (150., 120.), (200., 80.)]);
    let snap = app.snapshot();
    let shapes = shapes(&snap);
    assert_eq!(shapes.len(), 1, "{snap}");
    let stroke = &shapes[0];
    assert_eq!(stroke["kind"], json!("stroke"));
    assert_eq!(stroke["color"], json!("black"));
    assert!(stroke["points"].as_u64().unwrap_or(0) >= 4, "{stroke}");
    assert!(bounds_match(stroke, (50., 50., 150., 70.)), "{stroke}");
    assert!(bool_field(&snap, "can_undo"));
}

#[gpui_pre::test]
fn ac3_rectangle_tool_and_colours(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("r 2");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "tool"), "rect");
    assert_eq!(str_field(&snap, "color"), "red");
    draw(&mut app, &[(30., 40.), (80., 60.), (130., 90.)]);
    let snap = app.snapshot();
    let rect = &shapes(&snap)[0];
    assert_eq!(rect["kind"], json!("rect"));
    assert_eq!(rect["color"], json!("red"));
    assert!(bounds_match(rect, (30., 40., 100., 50.)), "{rect}");
    app.click("color-3");
    assert_eq!(str_field(&app.snapshot(), "color"), "blue");
    app.click("tool-pen");
    assert_eq!(str_field(&app.snapshot(), "tool"), "pen");
}

#[gpui_pre::test]
fn ac4_undo_redo(cx: &mut TestAppContext) {
    let mut app = open(cx);
    draw(&mut app, &[(20., 20.), (60., 60.)]);
    draw(&mut app, &[(80., 20.), (120., 60.)]);
    assert_eq!(shapes(&app.snapshot()).len(), 2);
    app.press("secondary-z");
    let snap = app.snapshot();
    assert_eq!(shapes(&snap).len(), 1);
    assert!(bool_field(&snap, "can_redo"));
    app.press("secondary-shift-z");
    assert_eq!(shapes(&app.snapshot()).len(), 2);
    app.press("secondary-z");
    draw(&mut app, &[(140., 20.), (180., 60.)]);
    let snap = app.snapshot();
    assert_eq!(shapes(&snap).len(), 2);
    assert!(!bool_field(&snap, "can_redo"));
}

#[gpui_pre::test]
fn ac5_clear_is_undoable(cx: &mut TestAppContext) {
    let mut app = open(cx);
    draw(&mut app, &[(20., 20.), (60., 60.)]);
    draw(&mut app, &[(80., 20.), (120., 60.)]);
    app.press("secondary-backspace");
    let snap = app.snapshot();
    assert!(shapes(&snap).is_empty());
    assert!(bool_field(&snap, "can_undo"));
    app.press("secondary-z");
    assert_eq!(shapes(&app.snapshot()).len(), 2);
}
