//! Single-selection toggle group with typed value events.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, Render, Window, actions, div,
    prelude::*, px,
};
use mkit_core::theme::Theme;
pub const KEY_CONTEXT: &str = "MkitToggleGroup";
actions!(toggle_group, [Right, Left, Down, Up, First, Last]);
pub fn default_key_bindings() -> [gpui_pre::KeyBinding; 6] {
    [
        gpui_pre::KeyBinding::new("right", Right, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("left", Left, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("down", Down, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("up", Up, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("home", First, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("end", Last, Some(KEY_CONTEXT)),
    ]
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueChanged(pub Option<String>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}
impl Item {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self { value: value.into(), label: label.into(), disabled: false }
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}
pub struct ToggleGroup {
    label: String,
    items: Vec<Item>,
    value: Option<String>,
    controlled: bool,
    disabled: bool,
    allow_empty: bool,
    orientation: Orientation,
    active: usize,
    item_focus: Vec<FocusHandle>,
}
impl EventEmitter<ValueChanged> for ToggleGroup {}
impl ToggleGroup {
    pub fn new(label: impl Into<String>, items: Vec<Item>, default_value: Option<String>) -> Self {
        let active = default_value
            .as_ref()
            .and_then(|v| items.iter().position(|i| &i.value == v))
            .unwrap_or(0);
        Self {
            label: label.into(),
            items,
            value: default_value,
            controlled: false,
            disabled: false,
            allow_empty: false,
            orientation: Orientation::Horizontal,
            active,
            item_focus: Vec::new(),
        }
    }
    pub fn controlled(label: impl Into<String>, items: Vec<Item>, value: Option<String>) -> Self {
        let mut s = Self::new(label, items, value);
        s.controlled = true;
        s
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn allow_empty(mut self, v: bool) -> Self {
        self.allow_empty = v;
        self
    }
    pub fn orientation(mut self, v: Orientation) -> Self {
        self.orientation = v;
        self
    }
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
    pub fn set_value(&mut self, v: Option<String>, cx: &mut Context<Self>) {
        self.value = v;
        cx.notify()
    }
    fn request(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.disabled || self.items.get(index).is_none_or(|i| i.disabled) {
            return;
        }
        let next = if self.allow_empty && self.value.as_deref() == Some(&self.items[index].value) {
            None
        } else {
            Some(self.items[index].value.clone())
        };
        self.active = index;
        if !self.controlled {
            self.value = next.clone();
            cx.notify()
        }
        cx.emit(ValueChanged(next));
    }
    fn move_by(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.items.is_empty() {
            return;
        }
        let n = self.items.len();
        for step in 1..=n {
            let idx =
                (self.active as isize + delta * step as isize).rem_euclid(n as isize) as usize;
            if !self.items[idx].disabled {
                self.active = idx;
                self.request(idx, cx);
                if let Some(focus) = self.item_focus.get(idx) {
                    window.focus(focus, cx);
                }
                return;
            }
        }
    }
    fn right(&mut self, _: &Right, window: &mut Window, cx: &mut Context<Self>) {
        if self.orientation == Orientation::Horizontal {
            self.move_by(1, window, cx);
        }
    }
    fn left(&mut self, _: &Left, window: &mut Window, cx: &mut Context<Self>) {
        if self.orientation == Orientation::Horizontal {
            self.move_by(-1, window, cx);
        }
    }
    fn down(&mut self, _: &Down, window: &mut Window, cx: &mut Context<Self>) {
        if self.orientation == Orientation::Vertical {
            self.move_by(1, window, cx);
        }
    }
    fn up(&mut self, _: &Up, window: &mut Window, cx: &mut Context<Self>) {
        if self.orientation == Orientation::Vertical {
            self.move_by(-1, window, cx);
        }
    }
    fn first(&mut self, _: &First, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(i) = self.items.iter().position(|item| !item.disabled) {
            self.active = i;
            self.request(i, cx);
            if let Some(focus) = self.item_focus.get(i) {
                window.focus(focus, cx);
            }
        }
    }
    fn last(&mut self, _: &Last, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(i) = self.items.iter().rposition(|item| !item.disabled) {
            self.active = i;
            self.request(i, cx);
            if let Some(focus) = self.item_focus.get(i) {
                window.focus(focus, cx);
            }
        }
    }
}
impl Render for ToggleGroup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let entity = cx.entity();
        while self.item_focus.len() < self.items.len() {
            self.item_focus.push(cx.focus_handle());
        }
        self.item_focus.truncate(self.items.len());
        for (index, focus) in self.item_focus.iter_mut().enumerate() {
            let enabled = !self.disabled && !self.items[index].disabled;
            *focus = focus
                .clone()
                .tab_stop(enabled && index == self.active)
                .tab_index(if index == self.active { 0 } else { 1 });
        }
        let row = self.orientation == Orientation::Horizontal;
        let mut root = div()
            .id(("mkit-toggle-group", cx.entity().entity_id()))
            .key_context(KEY_CONTEXT)
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.label.clone())
            .when(self.disabled, |e| {
                e.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .flex()
            .when(row, |e| e.flex_row())
            .when(!row, |e| e.flex_col())
            .rounded(px(t.radii.medium))
            .border(px(t.borders.regular))
            .border_color(t.colors.border)
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::first))
            .on_action(cx.listener(Self::last));
        for (index, item) in self.items.iter().enumerate() {
            let selected = self.value.as_deref() == Some(item.value.as_str());
            let enabled = !self.disabled && !item.disabled;
            let entity = entity.clone();
            let element = div()
                .id(item.value.clone())
                .track_focus(&self.item_focus[index])
                .role(gpui_pre::accesskit::Role::Button)
                .aria_label(item.label.clone())
                .aria_toggled(if selected {
                    gpui_pre::accesskit::Toggled::True
                } else {
                    gpui_pre::accesskit::Toggled::False
                })
                .when(!enabled, |e| e.aria_description("Unavailable"))
                .when(!enabled, |e| {
                    e.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
                })
                .on_click(move |_, _, cx| entity.update(cx, |s, cx| s.request(index, cx)))
                .h(px(t.controls.medium))
                .px(px(t.spacing.small))
                .flex()
                .items_center()
                .justify_center()
                .border(px(t.borders.hairline))
                .border_color(if selected { t.colors.accent } else { t.colors.border })
                .bg(if selected { t.colors.accent } else { t.colors.surface })
                .text_color(if !enabled {
                    t.colors.disabled
                } else if selected {
                    t.colors.accent_text
                } else {
                    t.colors.text
                })
                .text_size(px(t.typography.body))
                .focus_visible(|s| s.border_color(t.colors.focus))
                .child(item.label.clone());
            root = root.child(element)
        }
        root
    }
}

impl Focusable for ToggleGroup {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.item_focus
            .get(self.active)
            .cloned()
            .expect("toggle group focus handles initialize during render")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn group_models_allow_disabled_items_and_empty_selection() {
        let items = vec![Item::new("left", "Left"), Item::new("center", "Center").disabled(true)];
        let group = ToggleGroup::new("Alignment", items, None).allow_empty(true);
        assert_eq!(group.value(), None);
        assert!(group.items[1].disabled);
        assert!(group.allow_empty);
    }
}
