//! WCAG 2.x contrast measurement for theme colours.
//!
//! [`contrast_ratio`] implements the WCAG 2.x relative-luminance formula so theme
//! authors (and the E9.3 theme loader) can check token pairs such as `text` on
//! `background` or `focus` on `surface`. The built-in themes are audited against
//! these targets in this module's tests; see `docs/E9_CONTRAST.md` for the report.

use gpui_pre::Rgba;

/// WCAG 2.x AA minimum for normal-size text (SC 1.4.3).
pub const TEXT_AA: f32 = 4.5;
/// WCAG 2.x AAA minimum for normal-size text (SC 1.4.6). mkit uses it as the
/// target for text pairs in the high-contrast theme.
pub const TEXT_AAA: f32 = 7.0;
/// WCAG 2.x minimum for user-interface components, graphical objects, and focus
/// indicators (SC 1.4.11).
pub const NON_TEXT: f32 = 3.0;

fn linearize(channel: f32) -> f32 {
    let channel = channel.clamp(0.0, 1.0);
    if channel <= 0.04045 { channel / 12.92 } else { ((channel + 0.055) / 1.055).powf(2.4) }
}

/// Relative luminance of an opaque colour, from 0.0 (black) to 1.0 (white).
///
/// Alpha is ignored; composite translucent colours first with [`composite`].
pub fn relative_luminance(color: Rgba) -> f32 {
    0.2126 * linearize(color.r) + 0.7152 * linearize(color.g) + 0.0722 * linearize(color.b)
}

/// Composite `foreground` over an opaque `background` using its alpha.
pub fn composite(foreground: Rgba, background: Rgba) -> Rgba {
    let a = foreground.a.clamp(0.0, 1.0);
    let mix = |f: f32, b: f32| f * a + b * (1.0 - a);
    Rgba {
        r: mix(foreground.r, background.r),
        g: mix(foreground.g, background.g),
        b: mix(foreground.b, background.b),
        a: 1.0,
    }
}

