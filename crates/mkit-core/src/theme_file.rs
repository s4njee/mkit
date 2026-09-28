//! Theme files (E9.3): load a [`Theme`] from JSON and export one to JSON or CSS.
//!
//! A theme file is a JSON object with the same groups and token names as
//! [`Theme`]. Every token is required, and unknown tokens are rejected, so a
//! typo is reported instead of silently falling back to a default:
//!
//! ```json
//! {
//!   "name": "my-theme",
//!   "colors": { "background": "#17191f", "text": "#f1f3f7", "...": "..." },
//!   "typography": { "caption": 12, "body": 14, "...": 0 },
//!   "shadows": { "small": { "x": 0, "y": 1, "blur": 3, "spread": 0, "color": "#00000040" } },
//!   "motion": { "instant_ms": 0, "fast_ms": 100, "normal_ms": 180, "slow_ms": 280 }
//! }
//! ```
//!
//! Colours are `#rrggbb` or `#rrggbbaa` hex strings. Sizes are finite,
//! non-negative logical pixels; shadow offsets and spread may be negative.
//! Motion durations are whole milliseconds. Colours are stored with 8 bits per
//! channel, so a [`Theme`] built from arbitrary floats is quantized on export.
//! Use [`Theme::to_json`] on a built-in theme to get a complete starting file.

use crate::theme::{
    BorderTokens, ColorTokens, ControlTokens, DARK, HIGH_CONTRAST, LIGHT, MotionTokens,
    RadiusTokens, SHADCN_DARK, SHADCN_LIGHT, ShadowToken, ShadowTokens, SpacingTokens, Theme,
    TypographyTokens,
};
use gpui_pre::Rgba;
use serde_json::{Map, Value};
use std::{fmt, sync::Mutex};

/// Why a theme file could not be loaded. Every variant except
/// [`ThemeFileError::Syntax`] names the token by its dotted path, such as
/// `colors.focus` or `shadows.small.blur`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeFileError {
    /// The input is not valid JSON.
    Syntax(String),
    /// A required token (or group) is absent.
    Missing { token: String },
    /// A token is present but its value has the wrong type or range.
    Invalid { token: String, expected: &'static str, found: String },
    /// The file contains a token mkit does not define.
    Unknown { token: String },
}

impl fmt::Display for ThemeFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(message) => write!(f, "theme file is not valid JSON: {message}"),
            Self::Missing { token } => write!(f, "theme file is missing token `{token}`"),
            Self::Invalid { token, expected, found } => {
                write!(f, "theme token `{token}` must be {expected}, found {found}")
            }
            Self::Unknown { token } => write!(f, "theme file has unknown token `{token}`"),
        }
    }
}

impl std::error::Error for ThemeFileError {}

type Result<T> = std::result::Result<T, ThemeFileError>;

/// Reads one JSON object group, tracking which keys were consumed so unknown
/// keys can be reported.
struct Group<'a> {
    path: String,
    map: &'a Map<String, Value>,
    seen: Vec<&'static str>,
}

impl<'a> Group<'a> {
    fn new(path: String, value: &'a Value) -> Result<Self> {
        match value {
            Value::Object(map) => Ok(Self { path, map, seen: Vec::new() }),
            other if path.is_empty() => Err(invalid("(root)".to_owned(), "an object", other)),
            other => Err(invalid(path, "an object", other)),
        }
    }

    fn token_path(&self, key: &str) -> String {
        if self.path.is_empty() { key.to_owned() } else { format!("{}.{key}", self.path) }
    }

    fn get(&mut self, key: &'static str) -> Result<(&'a Value, String)> {
        self.seen.push(key);
        let path = self.token_path(key);
        match self.map.get(key) {
            Some(value) => Ok((value, path)),
            None => Err(ThemeFileError::Missing { token: path }),
        }
    }

