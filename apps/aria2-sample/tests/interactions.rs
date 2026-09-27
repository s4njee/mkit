extern crate gpui_pre as gpui;

use gpui_pre::{Modifiers, TestAppContext};
use mkit_aria2_sample::{Aria2Sample, default_key_bindings, install_fonts, model::Scene, theme};
use mkit_harness::{AccessibilityError, AccessibilitySnapshot};

fn setup(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::base::init(cx);
        install_fonts(cx);
        theme::install(cx, theme::LIGHT);
        cx.bind_keys(mkit::button::default_key_bindings());
        cx.bind_keys(mkit::text_field::default_key_bindings());
        cx.bind_keys(mkit::dialog::default_key_bindings());
        cx.bind_keys(mkit::data_table::default_key_bindings());
        cx.bind_keys(mkit::select::default_key_bindings());
        cx.bind_keys(mkit::slider::default_key_bindings());
        cx.bind_keys(default_key_bindings());
    });
}

#[gpui_pre::test]
fn pointer_filter_and_keyboard_selection(cx: &mut TestAppContext) {
    setup(cx);
    let (sample, visual) = cx.add_window_view(|_, _| Aria2Sample::new());
    visual.update(|window, cx| window.draw(cx).clear(cx));

    let first = visual.debug_bounds("download-0").expect("first download row").center();
    visual.simulate_click(first, Modifiers::default());
    sample.read_with(visual, |sample, _| {
        assert_eq!(sample.selected_name(), Some("ubuntu-24.04.2-desktop-amd64.iso"));
    });

    visual.simulate_keystrokes("down");
    sample.read_with(visual, |sample, _| {
        assert_eq!(sample.selected_name(), Some("Blender-4.5.0-linux-x64.tar.xz"));
    });

    let paused = visual.debug_bounds("category-Paused").expect("paused category").center();
    visual.simulate_click(paused, Modifiers::default());
    sample.read_with(visual, |sample, _| assert_eq!(sample.visible_count(), 1));
}

#[gpui_pre::test]
fn capture_selection_updates_download_count(cx: &mut TestAppContext) {
    setup(cx);
    let (sample, visual) = cx.add_window_view(|_, _| Aria2Sample::for_scene(Scene::Capture));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    sample.read_with(visual, |sample, _| assert_eq!(sample.captured_count(), 3));

    let torrent = visual.debug_bounds("captured-link-1").expect("second captured link").center();
    visual.simulate_click(torrent, Modifiers::default());
    sample.read_with(visual, |sample, _| assert_eq!(sample.captured_count(), 4));
}

#[gpui_pre::test]
fn accessibility_capture_reports_headless_gap(cx: &mut TestAppContext) {
    setup(cx);
    let (_, visual) = cx.add_window_view(|_, _| Aria2Sample::new());
    visual.update(|window, cx| window.draw(cx).clear(cx));
    assert_eq!(
        visual.update(|window, _| AccessibilitySnapshot::capture(window)),
        Err(AccessibilityError::Inactive)
    );
}
