// ANCHOR: text_field_main
use gpui_platform::application;
use gpui_pre::{App, AppContext, Focusable, WindowOptions};
use mkit_example_text_input::TextField;

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        cx.open_window(WindowOptions::default(), |window, cx| {
            let field = cx.new(TextField::new);
            let focus = field.read(cx).focus_handle(cx);
            window.focus(&focus, cx);
            field
        })
        .expect("open text field window");
        cx.activate(true);
    });
}
// ANCHOR_END: text_field_main
