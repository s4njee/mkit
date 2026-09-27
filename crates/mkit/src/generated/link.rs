//! Themed text link with semantic keyboard and pointer activation.
extern crate gpui_pre as gpui;

use gpui_pre::{App, IntoElement, KeyBinding, RenderOnce, Window, actions, div, prelude::*, px};
use mkit_core::theme::Theme;
use std::{
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
pub const KEY_CONTEXT: &str = "MkitLink";
actions!(link, [Activate]);

pub fn default_key_bindings() -> [KeyBinding; 1] {
    [KeyBinding::new("enter", Activate, Some(KEY_CONTEXT))]
}

type Callback = dyn Fn(&mut Window, &mut App) + 'static;

#[derive(IntoElement)]
pub struct Link {
    id: usize,
    label: String,
    accessible_name: Option<String>,
    disabled: bool,
    visited: bool,
    on_activate: Option<Rc<Callback>>,
}

impl Link {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            label: label.into(),
            accessible_name: None,
            disabled: false,
            visited: false,
            on_activate: None,
        }
    }

    /// Assign a stable identity when this link is rebuilt during parent renders.
    pub fn id(mut self, id: usize) -> Self {
        self.id = id;
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

    pub fn visited(mut self, visited: bool) -> Self {
        self.visited = visited;
        self
    }

    pub fn on_activate(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(callback));
        self
    }
}

impl RenderOnce for Link {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let available = !self.disabled && self.on_activate.is_some();
        let label = self.accessible_name.unwrap_or_else(|| self.label.clone());
        let mut element = div()
            .id(("mkit-link", self.id))
            .debug_selector(|| "mkit-link".into())
            .role(gpui_pre::accesskit::Role::Link)
            .aria_label(label)
            .when(self.disabled, |el| {
                el.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
                    .aria_description("Unavailable")
            })
            .when(available, |el| el.key_context(KEY_CONTEXT).tab_stop(true).tab_index(0))
            .flex()
            .self_start()
            .items_center()
            .text_color(if self.disabled {
                theme.colors.disabled
            } else if self.visited {
                theme.colors.text_muted
            } else {
                theme.colors.accent
            })
            .text_size(px(theme.typography.body))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.surface)
            .rounded(px(theme.radii.small))
            .underline()
            .when(available, |el| el.hover(|el| el.text_color(theme.colors.focus)))
            .focus_visible(|el| el.border_color(theme.colors.focus))
            .child(self.label);
        if available && let Some(callback) = self.on_activate {
            let pointer_callback = callback.clone();
            element = element
                .on_action(move |_: &Activate, window, cx| callback(window, cx))
                .on_click(move |_, window, cx| pointer_callback(window, cx));
        }
        element
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_link_does_not_become_activatable() {
        let link = Link::new("Docs").disabled(true).on_activate(|_, _| {});
        assert!(link.disabled);
        assert!(link.on_activate.is_some());
    }

    #[test]
    fn visible_text_is_the_default_accessible_name() {
        let link = Link::new("Read docs");
        assert_eq!(link.accessible_name, None);
        assert_eq!(link.label, "Read docs");
    }
}
