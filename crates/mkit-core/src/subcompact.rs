//! Subcompact: a high-density design system built on the mkit token set.
//!
//! Subcompact keeps every semantic role of [`Theme`] and changes the values:
//! controls are 18 to 26 logical pixels tall, the type scale starts at 10 px,
//! corners are square, and surfaces are flat. It is meant for keyboard-driven,
//! terminal-style tools (log viewers, consoles, dashboards, inspectors) that put
//! as much on screen as possible.
//!
//! The design system has two independent parts:
//!
//! - **Density.** [`TYPE`], [`SPACE`], [`RADII`], and [`CONTROLS`] replace the
//!   size scales. [`with_subcompact_density`] applies them to any theme, so
//!   `with_subcompact_density(SHADCN_DARK)` is a valid subcompact theme that
//!   keeps the shadcn colours.
//! - **Palettes.** [`PHOSPHOR`] (green on black, the default), [`AMBER`]
//!   (amber on black), and [`PAPER`] (ink on paper, the light variant) are
//!   complete themes with flat shadows and short motion. Every audited colour
//!   pair in them meets its WCAG target; see `docs/E9_CONTRAST.md`.
//!
//! Font family and line height are not theme tokens, because mkit leaves the
//! font to the application. Subcompact needs a monospace face and a tighter
//! line height to fit one line of text inside an 18 px control, so call
//! [`apply_text_style`] on the window's root element.

use crate::theme::{
    BorderTokens, ColorTokens, ControlTokens, MotionTokens, RadiusTokens, ShadowTokens,
    SpacingTokens, Theme, TypographyTokens, color, rgba, set_theme, shadow,
};
use gpui_pre::{App, Styled, px, relative};

/// Subcompact type scale. Body text is 11 px; captions are 10 px.
pub const TYPE: TypographyTokens = TypographyTokens {
    caption: 10.0,
    body: 11.0,
    body_emphasis: 11.0,
    heading_small: 12.0,
    heading: 14.0,
    heading_large: 18.0,
};

/// Subcompact spacing: the default scale shifted down one step, with a new
/// 2 px bottom step.
///
/// Components use spacing tokens for glyph sizes as well as gaps (an icon or a
/// checkbox is `spacing.large`), so shifting the scale keeps glyphs at a
/// legible 12 px. Halving it would shrink them to 8 px.
pub const SPACE: SpacingTokens = SpacingTokens {
    none: 0.0,
    xsmall: 2.0,
    small: 4.0,
    medium: 8.0,
    large: 12.0,
    xlarge: 16.0,
    xxlarge: 24.0,
};

// Icons, checkboxes, radio circles, and slider thumbs use `spacing.large`; check
// marks use `spacing.medium`; radio dots use `spacing.small`. Keep them legible.
const _: () = assert!(SPACE.large >= 12.0 && SPACE.medium >= 8.0 && SPACE.small >= 4.0);

/// Square corners. `pill` stays fully round, because components use it for
/// shapes that carry meaning, such as a radio button or a switch track.
pub const RADII: RadiusTokens =
    RadiusTokens { none: 0.0, small: 0.0, medium: 0.0, large: 0.0, pill: 999.0 };

/// Subcompact control heights. `medium`, the default, is 22 px.
pub const CONTROLS: ControlTokens =
    ControlTokens { xsmall: 18.0, small: 20.0, medium: 22.0, large: 26.0 };

/// Hairline borders. Subcompact separates regions with lines, not shadows.
pub const BORDERS: BorderTokens = BorderTokens { hairline: 1.0, regular: 1.0, strong: 2.0 };

/// Short motion: state changes land almost immediately.
pub const MOTION: MotionTokens =
    MotionTokens { instant_ms: 0, fast_ms: 50, normal_ms: 90, slow_ms: 150 };

/// No drop shadows. Overlays rely on `elevated_surface` and a border instead.
pub const SHADOWS: ShadowTokens = ShadowTokens {
    none: shadow(0.0, 0.0, rgba(0x00000000)),
    small: shadow(0.0, 0.0, rgba(0x00000000)),
    medium: shadow(0.0, 0.0, rgba(0x00000000)),
    large: shadow(0.0, 0.0, rgba(0x00000000)),
};

