use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use std::path::PathBuf;

fn main() {
    let fixture = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| "fixture".into());
    application().run(move |cx: &mut App| {
        bench_app::init(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| bench_app::root(&fixture)))
            .expect("open benchmark window");
        cx.activate(true);
    });
}
