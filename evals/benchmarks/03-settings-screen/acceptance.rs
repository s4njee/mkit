//! Acceptance tests for benchmark 03 (settings screen). See spec.md.

mod support;

use gpui_pre::TestAppContext;
use serde_json::json;
use support::{bool_field, open, str_field, u64_field};

#[gpui_pre::test]
fn ac1_defaults(cx: &mut TestAppContext) {
    let mut app = open(cx);
    let snap = app.snapshot();
    for (key, value) in [
        ("section", json!("general")),
        ("launch_at_login", json!(false)),
        ("restore_windows", json!(true)),
        ("theme", json!("light")),
        ("font_size", json!(14)),
        ("desktop_alerts", json!(true)),
        ("alert_sound", json!(false)),
        ("focus", json!("sidebar")),
    ] {
        assert_eq!(snap[key], value, "default `{key}` in {snap}");
    }
}

#[gpui_pre::test]
fn ac2_sidebar_click_switches_sections(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.click("section-appearance");
    assert_eq!(str_field(&app.snapshot(), "section"), "appearance");
    app.click("section-notifications");
    assert_eq!(str_field(&app.snapshot(), "section"), "notifications");
}

#[gpui_pre::test]
fn ac3_switches_toggle_by_click(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.click("launch-at-login");
    app.click("restore-windows");
    let snap = app.snapshot();
    assert!(bool_field(&snap, "launch_at_login"));
    assert!(!bool_field(&snap, "restore_windows"));
}

#[gpui_pre::test]
fn ac4_conditional_sound_row(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.click("section-notifications");
    assert!(app.bounds("alert-sound").is_some(), "alert-sound is shown while alerts are on");
    app.click("desktop-alerts");
    assert!(!bool_field(&app.snapshot(), "desktop_alerts"));
    assert!(app.bounds("alert-sound").is_none(), "alert-sound is hidden while alerts are off");
    app.click("desktop-alerts");
    assert!(app.bounds("alert-sound").is_some());
    assert!(!bool_field(&app.snapshot(), "alert_sound"));
}

#[gpui_pre::test]
fn ac5_font_size_clamps(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.click("section-appearance");
    for _ in 0..12 {
        app.click("font-larger");
    }
    assert_eq!(u64_field(&app.snapshot(), "font_size"), 24);
    for _ in 0..20 {
        app.click("font-smaller");
    }
    assert_eq!(u64_field(&app.snapshot(), "font_size"), 10);
    app.click("theme-dark");
    assert_eq!(str_field(&app.snapshot(), "theme"), "dark");
}

#[gpui_pre::test]
fn ac6_keyboard_sections_and_tab_focus(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("down down");
    assert_eq!(str_field(&app.snapshot(), "section"), "notifications");
    app.press("up");
    let snap = app.snapshot();
    assert_eq!(str_field(&snap, "section"), "appearance");
    assert_eq!(str_field(&snap, "focus"), "sidebar");
    app.press("up");
    assert_eq!(str_field(&app.snapshot(), "section"), "general");
    app.press("tab");
    assert_eq!(str_field(&app.snapshot(), "focus"), "launch-at-login");
    app.press("space");
    assert!(bool_field(&app.snapshot(), "launch_at_login"));
    app.press("tab");
    assert_eq!(str_field(&app.snapshot(), "focus"), "restore-windows");
    app.press("shift-tab shift-tab");
    assert_eq!(str_field(&app.snapshot(), "focus"), "sidebar");
}

#[gpui_pre::test]
fn ac8_accessibility_names_switches(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.check_a11y("ac8_accessibility_names_switches", &["Launch at login", "Restore windows"]);
}
