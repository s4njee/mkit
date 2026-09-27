// ANCHOR: reactivity_views_main
use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit_example_reactivity_views::ReactivityDemo;

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| ReactivityDemo::default()))
            .expect("open reactivity example window");
        cx.activate(true);
    });
}
// ANCHOR_END: reactivity_views_main
