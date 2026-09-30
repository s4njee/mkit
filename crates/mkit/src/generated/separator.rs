//! Themed visual or semantic separator.
extern crate gpui_pre as gpui;

use gpui_pre::{App, IntoElement, RenderOnce, Rgba, Window, div, prelude::*, px};
use mkit_core::{contrast::relative_luminance, theme::Theme};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

/// Line colour: shadcn's `--border`. Dark themes use `text` at 10% alpha, the docs-site mapping,
/// so the rule reads the same over any surface; light themes and high contrast use `border`.
fn line_color(t: &Theme) -> Rgba {
    let c = t.colors;
    if t.name != "high-contrast" && relative_luminance(c.background) < 0.5 {
        c.text.opacity(0.1)
    } else {
        c.border
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(IntoElement)]
pub struct Separator {
    id: usize,
    orientation: Orientation,
    decorative: bool,
    label: Option<String>,
}

impl Separator {
    pub fn new() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            orientation: Orientation::Horizontal,
            decorative: true,
            label: None,
        }
    }
    pub fn id(mut self, id: usize) -> Self {
        self.id = id;
        self
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Separator {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let horizontal = self.orientation == Orientation::Horizontal;
        div()
            .id(("mkit-separator", self.id))
            .when(!self.decorative, |el| {
                el.role(gpui_pre::accesskit::Role::Splitter)
                    .aria_orientation(if horizontal {
                        gpui_pre::accesskit::Orientation::Horizontal
                    } else {
                        gpui_pre::accesskit::Orientation::Vertical
                    })
                    .when_some(self.label, |el, label| el.aria_label(label))
            })
            .flex_none()
            .when(horizontal, |el| el.w_full().h(px(theme.borders.hairline)))
            .when(!horizontal, |el| el.h_full().w(px(theme.borders.hairline)))
            .bg(line_color(&theme))
    }
}
