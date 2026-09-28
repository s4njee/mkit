//! A live same-window integration of GPUI Kit and mkit on the pinned GPUI snapshot.
//!
//! mkit's `Theme` global is the source of truth. [`shared_theme::sync_kit_theme`]
//! projects its colour tokens onto GPUI Kit's `Theme` global, so both
//! libraries paint from the same palette in one window.

mod shared_theme;

use gpui_kit::component::button::{Button as KitButton, ButtonVariants as _};
use gpui_pre::{
    App, AppContext, Context, IntoElement, Render, Window, WindowOptions, div, prelude::*, px,
};
use mkit::{
    button::Button as MkitButton,
    core::theme::{LIGHT, Theme},
};

struct Coexistence;

impl Render for Coexistence {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let colors = theme.colors;
        let gap = theme.spacing.medium;
        let padding = theme.spacing.xlarge;
        div()
            .id("coexistence-root")
            .debug_selector(|| "coexistence-root".into())
            .size_full()
            .p(px(padding))
            .flex()
            .flex_col()
            .gap(px(gap))
            .bg(colors.background)
            .text_color(colors.text)
            .child("Both component libraries in one GPUI window, sharing mkit's theme colours")
            .child(
                div()
                    .flex()
                    .gap(px(gap))
                    .child(MkitButton::new("mkit button").id(1))
                    .child(KitButton::new("kit-primary-button").primary().label("GPUI Kit primary"))
                    .child(KitButton::new("kit-button").label("GPUI Kit button")),
            )
    }
}

fn initialize(cx: &mut App) {
    initialize_with(cx, LIGHT);
}

fn initialize_with(cx: &mut App, theme: Theme) {
    gpui_kit::init(cx);
    shared_theme::set_shared_theme(cx, theme);
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
    use gpui_kit::component::Theme as KitTheme;
    use gpui_pre::{Hsla, TestAppContext};
    use mkit::core::theme::DARK;

    #[gpui_kit::test]
    fn both_libraries_render_in_one_window(cx: &mut TestAppContext) {
        cx.update(initialize);
        let (_, window) = cx.add_window_view(|_, _| Coexistence);
        window.update(|window, cx| window.draw(cx).clear(cx));
        assert!(window.debug_bounds("coexistence-root").is_some());
        assert!(window.debug_bounds("mkit-button").is_some());
    }

    fn assert_kit_uses_mkit_colours(cx: &mut TestAppContext, theme: Theme, dark: bool) {
        cx.update(|cx| initialize_with(cx, theme));
        cx.read(|cx| {
            let mkit = cx.global::<Theme>().colors;
            let kit = KitTheme::global(cx);
            assert_eq!(kit.is_dark(), dark);
            assert_eq!(kit.background, Hsla::from(mkit.background));
            assert_eq!(kit.foreground, Hsla::from(mkit.text));
            assert_eq!(kit.border, Hsla::from(mkit.border));
            assert_eq!(kit.ring, Hsla::from(mkit.focus));
            assert_eq!(kit.primary, Hsla::from(mkit.accent));
            assert_eq!(kit.primary_foreground, Hsla::from(mkit.accent_text));
            assert_eq!(kit.danger, Hsla::from(mkit.danger));
            // The component tokens GPUI Kit's button actually paints with.
            assert_eq!(kit.tokens.button_primary.color, Hsla::from(mkit.accent));
            assert_eq!(kit.button_primary_foreground, Hsla::from(mkit.accent_text));
            assert_eq!(kit.tokens.button.color, Hsla::from(mkit.elevated_surface));
            assert_eq!(kit.button_foreground, Hsla::from(mkit.text));
        });
    }

    #[gpui_kit::test]
    fn gpui_kit_theme_follows_mkit_light_tokens(cx: &mut TestAppContext) {
        assert_kit_uses_mkit_colours(cx, LIGHT, false);
    }

    #[gpui_kit::test]
    fn gpui_kit_theme_follows_mkit_dark_tokens(cx: &mut TestAppContext) {
        assert_kit_uses_mkit_colours(cx, DARK, true);
    }
}
