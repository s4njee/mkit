//! Platform-aware, noninteractive keyboard shortcut display.
//!
//! Besides the `KeyHint` element, this module exposes the shared shortcut
//! formatter used by menus, CommandPalette, and ShortcutEditor:
//! [`KeyChord::parse`], [`KeyChord::label`], and [`shortcut_label`].
extern crate gpui_pre as gpui;

use gpui_pre::{App, FontWeight, IntoElement, RenderOnce, Rgba, Window, div, prelude::*, px};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::Theme,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyChord {
    pub key: String,
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub command: bool,
}

impl KeyChord {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into(), control: false, alt: false, shift: false, command: false }
    }
    pub fn control(mut self) -> Self {
        self.control = true;
        self
    }
    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }
    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }
    pub fn command(mut self) -> Self {
        self.command = true;
        self
    }

    /// Parses one shortcut written as a GPUI keystroke (`cmd-shift-s`,
    /// `secondary-k`), a `+`-separated label (`Ctrl+Shift+P`), or a macOS glyph
    /// label (`⌘⇧S`).
    ///
    /// Returns `None` for empty text, text with inner whitespace (such as
    /// multi-keystroke sequences), and unrecognized modifiers such as `fn`, so
    /// callers can show the original text instead.
    pub fn parse(text: &str) -> Option<Self> {
        let mut rest = text.trim();
        let mut chord = KeyChord::new("");
        'modifiers: loop {
            for (glyph, modifier) in GLYPH_MODIFIERS {
                if rest.len() > glyph.len_utf8()
                    && let Some(after) = rest.strip_prefix(*glyph)
                {
                    modifier.apply(&mut chord);
                    rest = after;
                    continue 'modifiers;
                }
            }
            for (name, modifier) in NAMED_MODIFIERS {
                let Some(head) = rest.get(..name.len()) else { continue };
                let after = &rest[name.len()..];
                if head.eq_ignore_ascii_case(name)
                    && after.len() > 1
                    && (after.starts_with('-') || after.starts_with('+'))
                {
                    modifier.apply(&mut chord);
                    rest = &after[1..];
                    continue 'modifiers;
                }
            }
            break;
        }
        let single_char = rest.chars().count() == 1;
        if rest.is_empty()
            || rest.contains(char::is_whitespace)
            || (!single_char && (rest.contains('-') || rest.contains('+')))
        {
            return None;
        }
        chord.key = display_key(rest);
        Some(chord)
    }

    /// Compact platform label: `⇧⌘S` on macOS, `Shift+Super+S` elsewhere.
    pub fn label(&self) -> String {
        let mut parts = self.visible_parts();
        parts.push(self.key.clone());
        if cfg!(target_os = "macos") { parts.concat() } else { parts.join("+") }
    }

    /// Spoken platform label, for example `Shift plus Command plus S`.
    pub fn spoken_label(&self) -> String {
        let mut parts = self.spoken_parts().into_iter().map(str::to_owned).collect::<Vec<_>>();
        parts.push(self.key.clone());
        parts.join(" plus ")
    }

    fn visible_parts(&self) -> Vec<String> {
        let mut parts = Vec::new();
        #[cfg(target_os = "macos")]
        {
            if self.control {
                parts.push("⌃".into());
            }
            if self.alt {
                parts.push("⌥".into());
            }
            if self.shift {
                parts.push("⇧".into());
            }
            if self.command {
                parts.push("⌘".into());
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            if self.control {
                parts.push("Ctrl".into());
            }
            if self.alt {
                parts.push("Alt".into());
            }
            if self.shift {
                parts.push("Shift".into());
            }
            if self.command {
                parts.push("Super".into());
            }
        }
        parts
    }

    fn spoken_parts(&self) -> Vec<&'static str> {
        let mut parts = Vec::new();
        #[cfg(target_os = "macos")]
        {
            if self.control {
                parts.push("Control");
            }
            if self.alt {
                parts.push("Option");
            }
            if self.shift {
                parts.push("Shift");
            }
            if self.command {
                parts.push("Command");
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            if self.control {
                parts.push("Control");
            }
            if self.alt {
                parts.push("Alt");
            }
            if self.shift {
                parts.push("Shift");
            }
            if self.command {
                parts.push("Super");
            }
        }
        parts
    }
}

