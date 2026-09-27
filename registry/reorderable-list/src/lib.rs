//! Stateful flat list with pointer and keyboard reordering.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render, Window,
    actions, div, prelude::*, px,
};
use mkit_core::theme::Theme;

pub const KEY_CONTEXT: &str = "MkitReorderableList";
actions!(reorderable_list, [MoveUp, MoveDown]);

pub fn default_key_bindings() -> [KeyBinding; 2] {
    [
        KeyBinding::new("alt-up", MoveUp, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-down", MoveDown, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem {
    pub id: String,
    pub label: String,
    pub disabled: bool,
}

impl ListItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), disabled: false }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReorderRequested {
    pub id: String,
    pub from: usize,
    pub to: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reordered(pub ReorderRequested);
impl EventEmitter<ReorderRequested> for ReorderableList {}
impl EventEmitter<Reordered> for ReorderableList {}

#[derive(Clone)]
struct DragItem {
    id: String,
    label: String,
}
struct DragPreview {
    label: String,
}
impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        div()
            .w(px(180.0))
            .h(px(theme.controls.small))
            .px(px(theme.spacing.medium))
            .py(px(theme.spacing.small))
            .rounded(px(theme.radii.small))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .bg(theme.colors.elevated_surface)
            .text_color(theme.colors.text)
            .child(self.label.clone())
    }
}

