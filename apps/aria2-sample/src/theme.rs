use gpui_pre::{App, Rgba};
use mkit::core::theme::{ColorTokens, SHADCN_LIGHT, Theme, set_theme};

const fn color(hex: u32) -> Rgba {
    Rgba {
        r: ((hex >> 16) & 0xff) as f32 / 255.0,
        g: ((hex >> 8) & 0xff) as f32 / 255.0,
        b: (hex & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

pub const LIGHT: Theme = Theme {
    name: "aria2-light",
    colors: ColorTokens {
        background: color(0xeceeef),
        surface: color(0xffffff),
        elevated_surface: color(0xfafbfb),
        text: color(0x14181b),
        text_muted: color(0x6b7580),
        border: color(0xe3e6e8),
        accent: color(0x007aff),
        accent_text: color(0xffffff),
        focus: color(0x007aff),
        success: color(0x0e9f6e),
        warning: color(0xd97706),
        danger: color(0xdc2626),
        disabled: color(0x9aa5ae),
    },
    typography: mkit::core::theme::TypographyTokens {
        caption: 10.5,
        body: 12.5,
        body_emphasis: 12.5,
        heading_small: 14.0,
        heading: 15.0,
        heading_large: 20.0,
    },
    spacing: mkit::core::theme::SpacingTokens {
        none: 0.0,
        xsmall: 4.0,
        small: 7.0,
        medium: 11.0,
        large: 16.0,
        xlarge: 20.0,
        xxlarge: 24.0,
    },
    radii: mkit::core::theme::RadiusTokens {
        none: 0.0,
        small: 4.0,
        medium: 7.0,
        large: 10.0,
        pill: 999.0,
    },
    controls: mkit::core::theme::ControlTokens {
        xsmall: 24.0,
        small: 30.0,
        medium: 32.0,
        large: 38.0,
    },
    ..SHADCN_LIGHT
};

pub const DARK: Theme = Theme {
    name: "aria2-dark",
    colors: ColorTokens {
        background: color(0x12171a),
        surface: color(0x12171a),
        elevated_surface: color(0x171d21),
        text: color(0xdee5e9),
        text_muted: color(0x9fadb6),
        border: color(0x262f35),
        accent: color(0x0a84ff),
        accent_text: color(0xffffff),
        focus: color(0x6fb4ff),
        success: color(0x3fcf96),
        warning: color(0xe8b04b),
        danger: color(0xf07575),
        disabled: color(0x67757e),
    },
    ..LIGHT
};

pub const PAPER: Theme = Theme {
    name: "aria2-paper",
    colors: ColorTokens {
        background: color(0xfbf9f5),
        surface: color(0xfbf9f5),
        elevated_surface: color(0xf4f0e8),
        text: color(0x2b2621),
        text_muted: color(0x5c5347),
        border: color(0xdfd8cb),
        accent: color(0xc98a21),
        accent_text: color(0xffffff),
        focus: color(0x8a5a11),
        success: color(0x0b6b4b),
        warning: color(0x8a5a11),
        danger: color(0xb21d1d),
        disabled: color(0x9c9280),
    },
    radii: mkit::core::theme::RadiusTokens {
        none: 0.0,
        small: 2.0,
        medium: 3.0,
        large: 3.0,
        pill: 3.0,
    },
    ..LIGHT
};

pub fn install(cx: &mut App, theme: Theme) {
    set_theme(cx, theme);
}

pub fn tint(t: Theme) -> Rgba {
    match t.name {
        "aria2-dark" => color(0x183047),
        "aria2-paper" => color(0xf6e7cb),
        _ => color(0xe5f0ff),
    }
}

pub fn subtle(t: Theme) -> Rgba {
    match t.name {
        "aria2-dark" => color(0x171d21),
        "aria2-paper" => color(0xf4f0e8),
        _ => color(0xf2f4f5),
    }
}

pub fn rule(t: Theme) -> Rgba {
    match t.name {
        "aria2-dark" => color(0x1d2429),
        "aria2-paper" => color(0xebe4d7),
        _ => color(0xf0f2f3),
    }
}

pub fn meta(t: Theme) -> Rgba {
    match t.name {
        "aria2-dark" => color(0x67757e),
        "aria2-paper" => color(0x9c9280),
        _ => color(0x9aa5ae),
    }
}

pub fn track(t: Theme) -> Rgba {
    match t.name {
        "aria2-dark" => color(0x212a2f),
        "aria2-paper" => color(0xdfd8cb),
        _ => color(0xebeef0),
    }
}
