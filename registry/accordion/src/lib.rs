//! Grouped disclosures with single-open or multiple-open state.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnyElement, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render,
    Window, actions, div, prelude::*, px,
};
use mkit_core::theme::Theme;

pub const KEY_CONTEXT: &str = "Accordion";
actions!(accordion, [Toggle, FocusNext, FocusPrevious]);
pub fn default_key_bindings() -> [KeyBinding; 4] {
    [
        KeyBinding::new("space", Toggle, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Toggle, Some(KEY_CONTEXT)),
        KeyBinding::new("tab", FocusNext, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-tab", FocusPrevious, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Single,
    Multiple,
}

pub struct Item {
    pub id: String,
    pub label: String,
    pub expanded: bool,
    pub disabled: bool,
    content: Box<dyn Fn() -> AnyElement>,
}
impl Item {
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
            disabled: false,
            content: Box::new(move || content().into_any_element()),
        }
    }
    pub fn expanded(mut self, value: bool) -> Self {
        self.expanded = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpandedChanged(pub Vec<String>);
impl EventEmitter<ExpandedChanged> for Accordion {}

pub struct Accordion {
    items: Vec<Item>,
    mode: Mode,
    open: Vec<String>,
    controlled: bool,
    focus: Vec<FocusHandle>,
}
impl Accordion {
    pub fn new(items: Vec<Item>, mode: Mode) -> Self {
        let open = items.iter().filter(|i| i.expanded).map(|i| i.id.clone()).collect();
        Self { items, mode, open, controlled: false, focus: Vec::new() }
    }
    pub fn controlled(items: Vec<Item>, mode: Mode, expanded_ids: Vec<String>) -> Self {
        Self { items, mode, open: expanded_ids, controlled: true, focus: Vec::new() }
    }
    pub fn expanded_ids(&self) -> &[String] {
        &self.open
    }
    pub fn set_expanded(&mut self, ids: Vec<String>, cx: &mut Context<Self>) {
        self.open = self.normalize(ids);
        cx.notify();
    }
    fn normalize(&self, ids: Vec<String>) -> Vec<String> {
        let mut result = Vec::new();
        for id in ids {
            if self.items.iter().any(|i| i.id == id) && !result.contains(&id) {
                result.push(id);
            }
        }
        if self.mode == Mode::Single {
            result.truncate(1);
        }
        result
    }
    fn request(&mut self, id: &str, cx: &mut Context<Self>) {
        if self.items.iter().find(|i| i.id == id).is_none_or(|i| i.disabled) {
            return;
        }
        let mut next = self.open.clone();
        if next.iter().any(|v| v == id) {
            next.retain(|v| v != id);
        } else {
            if self.mode == Mode::Single {
                next.clear();
            }
            next.push(id.to_owned());
        }
        if next == self.open {
            return;
        }
        if !self.controlled {
            self.open = next.clone();
        }
        cx.emit(ExpandedChanged(next));
        cx.notify();
    }
    fn focus_next(&mut self, _: &FocusNext, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }
    fn focus_previous(&mut self, _: &FocusPrevious, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }
}
impl Focusable for Accordion {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.first().cloned().expect("focus initialized")
    }
}
impl Render for Accordion {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        while self.focus.len() < self.items.len() {
            let index = self.focus.len();
            self.focus.push(cx.focus_handle().tab_index(0).tab_stop(!self.items[index].disabled));
        }
        let mut root = div()
            .id("mkit-accordion")
            .key_context(KEY_CONTEXT)
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label("Accordion")
            .w_full()
            .flex()
            .flex_col();
        for (index, item) in self.items.iter().enumerate() {
            let open = self.open.contains(&item.id);
            let id = item.id.clone();
            let label = item.label.clone();
            let disabled = item.disabled;
            let focus = self.focus[index].clone();
            let click_id = id.clone();
            root = root.child(
                div()
                    .id(format!("accordion-trigger-{id}"))
                    .key_context(KEY_CONTEXT)
                    .on_action(cx.listener(Self::focus_next))
                    .on_action(cx.listener(Self::focus_previous))
                    .track_focus(&focus)
                    .tab_stop(!disabled)
                    .debug_selector({
                        let s = format!("accordion-trigger-{id}");
                        move || s.clone()
                    })
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label(label.clone())
                    .aria_expanded(open)
                    .when(disabled, |el| {
                        el.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
                    })
                    .tab_index(if disabled { -1 } else { 0 })
                    .w_full()
                    .h(px(theme.controls.small))
                    .px(px(theme.spacing.medium))
                    .flex()
                    .items_center()
                    .justify_between()
                    .border_b(px(theme.borders.hairline))
                    .border_color(theme.colors.border)
                    .text_color(if disabled { theme.colors.disabled } else { theme.colors.text })
                    .text_size(px(theme.typography.body_emphasis))
                    .when(!disabled, |e| {
                        e.on_click(cx.listener({
                            let id = click_id.clone();
                            move |this, _, _, cx| this.request(&id, cx)
                        }))
                    })
                    .on_action(
                        cx.listener(move |this, _: &Toggle, _, cx| this.request(&click_id, cx)),
                    )
                    .child(label.clone())
                    .child(if open { "⌄" } else { "›" }),
            );
            if open {
                root = root.child(
                    div()
                        .id(format!("accordion-panel-{id}"))
                        .role(gpui_pre::accesskit::Role::Group)
                        .aria_label(label)
                        .px(px(theme.spacing.medium))
                        .py(px(theme.spacing.small))
                        .text_color(theme.colors.text)
                        .child((item.content)()),
                );
            }
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use std::{cell::RefCell, rc::Rc};

    fn items() -> Vec<Item> {
        vec![
            Item::new("one", "One", || div().child("First")),
            Item::new("two", "Two", || div().child("Second")),
        ]
    }

    #[gpui::test]
    fn keyboard_toggle_obeys_mode_and_controlled_contract(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (view, window) = cx.add_window_view(|_, _| Accordion::new(items(), Mode::Single));
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        window.simulate_keystrokes("space");
        assert_eq!(view.read_with(window, |a, _| a.expanded_ids().to_vec()), vec!["one"]);

        let events = Rc::new(RefCell::new(Vec::new()));
        let (controlled, window) =
            cx.add_window_view(|_, _| Accordion::controlled(items(), Mode::Multiple, Vec::new()));
        let out = events.clone();
        let _subscription = window.update(|_, cx| {
            cx.subscribe(&controlled, move |_, event: &ExpandedChanged, _| {
                out.borrow_mut().push(event.0.clone())
            })
        });
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            controlled.focus_handle(cx).focus(w, cx);
        });
        window.simulate_keystrokes("enter");
        assert!(controlled.read_with(window, |a, _| a.expanded_ids().is_empty()));
        assert_eq!(*events.borrow(), vec![vec!["one".to_string()]]);
    }

    #[gpui::test]
    fn tab_to_second_header_and_activate_uses_focused_item(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (view, window) = cx.add_window_view(|_, _| Accordion::new(items(), Mode::Multiple));
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        let (first_focus, second_focus) =
            view.read_with(window, |a, _| (a.focus[0].clone(), a.focus[1].clone()));
        assert!(window.update(|w, _| first_focus.is_focused(w)));
        window.simulate_keystrokes("tab enter");
        assert!(window.update(|w, _| second_focus.is_focused(w)));
        assert_eq!(view.read_with(window, |a, _| a.expanded_ids().to_vec()), vec!["two"]);
    }

    #[gpui::test]
    fn multiple_mode_keeps_independent_expanded_items(cx: &mut TestAppContext) {
        cx.update(|app| mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT));
        let (view, window) = cx.add_window_view(|_, _| Accordion::new(items(), Mode::Multiple));
        window.update(|w, cx| w.draw(cx).clear(cx));
        for selector in ["accordion-trigger-one", "accordion-trigger-two"] {
            let bounds = window.debug_bounds(selector).expect("accordion trigger is rendered");
            window.simulate_click(bounds.center(), gpui::Modifiers::default());
        }
        assert_eq!(view.read_with(window, |a, _| a.expanded_ids().to_vec()), vec!["one", "two"]);
    }
}
