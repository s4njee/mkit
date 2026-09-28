//! Stateless icon-only action button. Accessible label is required by construction.
extern crate gpui_pre as gpui;
use gpui_pre::{
    AnyElement, App, IntoElement, RenderOnce, Rgba, Window, actions, div, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
use std::{
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};
/// Resolved colours for one variant; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    bg: Rgba,
    fg: Rgba,
    border: Rgba,
    /// Whether the resting button draws `shadows.small` (shadcn `shadow-xs`).
    shadow: bool,
    hover_bg: Option<Rgba>,
    hover_border: Option<Rgba>,
    /// Opaque fill used while the focus ring is drawn.
    focus_bg: Rgba,
    ring: Rgba,
    /// Colours used while disabled or loading; `None` dims the resting look to 50% instead.
    unavailable: Option<(Rgba, Rgba, Rgba)>,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight, ..foreground }, Rgba { a: 1.0, ..base })
}
fn look(t: &Theme, variant: Variant) -> Look {
    let c = t.colors;
    let transparent = c.background.opacity(0.);
    if t.name == "high-contrast" {
        let (bg, fg, border, hover_border) = match variant {
            Variant::Default => (c.accent, c.accent_text, c.accent, c.text),
            Variant::Secondary | Variant::Outline => (c.background, c.text, c.border, c.accent),
            Variant::Ghost => (transparent, c.text, transparent, c.accent),
            Variant::Destructive => (c.danger, c.accent_text, c.danger, c.text),
        };
        let outlined = variant != Variant::Ghost;
        return Look {
            bg,
            fg,
            border,
            shadow: false,
            hover_bg: None,
            hover_border: Some(hover_border),
            focus_bg: if bg.a < 1.0 { c.background } else { bg },
            ring: c.focus,
            unavailable: Some((
                if outlined { c.background } else { transparent },
                c.disabled,
                if outlined { c.disabled } else { transparent },
            )),
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn "secondary"/"accent"/"muted": text mixed into the background.
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    let outline_border = if dark { c.text.opacity(0.1) } else { c.border };
    // Near-black and near-white from the theme, for the destructive fill.
    let (ink, on_danger) = if dark { (c.background, c.text) } else { (c.text, c.background) };
    let (bg, fg, border, shadow, hover_bg) = match variant {
        Variant::Default => {
            (c.accent, c.accent_text, transparent, true, mix(c.accent, c.background, 0.9))
        }
        // color-mix(in srgb, secondary 80%, foreground 6%): the 86% total becomes alpha.
        Variant::Secondary => (
            muted,
            c.text,
            transparent,
            true,
            composite(Rgba { a: 0.86, ..mix(c.text, muted, 6. / 86.) }, c.background),
        ),
        Variant::Outline => (c.background, c.text, outline_border, true, muted),
        Variant::Ghost => (transparent, c.text, transparent, false, muted),
        Variant::Destructive => (c.danger, on_danger, transparent, true, mix(ink, c.danger, 0.1)),
    };
    Look {
        bg,
        fg,
        border,
        shadow,
        hover_bg: Some(hover_bg),
        hover_border: None,
        // GPUI fills the inside of drop shadows, so the fill under the focus ring is opaque.
        focus_bg: if bg.a < 1.0 { c.background } else { bg },
        ring: c.focus.opacity(0.5),
        unavailable: None,
    }
}
fn box_shadow(shadow: ShadowToken) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: shadow.color.into(),
        offset: gpui_pre::point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}
/// shadcn/ui focus ring width, drawn outside the icon button.
const FOCUS_RING_WIDTH: f32 = 3.0;
fn focus_ring(color: Rgba) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: color.into(),
        offset: gpui_pre::point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(FOCUS_RING_WIDTH),
        inset: false,
    }
}
static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
type ClickHandler = dyn Fn(&gpui_pre::ClickEvent, &mut Window, &mut App) + 'static;
type ActivateHandler = dyn Fn(&mut Window, &mut App) + 'static;
pub const KEY_CONTEXT: &str = "MkitIconButton";
actions!(icon_button, [Activate]);
pub fn default_key_bindings() -> [gpui_pre::KeyBinding; 2] {
    [
        gpui_pre::KeyBinding::new("enter", Activate, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("space", Activate, Some(KEY_CONTEXT)),
    ]
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    Small,
    #[default]
    Default,
    Large,
}
#[derive(IntoElement)]
pub struct IconButton {
    id: usize,
    label: String,
    icon: AnyElement,
    variant: Variant,
    size: Size,
    disabled: bool,
    on_click: Option<Box<ClickHandler>>,
    on_activate: Option<Rc<ActivateHandler>>,
}
impl IconButton {
    pub fn new(label: impl Into<String>, icon: impl IntoElement) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            label: label.into(),
            icon: icon.into_any_element(),
            variant: Variant::Default,
            size: Size::Default,
            disabled: false,
            on_click: None,
            on_activate: None,
        }
    }
    pub fn id(mut self, id: usize) -> Self {
        self.id = id;
        self
    }
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn on_activate(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(f));
        self
    }
    pub fn on_click(
        mut self,
        f: impl Fn(&gpui_pre::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}
impl RenderOnce for IconButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let look = look(&t, self.variant);
        let h = match self.size {
            Size::Small => t.controls.small,
            Size::Default => t.controls.medium,
            Size::Large => t.controls.large,
        };
        let (bg, fg, border) = match look.unavailable {
            Some(colors) if self.disabled => colors,
            _ => (look.bg, look.fg, look.border),
        };
        let mut e = div()
            .id(("mkit-icon-button", self.id))
            .debug_selector(|| "mkit-icon-button".into())
            .key_context(KEY_CONTEXT)
            .tab_index(if self.disabled { -1 } else { 0 })
            .role(gpui_pre::accesskit::Role::Button)
            .aria_label(self.label.clone())
            .when(self.disabled, |x| x.aria_description("Unavailable"))
            .when(self.disabled, |x| {
                x.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .h(px(h))
            .w(px(h))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(t.radii.medium))
            .border(px(t.borders.regular))
            .border_color(border)
            .bg(bg)
            .when(look.shadow, |x| x.shadow(vec![box_shadow(t.shadows.small)]))
            .text_color(fg)
            .text_size(px(t.typography.body))
            .when(self.disabled && look.unavailable.is_none(), |x| x.opacity(0.5))
            .when(!self.disabled, |x| {
                x.hover(move |s| {
                    let s = match look.hover_bg {
                        Some(color) => s.bg(color),
                        None => s,
                    };
                    match look.hover_border {
                        Some(color) => s.border_color(color),
                        None => s,
                    }
                })
            })
            .focus_visible(move |s| {
                s.border_color(t.colors.focus).bg(look.focus_bg).shadow(vec![focus_ring(look.ring)])
            })
            .child(self.icon);
        let enabled = !self.disabled;
        let on_click = self.on_click.filter(|_| enabled);
        let on_activate = self.on_activate.filter(|_| enabled);
        if let Some(callback) = on_activate.clone() {
            e = e.on_action(move |_: &Activate, w, cx| callback(w, cx));
        }
        if on_click.is_some() || on_activate.is_some() {
            e = e.on_click(move |event, w, cx| {
                if let Some(callback) = on_click.as_ref() {
                    callback(event, w, cx);
                }
                if let Some(callback) = on_activate.as_ref() {
                    callback(w, cx);
                }
            });
        }
        e
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn variants_and_sizes_remain_available() {
        for v in [
            Variant::Default,
            Variant::Secondary,
            Variant::Outline,
            Variant::Ghost,
            Variant::Destructive,
        ] {
            let _ = v;
        }
        for s in [Size::Small, Size::Default, Size::Large] {
            let _ = s;
        }
    }
}
