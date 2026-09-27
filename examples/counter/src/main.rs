// ANCHOR: counter_main
use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit_example_counter::Counter;

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| Counter::default()))
            .expect("open counter window");
        cx.activate(true);
    });
}
// ANCHOR_END: counter_main
