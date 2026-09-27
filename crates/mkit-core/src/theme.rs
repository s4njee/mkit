//! Shared design tokens and built-in themes.
//!
//! Components should read these values from [`Theme`] instead of defining visual
//! constants locally. Theme changes are installed as a GPUI global and schedule
//! every application window for redraw.

use gpui_pre::{App, Global, Rgba};

const fn color(hex: u32) -> Rgba {
    Rgba {
        r: ((hex >> 16) & 0xff) as f32 / 255.0,
        g: ((hex >> 8) & 0xff) as f32 / 255.0,
        b: (hex & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

const fn rgba(hex: u32) -> Rgba {
    Rgba {
        r: ((hex >> 24) & 0xff) as f32 / 255.0,
        g: ((hex >> 16) & 0xff) as f32 / 255.0,
        b: ((hex >> 8) & 0xff) as f32 / 255.0,
        a: (hex & 0xff) as f32 / 255.0,
    }
}

/// Semantic colors used by mkit components.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTokens {
    pub background: Rgba,
    pub surface: Rgba,
    pub elevated_surface: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub border: Rgba,
    pub accent: Rgba,
    pub accent_text: Rgba,
    pub focus: Rgba,
    pub success: Rgba,
    pub warning: Rgba,
    pub danger: Rgba,
    pub disabled: Rgba,
}

/// Typography sizes in GPUI logical pixels. Font families remain application-owned.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypographyTokens {
    pub caption: f32,
    pub body: f32,
    pub body_emphasis: f32,
    pub heading_small: f32,
    pub heading: f32,
    pub heading_large: f32,
}

/// Spacing values in GPUI logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpacingTokens {
    pub none: f32,
    pub xsmall: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub xlarge: f32,
    pub xxlarge: f32,
}

/// Corner radii in GPUI logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadiusTokens {
    pub none: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub pill: f32,
}

/// Border widths in GPUI logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderTokens {
    pub hairline: f32,
    pub regular: f32,
    pub strong: f32,
}

/// Standard interactive control heights in GPUI logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlTokens {
    pub xsmall: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
}

/// Box shadow description. Offset and blur values are logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowToken {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Rgba,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowTokens {
    pub none: ShadowToken,
    pub small: ShadowToken,
    pub medium: ShadowToken,
    pub large: ShadowToken,
}

/// Motion durations in milliseconds. Zero disables transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MotionTokens {
    pub instant_ms: u16,
    pub fast_ms: u16,
    pub normal_ms: u16,
    pub slow_ms: u16,
}

/// Complete semantic theme, suitable for storage as a GPUI global.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub colors: ColorTokens,
    pub typography: TypographyTokens,
    pub spacing: SpacingTokens,
    pub radii: RadiusTokens,
    pub borders: BorderTokens,
    pub controls: ControlTokens,
    pub shadows: ShadowTokens,
    pub motion: MotionTokens,
}

impl Global for Theme {}

const TYPE: TypographyTokens = TypographyTokens {
    caption: 12.0,
    body: 14.0,
    body_emphasis: 14.0,
    heading_small: 16.0,
    heading: 20.0,
    heading_large: 28.0,
};
const SPACE: SpacingTokens = SpacingTokens {
    none: 0.0,
    xsmall: 4.0,
    small: 8.0,
    medium: 12.0,
    large: 16.0,
    xlarge: 24.0,
    xxlarge: 32.0,
};
const RADII: RadiusTokens =
    RadiusTokens { none: 0.0, small: 4.0, medium: 8.0, large: 12.0, pill: 999.0 };
const BORDERS: BorderTokens = BorderTokens { hairline: 1.0, regular: 1.0, strong: 2.0 };
const CONTROLS: ControlTokens =
    ControlTokens { xsmall: 28.0, small: 32.0, medium: 36.0, large: 40.0 };
const MOTION: MotionTokens =
    MotionTokens { instant_ms: 0, fast_ms: 100, normal_ms: 180, slow_ms: 280 };

const fn shadow(y: f32, blur: f32, shadow_color: Rgba) -> ShadowToken {
    ShadowToken { x: 0.0, y, blur, spread: 0.0, color: shadow_color }
}

/// Standard light palette.
pub const LIGHT: Theme = Theme {
    name: "light",
    colors: ColorTokens {
        background: color(0xf7f8fa),
        surface: color(0xffffff),
        elevated_surface: color(0xffffff),
        text: color(0x17191f),
        text_muted: color(0x5b6270),
        border: color(0xd8dce3),
        accent: color(0x315eea),
        accent_text: color(0xffffff),
        focus: color(0x1749d1),
        success: color(0x16834a),
        warning: color(0x9a5b00),
        danger: color(0xc33030),
        disabled: color(0x858b96),
    },
    typography: TYPE,
    spacing: SPACE,
    radii: RADII,
    borders: BORDERS,
    controls: CONTROLS,
    shadows: ShadowTokens {
        none: shadow(0.0, 0.0, rgba(0x00000000)),
        small: shadow(1.0, 3.0, rgba(0x00000018)),
        medium: shadow(3.0, 8.0, rgba(0x00000020)),
        large: shadow(6.0, 18.0, rgba(0x00000028)),
    },
    motion: MOTION,
};

