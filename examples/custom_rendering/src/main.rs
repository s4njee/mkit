// ANCHOR: pan_zoom_main
use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit_example_custom_rendering::PanZoomView;

fn main() {
    application().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| PanZoomView::default()))
            .expect("open pan-and-zoom canvas window");
        cx.activate(true);
    });
}
// ANCHOR_END: pan_zoom_main