pub struct ReorderableList {
    id: String,
    label: String,
    items: Vec<ListItem>,
    empty_text: Option<String>,
    active: Option<String>,
    controlled: bool,
    focus: Option<FocusHandle>,
}
impl ReorderableList {
    pub fn new(label: impl Into<String>, items: Vec<ListItem>) -> Self {
        let active = items.iter().find(|item| !item.disabled).map(|item| item.id.clone());
        Self {
            id: "mkit-reorderable-list".into(),
            label: label.into(),
            items,
            empty_text: None,
            active,
            controlled: false,
            focus: None,
        }
    }
    pub fn controlled(label: impl Into<String>, items: Vec<ListItem>) -> Self {
        Self { controlled: true, ..Self::new(label, items) }
    }
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }
    pub fn empty_text(mut self, text: impl Into<String>) -> Self {
        self.empty_text = Some(text.into());
        self
    }
    pub fn active_id(mut self, id: impl Into<String>) -> Self {
        self.active = Some(id.into());
        self
    }
    pub fn items(&self) -> &[ListItem] {
        &self.items
    }
    pub fn active_id_value(&self) -> Option<&str> {
        self.active.as_deref()
    }
    pub fn set_items(&mut self, items: Vec<ListItem>, cx: &mut Context<Self>) {
        if !items.iter().any(|item| self.active.as_deref() == Some(&item.id) && !item.disabled) {
            self.active = items.iter().find(|item| !item.disabled).map(|item| item.id.clone());
        }
        self.items = items;
        cx.notify();
    }
    pub fn set_active_id(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        self.active =
            id.filter(|id| self.items.iter().any(|item| item.id == *id && !item.disabled));
        cx.notify();
    }
    fn reorder(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some(id) = self.active.clone() else { return };
        let Some(from) = self.items.iter().position(|item| item.id == id && !item.disabled) else {
            return;
        };
        let mut to = from as isize + delta;
        while (0..self.items.len() as isize).contains(&to) && self.items[to as usize].disabled {
            to += delta;
        }
        if !(0..self.items.len() as isize).contains(&to) {
            return;
        }
        self.move_to(id, from, to as usize, cx);
    }
    fn move_to(&mut self, id: String, from: usize, to: usize, cx: &mut Context<Self>) {
        if from == to
            || from >= self.items.len()
            || to >= self.items.len()
            || self.items[from].disabled
            || self.items[to].disabled
        {
            return;
        }
        let event = ReorderRequested { id: id.clone(), from, to };
        if !self.controlled {
            let item = self.items.remove(from);
            self.items.insert(to, item);
            self.active = Some(id);
            cx.emit(Reordered(event.clone()));
        }
        cx.emit(event);
        cx.notify();
    }
    fn move_before(&mut self, drag: &DragItem, target_id: &str, cx: &mut Context<Self>) {
        let Some(from) = self.items.iter().position(|item| item.id == drag.id) else {
            return;
        };
        let Some(target) = self.items.iter().position(|item| item.id == target_id) else {
            return;
        };
        if from == target || self.items[from].disabled || self.items[target].disabled {
            return;
        }
        let to = if from < target { target - 1 } else { target };
        if from != to {
            self.move_to(drag.id.clone(), from, to, cx);
        }
    }
    fn on_move_up(&mut self, _: &MoveUp, _: &mut Window, cx: &mut Context<Self>) {
        self.reorder(-1, cx);
    }
    fn on_move_down(&mut self, _: &MoveDown, _: &mut Window, cx: &mut Context<Self>) {
        self.reorder(1, cx);
    }
}
impl Focusable for ReorderableList {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone().expect("focus initializes during render")
    }
}
impl Render for ReorderableList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(0)).clone();
        let mut root = div()
            .id(self.id.clone())
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .on_action(cx.listener(Self::on_move_up))
            .on_action(cx.listener(Self::on_move_down))
            .role(gpui_pre::accesskit::Role::List)
            .aria_label(self.label.clone())
            .w_full()
            .flex()
            .flex_col()
            .rounded(px(theme.radii.small))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .bg(theme.colors.surface);
        if self.items.is_empty() {
            if let Some(text) = &self.empty_text {
                root = root.child(
                    div()
                        .px(px(theme.spacing.medium))
                        .py(px(theme.spacing.small))
                        .text_color(theme.colors.text_muted)
                        .child(text.clone()),
                );
            }
            return root;
        }
        for (index, item) in self.items.iter().enumerate() {
            let active = self.active.as_deref() == Some(item.id.as_str());
            let item_id = item.id.clone();
            let row_id = format!("{}-{}", self.id, item.id);
            let row_selector = row_id.clone();
            let mut row = div()
                .id(row_id)
                .debug_selector(move || row_selector.clone())
                .key_context(KEY_CONTEXT)
                .role(gpui_pre::accesskit::Role::ListItem)
                .aria_label(item.label.clone())
                .aria_position_in_set(index + 1)
                .aria_size_of_set(self.items.len())
                .tab_index(if active { 0 } else { -1 })
                .h(px(theme.controls.small))
                .px(px(theme.spacing.medium))
                .flex()
                .items_center()
                .text_color(if item.disabled { theme.colors.disabled } else { theme.colors.text })
                .bg(if active { theme.colors.elevated_surface } else { theme.colors.surface })
                .when(active, |e| {
                    e.border_l(px(theme.borders.strong)).border_color(theme.colors.focus)
                })
                .when(!item.disabled, |e| {
                    e.on_click(cx.listener(move |this, _, _, cx| {
                        this.active = Some(item_id.clone());
                        cx.notify();
                    }))
                })
                .child(div().flex_1().child(item.label.clone()));
            let drag = DragItem { id: item.id.clone(), label: item.label.clone() };
            let target = item.id.clone();
            if !item.disabled {
                let hover_target = target.clone();
                let drop_target = target.clone();
                row = row
                    .drag_over::<DragItem>(move |style, dragged, _, _| {
                        if dragged.id != hover_target {
                            style
                                .border_t(px(theme.borders.strong))
                                .border_color(theme.colors.accent)
                        } else {
                            style
                        }
                    })
                    .on_drag(drag.clone(), |drag, _position, _, cx| {
                        cx.new(|_| DragPreview { label: drag.label.clone() })
                    })
                    .can_drop(move |value, _, _| {
                        value
                            .downcast_ref::<DragItem>()
                            .is_some_and(|dragged| dragged.id != drop_target)
                    })
                    .on_drop(cx.listener(move |this, dragged: &DragItem, _, cx| {
                        this.move_before(dragged, &target, cx)
                    }));
            }
            root = root.child(row);
        }
        root
    }
}

