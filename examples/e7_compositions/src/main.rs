use gpui_platform::application;
use gpui_pre::{App, AppContext, WindowOptions};
use mkit::core::theme::set_shadcn_light_theme;
use mkit_example_e7_compositions::{ModalKind, StepperModalScene, bind_keys};

fn main() {
    // Pass `sheet` to host the stepper in a Sheet instead of a Dialog.
    let kind = match std::env::args().nth(1).as_deref() {
        Some("sheet") => ModalKind::Sheet,
        _ => ModalKind::Dialog,
    };
    application().run(move |cx: &mut App| {
        gpui_kit::init(cx);
        set_shadcn_light_theme(cx);
        bind_keys(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| StepperModalScene::new(kind)))
            .expect("open stepper modal scene");
        cx.activate(true);
    });
}