    fn group(&mut self, key: &'static str) -> Result<Group<'a>> {
        let (value, path) = self.get(key)?;
        Group::new(path, value)
    }

    fn color(&mut self, key: &'static str) -> Result<Rgba> {
        let (value, path) = self.get(key)?;
        parse_color(value).ok_or_else(|| invalid(path, "a #rrggbb or #rrggbbaa colour", value))
    }

    fn size(&mut self, key: &'static str) -> Result<f32> {
        let (value, path) = self.get(key)?;
        match value.as_f64() {
            Some(n) if n.is_finite() && n >= 0.0 && n <= f32::MAX as f64 => Ok(n as f32),
            _ => Err(invalid(path, "a finite, non-negative number", value)),
        }
    }

    fn offset(&mut self, key: &'static str) -> Result<f32> {
        let (value, path) = self.get(key)?;
        match value.as_f64() {
            Some(n) if n.is_finite() && n.abs() <= f32::MAX as f64 => Ok(n as f32),
            _ => Err(invalid(path, "a finite number", value)),
        }
    }

    fn millis(&mut self, key: &'static str) -> Result<u16> {
        let (value, path) = self.get(key)?;
        value
            .as_u64()
            .and_then(|n| u16::try_from(n).ok())
            .ok_or_else(|| invalid(path, "whole milliseconds from 0 to 65535", value))
    }

    fn string(&mut self, key: &'static str) -> Result<&'a str> {
        let (value, path) = self.get(key)?;
        match value.as_str() {
            Some(text) if !text.trim().is_empty() => Ok(text),
            _ => Err(invalid(path, "a non-empty string", value)),
        }
    }

    /// Reject any key that was not read.
    fn finish(self) -> Result<()> {
        let mut unknown: Vec<&String> =
            self.map.keys().filter(|key| !self.seen.contains(&key.as_str())).collect();
        unknown.sort();
        match unknown.first() {
            Some(key) => Err(ThemeFileError::Unknown { token: self.token_path(key) }),
            None => Ok(()),
        }
    }
}

fn invalid(token: String, expected: &'static str, found: &Value) -> ThemeFileError {
    let mut found = found.to_string();
    if found.len() > 40 {
        found.truncate(found.char_indices().nth(37).map_or(found.len(), |(i, _)| i));
        found.push_str("...");
    }
    ThemeFileError::Invalid { token, expected, found }
}

fn parse_color(value: &Value) -> Option<Rgba> {
    let digits = value.as_str()?.strip_prefix('#')?;
    if !(digits.len() == 6 || digits.len() == 8) || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let channel =
        |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok().map(|v| v as f32 / 255.0);
    Some(Rgba {
        r: channel(0)?,
        g: channel(2)?,
        b: channel(4)?,
        a: if digits.len() == 8 { channel(6)? } else { 1.0 },
    })
}

fn color_hex(color: Rgba) -> String {
    let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    let (r, g, b, a) = (byte(color.r), byte(color.g), byte(color.b), byte(color.a));
    if a == 255 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

/// Theme names are `&'static str` so `Theme` stays `Copy`. Loaded names reuse a
/// built-in name when they match one; otherwise each distinct name is leaked
/// once and reused on later loads, so reloading a file does not grow memory.
fn intern_name(name: &str) -> &'static str {
    static NAMES: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
    for builtin in [LIGHT, DARK, HIGH_CONTRAST, SHADCN_LIGHT, SHADCN_DARK] {
        if builtin.name == name {
            return builtin.name;
        }
    }
    let mut names = NAMES.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(existing) = names.iter().find(|existing| **existing == name) {
        return existing;
    }
    let leaked: &'static str = Box::leak(name.to_owned().into_boxed_str());
    names.push(leaked);
    leaked
}

fn read_theme(root: &Value) -> Result<Theme> {
    let mut root = Group::new(String::new(), root)?;
    let name = intern_name(root.string("name")?);

    let mut c = root.group("colors")?;
    let colors = ColorTokens {
        background: c.color("background")?,
        surface: c.color("surface")?,
        elevated_surface: c.color("elevated_surface")?,
        text: c.color("text")?,
        text_muted: c.color("text_muted")?,
        border: c.color("border")?,
        accent: c.color("accent")?,
        accent_text: c.color("accent_text")?,
        focus: c.color("focus")?,
        success: c.color("success")?,
        warning: c.color("warning")?,
        danger: c.color("danger")?,
        disabled: c.color("disabled")?,
    };
    c.finish()?;

    let mut t = root.group("typography")?;
    let typography = TypographyTokens {
        caption: t.size("caption")?,
        body: t.size("body")?,
        body_emphasis: t.size("body_emphasis")?,
        heading_small: t.size("heading_small")?,
        heading: t.size("heading")?,
        heading_large: t.size("heading_large")?,
    };
    t.finish()?;

    let mut s = root.group("spacing")?;
    let spacing = SpacingTokens {
        none: s.size("none")?,
        xsmall: s.size("xsmall")?,
        small: s.size("small")?,
        medium: s.size("medium")?,
        large: s.size("large")?,
        xlarge: s.size("xlarge")?,
        xxlarge: s.size("xxlarge")?,
    };
    s.finish()?;

    let mut r = root.group("radii")?;
    let radii = RadiusTokens {
        none: r.size("none")?,
        small: r.size("small")?,
        medium: r.size("medium")?,
        large: r.size("large")?,
        pill: r.size("pill")?,
    };
    r.finish()?;

    let mut b = root.group("borders")?;
    let borders = BorderTokens {
        hairline: b.size("hairline")?,
        regular: b.size("regular")?,
        strong: b.size("strong")?,
    };
    b.finish()?;

    let mut k = root.group("controls")?;
    let controls = ControlTokens {
        xsmall: k.size("xsmall")?,
        small: k.size("small")?,
        medium: k.size("medium")?,
        large: k.size("large")?,
    };
    k.finish()?;

    let mut sh = root.group("shadows")?;
    let mut shadow = |key: &'static str| -> Result<ShadowToken> {
        let mut g = sh.group(key)?;
        let token = ShadowToken {
            x: g.offset("x")?,
            y: g.offset("y")?,
            blur: g.size("blur")?,
            spread: g.offset("spread")?,
            color: g.color("color")?,
        };
        g.finish()?;
        Ok(token)
    };
    let shadows = ShadowTokens {
        none: shadow("none")?,
        small: shadow("small")?,
        medium: shadow("medium")?,
        large: shadow("large")?,
    };
    sh.finish()?;

    let mut m = root.group("motion")?;
    let motion = MotionTokens {
        instant_ms: m.millis("instant_ms")?,
        fast_ms: m.millis("fast_ms")?,
        normal_ms: m.millis("normal_ms")?,
        slow_ms: m.millis("slow_ms")?,
    };
    m.finish()?;
    root.finish()?;

    Ok(Theme { name, colors, typography, spacing, radii, borders, controls, shadows, motion })
}

/// A token value in export order, shared by the JSON and CSS writers.
enum Token {
    Color(Rgba),
    Size(f32),
    Shadow(ShadowToken),
    Millis(u16),
}

fn groups(theme: &Theme) -> Vec<(&'static str, Vec<(&'static str, Token)>)> {
    use Token::{Color, Millis, Shadow, Size};
    let c = &theme.colors;
    let t = &theme.typography;
    let s = &theme.spacing;
    let r = &theme.radii;
    let b = &theme.borders;
    let k = &theme.controls;
    let sh = &theme.shadows;
    let m = &theme.motion;
    vec![
        (
            "colors",
            vec![
                ("background", Color(c.background)),
                ("surface", Color(c.surface)),
                ("elevated_surface", Color(c.elevated_surface)),
                ("text", Color(c.text)),
                ("text_muted", Color(c.text_muted)),
                ("border", Color(c.border)),
                ("accent", Color(c.accent)),
                ("accent_text", Color(c.accent_text)),
                ("focus", Color(c.focus)),
                ("success", Color(c.success)),
                ("warning", Color(c.warning)),
                ("danger", Color(c.danger)),
                ("disabled", Color(c.disabled)),
            ],
        ),
        (
            "typography",
            vec![
                ("caption", Size(t.caption)),
                ("body", Size(t.body)),
                ("body_emphasis", Size(t.body_emphasis)),
                ("heading_small", Size(t.heading_small)),
                ("heading", Size(t.heading)),
                ("heading_large", Size(t.heading_large)),
            ],
        ),
        (
            "spacing",
            vec![
                ("none", Size(s.none)),
                ("xsmall", Size(s.xsmall)),
                ("small", Size(s.small)),
                ("medium", Size(s.medium)),
                ("large", Size(s.large)),
                ("xlarge", Size(s.xlarge)),
                ("xxlarge", Size(s.xxlarge)),
            ],
        ),
        (
            "radii",
            vec![
                ("none", Size(r.none)),
                ("small", Size(r.small)),
                ("medium", Size(r.medium)),
                ("large", Size(r.large)),
                ("pill", Size(r.pill)),
            ],
        ),
        (
            "borders",
            vec![
                ("hairline", Size(b.hairline)),
                ("regular", Size(b.regular)),
                ("strong", Size(b.strong)),
            ],
        ),
        (
            "controls",
            vec![
                ("xsmall", Size(k.xsmall)),
                ("small", Size(k.small)),
                ("medium", Size(k.medium)),
                ("large", Size(k.large)),
            ],
        ),
        (
            "shadows",
            vec![
                ("none", Shadow(sh.none)),
                ("small", Shadow(sh.small)),
                ("medium", Shadow(sh.medium)),
                ("large", Shadow(sh.large)),
            ],
        ),
        (
            "motion",
            vec![
                ("instant_ms", Millis(m.instant_ms)),
                ("fast_ms", Millis(m.fast_ms)),
                ("normal_ms", Millis(m.normal_ms)),
                ("slow_ms", Millis(m.slow_ms)),
            ],
        ),
    ]
}

