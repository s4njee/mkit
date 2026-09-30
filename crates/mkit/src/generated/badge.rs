//! Compact, theme-driven count and status badges.
extern crate gpui_pre as gpui;

use gpui_pre::{App, FontWeight, IntoElement, RenderOnce, Rgba, Window, div, prelude::*, px};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::Theme,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Neutral,
    Success,
    Warning,
    Danger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Count(u64),
    Status(Status),
}

/// Small count or status metadata. It is static content, never a live region.
#[derive(IntoElement)]
pub struct Badge {
    id: usize,
    kind: Kind,
    label: Option<String>,
    accessible_name: Option<String>,
    max_count: u64,
    overflow_label: String,
    disabled: bool,
}

impl Badge {
    pub fn count(count: u64) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            kind: Kind::Count(count),
            label: None,
            accessible_name: None,
            max_count: 99,
            overflow_label: "99+".into(),
            disabled: false,
        }
    }

    pub fn status(status: Status, label: impl Into<String>) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            kind: Kind::Status(status),
            label: Some(label.into()),
            accessible_name: None,
            max_count: 99,
            overflow_label: "99+".into(),
            disabled: false,
        }
    }

    pub fn max_count(mut self, max_count: u64) -> Self {
        self.max_count = max_count;
        self
    }

    pub fn overflow_label(mut self, label: impl Into<String>) -> Self {
        self.overflow_label = label.into();
        self
    }

    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.accessible_name = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}

/// shadcn's `opacity: .5` applied as one layer: composite over `base`, then mix 50%.
fn dim(color: Rgba, base: Rgba) -> Rgba {
    mix(composite(color, base), base, 0.5)
}

/// Resolved colours for one badge; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Look {
    bg: Rgba,
    fg: Rgba,
    border: Rgba,
    dot: Rgba,
}

fn look(t: &Theme, kind: Kind, disabled: bool) -> Look {
    let c = t.colors;
    let status_color = |status: Status| match status {
        Status::Neutral => c.text_muted,
        Status::Success => c.success,
        Status::Warning => c.warning,
        Status::Danger => c.danger,
    };
    if t.name == "high-contrast" {
        let look = match kind {
            Kind::Count(_) => {
                Look { bg: c.accent, fg: c.accent_text, border: c.accent, dot: c.accent_text }
            }
            Kind::Status(status) => {
                Look { bg: c.background, fg: c.text, border: c.border, dot: status_color(status) }
            }
        };
        return if disabled {
            Look { bg: c.background, fg: c.disabled, border: c.disabled, dot: c.disabled }
        } else {
            look
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    let border = if dark { mix(c.text, c.background, 0.1) } else { c.border };
    // The theme's near-white, for text on the destructive fill.
    let on_danger = if dark { c.text } else { c.background };
    let look = match kind {
        Kind::Count(_) => {
            Look { bg: c.accent, fg: c.accent_text, border: c.accent, dot: c.accent_text }
        }
        Kind::Status(Status::Neutral) => {
            Look { bg: muted, fg: c.text, border: muted, dot: c.text_muted }
        }
        Kind::Status(status @ (Status::Success | Status::Warning)) => {
            let tint = status_color(status);
            Look {
                bg: mix(tint, c.background, 0.12),
                fg: c.text,
                border: mix(tint, border, 0.3),
                dot: tint,
            }
        }
        Kind::Status(Status::Danger) => {
            Look { bg: c.danger, fg: on_danger, border: c.danger, dot: on_danger }
        }
    };
    if disabled {
        let bg = c.background;
        Look {
            bg: dim(look.bg, bg),
            fg: dim(look.fg, bg),
            border: dim(look.border, bg),
            dot: dim(look.dot, bg),
        }
    } else {
        look
    }
}

impl RenderOnce for Badge {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let look = look(&theme, self.kind, self.disabled);
        let text = match self.kind {
            Kind::Count(count) => {
                if count > self.max_count {
                    self.overflow_label
                } else {
                    count.to_string()
                }
            }
            Kind::Status(_) => self.label.unwrap_or_default(),
        };
        let border = theme.borders.regular;
        let icon = theme.typography.caption;
        div()
            .id(("mkit-badge", self.id))
            .debug_selector(|| "mkit-badge".into())
            .when_some(self.accessible_name, |el, name| el.aria_label(name))
            .flex()
            .flex_none()
            .self_start()
            .items_center()
            .gap(px(theme.spacing.xsmall))
            .h(px(theme.spacing.large + theme.spacing.xsmall + 2.0 * border))
            .px(px(theme.spacing.small))
            .rounded(px(theme.radii.medium))
            .border(px(border))
            .border_color(look.border)
            .bg(look.bg)
            .text_color(look.fg)
            .text_size(px(theme.typography.caption))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .when(matches!(self.kind, Kind::Status(_)), |el| {
                el.child(
                    div().size(px(icon)).flex().items_center().justify_center().child(
                        div().size(px(icon * 0.5)).rounded(px(theme.radii.pill)).bg(look.dot),
                    ),
                )
            })
            .child(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_overflow_uses_configured_label() {
        let badge = Badge::count(120).max_count(10).overflow_label("10 or more");
        assert_eq!(badge.max_count, 10);
        assert_eq!(badge.overflow_label, "10 or more");
    }

    #[test]
    fn disabled_badges_dim_every_colour_but_high_contrast_stays_solid() {
        use mkit_core::theme::{HIGH_CONTRAST, SHADCN_LIGHT};
        let kind = Kind::Status(Status::Danger);
        let resting = look(&SHADCN_LIGHT, kind, false);
        let disabled = look(&SHADCN_LIGHT, kind, true);
        assert_eq!(disabled.bg, dim(resting.bg, SHADCN_LIGHT.colors.background));
        assert_eq!(disabled.fg, dim(resting.fg, SHADCN_LIGHT.colors.background));
        assert_ne!(disabled.bg, resting.bg);
        let hc = look(&HIGH_CONTRAST, kind, true);
        assert_eq!(hc.fg, HIGH_CONTRAST.colors.disabled);
        assert_eq!(hc.bg, HIGH_CONTRAST.colors.background);
    }

    #[test]
    fn status_badge_keeps_a_visible_text_label() {
        let badge = Badge::status(Status::Warning, "Pending");
        assert_eq!(badge.label.as_deref(), Some("Pending"));
        assert_eq!(badge.kind, Kind::Status(Status::Warning));
    }
}