pub fn reorder_indices(items: &[ListItem], from: usize, to: usize) -> Option<Vec<ListItem>> {
    if from >= items.len()
        || to >= items.len()
        || from == to
        || items[from].disabled
        || items[to].disabled
    {
        return None;
    }
    let mut reordered = items.to_vec();
    let item = reordered.remove(from);
    reordered.insert(to, item);
    Some(reordered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Modifiers, MouseButton, TestAppContext};
    use std::{cell::RefCell, rc::Rc};
    #[test]
    fn indices_reorder_preserving_neighbor_order() {
        let items = vec![ListItem::new("a", "A"), ListItem::new("b", "B"), ListItem::new("c", "C")];
        let result = reorder_indices(&items, 0, 2).unwrap();
        assert_eq!(result.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(), ["b", "c", "a"]);
    }
    #[test]
    fn invalid_and_disabled_moves_are_rejected() {
        let items = vec![ListItem::new("a", "A"), ListItem::new("b", "B").disabled(true)];
        assert!(reorder_indices(&items, 0, 1).is_none());
        assert!(reorder_indices(&items, 3, 0).is_none());
    }

    fn items() -> Vec<ListItem> {
        vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Beta"), ListItem::new("c", "Gamma")]
    }

    #[gpui::test]
    fn keyboard_move_emits_and_updates_uncontrolled_order(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (view, window) = cx.add_window_view(|_, _| ReorderableList::new("Tasks", items()));
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        let requests = Rc::new(RefCell::new(Vec::new()));
        let applied = Rc::new(RefCell::new(Vec::new()));
        let request_log = requests.clone();
        let applied_log = applied.clone();
        let _r = window.update(|_, cx| {
            cx.subscribe(&view, move |_, event: &ReorderRequested, _| {
                request_log.borrow_mut().push(event.clone())
            })
        });
        let _a = window.update(|_, cx| {
            cx.subscribe(&view, move |_, event: &Reordered, _| {
                applied_log.borrow_mut().push(event.clone())
            })
        });
        window.simulate_keystrokes("alt-down");
        assert_eq!(
            view.read_with(window, |list, _| list
                .items
                .iter()
                .map(|item| item.id.clone())
                .collect::<Vec<_>>()),
            ["b", "a", "c"]
        );
        assert_eq!(
            requests.borrow().as_slice(),
            &[ReorderRequested { id: "a".into(), from: 0, to: 1 }]
        );
        assert_eq!(
            applied.borrow().as_slice(),
            &[Reordered(ReorderRequested { id: "a".into(), from: 0, to: 1 })]
        );
    }

    #[gpui::test]
    fn controlled_move_requests_owner_update_without_mutating(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (view, window) =
            cx.add_window_view(|_, _| ReorderableList::controlled("Tasks", items()));
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        let requests = Rc::new(RefCell::new(Vec::new()));
        let log = requests.clone();
        let _r = window.update(|_, cx| {
            cx.subscribe(&view, move |_, event: &ReorderRequested, _| {
                log.borrow_mut().push(event.clone())
            })
        });
        window.simulate_keystrokes("alt-down");
        assert_eq!(
            view.read_with(window, |list, _| list
                .items
                .iter()
                .map(|item| item.id.clone())
                .collect::<Vec<_>>()),
            ["a", "b", "c"]
        );
        assert_eq!(
            requests.borrow().as_slice(),
            &[ReorderRequested { id: "a".into(), from: 0, to: 1 }]
        );
    }

    #[gpui::test]
    fn pointer_drag_moves_row_before_drop_target(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (view, window) = cx.add_window_view(|_, _| ReorderableList::new("Tasks", items()));
        window.update(|w, cx| w.draw(cx).clear(cx));
        let from = window.debug_bounds("mkit-reorderable-list-a").expect("first row").center();
        let to = window.debug_bounds("mkit-reorderable-list-c").expect("third row").center();
        window.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
        window.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::default());
        window.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
        assert_eq!(
            view.read_with(window, |list, _| list
                .items
                .iter()
                .map(|item| item.id.clone())
                .collect::<Vec<_>>()),
            ["b", "a", "c"]
        );
    }
}
