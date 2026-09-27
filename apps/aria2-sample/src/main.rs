use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowBounds, WindowOptions, px, size};
use mkit_aria2_sample::{Aria2Sample, install_fonts, theme};

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        install_fonts(cx);
        theme::install(cx, theme::LIGHT);
        cx.bind_keys(mkit::button::default_key_bindings());
        cx.bind_keys(mkit::text_field::default_key_bindings());
        cx.bind_keys(mkit::dialog::default_key_bindings());
        cx.bind_keys(mkit::data_table::default_key_bindings());
        cx.bind_keys(mkit::select::default_key_bindings());
        cx.bind_keys(mkit::slider::default_key_bindings());
        cx.bind_keys(mkit_aria2_sample::default_key_bindings());
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1080.0), px(640.0)), cx)),
            ..WindowOptions::default()
        };
        cx.open_window(window_options, |_, cx| cx.new(|_| Aria2Sample::new()))
            .expect("open Aria2 sample window");
        cx.activate(true);
    });
}
