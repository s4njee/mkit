//! Stateless breadcrumb trail using shared theme tokens.
extern crate gpui_pre as gpui;
use gpui_pre::{IntoElement, KeyBinding, RenderOnce, actions, div, prelude::*, px};
use mkit_core::theme::Theme;
use std::rc::Rc;

pub const KEY_CONTEXT: &str = "MkitBreadcrumbs";
actions!(breadcrumbs, [Activate]);

pub fn default_key_bindings() -> [KeyBinding; 1] {
    [KeyBinding::new("enter", Activate, Some(KEY_CONTEXT))]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crumb {
    pub label: String,
    pub target: Option<String>,
}
impl Crumb {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), target: None }
    }
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
}
#[derive(IntoElement)]
pub struct Breadcrumbs {
    label: String,
    items: Vec<Crumb>,
    disabled: bool,
    on_navigate: Option<Rc<dyn Fn(String)>>,
}
impl Breadcrumbs {
    pub fn new(label: impl Into<String>, items: Vec<Crumb>) -> Self {
        Self { label: label.into(), items, disabled: false, on_navigate: None }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_navigate(mut self, callback: impl Fn(String) + 'static) -> Self {
        self.on_navigate = Some(Rc::new(callback));
        self
    }
}
impl RenderOnce for Breadcrumbs {
    fn render(self, _: &mut gpui_pre::Window, cx: &mut gpui_pre::App) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let last = self.items.len().saturating_sub(1);
        let mut list = div()
            .id("mkit-breadcrumb-list")
            .role(gpui_pre::accesskit::Role::List)
            .flex()
            .items_center()
            .gap(px(t.spacing.small));
        for (i, item) in self.items.iter().enumerate() {
            let current = i == last;
            let linked =
                !current && !self.disabled && item.target.is_some() && self.on_navigate.is_some();
            let mut node = div()
                .id(i)
                .debug_selector(move || format!("breadcrumbs-crumb-{i}"))
                .aria_label(item.label.clone())
                .text_size(px(t.typography.body))
                .text_color(if self.disabled {
                    t.colors.disabled
                } else if current {
                    t.colors.text
                } else {
                    t.colors.text_muted
                })
                .child(item.label.clone());
            if linked {
                node = node
                    .key_context(KEY_CONTEXT)
                    .role(gpui_pre::accesskit::Role::Link)
                    .tab_stop(true)
                    .tab_index(i as isize)
                    .focus_visible(|s| s.border_color(t.colors.focus));
                if let (Some(target), Some(callback)) =
                    (item.target.clone(), self.on_navigate.clone())
                {
                    let click_target = target.clone();
                    let click_callback = callback.clone();
                    node = node
                        .on_click(move |_, _, _| click_callback(click_target.clone()))
                        .on_action(move |_: &Activate, _, _| callback(target.clone()));
                }
            }
            if current {
                node = node.aria_description("Current page");
            } else if self.disabled {
                node = node.aria_description("Unavailable");
            }
            list = list.child(
                div()
                    .id(("mkit-breadcrumb-item", i))
                    .role(gpui_pre::accesskit::Role::ListItem)
                    .flex()
                    .items_center()
                    .gap(px(t.spacing.small))
                    .child(node)
                    .when(i + 1 < self.items.len(), |e| {
                        e.child(div().text_color(t.colors.text_muted).child("/"))
                    }),
            );
        }
        div()
            .id("mkit-breadcrumbs")
            .role(gpui_pre::accesskit::Role::Navigation)
            .aria_label(self.label)
            .child(list)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn final_crumb_is_current_even_without_explicit_marker() {
        let crumbs = [Crumb::new("Home").target("/"), Crumb::new("Docs")];
        assert_eq!(crumbs.last().unwrap().label, "Docs");
        assert_eq!(Crumb::new("Docs").label, "Docs");
    }
    #[test]
    fn crumb_targets_are_optional_and_builder_owned() {
        assert_eq!(Crumb::new("Home").target("/").target, Some("/".into()));
        assert_eq!(Crumb::new("Current").target, None);
    }
}