/// Standard dark palette.
pub const DARK: Theme = Theme {
    name: "dark",
    colors: ColorTokens {
        background: color(0x17191f),
        surface: color(0x20232b),
        elevated_surface: color(0x292d37),
        text: color(0xf1f3f7),
        text_muted: color(0xb0b6c2),
        border: color(0x454b58),
        accent: color(0x7897ff),
        accent_text: color(0x101a3b),
        focus: color(0x9bb1ff),
        success: color(0x65d696),
        warning: color(0xffc66d),
        danger: color(0xff8585),
        disabled: color(0x858b96),
    },
    typography: TYPE,
    spacing: SPACE,
    radii: RADII,
    borders: BORDERS,
    controls: CONTROLS,
    shadows: ShadowTokens {
        none: shadow(0.0, 0.0, rgba(0x00000000)),
        small: shadow(1.0, 3.0, rgba(0x00000040)),
        medium: shadow(3.0, 8.0, rgba(0x00000060)),
        large: shadow(6.0, 18.0, rgba(0x00000080)),
    },
    motion: MOTION,
};

/// High-contrast palette with stronger text, borders, and focus indication.
pub const HIGH_CONTRAST: Theme = Theme {
    name: "high-contrast",
    colors: ColorTokens {
        background: color(0x000000),
        surface: color(0x000000),
        elevated_surface: color(0x000000),
        text: color(0xffffff),
        text_muted: color(0xf0f0f0),
        border: color(0xffffff),
        accent: color(0xffff00),
        accent_text: color(0x000000),
        focus: color(0x00ffff),
        success: color(0x00ff66),
        warning: color(0xffff00),
        danger: color(0xff5555),
        disabled: color(0xc0c0c0),
    },
    typography: TYPE,
    spacing: SPACE,
    radii: RADII,
    borders: BorderTokens { hairline: 2.0, regular: 2.0, strong: 3.0 },
    controls: CONTROLS,
    shadows: ShadowTokens {
        none: shadow(0.0, 0.0, rgba(0x00000000)),
        small: shadow(0.0, 0.0, rgba(0x00000000)),
        medium: shadow(0.0, 0.0, rgba(0x00000000)),
        large: shadow(0.0, 0.0, rgba(0x00000000)),
    },
    motion: MotionTokens { instant_ms: 0, fast_ms: 0, normal_ms: 0, slow_ms: 0 },
};

const SHADCN_RADII: RadiusTokens =
    RadiusTokens { none: 0.0, small: 4.0, medium: 6.0, large: 8.0, pill: 999.0 };

/// Neutral, compact palette inspired by shadcn/ui's semantic token system.
pub const SHADCN_LIGHT: Theme = Theme {
    name: "shadcn-light",
    colors: ColorTokens {
        background: color(0xffffff),
        surface: color(0xffffff),
        elevated_surface: color(0xfafafa),
        text: color(0x0a0a0a),
        text_muted: color(0x737373),
        border: color(0xe5e5e5),
        accent: color(0x171717),
        accent_text: color(0xfafafa),
        focus: color(0x737373),
        success: color(0x15803d),
        warning: color(0xa16207),
        danger: color(0xdc2626),
        disabled: color(0xa3a3a3),
    },
    typography: TYPE,
    spacing: SPACE,
    radii: SHADCN_RADII,
    borders: BORDERS,
    controls: CONTROLS,
    shadows: ShadowTokens {
        none: shadow(0.0, 0.0, rgba(0x00000000)),
        small: shadow(1.0, 2.0, rgba(0x00000012)),
        medium: shadow(2.0, 6.0, rgba(0x0000001a)),
        large: shadow(4.0, 12.0, rgba(0x00000024)),
    },
    motion: MOTION,
};

/// Dark companion to [`SHADCN_LIGHT`].
pub const SHADCN_DARK: Theme = Theme {
    name: "shadcn-dark",
    colors: ColorTokens {
        background: color(0x0a0a0a),
        surface: color(0x171717),
        elevated_surface: color(0x262626),
        text: color(0xfafafa),
        text_muted: color(0xa3a3a3),
        border: color(0x404040),
        accent: color(0xfafafa),
        accent_text: color(0x171717),
        focus: color(0xa3a3a3),
        success: color(0x4ade80),
        warning: color(0xfacc15),
        danger: color(0xf87171),
        disabled: color(0x737373),
    },
    typography: TYPE,
    spacing: SPACE,
    radii: SHADCN_RADII,
    borders: BORDERS,
    controls: CONTROLS,
    shadows: ShadowTokens {
        none: shadow(0.0, 0.0, rgba(0x00000000)),
        small: shadow(1.0, 2.0, rgba(0x00000040)),
        medium: shadow(2.0, 6.0, rgba(0x00000060)),
        large: shadow(4.0, 12.0, rgba(0x00000080)),
    },
    motion: MOTION,
};

