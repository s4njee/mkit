//! Reads semantic theme tokens.
//!
//! This module backs the "Using the tokens" sample in the design-language
//! chapter. Keep the anchored region small; the chapter includes only it.

use gpui_pre::{App, IntoElement, div, prelude::*, px};
use mkit_core::theme::Theme;

// ANCHOR: theme_token_read
/// Build a themed panel from semantic tokens instead of literal values.
pub fn themed_panel(cx: &mut App) -> impl IntoElement {
    let theme = *cx.global::<Theme>();
    let colors = theme.colors;
    div()
        .bg(colors.surface)
        .text_color(colors.text)
        .border_1()
        .border_color(colors.border)
        .rounded(px(theme.radii.medium))
        .p(px(theme.spacing.medium))
}
// ANCHOR_END: theme_token_read
