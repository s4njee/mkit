// ANCHOR: interaction_main
use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit_example_interaction::{InteractionList, bind_interaction_keys, set_interaction_menus};

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        let alternate_down_key =
            std::env::var("MKIT_MOVE_DOWN_KEY").unwrap_or_else(|_| "alt-j".into());
        bind_interaction_keys(cx, &alternate_down_key)
            .expect("MKIT_MOVE_DOWN_KEY must be one valid keystroke");
        set_interaction_menus(cx);
        cx.open_window(WindowOptions::default(), move |_, cx| {
            cx.new(|_| InteractionList::with_alternate_down_key(alternate_down_key))
        })
        .expect("open interaction example window");
        cx.activate(true);
    });
}
// ANCHOR_END: interaction_main
