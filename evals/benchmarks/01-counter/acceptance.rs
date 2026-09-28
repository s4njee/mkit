//! Acceptance tests for benchmark 01 (counter). See spec.md.

mod support;

use gpui_pre::TestAppContext;
use support::{open, u64_field};

#[gpui_pre::test]
fn ac1_initial_count_is_zero(cx: &mut TestAppContext) {
    let mut app = open(cx);
    assert_eq!(u64_field(&app.snapshot(), "count"), 0);
}

#[gpui_pre::test]
fn ac2_buttons_change_the_count(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.run_script("click @increment\nclick @increment\nclick @increment\nclick @decrement");
    assert_eq!(u64_field(&app.snapshot(), "count"), 2);
    app.click("reset");
    assert_eq!(u64_field(&app.snapshot(), "count"), 0);
}

#[gpui_pre::test]
fn ac3_keyboard_commands_change_the_count(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("up up up down");
    assert_eq!(u64_field(&app.snapshot(), "count"), 2);
    app.press("escape");
    assert_eq!(u64_field(&app.snapshot(), "count"), 0);
}

#[gpui_pre::test]
fn ac4_count_never_goes_below_zero(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.press("down");
    assert_eq!(u64_field(&app.snapshot(), "count"), 0);
    app.click("decrement");
    assert_eq!(u64_field(&app.snapshot(), "count"), 0);
}

#[gpui_pre::test]
fn ac6_accessibility_names_the_increment_button(cx: &mut TestAppContext) {
    let mut app = open(cx);
    app.check_a11y("ac6_accessibility_names_the_increment_button", &["Increment"]);
}
