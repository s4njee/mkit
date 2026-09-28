//! Acceptance tests for benchmark 07 (image viewer). See spec.md.

mod support;

use gpui_pre::{Modifiers, TestAppContext, point, px};
use serde_json::{Value, json};
use support::{Bench, approx, f64_field, open, open_with_fixture, str_field};

fn zoom(app: &mut Bench<'_>) -> f64 {
    f64_field(&app.snapshot(), "zoom")
}

fn pan(snapshot: &Value) -> (f64, f64) {
    (snapshot["pan"]["x"].as_f64().expect("pan.x"), snapshot["pan"]["y"].as_f64().expect("pan.y"))
}

#[gpui_pre::test]
fn ac1_loads_and_fits(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let snap = app.snapshot();
    assert_eq!(snap["image"], json!({"width": 400, "height": 300}));
    assert_eq!(snap["error"], Value::Null);
    assert_eq!(str_field(&snap, "mode"), "fit");
    let fit = f64_field(&snap, "fit_zoom");
    assert!(fit > 0.0);
    assert!(approx(f64_field(&snap, "zoom"), fit, 1e-3));
    assert_eq!(pan(&snap), (0.0, 0.0));
}

#[gpui_pre::test]
fn ac2_keyboard_zoom_steps_and_clamps(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("secondary-1");
    assert!(approx(zoom(&mut app), 1.0, 1e-3));
    assert_eq!(str_field(&app.snapshot(), "mode"), "manual");
    app.press("secondary-=");
    assert!(approx(zoom(&mut app), 1.25, 1e-3));
    for _ in 0..20 {
        app.press("secondary-=");
    }
    assert!(approx(zoom(&mut app), 8.0, 1e-3));
    for _ in 0..40 {
        app.press("secondary--");
    }
    assert!(approx(zoom(&mut app), 0.1, 1e-3));
}

#[gpui_pre::test]
fn ac3_actual_size_and_fit(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("secondary-1 right");
    app.press("secondary-0");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "mode"), "fit");
    assert!(approx(f64_field(&snap, "zoom"), f64_field(&snap, "fit_zoom"), 1e-3));
    assert_eq!(pan(&snap), (0.0, 0.0));
}

#[gpui_pre::test]
fn ac4_drag_and_arrow_pan(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let center = app.target("viewport").center();
    app.drag(center, point(center.x + px(40.), center.y - px(30.)), 5);
    let (x, y) = pan(&app.snapshot());
    assert!(approx(x, 40.0, 1.0) && approx(y, -30.0, 1.0), "pan after drag: ({x}, {y})");
    app.press("right down");
    let (x, y) = pan(&app.snapshot());
    assert!(approx(x, 90.0, 1.0) && approx(y, 20.0, 1.0), "pan after arrows: ({x}, {y})");
}

#[gpui_pre::test]
fn ac5_ctrl_scroll_zooms(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("secondary-1");
    let center = app.target("viewport").center();
    let ctrl = Modifiers { control: true, ..Modifiers::default() };
    app.scroll(center, 120.0, ctrl);
    assert!(approx(zoom(&mut app), 1.25, 1e-3), "ctrl-scroll up zooms in");
    app.scroll(center, -120.0, ctrl);
    assert!(approx(zoom(&mut app), 1.0, 1e-3), "ctrl-scroll down zooms out");
}

#[gpui_pre::test]
fn ac6_missing_image_reports_error(cx: &mut TestAppContext) {
    let fixture = support::fresh_fixture();
    let _ = std::fs::remove_file(fixture.join("image.png"));
    let mut app = open_with_fixture(cx, fixture);
    let snap = app.snapshot();
    assert_eq!(snap["image"], Value::Null);
    assert!(!str_field(&snap, "error").is_empty());
}
