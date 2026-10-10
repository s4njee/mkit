use gpui_platform::application;
use gpui_pre::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use mkit::core::subcompact::{set_amber_theme, set_paper_theme, set_phosphor_theme};
use mkit_example_subcompact_console::{SubcompactConsole, bind_keys};

fn main() {
    // Pass `amber` or `paper` to start in another palette.
    let palette = std::env::args().nth(1);
    application().run(move |cx: &mut App| {
        match palette.as_deref() {
            Some("amber") => set_amber_theme(cx),
            Some("paper") => set_paper_theme(cx),
            _ => set_phosphor_theme(cx),
        }
        bind_keys(cx);
        let bounds = Bounds::centered(None, size(px(960.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(SubcompactConsole::new),
        )
        .expect("open subcompact console");
        cx.activate(true);
    });
}
