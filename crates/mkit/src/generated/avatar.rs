//! Theme-driven avatar frame with an optional application-provided image and initials fallback.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, FontWeight, ImageSource, IntoElement, PathBuilder, RenderOnce, Rgba, Window, canvas, div,
    img, point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::Theme,
};
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

/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}

/// Resolved colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Look {
    fill: Rgba,
    border: Rgba,
    text: Rgba,
    icon: Rgba,
}

fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look { fill: c.background, border: c.border, text: c.text, icon: c.text };
    }
    let dark = relative_luminance(c.background) < 0.5;
    Look {
        fill: mix(c.text, c.background, if dark { 0.12 } else { 0.04 }),
        border: if dark { mix(c.text, c.background, 0.1) } else { c.border },
        text: c.text,
        icon: c.text_muted,
    }
}

/// One step of a Lucide icon path on a 24-unit grid.
#[derive(Clone, Copy)]
enum Seg {
    Move(f32, f32),
    Line(f32, f32),
    /// SVG arc with a circular radius and sweep flag, ending at the point.
    Arc(f32, bool, f32, f32),
}

/// Lucide `user`: shoulders, then the head as two half circles.
const USER: &[Seg] = &[
    Seg::Move(19., 21.),
    Seg::Line(19., 19.),
    Seg::Arc(4., false, 15., 15.),
    Seg::Line(9., 15.),
    Seg::Arc(4., false, 5., 19.),
    Seg::Line(5., 21.),
    Seg::Move(16., 7.),
    Seg::Arc(4., true, 8., 7.),
    Seg::Arc(4., true, 16., 7.),
];

/// Decorative Lucide icon drawn as a vector stroke so it stays crisp at every scale; the stroke is
/// 2 units, Lucide's default.
fn icon(size: f32, segs: &'static [Seg], color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let at = |x: f32, y: f32| bounds.origin + point(unit * x, unit * y);
            let mut path = PathBuilder::stroke(unit * 2.0);
            for seg in segs {
                match *seg {
                    Seg::Move(x, y) => path.move_to(at(x, y)),
                    Seg::Line(x, y) => path.line_to(at(x, y)),
                    Seg::Arc(r, sweep, x, y) => {
                        path.arc_to(point(unit * r, unit * r), px(0.), false, sweep, at(x, y))
                    }
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
}

impl RenderOnce for Avatar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let look = look(&theme);
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
            .flex_none()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .rounded(px(theme.radii.pill))
            .border(px(theme.borders.hairline))
            .border_color(look.border)
            .bg(look.fill)
            .text_color(look.text)
            .text_size(px(theme.typography.caption))
            .font_weight(FontWeight::MEDIUM);
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
            frame = frame.child(icon(theme.spacing.large, USER, look.icon));
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