/// Formats shortcut text with the platform label used by every mkit shortcut
/// surface. Text that [`KeyChord::parse`] cannot read is returned trimmed but
/// otherwise unchanged.
pub fn shortcut_label(text: &str) -> String {
    KeyChord::parse(text).map_or_else(|| text.trim().to_owned(), |chord| chord.label())
}

#[derive(Clone, Copy)]
enum Modifier {
    Control,
    Alt,
    Shift,
    Command,
    /// GPUI's `secondary`: Command on macOS, Control elsewhere.
    Secondary,
}

impl Modifier {
    fn apply(self, chord: &mut KeyChord) {
        match self {
            Self::Control => chord.control = true,
            Self::Alt => chord.alt = true,
            Self::Shift => chord.shift = true,
            Self::Command => chord.command = true,
            Self::Secondary if cfg!(target_os = "macos") => chord.command = true,
            Self::Secondary => chord.control = true,
        }
    }
}

const GLYPH_MODIFIERS: &[(char, Modifier)] = &[
    ('⌃', Modifier::Control),
    ('⌥', Modifier::Alt),
    ('⇧', Modifier::Shift),
    ('⌘', Modifier::Command),
];

const NAMED_MODIFIERS: &[(&str, Modifier)] = &[
    ("ctrl", Modifier::Control),
    ("control", Modifier::Control),
    ("alt", Modifier::Alt),
    ("option", Modifier::Alt),
    ("opt", Modifier::Alt),
    ("shift", Modifier::Shift),
    ("cmd", Modifier::Command),
    ("command", Modifier::Command),
    ("super", Modifier::Command),
    ("win", Modifier::Command),
    ("meta", Modifier::Command),
    ("platform", Modifier::Command),
    ("secondary", Modifier::Secondary),
];

fn display_key(key: &str) -> String {
    if key.chars().count() == 1 {
        return key.to_uppercase();
    }
    if key.chars().any(char::is_uppercase) {
        return key.to_owned();
    }
    let named = match key {
        "up" => "↑",
        "down" => "↓",
        "left" => "←",
        "right" => "→",
        "escape" | "esc" => "Esc",
        "pageup" => "Page Up",
        "pagedown" => "Page Down",
        _ => "",
    };
    if !named.is_empty() {
        return named.to_owned();
    }
    let mut chars = key.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Keycap colours (fill, border, text); see the spec's "Theme tokens used" table.
fn keycap_colors(t: &Theme) -> (Rgba, Option<Rgba>, Rgba) {
    let c = t.colors;
    if t.name == "high-contrast" {
        return (c.background, Some(c.border), c.text);
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn's `muted`: `text` mixed into `background`, like CSS `color-mix(in srgb, ...)`.
    let muted = composite(Rgba { a: if dark { 0.12 } else { 0.04 }, ..c.text }, c.background);
    (muted, None, c.text_muted)
}

#[derive(IntoElement)]
pub struct KeyHint {
    id: usize,
    chord: KeyChord,
    accessible_name: Option<String>,
    inline: bool,
}

impl KeyHint {
    pub fn new(chord: KeyChord) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            chord,
            accessible_name: None,
            inline: false,
        }
    }

    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.accessible_name = Some(label.into());
        self
    }

    /// Renders the platform label as one text run without keycaps. The text
    /// inherits color and size from its parent, for dense rows such as menus.
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }
}

