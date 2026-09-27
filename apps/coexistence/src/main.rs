//! A live same-window integration of GPUI Kit and mkit on the pinned GPUI snapshot.

use gpui_kit::component::button::Button as KitButton;
use gpui_pre::{
    App, AppContext, Context, IntoElement, Render, Window, WindowOptions, div, prelude::*, px,
};
use mkit::{
    button::Button as MkitButton,
    core::theme::{self, LIGHT, Theme},
};

struct Coexistence;

impl Render for Coexistence {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;
        div()
            .id("coexistence-root")
            .debug_selector(|| "coexistence-root".into())
            .size_full()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            .bg(colors.background)
            .text_color(colors.text)
            .child("Both component libraries in one GPUI window")
            .child(
                div()
                    .flex()
                    .gap(px(12.0))
                    .child(MkitButton::new("mkit button").id(1))
                    .child(KitButton::new("kit-button").label("GPUI Kit button")),
            )
    }
}

fn initialize(cx: &mut App) {
    gpui_kit::init(cx);
    theme::set_theme(cx, LIGHT);
}

fn main() {
    gpui_kit::application().run(|cx: &mut App| {
        initialize(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| Coexistence))
            .expect("open coexistence window");
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::TestAppContext;

    #[gpui_kit::test]
    fn both_libraries_render_in_one_window(cx: &mut TestAppContext) {
        cx.update(initialize);
        let (_, window) = cx.add_window_view(|_, _| Coexistence);
        window.update(|window, cx| window.draw(cx).clear(cx));
        assert!(window.debug_bounds("coexistence-root").is_some());
        assert!(window.debug_bounds("mkit-button").is_some());
    }
}