/// Line height, as a multiple of the font size, that [`apply_text_style`]
/// installs. At the 11 px body size a line is 14.3 px, which fits inside the
/// 16 px content box of an 18 px control with 1 px borders.
pub const LINE_HEIGHT: f32 = 1.3;

/// The monospace family [`apply_text_style`] installs: one that ships with the
/// operating system, so no font has to be bundled.
#[cfg(target_os = "macos")]
pub const FONT_FAMILY: &str = "Menlo";
/// The monospace family [`apply_text_style`] installs: one that ships with the
/// operating system, so no font has to be bundled.
#[cfg(target_os = "windows")]
pub const FONT_FAMILY: &str = "Consolas";
/// The monospace family [`apply_text_style`] installs: one that ships with the
/// operating system, so no font has to be bundled.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub const FONT_FAMILY: &str = "DejaVu Sans Mono";

/// Green phosphor on black. The default subcompact palette.
pub const PHOSPHOR: Theme = Theme {
    name: "subcompact-phosphor",
    colors: ColorTokens {
        background: color(0x050805),
        surface: color(0x0a110b),
        elevated_surface: color(0x101a12),
        text: color(0xc4f5cc),
        text_muted: color(0x7cb487),
        border: color(0x3f7049),
        accent: color(0x3dff7f),
        accent_text: color(0x02140a),
        focus: color(0xc85a00),
        success: color(0x4ade80),
        warning: color(0xffc23d),
        danger: color(0xff5f56),
        disabled: color(0x52705a),
    },
    typography: TYPE,
    spacing: SPACE,
    radii: RADII,
    borders: BORDERS,
    controls: CONTROLS,
    shadows: SHADOWS,
    motion: MOTION,
};

/// Amber monochrome on black, with a blue focus cue.
pub const AMBER: Theme = Theme {
    name: "subcompact-amber",
    colors: ColorTokens {
        background: color(0x0a0700),
        surface: color(0x120d03),
        elevated_surface: color(0x1a1407),
        text: color(0xffd58a),
        text_muted: color(0xc0904a),
        border: color(0x86652b),
        accent: color(0xffb000),
        accent_text: color(0x1a0f00),
        focus: color(0x2266b8),
        success: color(0x9be15d),
        warning: color(0xfff06a),
        danger: color(0xff6a50),
        disabled: color(0x6e5a3a),
    },
    typography: TYPE,
    spacing: SPACE,
    radii: RADII,
    borders: BORDERS,
    controls: CONTROLS,
    shadows: SHADOWS,
    motion: MOTION,
};

/// Dark ink on paper. The light subcompact palette.
pub const PAPER: Theme = Theme {
    name: "subcompact-paper",
    colors: ColorTokens {
        background: color(0xf3f0e6),
        surface: color(0xfaf8f1),
        elevated_surface: color(0xffffff),
        text: color(0x0e130f),
        text_muted: color(0x4a554c),
        border: color(0x7f8a80),
        accent: color(0x0a3018),
        accent_text: color(0xf3f0e6),
        focus: color(0xc85a00),
        success: color(0x146c32),
        warning: color(0x8a5300),
        danger: color(0xb3261e),
        disabled: color(0x7f8a80),
    },
    typography: TYPE,
    spacing: SPACE,
    radii: RADII,
    borders: BORDERS,
    controls: CONTROLS,
    shadows: SHADOWS,
    motion: MOTION,
};

/// The three subcompact palettes, dark-first.
pub const PALETTES: [Theme; 3] = [PHOSPHOR, AMBER, PAPER];

/// Return `theme` with the subcompact type, spacing, radius, and control
/// scales. Colours, borders, shadows, motion, and the name are kept, so a
/// component that checks the theme name (such as the high-contrast look)
/// still recognises it.
pub const fn with_subcompact_density(theme: Theme) -> Theme {
    Theme { typography: TYPE, spacing: SPACE, radii: RADII, controls: CONTROLS, ..theme }
}

/// Set the subcompact font family, line height, body size, and text colour on
/// an element. Children inherit them, so apply it once to the window's root.
pub fn apply_text_style<E: Styled>(element: E, theme: &Theme) -> E {
    element
        .font_family(FONT_FAMILY)
        .line_height(relative(LINE_HEIGHT))
        .text_size(px(theme.typography.body))
        .text_color(theme.colors.text)
}

