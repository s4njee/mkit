use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit::core::theme::set_shadcn_light_theme;
use mkit_gallery::e7::EverydayGallery;

fn selected_scene() -> EverydayGallery {
    let mut args = std::env::args().skip(1);
    match (args.next(), args.next(), args.next()) {
        (None, None, None) => EverydayGallery::new(),
        (Some(flag), Some(name), None) if flag == "--scene" => match name.as_str() {
            "main" => EverydayGallery::new(),
            "inputs" => EverydayGallery::inputs_preview(),
            "menus" => EverydayGallery::menus_preview(),
            "overlays" => EverydayGallery::overlays_preview(),
            "navigation" => EverydayGallery::navigation_preview(),
            "collections" => EverydayGallery::collections_preview(),
            "layout" => EverydayGallery::layout_preview(),
            _ => panic!("unknown E7 gallery scene: {name}"),
        },
        _ => panic!(
            "usage: e7_gallery [--scene main|inputs|menus|overlays|navigation|collections|layout]"
        ),
    }
}

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        set_shadcn_light_theme(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| selected_scene()))
            .expect("open E7 gallery window");
        cx.activate(true);
    });
}
