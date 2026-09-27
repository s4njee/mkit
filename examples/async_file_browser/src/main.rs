// ANCHOR: file_browser_main
use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit_example_async_file_browser::{FileBrowser, install_browser_settings, sample_directory};

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        install_browser_settings(cx);
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|cx| {
                let mut browser = FileBrowser::new(sample_directory());
                browser.reload(cx).detach();
                browser
            })
        })
        .expect("open file browser window");
        cx.activate(true);
    });
}
// ANCHOR_END: file_browser_main
