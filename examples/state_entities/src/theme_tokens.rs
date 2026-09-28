//! Reads semantic theme tokens.
//!
//! This module backs the "Using the tokens" and "Theme files" samples in the
//! design-language chapter. Keep the anchored regions small; the chapter
//! includes only them.

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

// ANCHOR: theme_file
use mkit_core::theme::set_theme;
use mkit_core::theme_file::ThemeFileError;

/// Install a theme from JSON theme-file text. On failure nothing changes and
/// the error names the missing or invalid token, such as `colors.focus`.
pub fn install_theme_file(cx: &mut App, json: &str) -> Result<(), ThemeFileError> {
    let theme = Theme::from_json(json)?;
    set_theme(cx, theme);
    Ok(())
}

/// Export the installed theme as a theme file and as CSS custom properties.
pub fn export_installed_theme(cx: &App) -> (String, String) {
    let theme = cx.global::<Theme>();
    (theme.to_json(), theme.to_css_custom_properties(":root"))
}
// ANCHOR_END: theme_file

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::TestAppContext;
    use mkit_core::theme::{DARK, LIGHT};

    #[gpui_pre::test]
    fn theme_file_installs_and_reports_the_failing_token(cx: &mut TestAppContext) {
        cx.set_global(LIGHT);
        cx.update(|app| install_theme_file(app, &DARK.to_json())).unwrap();
        assert_eq!(cx.read(|app| *app.global::<Theme>()), DARK);

        let (json, css) = cx.read(export_installed_theme);
        assert_eq!(Theme::from_json(&json).unwrap(), DARK);
        assert!(css.contains("--mkit-colors-background: #17191f;"));

        let broken = LIGHT.to_json().replace("\"focus\"", "\"focus_ring\"");
        let error = cx.update(|app| install_theme_file(app, &broken)).unwrap_err();
        assert_eq!(error.to_string(), "theme file is missing token `colors.focus`");
        assert_eq!(cx.read(|app| app.global::<Theme>().name), "dark");
    }
}
