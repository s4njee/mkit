// ANCHOR: hello_main
use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit_example_hello::InspectorFixture;

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| InspectorFixture::hello()))
            .expect("open hello window");
        cx.activate(true);
    });
}
// ANCHOR_END: hello_main
