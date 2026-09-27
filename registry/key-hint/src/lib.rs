//! Platform-aware, noninteractive keyboard shortcut display.
extern crate gpui_pre as gpui;

use gpui_pre::{App, IntoElement, RenderOnce, Window, div, prelude::*, px};
use mkit_core::theme::Theme;
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

#[derive(IntoElement)]
pub struct KeyHint {
    id: usize,
    chord: KeyChord,
    accessible_name: Option<String>,
}

impl KeyHint {
    pub fn new(chord: KeyChord) -> Self {
        Self { id: NEXT_ID.fetch_add(1, Ordering::Relaxed), chord, accessible_name: None }
    }

    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.accessible_name = Some(label.into());
        self
    }
}

impl RenderOnce for KeyHint {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let mut visible = self.chord.visible_parts();
        visible.push(self.chord.key.clone());
        let mut spoken =
            self.chord.spoken_parts().into_iter().map(str::to_owned).collect::<Vec<_>>();
        spoken.push(self.chord.key.clone());
        let accessible_name = self.accessible_name.unwrap_or_else(|| spoken.join(" plus "));
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
                    .px(px(theme.spacing.xsmall))
                    .py(px(theme.spacing.xsmall))
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.hairline))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.elevated_surface)
                    .text_color(theme.colors.text_muted)
                    .text_size(px(theme.typography.caption))
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
}