impl RenderOnce for KeyHint {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let accessible_name = self.accessible_name.unwrap_or_else(|| self.chord.spoken_label());
        if self.inline {
            return div()
                .id(("mkit-key-hint", self.id))
                .debug_selector(|| "mkit-key-hint".into())
                .aria_label(accessible_name)
                .child(self.chord.label());
        }
        let theme = *cx.global::<Theme>();
        let (fill, border, text) = keycap_colors(&theme);
        let cap = theme.spacing.large + theme.spacing.xsmall;
        let mut visible = self.chord.visible_parts();
        visible.push(self.chord.key.clone());
        let mut row = div()
            .id(("mkit-key-hint", self.id))
            .debug_selector(|| "mkit-key-hint".into())
            .aria_label(accessible_name)
            .flex()
            .items_center()
            .gap(px(theme.spacing.xsmall));
        for (index, part) in visible.iter().enumerate() {
            row = row.child(
                div()
                    .id(index)
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .h(px(cap))
                    .min_w(px(cap))
                    .px(px(theme.spacing.xsmall))
                    .rounded(px(theme.radii.small))
                    .when_some(border, |el, color| {
                        el.border(px(theme.borders.hairline)).border_color(color)
                    })
                    .bg(fill)
                    .text_color(text)
                    .text_size(px(theme.typography.caption))
                    .font_weight(FontWeight::MEDIUM)
                    .whitespace_nowrap()
                    .child(part.clone()),
            );
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chord_retains_all_requested_modifiers_and_main_key() {
        let chord = KeyChord::new("P").control().alt().shift().command();
        assert_eq!(chord.key, "P");
        assert!(chord.control && chord.alt && chord.shift && chord.command);
        assert_eq!(chord.visible_parts().len(), 4);
    }

    #[test]
    fn single_key_has_no_modifiers() {
        let chord = KeyChord::new("Escape");
        assert!(chord.visible_parts().is_empty());
        assert_eq!(chord.key, "Escape");
    }

    #[test]
    fn parse_reads_gpui_keystrokes_glyph_labels_and_plus_labels() {
        let expected = KeyChord::new("S").shift().command();
        assert_eq!(KeyChord::parse("cmd-shift-s"), Some(expected.clone()));
        assert_eq!(KeyChord::parse(" CMD-Shift-S "), Some(expected.clone()));
        assert_eq!(KeyChord::parse("⌘⇧S"), Some(expected.clone()));
        assert_eq!(KeyChord::parse("Shift+Super+S"), Some(expected));
        assert_eq!(
            KeyChord::parse("ctrl-alt-delete"),
            Some(KeyChord::new("Delete").control().alt())
        );
        assert_eq!(KeyChord::parse("option-up"), Some(KeyChord::new("↑").alt()));
        assert_eq!(KeyChord::parse("F2"), Some(KeyChord::new("F2")));
        assert_eq!(KeyChord::parse("f12"), Some(KeyChord::new("F12")));
        assert_eq!(KeyChord::parse("⌘]"), Some(KeyChord::new("]").command()));
        assert_eq!(KeyChord::parse("ctrl--"), Some(KeyChord::new("-").control()));
        assert_eq!(KeyChord::parse("cmd-+"), Some(KeyChord::new("+").command()));
        assert_eq!(KeyChord::parse("escape"), Some(KeyChord::new("Esc")));
    }

    #[test]
    fn parse_maps_secondary_to_the_platform_primary_modifier() {
        let chord = KeyChord::parse("secondary-k").expect("secondary chord");
        assert_eq!(chord.command, cfg!(target_os = "macos"));
        assert_eq!(chord.control, !cfg!(target_os = "macos"));
    }

    #[test]
    fn parse_rejects_sequences_unknown_modifiers_and_empty_text() {
        assert_eq!(KeyChord::parse(""), None);
        assert_eq!(KeyChord::parse("   "), None);
        assert_eq!(KeyChord::parse("cmd-k cmd-s"), None);
        assert_eq!(KeyChord::parse("⌘ N"), None);
        assert_eq!(KeyChord::parse("fn-f1"), None);
        assert_eq!(KeyChord::parse("cmd-"), None);
        assert_eq!(shortcut_label("fn-f1"), "fn-f1");
        assert_eq!(shortcut_label(" ⌘ N "), "⌘ N");
    }

    #[test]
    fn labels_follow_the_platform_modifier_order() {
        let chord = KeyChord::parse("⌘⇧S").unwrap();
        #[cfg(target_os = "macos")]
        {
            assert_eq!(chord.label(), "⇧⌘S");
            assert_eq!(chord.spoken_label(), "Shift plus Command plus S");
            assert_eq!(shortcut_label("cmd-o"), "⌘O");
            assert_eq!(shortcut_label("ctrl-alt-shift-cmd-p"), "⌃⌥⇧⌘P");
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert_eq!(chord.label(), "Shift+Super+S");
            assert_eq!(chord.spoken_label(), "Shift plus Super plus S");
            assert_eq!(shortcut_label("cmd-o"), "Super+O");
            assert_eq!(shortcut_label("ctrl-alt-shift-cmd-p"), "Ctrl+Alt+Shift+Super+P");
        }
    }
}
