//! Single collapsible region following the WAI-ARIA Disclosure pattern.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnyElement, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render,
    Window, actions, div, prelude::*, px,
};
use mkit_core::theme::Theme;

pub const KEY_CONTEXT: &str = "Disclosure";
actions!(disclosure, [Toggle]);

pub fn default_key_bindings() -> [KeyBinding; 2] {
    [
        KeyBinding::new("space", Toggle, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Toggle, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpandedChanged(pub bool);
impl EventEmitter<ExpandedChanged> for Disclosure {}

pub struct Disclosure {
    id: String,
    label: String,
    expanded: bool,
    controlled: bool,
    disabled: bool,
    content: Box<dyn Fn() -> AnyElement>,
    focus: Option<FocusHandle>,
}

impl Disclosure {
    pub fn new<E>(
        id: impl Into<String>,
        label: impl Into<String>,
        content: impl Fn() -> E + 'static,
    ) -> Self
    where
        E: IntoElement + 'static,
    {
        Self {
            id: id.into(),
            label: label.into(),
            expanded: false,
            controlled: false,
            disabled: false,
            content: Box::new(move || content().into_any_element()),
            focus: None,
        }
    }

    pub fn controlled<E>(
        id: impl Into<String>,
        label: impl Into<String>,
        expanded: bool,
        content: impl Fn() -> E + 'static,
    ) -> Self
    where
        E: IntoElement + 'static,
    {
        let mut this = Self::new(id, label, content);
        this.expanded = expanded;
        this.controlled = true;
        this
    }

    pub fn expanded(mut self, value: bool) -> Self {
        self.expanded = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }
    pub fn set_expanded(&mut self, value: bool, cx: &mut Context<Self>) {
        self.expanded = value;
        cx.notify();
    }

    fn request_toggle(&mut self, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let next = !self.expanded;
        if next == self.expanded {
            return;
        }
        if !self.controlled {
            self.expanded = next;
        }
        cx.emit(ExpandedChanged(next));
        cx.notify();
    }
    fn toggle(&mut self, _: &Toggle, _: &mut Window, cx: &mut Context<Self>) {
        self.request_toggle(cx);
    }
}

impl Focusable for Disclosure {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("focus initialized during render")
    }
}

impl Render for Disclosure {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focus = self
            .focus
            .get_or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(!self.disabled))
            .clone();
        let id = self.id.clone();
        let label = self.label.clone();
        let expanded = self.expanded;
        let disabled = self.disabled;
        let mut root = div()
            .id(format!("disclosure-{id}"))
            .key_context(KEY_CONTEXT)
            .flex()
            .flex_col()
            .w_full()
            .on_action(cx.listener(Self::toggle));
        root = root.child(
            div()
                .id(format!("disclosure-trigger-{id}"))
                .track_focus(&focus)
                .debug_selector({
                    let s = format!("disclosure-trigger-{id}");
                    move || s.clone()
                })
                .role(gpui_pre::accesskit::Role::Button)
                .aria_label(label.clone())
                .aria_expanded(expanded)
                .when(disabled, |e| {
                    e.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
                })
                .tab_index(if disabled { -1 } else { 0 })
                .w_full()
                .h(px(theme.controls.small))
                .px(px(theme.spacing.medium))
                .flex()
                .items_center()
                .justify_between()
                .text_color(if disabled { theme.colors.disabled } else { theme.colors.text })
                .text_size(px(theme.typography.body_emphasis))
                .bg(theme.colors.surface)
                .when(!disabled, |e| {
                    e.on_click(cx.listener(|this, _, _, cx| this.request_toggle(cx)))
                })
                .child(label.clone())
                .child(if expanded { "⌄" } else { "›" }),
        );
        if expanded {
            root = root.child(
                div()
                    .id(format!("disclosure-panel-{id}"))
                    .role(gpui_pre::accesskit::Role::Group)
                    .aria_label(label)
                    .pl(px(theme.spacing.medium))
                    .py(px(theme.spacing.small))
                    .text_color(theme.colors.text)
                    .child((self.content)()),
            );
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use std::{cell::RefCell, rc::Rc};

    #[gpui::test]
    fn keyboard_toggles_and_controlled_mode_waits_for_owner(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (view, window) =
            cx.add_window_view(|_, _| Disclosure::new("notes", "Notes", || div().child("Details")));
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        window.simulate_keystrokes("space");
        assert!(view.read_with(window, |d, _| d.is_expanded()));

        let events = Rc::new(RefCell::new(Vec::new()));
        let (controlled, window) = cx.add_window_view(|_, _| {
            Disclosure::controlled("controlled", "Controlled", false, || div())
        });
        let out = events.clone();
        let _subscription = window.update(|_, cx| {
            cx.subscribe(&controlled, move |_, event: &ExpandedChanged, _| {
                out.borrow_mut().push(event.0)
            })
        });
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            controlled.focus_handle(cx).focus(w, cx);
        });
        window.simulate_keystrokes("enter");
        assert!(!controlled.read_with(window, |d, _| d.is_expanded()));
        assert_eq!(*events.borrow(), vec![true]);
    }

    #[gpui::test]
    fn disabled_disclosure_ignores_activation(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (view, window) =
            cx.add_window_view(|_, _| Disclosure::new("locked", "Locked", || div()).disabled(true));
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        window.simulate_keystrokes("space");
        assert!(!view.read_with(window, |d, _| d.is_expanded()));
    }
}