/// Install the phosphor palette and redraw all application windows.
pub fn set_phosphor_theme(cx: &mut App) {
    set_theme(cx, PHOSPHOR);
}

/// Install the amber palette and redraw all application windows.
pub fn set_amber_theme(cx: &mut App) {
    set_theme(cx, AMBER);
}

/// Install the paper palette and redraw all application windows.
pub fn set_paper_theme(cx: &mut App) {
    set_theme(cx, PAPER);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{DARK, HIGH_CONTRAST, SHADCN_DARK};
    use gpui_pre as gpui;
    use gpui_pre::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*};

    #[test]
    fn density_is_the_default_scale_shifted_down() {
        let (sub, default) = (PHOSPHOR, DARK);
        assert!(sub.controls.xsmall < sub.controls.small);
        assert!(sub.controls.small < sub.controls.medium);
        assert!(sub.controls.medium < sub.controls.large);
        assert!(sub.controls.large < default.controls.xsmall);
        assert!(sub.typography.body < default.typography.caption);

        let (s, d) = (sub.spacing, default.spacing);
        assert!(s.xsmall < d.xsmall);
        assert_eq!(
            [s.small, s.medium, s.large, s.xlarge, s.xxlarge],
            [d.xsmall, d.small, d.medium, d.large, d.xlarge]
        );
    }

    #[test]
    fn one_line_of_body_text_fits_inside_the_smallest_control() {
        for theme in PALETTES {
            let line = theme.typography.body * LINE_HEIGHT;
            let content = theme.controls.xsmall - 2.0 * theme.borders.regular;
            assert!(line <= content, "{}: {line} px line in a {content} px box", theme.name);
            // A heading-sized line still fits the large control.
            let heading = theme.typography.heading * LINE_HEIGHT;
            assert!(heading <= theme.controls.large - 2.0 * theme.borders.regular);
        }
    }

    #[test]
    fn palettes_are_flat_square_and_distinct() {
        for theme in PALETTES {
            assert!(theme.name.starts_with("subcompact-"));
            for token in [theme.shadows.small, theme.shadows.medium, theme.shadows.large] {
                assert_eq!(token.color.a, 0.0, "{}: shadows must be transparent", theme.name);
            }
            assert_eq!(theme.radii.medium, 0.0);
            assert_eq!(theme.radii.pill, 999.0);
            assert_ne!(theme.colors.focus, theme.colors.accent);
        }
        assert_ne!(PHOSPHOR.colors, AMBER.colors);
        assert_ne!(AMBER.colors, PAPER.colors);
    }

    #[test]
    fn density_applies_to_any_palette_and_keeps_its_identity() {
        let dense = with_subcompact_density(SHADCN_DARK);
        assert_eq!(dense.name, SHADCN_DARK.name);
        assert_eq!(dense.colors, SHADCN_DARK.colors);
        assert_eq!(dense.shadows, SHADCN_DARK.shadows);
        assert_eq!(dense.controls, CONTROLS);
        assert_eq!(dense.typography, TYPE);

        // High contrast keeps its wider borders and its name-based look.
        let dense = with_subcompact_density(HIGH_CONTRAST);
        assert_eq!(dense.name, "high-contrast");
        assert_eq!(dense.borders, HIGH_CONTRAST.borders);
        assert_eq!(dense.controls, CONTROLS);
    }

    struct Root;

    impl Render for Root {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let theme = *cx.global::<Theme>();
            apply_text_style(div(), &theme).child("ready")
        }
    }

    #[gpui_pre::test]
    fn helpers_install_each_palette(cx: &mut TestAppContext) {
        cx.update(set_phosphor_theme);
        assert_eq!(cx.read(|app| app.global::<Theme>().name), "subcompact-phosphor");
        cx.update(set_amber_theme);
        assert_eq!(cx.read(|app| app.global::<Theme>().name), "subcompact-amber");
        cx.update(set_paper_theme);
        assert_eq!(cx.read(|app| *app.global::<Theme>()), PAPER);

        // The root text style renders with the platform monospace family.
        cx.add_window(|_, _| Root);
        cx.run_until_parked();
    }
}