/// Install a theme and redraw all application windows.
pub fn set_theme(cx: &mut App, theme: Theme) {
    cx.set_global(theme);
    cx.refresh_windows();
}

/// Install the standard light theme and redraw all application windows.
pub fn set_light_theme(cx: &mut App) {
    set_theme(cx, LIGHT);
}

/// Install the standard dark theme and redraw all application windows.
pub fn set_dark_theme(cx: &mut App) {
    set_theme(cx, DARK);
}

/// Install the neutral shadcn-inspired light theme.
pub fn set_shadcn_light_theme(cx: &mut App) {
    set_theme(cx, SHADCN_LIGHT);
}

/// Install the neutral shadcn-inspired dark theme.
pub fn set_shadcn_dark_theme(cx: &mut App) {
    set_theme(cx, SHADCN_DARK);
}

/// Install the high-contrast theme and redraw all application windows.
pub fn set_high_contrast_theme(cx: &mut App) {
    set_theme(cx, HIGH_CONTRAST);
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre as gpui;
    use gpui_pre::{Context, IntoElement, Render, TestAppContext, Window, div};
    use std::{cell::RefCell, rc::Rc};

    struct ThemeReader(Rc<RefCell<Vec<&'static str>>>);

    impl Render for ThemeReader {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.0.borrow_mut().push(cx.global::<Theme>().name);
            div()
        }
    }

    #[test]
    fn builtins_have_complete_distinct_color_palettes() {
        assert_ne!(LIGHT.colors.background, DARK.colors.background);
        assert_ne!(DARK.colors.background, HIGH_CONTRAST.colors.background);
        let shadow_color = LIGHT.shadows.small.color;
        assert_eq!((shadow_color.r, shadow_color.g, shadow_color.b), (0.0, 0.0, 0.0));
        assert!(shadow_color.a > 0.0 && shadow_color.a < 1.0);
        for theme in [LIGHT, DARK, HIGH_CONTRAST, SHADCN_LIGHT, SHADCN_DARK] {
            assert_eq!(theme.shadows.none.color, rgba(0x00000000));
            assert!(theme.controls.xsmall < theme.controls.small);
            assert!(theme.controls.small < theme.controls.medium);
            assert!(theme.controls.medium < theme.controls.large);
        }
        assert_eq!(HIGH_CONTRAST.borders.strong, 3.0);
        assert_eq!(HIGH_CONTRAST.motion.normal_ms, 0);
        for theme in [LIGHT, DARK, HIGH_CONTRAST, SHADCN_LIGHT, SHADCN_DARK] {
            assert!(theme.typography.body > 0.0);
            assert!(theme.spacing.medium > 0.0);
            assert!(theme.colors.focus.a > 0.0);
        }
        assert_eq!(SHADCN_LIGHT.colors.accent, color(0x171717));
        assert_eq!(SHADCN_DARK.colors.accent, color(0xfafafa));
        assert_eq!(SHADCN_LIGHT.radii.medium, 6.0);
    }

    #[gpui_pre::test]
    fn theme_is_a_global_and_can_be_switched_at_runtime(cx: &mut TestAppContext) {
        cx.set_global(LIGHT);
        assert_eq!(cx.read(|app| app.global::<Theme>().name), "light");
        cx.update(set_dark_theme);
        assert_eq!(cx.read(|app| app.global::<Theme>().name), "dark");
        cx.update(set_high_contrast_theme);
        assert_eq!(cx.read(|app| app.global::<Theme>().name), "high-contrast");
    }

    #[gpui_pre::test]
    fn theme_switch_redraws_readers_in_each_open_window(cx: &mut TestAppContext) {
        let first = Rc::new(RefCell::new(Vec::new()));
        let second = Rc::new(RefCell::new(Vec::new()));
        cx.set_global(LIGHT);
        cx.add_window({
            let first = first.clone();
            move |_, _| ThemeReader(first)
        });
        cx.add_window({
            let second = second.clone();
            move |_, _| ThemeReader(second)
        });
        cx.run_until_parked();

        assert_eq!(first.borrow().last(), Some(&"light"));
        assert_eq!(second.borrow().last(), Some(&"light"));
        let first_render_count = first.borrow().len();
        let second_render_count = second.borrow().len();

        cx.update(set_dark_theme);
        cx.run_until_parked();

        assert!(first.borrow().len() > first_render_count);
        assert!(second.borrow().len() > second_render_count);
        assert_eq!(first.borrow().last(), Some(&"dark"));
        assert_eq!(second.borrow().last(), Some(&"dark"));
    }
}
