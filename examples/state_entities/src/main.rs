// ANCHOR: state_entities_main
use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit_example_state_entities::{CounterModel, CounterSettings, CounterWindow};

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        cx.set_global(CounterSettings::default());

        cx.open_window(WindowOptions::default(), |_, cx| {
            let counter = cx.new(|_| CounterModel::default());
            cx.new(|_| CounterWindow::new(counter))
        })
        .expect("open state and entities window");
        cx.activate(true);
    });
}
// ANCHOR_END: state_entities_main
