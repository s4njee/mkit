//! Stateless shadcn-style action button.
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
        let (bg, fg, border) = match self.variant {
            Variant::Default => (t.colors.accent, t.colors.accent_text, t.colors.accent),
            Variant::Secondary => (t.colors.elevated_surface, t.colors.text, t.colors.border),
            Variant::Outline => (t.colors.surface, t.colors.text, t.colors.border),
            Variant::Ghost => (t.colors.background, t.colors.text, t.colors.background),
            Variant::Destructive => (t.colors.danger, t.colors.accent_text, t.colors.danger),
            Variant::Link => (t.colors.background, t.colors.accent, t.colors.background),
        };
        let h = match self.size {
            Size::Small => t.controls.xsmall,
            Size::Default => t.controls.medium,
            Size::Large => t.controls.large,
        };
        let pad = if self.size == Size::Small { t.spacing.small } else { t.spacing.medium };
        let unavailable = self.disabled || self.loading;
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
            .border_color(if self.disabled { t.colors.border } else { border })
            .bg(if self.disabled { t.colors.surface } else { bg })
            .text_color(if self.disabled { t.colors.disabled } else { fg })
            .text_size(px(t.typography.body))
            .focus_visible(|s| s.border_color(t.colors.focus));
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
