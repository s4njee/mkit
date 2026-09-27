//! Compact, theme-driven count and status badges.
extern crate gpui_pre as gpui;

use gpui_pre::{App, IntoElement, RenderOnce, Window, div, prelude::*, px};
use mkit_core::theme::Theme;
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

impl RenderOnce for Badge {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let (text, color) = match self.kind {
            Kind::Count(count) => (
                if count > self.max_count { self.overflow_label } else { count.to_string() },
                theme.colors.accent,
            ),
            Kind::Status(status) => (
                self.label.unwrap_or_default(),
                match status {
                    Status::Neutral => theme.colors.text_muted,
                    Status::Success => theme.colors.success,
                    Status::Warning => theme.colors.warning,
                    Status::Danger => theme.colors.danger,
                },
            ),
        };
        div()
            .id(("mkit-badge", self.id))
            .debug_selector(|| "mkit-badge".into())
            .when_some(self.accessible_name, |el, name| el.aria_label(name))
            .flex()
            .self_start()
            .items_center()
            .gap(px(theme.spacing.xsmall))
            .px(px(theme.spacing.small))
            .py(px(theme.spacing.xsmall))
            .rounded(px(theme.radii.pill))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .text_color(if self.disabled { theme.colors.disabled } else { theme.colors.text })
            .text_size(px(theme.typography.caption))
            .when(matches!(self.kind, Kind::Status(_)), |el| {
                el.child(
                    div()
                        .w(px(theme.spacing.small))
                        .h(px(theme.spacing.small))
                        .rounded(px(theme.radii.pill))
                        .bg(if self.disabled { theme.colors.disabled } else { color }),
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
    fn status_badge_keeps_a_visible_text_label() {
        let badge = Badge::status(Status::Warning, "Pending");
        assert_eq!(badge.label.as_deref(), Some("Pending"));
        assert_eq!(badge.kind, Kind::Status(Status::Warning));
    }
}
