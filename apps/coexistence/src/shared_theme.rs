//! Share one palette between mkit and GPUI Kit (plan item E11.3).
//!
//! mkit components read `mkit::core::theme::Theme`; GPUI Kit components read
//! `gpui_kit::component::Theme`. The two globals are independent, so an app
//! that uses both libraries must keep them in step. This module treats mkit's
//! tokens as the source of truth and writes them into the GPUI Kit fields that
//! have a clear counterpart. GPUI Kit fields without an mkit counterpart (for
//! example chart, table, and tab colours) keep GPUI Kit's own values for the
//! matching light or dark mode.
//!
//! mkit has no hover or pressed colour tokens, so GPUI Kit's hover and active
//! states reuse the base token of the same role instead of inventing colours.

use gpui_kit::component::{Theme as KitTheme, ThemeMode, ThemeTokens};
use gpui_pre::{App, Hsla};
use mkit::core::theme::{self, Theme};

/// Install `theme` as mkit's theme and project its colours onto GPUI Kit.
pub fn set_shared_theme(cx: &mut App, theme: Theme) {
    theme::set_theme(cx, theme);
    sync_kit_theme(cx);
}

/// Copy the current mkit theme colours into GPUI Kit's theme global.
///
/// Call this after every mkit theme change. GPUI Kit's mode follows the
/// lightness of mkit's background token so mode-dependent Kit styling (such as
/// its dark-mode input background) agrees with the palette.
pub fn sync_kit_theme(cx: &mut App) {
    let colors = cx.global::<Theme>().colors;
    let background = Hsla::from(colors.background);
    let mode = if background.l < 0.5 { ThemeMode::Dark } else { ThemeMode::Light };
    KitTheme::change(mode, None, cx);

    let text = Hsla::from(colors.text);
    let text_muted = Hsla::from(colors.text_muted);
    let surface = Hsla::from(colors.surface);
    let elevated = Hsla::from(colors.elevated_surface);
    let border = Hsla::from(colors.border);
    let accent = Hsla::from(colors.accent);
    let accent_text = Hsla::from(colors.accent_text);
    let focus = Hsla::from(colors.focus);
    let success = Hsla::from(colors.success);
    let warning = Hsla::from(colors.warning);
    let danger = Hsla::from(colors.danger);

    let kit = KitTheme::global_mut(cx);
    let c = &mut kit.colors;

    // Generic roles.
    c.background = background;
    c.foreground = text;
    c.caret = text;
    c.border = border;
    c.input = border;
    c.ring = focus;
    c.muted = surface;
    c.muted_foreground = text_muted;
    c.popover = elevated;
    c.popover_foreground = text;
    c.primary = accent;
    c.primary_hover = accent;
    c.primary_active = accent;
    c.primary_foreground = accent_text;
    c.secondary = elevated;
    c.secondary_hover = surface;
    c.secondary_active = surface;
    c.secondary_foreground = text;
    c.danger = danger;
    c.danger_hover = danger;
    c.danger_active = danger;
    c.danger_foreground = accent_text;
    c.success = success;
    c.success_hover = success;
    c.success_active = success;
    c.success_foreground = accent_text;
    c.warning = warning;
    c.warning_hover = warning;
    c.warning_active = warning;
    c.warning_foreground = accent_text;
    c.link = accent;
    c.link_hover = accent;
    c.link_active = accent;

    // Buttons: Kit's default button matches mkit's secondary button, and
    // Kit's primary and danger buttons match mkit's default and destructive.
    c.button = elevated;
    c.button_hover = surface;
    c.button_active = surface;
    c.button_foreground = text;
    c.button_primary = accent;
    c.button_primary_hover = accent;
    c.button_primary_active = accent;
    c.button_primary_foreground = accent_text;
    c.button_secondary = elevated;
    c.button_secondary_hover = surface;
    c.button_secondary_active = surface;
    c.button_secondary_foreground = text;
    c.button_danger = danger;
    c.button_danger_hover = danger;
    c.button_danger_active = danger;
    c.button_danger_foreground = accent_text;

    // Window chrome.
    c.title_bar = surface;
    c.title_bar_border = border;
    c.status_bar = surface;
    c.status_bar_border = border;
    c.window_border = border;

    // GPUI Kit paints some components from the resolved legacy tokens, so
    // rebuild them from the updated colours, then refresh GPUI Kit's base layer.
    kit.tokens = ThemeTokens::from(&kit.colors);
    KitTheme::sync_base(cx);
    cx.refresh_windows();
}