/// JSON number text for a finite float; non-finite values (which the loader
/// rejects) are written as 0 so the output always parses.
fn number(value: f32) -> String {
    if value.is_finite() { format!("{value}") } else { "0".to_owned() }
}

impl Theme {
    /// Parse and validate a theme from JSON theme-file text.
    ///
    /// Every token must be present and valid; errors name the offending token.
    pub fn from_json(text: &str) -> Result<Theme> {
        let value: Value = serde_json::from_str(text)
            .map_err(|error| ThemeFileError::Syntax(error.to_string()))?;
        read_theme(&value)
    }

    /// Export every token as a pretty-printed JSON theme file.
    ///
    /// Keys follow the struct field order, so the output is stable for diffs.
    /// [`Theme::from_json`] reads the output back to an equal theme whenever the
    /// colours are 8-bit values (true for every built-in theme and loaded file).
    pub fn to_json(&self) -> String {
        let mut out = String::from("{\n");
        out.push_str(&format!("  \"name\": {},\n", Value::from(self.name)));
        let groups = groups(self);
        for (group_index, (group, tokens)) in groups.iter().enumerate() {
            out.push_str(&format!("  \"{group}\": {{\n"));
            for (index, (key, token)) in tokens.iter().enumerate() {
                let value = match token {
                    Token::Color(color) => format!("\"{}\"", color_hex(*color)),
                    Token::Size(size) => number(*size),
                    Token::Millis(ms) => ms.to_string(),
                    Token::Shadow(s) => format!(
                        "{{ \"x\": {}, \"y\": {}, \"blur\": {}, \"spread\": {}, \"color\": \"{}\" }}",
                        number(s.x),
                        number(s.y),
                        number(s.blur),
                        number(s.spread),
                        color_hex(s.color)
                    ),
                };
                let comma = if index + 1 < tokens.len() { "," } else { "" };
                out.push_str(&format!("    \"{key}\": {value}{comma}\n"));
            }
            let comma = if group_index + 1 < groups.len() { "," } else { "" };
            out.push_str(&format!("  }}{comma}\n"));
        }
        out.push_str("}\n");
        out
    }

