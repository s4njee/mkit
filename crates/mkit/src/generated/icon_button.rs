//! Stateless icon-only action button. Accessible label is required by construction.
extern crate gpui_pre as gpui;
use gpui_pre::{AnyElement, App, IntoElement, RenderOnce, Window, actions, div, prelude::*, px};
use mkit_core::theme::Theme;
use std::{
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};
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
        let (bg, fg, border) = match self.variant {
            Variant::Default => (t.colors.accent, t.colors.accent_text, t.colors.accent),
            Variant::Secondary => (t.colors.elevated_surface, t.colors.text, t.colors.border),
            Variant::Outline => (t.colors.surface, t.colors.text, t.colors.border),
            Variant::Ghost => (t.colors.background, t.colors.text, t.colors.background),
            Variant::Destructive => (t.colors.danger, t.colors.accent_text, t.colors.danger),
        };
        let h = match self.size {
            Size::Small => t.controls.xsmall,
            Size::Default => t.controls.medium,
            Size::Large => t.controls.large,
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
            .border_color(if self.disabled { t.colors.border } else { border })
            .bg(if self.disabled { t.colors.surface } else { bg })
            .text_color(if self.disabled { t.colors.disabled } else { fg })
            .focus_visible(|s| s.border_color(t.colors.focus))
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
