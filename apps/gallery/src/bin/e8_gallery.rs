use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit::core::theme::set_shadcn_light_theme;
use mkit_gallery::e8::ProGallery;

fn selected_scene() -> ProGallery {
    let mut args = std::env::args().skip(1);
    match (args.next(), args.next(), args.next()) {
        (None, None, None) => ProGallery::new(),
        (Some(flag), Some(name), None) if flag == "--scene" => match name.as_str() {
            "canvas" => ProGallery::canvas_preview(),
            "adjust" => ProGallery::adjust_preview(),
            "inspect" => ProGallery::inspect_preview(),
            "command" => ProGallery::command_preview(),
            "time" => ProGallery::time_preview(),
            "graph" => ProGallery::graph_preview(),
            _ => panic!("unknown E8 gallery scene: {name}"),
        },
        _ => panic!("usage: e8_gallery [--scene canvas|adjust|inspect|command|time|graph]"),
    }
}

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        set_shadcn_light_theme(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| selected_scene()))
            .expect("open E8 gallery window");
        cx.activate(true);
    });
}