    /// Export every token as CSS custom properties inside `selector`, for
    /// example `":root"` or `"[data-theme=\"dark\"]"`.
    ///
    /// Properties are named `--mkit-<group>-<token>` with underscores turned
    /// into hyphens (`--mkit-colors-text-muted`). Sizes use `px`, motion uses
    /// `ms`, and shadows use the `box-shadow` shorthand. Colours are hex.
    pub fn to_css_custom_properties(&self, selector: &str) -> String {
        let mut out = format!("{selector} {{\n");
        for (group, tokens) in groups(self) {
            for (key, token) in tokens {
                let key = key.strip_suffix("_ms").unwrap_or(key).replace('_', "-");
                let value = match token {
                    Token::Color(color) => color_hex(color),
                    Token::Size(size) => format!("{}px", number(size)),
                    Token::Millis(ms) => format!("{ms}ms"),
                    Token::Shadow(s) => format!(
                        "{}px {}px {}px {}px {}",
                        number(s.x),
                        number(s.y),
                        number(s.blur),
                        number(s.spread),
                        color_hex(s.color)
                    ),
                };
                out.push_str(&format!("  --mkit-{group}-{key}: {value};\n"));
            }
        }
        out.push_str("}\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUILTINS: [Theme; 5] = [LIGHT, DARK, HIGH_CONTRAST, SHADCN_LIGHT, SHADCN_DARK];

    fn edit(theme: &Theme, f: impl FnOnce(&mut Value)) -> String {
        let mut value: Value = serde_json::from_str(&theme.to_json()).unwrap();
        f(&mut value);
        value.to_string()
    }

    #[test]
    fn every_builtin_theme_round_trips_through_json() {
        for theme in BUILTINS {
            let json = theme.to_json();
            let loaded = Theme::from_json(&json).unwrap_or_else(|e| panic!("{}: {e}", theme.name));
            assert_eq!(loaded, theme, "{} did not round-trip", theme.name);
            // Export is deterministic.
            assert_eq!(loaded.to_json(), json);
        }
    }

    #[test]
    fn export_uses_readable_hex_colours_and_field_order() {
        let json = DARK.to_json();
        assert!(json.starts_with(
            "{\n  \"name\": \"dark\",\n  \"colors\": {\n    \"background\": \"#17191f\","
        ));
        assert!(json.contains("\"small\": { \"x\": 0, \"y\": 1, \"blur\": 3, \"spread\": 0, \"color\": \"#00000040\" }"));
        assert!(json.contains("\"normal_ms\": 180,\n"));
    }

    #[test]
    fn custom_names_are_interned_once() {
        let json = edit(&DARK, |v| v["name"] = "studio-night".into());
        let first = Theme::from_json(&json).unwrap();
        let second = Theme::from_json(&json).unwrap();
        assert_eq!(first.name, "studio-night");
        assert!(std::ptr::eq(first.name, second.name));
        assert_eq!(first.colors, DARK.colors);
    }

    #[test]
    fn missing_tokens_are_named() {
        let json = edit(&LIGHT, |v| {
            v["colors"].as_object_mut().unwrap().remove("focus");
        });
        let error = Theme::from_json(&json).unwrap_err();
        assert_eq!(error, ThemeFileError::Missing { token: "colors.focus".into() });
        assert_eq!(error.to_string(), "theme file is missing token `colors.focus`");

        let json = edit(&LIGHT, |v| {
            v["shadows"]["small"].as_object_mut().unwrap().remove("blur");
        });
        assert_eq!(
            Theme::from_json(&json).unwrap_err(),
            ThemeFileError::Missing { token: "shadows.small.blur".into() }
        );

        let json = edit(&LIGHT, |v| {
            v.as_object_mut().unwrap().remove("motion");
        });
        assert_eq!(
            Theme::from_json(&json).unwrap_err(),
            ThemeFileError::Missing { token: "motion".into() }
        );
    }

    #[test]
    fn invalid_tokens_are_named_with_the_expected_form() {
        type Mutate = fn(&mut Value);
        let cases: [(Mutate, &str); 8] = [
            (|v| v["colors"]["accent"] = "blue".into(), "colors.accent"),
            (|v| v["colors"]["accent"] = "#12345".into(), "colors.accent"),
            (|v| v["colors"]["accent"] = "#12345g".into(), "colors.accent"),
            (|v| v["spacing"]["medium"] = (-4).into(), "spacing.medium"),
            (|v| v["typography"]["body"] = "14px".into(), "typography.body"),
            (|v| v["motion"]["fast_ms"] = 12.5.into(), "motion.fast_ms"),
            (|v| v["motion"]["slow_ms"] = 70000.into(), "motion.slow_ms"),
            (|v| v["name"] = " ".into(), "name"),
        ];
        for (mutate, token) in cases {
            let json = edit(&DARK, mutate);
            match Theme::from_json(&json).unwrap_err() {
                ThemeFileError::Invalid { token: found, .. } => assert_eq!(found, token),
                other => panic!("expected invalid {token}, got {other:?}"),
            }
        }
        let error =
            Theme::from_json(&edit(&DARK, |v| v["colors"]["accent"] = "blue".into())).unwrap_err();
        assert_eq!(
            error.to_string(),
            "theme token `colors.accent` must be a #rrggbb or #rrggbbaa colour, found \"blue\""
        );
        // Shadow offsets may be negative; blur may not.
        assert!(
            Theme::from_json(&edit(&DARK, |v| v["shadows"]["large"]["y"] = (-2).into())).is_ok()
        );
        assert!(
            Theme::from_json(&edit(&DARK, |v| v["shadows"]["large"]["blur"] = (-2).into()))
                .is_err()
        );
        // Groups must be objects.
        assert!(matches!(
            Theme::from_json(&edit(&DARK, |v| v["radii"] = 4.into())).unwrap_err(),
            ThemeFileError::Invalid { token, .. } if token == "radii"
        ));
    }

    #[test]
    fn unknown_tokens_and_syntax_errors_are_rejected() {
        let json = edit(&LIGHT, |v| v["colors"]["focus_ring"] = "#000000".into());
        assert_eq!(
            Theme::from_json(&json).unwrap_err(),
            ThemeFileError::Unknown { token: "colors.focus_ring".into() }
        );
        let json = edit(&LIGHT, |v| v["palette"] = Value::Object(Map::new()));
        assert_eq!(
            Theme::from_json(&json).unwrap_err(),
            ThemeFileError::Unknown { token: "palette".into() }
        );
        assert!(matches!(Theme::from_json("{ \"name\": "), Err(ThemeFileError::Syntax(_))));
        assert!(matches!(
            Theme::from_json("[]"),
            Err(ThemeFileError::Invalid { token, .. }) if token == "(root)"
        ));
    }

    #[test]
    fn alpha_colours_use_eight_digit_hex() {
        let json = edit(&LIGHT, |v| v["colors"]["disabled"] = "#858b9680".into());
        let theme = Theme::from_json(&json).unwrap();
        assert!((theme.colors.disabled.a - 128.0 / 255.0).abs() < 1e-6);
        assert!(theme.to_json().contains("\"disabled\": \"#858b9680\""));
    }

    #[test]
    fn css_custom_properties_cover_every_token() {
        let css = HIGH_CONTRAST.to_css_custom_properties("[data-theme=\"high-contrast\"]");
        assert!(css.starts_with("[data-theme=\"high-contrast\"] {\n"));
        assert!(css.ends_with("}\n"));
        assert!(css.contains("  --mkit-colors-text-muted: #f0f0f0;\n"));
        assert!(css.contains("  --mkit-borders-strong: 3px;\n"));
        assert!(css.contains("  --mkit-motion-normal: 0ms;\n"));
        let css = LIGHT.to_css_custom_properties(":root");
        assert!(css.contains("  --mkit-shadows-small: 0px 1px 3px 0px #00000018;\n"));
        assert!(css.contains("  --mkit-typography-heading-large: 28px;\n"));
        let properties =
            css.lines().filter(|line| line.trim_start().starts_with("--mkit-")).count();
        assert_eq!(properties, 13 + 6 + 7 + 5 + 3 + 4 + 4 + 4);
    }
}