/// WCAG 2.x contrast ratio between a foreground and an opaque background, from
/// 1.0 (identical) to 21.0 (black on white).
///
/// A translucent foreground is composited over the background first. The
/// background's own alpha is ignored.
pub fn contrast_ratio(foreground: Rgba, background: Rgba) -> f32 {
    let background = Rgba { a: 1.0, ..background };
    let foreground = composite(foreground, background);
    let (a, b) = (relative_luminance(foreground), relative_luminance(background));
    let (lighter, darker) = if a >= b { (a, b) } else { (b, a) };
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ColorTokens, DARK, HIGH_CONTRAST, LIGHT, SHADCN_DARK, SHADCN_LIGHT, Theme};

    fn hex(value: u32) -> Rgba {
        Rgba {
            r: ((value >> 16) & 0xff) as f32 / 255.0,
            g: ((value >> 8) & 0xff) as f32 / 255.0,
            b: (value & 0xff) as f32 / 255.0,
            a: 1.0,
        }
    }

    #[test]
    fn contrast_ratio_matches_wcag_reference_values() {
        let (black, white) = (hex(0x000000), hex(0xffffff));
        assert!((contrast_ratio(black, white) - 21.0).abs() < 1e-3);
        assert!((contrast_ratio(white, black) - 21.0).abs() < 1e-3);
        assert!((contrast_ratio(white, white) - 1.0).abs() < 1e-6);
        // #767676 is the lightest grey that passes 4.5:1 on white; #777777 fails.
        assert!(contrast_ratio(hex(0x767676), white) >= TEXT_AA);
        assert!(contrast_ratio(hex(0x777777), white) < TEXT_AA);
        // A fully transparent foreground has no contrast with its background.
        let clear = Rgba { a: 0.0, ..black };
        assert!((contrast_ratio(clear, white) - 1.0).abs() < 1e-6);
        // 50% black over white composites to 50% grey (about 3.98:1).
        let half = Rgba { a: 0.5, ..black };
        assert!((contrast_ratio(half, white) - 3.98).abs() < 0.01);
    }

    /// Which WCAG target a token pair must meet.
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Kind {
        /// Text or icons that carry meaning (4.5:1, or 7:1 in high contrast).
        Text,
        /// Control boundaries, fills, and focus indicators (3:1).
        NonText,
    }

    type Pick = fn(&ColorTokens) -> Rgba;

    /// Meaningful foreground/background pairs, based on how registry components
    /// combine the tokens (for example the destructive button draws
    /// `accent_text` on `danger`, and the link button draws `accent` on
    /// `background`).
    const PAIRS: &[(&str, Pick, &str, Pick, Kind)] = &[
        ("text", |c| c.text, "background", |c| c.background, Kind::Text),
        ("text", |c| c.text, "surface", |c| c.surface, Kind::Text),
        ("text", |c| c.text, "elevated_surface", |c| c.elevated_surface, Kind::Text),
        ("text_muted", |c| c.text_muted, "background", |c| c.background, Kind::Text),
        ("text_muted", |c| c.text_muted, "surface", |c| c.surface, Kind::Text),
        ("text_muted", |c| c.text_muted, "elevated_surface", |c| c.elevated_surface, Kind::Text),
        ("accent_text", |c| c.accent_text, "accent", |c| c.accent, Kind::Text),
        ("accent_text", |c| c.accent_text, "danger", |c| c.danger, Kind::Text),
        ("accent", |c| c.accent, "background", |c| c.background, Kind::Text),
        ("accent", |c| c.accent, "surface", |c| c.surface, Kind::Text),
        ("danger", |c| c.danger, "background", |c| c.background, Kind::Text),
        ("danger", |c| c.danger, "surface", |c| c.surface, Kind::Text),
        ("success", |c| c.success, "background", |c| c.background, Kind::Text),
        ("success", |c| c.success, "surface", |c| c.surface, Kind::Text),
        ("warning", |c| c.warning, "background", |c| c.background, Kind::Text),
        ("warning", |c| c.warning, "surface", |c| c.surface, Kind::Text),
        ("focus", |c| c.focus, "background", |c| c.background, Kind::NonText),
        ("focus", |c| c.focus, "surface", |c| c.surface, Kind::NonText),
        ("focus", |c| c.focus, "elevated_surface", |c| c.elevated_surface, Kind::NonText),
        ("focus", |c| c.focus, "accent", |c| c.accent, Kind::NonText),
        ("border", |c| c.border, "background", |c| c.background, Kind::NonText),
        ("border", |c| c.border, "surface", |c| c.surface, Kind::NonText),
        ("border", |c| c.border, "elevated_surface", |c| c.elevated_surface, Kind::NonText),
    ];

    /// Known failures awaiting maintainer review, with the measured ratio
    /// (two decimals). See `docs/E9_CONTRAST.md` for proposed fixes. A pair
    /// listed here must still measure exactly this value, so a regression or a
    /// fix both force this list (and the report) to be updated.
    const KNOWN_FAILURES: &[(&str, &str, &str, f32)] = &[
        ("light", "focus", "accent", 1.35),
        ("light", "border", "background", 1.29),
        ("light", "border", "surface", 1.38),
        ("light", "border", "elevated_surface", 1.38),
        ("dark", "focus", "accent", 1.32),
        ("dark", "border", "background", 2.01),
        ("dark", "border", "surface", 1.80),
        ("dark", "border", "elevated_surface", 1.57),
        ("high-contrast", "accent_text", "danger", 6.68),
        ("high-contrast", "danger", "background", 6.68),
        ("high-contrast", "danger", "surface", 6.68),
        ("high-contrast", "focus", "accent", 1.17),
        ("shadcn-light", "border", "background", 1.26),
        ("shadcn-light", "border", "surface", 1.26),
        ("shadcn-light", "border", "elevated_surface", 1.21),
        ("shadcn-dark", "focus", "accent", 2.42),
        ("shadcn-dark", "border", "background", 1.91),
        ("shadcn-dark", "border", "surface", 1.73),
        ("shadcn-dark", "border", "elevated_surface", 1.46),
    ];

    fn target(theme: &Theme, kind: Kind) -> f32 {
        match (kind, theme.name) {
            (Kind::Text, "high-contrast") => TEXT_AAA,
            (Kind::Text, _) => TEXT_AA,
            (Kind::NonText, _) => NON_TEXT,
        }
    }

    #[test]
    fn builtin_theme_token_pairs_meet_wcag_targets_or_are_known_failures() {
        let mut problems = Vec::new();
        let mut used = vec![false; KNOWN_FAILURES.len()];
        let mut passes = 0;
        for theme in [LIGHT, DARK, HIGH_CONTRAST, SHADCN_LIGHT, SHADCN_DARK] {
            for &(fg_name, fg, bg_name, bg, kind) in PAIRS {
                let ratio = contrast_ratio(fg(&theme.colors), bg(&theme.colors));
                let rounded = (ratio * 100.0).round() / 100.0;
                let required = target(&theme, kind);
                let known = KNOWN_FAILURES
                    .iter()
                    .position(|&(t, f, b, _)| (t, f, b) == (theme.name, fg_name, bg_name));
                match known {
                    Some(index) => {
                        used[index] = true;
                        let recorded = KNOWN_FAILURES[index].3;
                        if ratio >= required {
                            problems.push(format!(
                                "{}: {fg_name} on {bg_name} now passes ({rounded:.2}:1 >= {required}:1); \
                                 remove it from KNOWN_FAILURES and docs/E9_CONTRAST.md",
                                theme.name
                            ));
                        } else if (rounded - recorded).abs() > 0.005 {
                            problems.push(format!(
                                "{}: {fg_name} on {bg_name} changed from recorded {recorded:.2}:1 to \
                                 {rounded:.2}:1 (target {required}:1)",
                                theme.name
                            ));
                        }
                    }
                    None if ratio < required => problems.push(format!(
                        "{}: {fg_name} on {bg_name} is {rounded:.2}:1, below {required}:1",
                        theme.name
                    )),
                    None => passes += 1,
                }
            }
        }
        for (index, was_used) in used.iter().enumerate() {
            if !was_used {
                let (t, f, b, _) = KNOWN_FAILURES[index];
                problems.push(format!("stale KNOWN_FAILURES entry: {t}: {f} on {b}"));
            }
        }
        assert!(problems.is_empty(), "contrast audit problems:\n{}", problems.join("\n"));
        assert_eq!(passes + KNOWN_FAILURES.len(), 5 * PAIRS.len());
    }
}
