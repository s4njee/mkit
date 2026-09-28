//! Stateless shadcn-style action button.
extern crate gpui_pre as gpui;
use gpui_pre::{
    AnyElement, App, FontWeight, IntoElement, RenderOnce, Rgba, Window, actions, div, prelude::*,
    px,
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
            Variant::Link => (transparent, c.accent, transparent, transparent),
        };
        let outlined = !matches!(variant, Variant::Ghost | Variant::Link);
        return Look {
            bg,
            fg,
            border,
            shadow: false,
            hover_bg: None,
            hover_border: (variant != Variant::Link).then_some(hover_border),
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
        Variant::Link => (transparent, c.accent, transparent, false, transparent),
    };
    Look {
        bg,
        fg,
        border,
        shadow,
        hover_bg: (variant != Variant::Link).then_some(hover_bg),
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
/// shadcn/ui focus ring width, drawn outside the button.
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

pub const KEY_CONTEXT: &str = "MkitButton";
actions!(button, [Activate]);
fn accessible_name(label: &str, override_label: Option<String>) -> String {
    override_label.unwrap_or_else(|| label.to_owned())
}

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
    Link,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    Small,
    #[default]
    Default,
    Large,
}
#[derive(IntoElement)]
pub struct Button {
    id: usize,
    label: String,
    variant: Variant,
    size: Size,
    disabled: bool,
    loading: bool,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    accessible_label: Option<String>,
    on_click: Option<Box<ClickHandler>>,
    on_activate: Option<Rc<ActivateHandler>>,
}
impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            label: label.into(),
            variant: Variant::Default,
            size: Size::Default,
            disabled: false,
            loading: false,
            leading: None,
            trailing: None,
            accessible_label: None,
            on_click: None,
            on_activate: None,
        }
    }
    pub fn id(mut self, id: usize) -> Self {
        self.id = id;
        self
    }
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
    pub fn leading(mut self, icon: impl IntoElement) -> Self {
        self.leading = Some(icon.into_any_element());
        self
    }
    pub fn trailing(mut self, icon: impl IntoElement) -> Self {
        self.trailing = Some(icon.into_any_element());
        self
    }
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }
    pub fn on_activate(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(callback));
        self
    }
    pub fn on_click(
        mut self,
        callback: impl Fn(&gpui_pre::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }
}
impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let accessible_name = accessible_name(&self.label, self.accessible_label);
        let look = look(&t, self.variant);
        let (h, pad) = match self.size {
            Size::Small => (t.controls.small, t.spacing.medium),
            Size::Default => (t.controls.medium, t.spacing.large),
            Size::Large => (t.controls.large, t.spacing.xlarge),
        };
        let unavailable = self.disabled || self.loading;
        let is_link = self.variant == Variant::Link;
        let (bg, fg, border) = match look.unavailable {
            Some(colors) if unavailable => colors,
            _ => (look.bg, look.fg, look.border),
        };
        let mut el = div()
            .id(("mkit-button", self.id))
            .debug_selector(|| "mkit-button".into())
            .key_context(KEY_CONTEXT)
            .tab_index(if self.disabled || self.loading { -1 } else { 0 })
            .role(gpui_pre::accesskit::Role::Button)
            .aria_label(accessible_name)
            .when(self.disabled, |e| e.aria_description("Unavailable"))
            .when(self.loading, |e| e.aria_description("Loading"))
            .when(unavailable, |e| {
                e.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .flex()
            .items_center()
            .justify_center()
            .gap(px(t.spacing.small))
            .h(px(h))
            .px(px(pad))
            .rounded(px(t.radii.medium))
            .border(px(t.borders.regular))
            .border_color(border)
            .bg(bg)
            .when(look.shadow, |e| e.shadow(vec![box_shadow(t.shadows.small)]))
            .text_color(fg)
            .text_size(px(t.typography.body))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .when(unavailable && look.unavailable.is_none(), |e| e.opacity(0.5))
            .when(!unavailable, |e| {
                e.hover(move |s| {
                    let s = match look.hover_bg {
                        Some(color) => s.bg(color),
                        None => s,
                    };
                    let s = match look.hover_border {
                        Some(color) => s.border_color(color),
                        None => s,
                    };
                    if is_link { s.underline() } else { s }
                })
            })
            .focus_visible(move |s| {
                s.border_color(t.colors.focus).bg(look.focus_bg).shadow(vec![focus_ring(look.ring)])
            });
        let enabled = !self.disabled && !self.loading;
        let on_activate = self.on_activate.filter(|_| enabled);
        if let Some(cb) = on_activate.clone() {
            el = el.on_action(move |_: &Activate, window, cx| cb(window, cx));
        }
        let on_click = self.on_click.filter(|_| enabled);
        if on_click.is_some() || on_activate.is_some() {
            el = el.on_click(move |event, window, cx| {
                if let Some(cb) = on_click.as_ref() {
                    cb(event, window, cx);
                }
                if let Some(cb) = on_activate.as_ref() {
                    cb(window, cx);
                }
            });
        }
        if let Some(icon) = self.leading {
            el = el.child(icon);
        }
        el = el.child(self.label);
        if let Some(icon) = self.trailing {
            el = el.child(icon);
        }
        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visible_label_names_button_unless_explicitly_overridden() {
        assert_eq!(accessible_name("Save", None), "Save");
        assert_eq!(accessible_name("Save", Some("Commit changes".into())), "Commit changes");
    }

    #[test]
    fn all_variants_and_sizes_are_explicit() {
        assert_eq!(Variant::default(), Variant::Default);
        assert_eq!(Size::default(), Size::Default);
        for v in [
            Variant::Secondary,
            Variant::Outline,
            Variant::Ghost,
            Variant::Destructive,
            Variant::Link,
        ] {
            let _ = Button::new("action").variant(v);
        }
        for s in [Size::Small, Size::Default, Size::Large] {
            let _ = Button::new("action").size(s);
        }
    }
    #[test]
    fn generated_ids_are_unique() {
        let a = Button::new("same");
        let b = Button::new("same");
        assert_ne!(a.id, b.id);
    }
    #[test]
    fn disabled_loading_builder_states_are_independent() {
        let b = Button::new("save").disabled(true).loading(true);
        assert!(b.disabled && b.loading);
    }
}
