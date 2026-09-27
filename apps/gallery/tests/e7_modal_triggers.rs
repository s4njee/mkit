extern crate gpui_pre as gpui;

use gpui_pre::{Modifiers, TestAppContext};
use mkit::core::theme;
use mkit_gallery::e7::EverydayGallery;

#[gpui_pre::test]
fn dialog_gallery_trigger_opens_a_single_modal(cx: &mut TestAppContext) {
    cx.update(|app| {
        gpui_kit::base::init(app);
        theme::set_light_theme(app);
    });
    let (_, visual) = cx.add_window_view(|_, _| EverydayGallery::overlays_preview());
    visual.update(|window, cx| window.draw(cx).clear(cx));
    assert!(visual.debug_bounds("mkit-dialog-surface").is_none());
    let trigger = visual.debug_bounds("e7-dialog-trigger").expect("dialog trigger").center();
    visual.simulate_click(trigger, Modifiers::default());
    visual.update(|window, cx| window.draw(cx).clear(cx));
    assert!(visual.debug_bounds("mkit-dialog-surface").is_some());
    assert!(visual.debug_bounds("mkit-sheet-surface").is_none());
}

#[gpui_pre::test]
fn sheet_gallery_trigger_opens_a_single_modal(cx: &mut TestAppContext) {
    cx.update(|app| {
        gpui_kit::base::init(app);
        theme::set_light_theme(app);
    });
    let (_, visual) = cx.add_window_view(|_, _| EverydayGallery::overlays_preview());
    visual.update(|window, cx| window.draw(cx).clear(cx));
    assert!(visual.debug_bounds("mkit-sheet-surface").is_none());
    let trigger = visual.debug_bounds("e7-sheet-trigger").expect("sheet trigger").center();
    visual.simulate_click(trigger, Modifiers::default());
    visual.update(|window, cx| window.draw(cx).clear(cx));
    assert!(visual.debug_bounds("mkit-sheet-surface").is_some());
    assert!(visual.debug_bounds("mkit-dialog-surface").is_none());
}
