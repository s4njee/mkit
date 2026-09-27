//! Theme-driven avatar frame with an optional application-provided image and initials fallback.
extern crate gpui_pre as gpui;

use gpui_pre::{App, ImageSource, IntoElement, RenderOnce, Window, div, img, prelude::*, px};
use mkit_core::theme::Theme;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(IntoElement)]
pub struct Avatar {
    id: usize,
    image: Option<ImageSource>,
    initials: Option<String>,
    accessible_name: Option<String>,
    size: Size,
}

impl Avatar {
    pub fn new() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            image: None,
            initials: None,
            accessible_name: None,
            size: Size::Medium,
        }
    }

    pub fn image(mut self, source: impl Into<ImageSource>) -> Self {
        self.image = Some(source.into());
        self
    }

    pub fn initials(mut self, initials: impl Into<String>) -> Self {
        self.initials = Some(initials.into());
        self
    }

    pub fn aria_label(mut self, name: impl Into<String>) -> Self {
        self.accessible_name = Some(name.into());
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Avatar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let extent = match self.size {
            Size::Small => theme.controls.xsmall,
            Size::Medium => theme.controls.medium,
            Size::Large => theme.controls.large,
        };
        let name = self.accessible_name.or_else(|| self.initials.clone());
        let mut frame = div()
            .id(("mkit-avatar", self.id))
            .debug_selector(|| "mkit-avatar".into())
            .when_some(name, |el, name| el.role(gpui_pre::accesskit::Role::Image).aria_label(name))
            .w(px(extent))
            .h(px(extent))
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .rounded(px(theme.radii.pill))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .bg(theme.colors.elevated_surface)
            .text_color(theme.colors.text_muted)
            .text_size(px(theme.typography.body_emphasis));
        if let Some(source) = self.image {
            frame = frame.child(
                img(source)
                    .w_full()
                    .h_full()
                    .object_fit(gpui_pre::ObjectFit::Cover)
                    .rounded(px(theme.radii.pill)),
            );
        } else if let Some(initials) = self.initials {
            frame = frame.child(initials);
        } else {
            frame = frame.child(
                div()
                    .w(px(extent * 0.65))
                    .h(px(extent * 0.72))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_end()
                    .gap(px(extent * 0.05))
                    .child(
                        div()
                            .w(px(extent * 0.31))
                            .h(px(extent * 0.31))
                            .rounded(px(theme.radii.pill))
                            .bg(theme.colors.text_muted),
                    )
                    .child(
                        div()
                            .w(px(extent * 0.65))
                            .h(px(extent * 0.34))
                            .rounded(px(theme.radii.medium))
                            .bg(theme.colors.text_muted),
                    ),
            );
        }
        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_and_accessible_name_are_explicit_fallbacks() {
        let avatar = Avatar::new().initials("JD");
        assert_eq!(avatar.initials.as_deref(), Some("JD"));
        assert_eq!(avatar.accessible_name, None);
    }

    #[test]
    fn image_and_size_are_caller_owned() {
        let avatar = Avatar::new().image("avatar-key").size(Size::Large);
        assert!(avatar.image.is_some());
        assert_eq!(avatar.size, Size::Large);
    }
}
