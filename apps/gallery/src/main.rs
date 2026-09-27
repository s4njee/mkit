use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit::core::theme::set_light_theme;
use mkit_gallery::ComponentGallery;

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        set_light_theme(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| ComponentGallery::new()))
            .expect("open gallery window");
        cx.activate(true);
    });
}
