// ANCHOR: platform_shipping_main
use gpui_platform::application;
use gpui_pre::{App, AppContext};
use mkit_example_platform_shipping::{
    ShippingDemo, install_diagnostics, install_panic_hook, open_diagnostics,
    primary_window_options, record_tagged_error,
};

fn main() {
    application().run(|cx: &mut App| {
        gpui_kit::init(cx);
        let sink = install_diagnostics().expect("install the local diagnostics logger");
        install_panic_hook(sink);
        record_tagged_error("APP_START");

        cx.open_window(primary_window_options(), |_, cx| cx.new(|_| ShippingDemo::default()))
            .expect("open primary window");
        open_diagnostics(cx).expect("open diagnostics window");
        cx.activate(true);
    });
}
// ANCHOR_END: platform_shipping_main
