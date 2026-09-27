use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit_example_cookbook::{Cookbook, install_cookbook_keys};

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        install_cookbook_keys(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| Cookbook::default()))
            .expect("open cookbook window");
        cx.activate(true);
    });
}
