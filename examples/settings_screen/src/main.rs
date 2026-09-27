// ANCHOR: settings_screen_main
use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit_example_settings_screen::SettingsScreen;

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| SettingsScreen::default()))
            .expect("open settings screen");
        cx.activate(true);
    });
}
// ANCHOR_END: settings_screen_main
