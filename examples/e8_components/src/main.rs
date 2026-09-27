use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit::core::theme::set_shadcn_light_theme;
use mkit_example_e8_components::E8Preview;

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        set_shadcn_light_theme(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| E8Preview::default()))
            .expect("open E8 preview window");
        cx.activate(true);
    });
}
