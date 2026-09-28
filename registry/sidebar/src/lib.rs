//! Labeled navigation landmark with link-style items and a current-page marker.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement, Render, Rgba, Window,
    actions, div, prelude::*, px,
};
use mkit_core::theme::Theme;

/// Colours derived from theme tokens for the shadcn-style sidebar. The
/// shadcn themes mix `text` into `background` the same way DropdownMenu,
/// ContextMenu, MenuBar and TagInput derive their active rows; other themes
/// keep solid theme roles so high contrast stays high contrast.
struct Palette {
    /// Pane background (shadcn "muted").
    pane: Rgba,
    /// Hover and current-page row fill (shadcn "accent", one step above the pane).
    active: Rgba,
    /// Text colour on the current-page row.
    active_text: Rgba,
    /// Trailing-edge divider.
    divider: Rgba,
    /// Whether hover is shown as a fill (shadcn) or as a border (other themes).
    fill_hover: bool,
}
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    let mix = |f: f32, b: f32| f * weight + b * (1.0 - weight);
    Rgba {
        r: mix(foreground.r, base.r),
        g: mix(foreground.g, base.g),
        b: mix(foreground.b, base.b),
        a: 1.0,
    }
}
fn palette(t: &Theme) -> Palette {
    let c = t.colors;
    let (weight, dark) = match t.name {
        "shadcn-light" => (0.04, false),
        "shadcn-dark" => (0.12, true),
        _ => {
            return Palette {
                pane: c.surface,
                active: c.accent,
                active_text: c.accent_text,
                divider: c.border,
                fill_hover: false,
            };
        }
    };
    let pane = mix(c.text, c.background, weight);
    Palette {
        pane,
        active: mix(c.text, pane, weight),
        active_text: c.text,
        divider: if dark { Rgba { a: 0.1, ..c.text } } else { c.border },
        fill_hover: true,
    }
}

pub const KEY_CONTEXT: &str = "MkitSidebar";
actions!(sidebar, [Activate]);
pub fn default_key_bindings() -> [gpui_pre::KeyBinding; 1] {
    [gpui_pre::KeyBinding::new("enter", Activate, Some(KEY_CONTEXT))]
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

pub struct Sidebar {
    label: String,
    items: Vec<Item>,
    value: Option<String>,
    controlled: bool,
    disabled: bool,
    orientation: Orientation,
    item_focus: Vec<FocusHandle>,
}
impl EventEmitter<ValueChanged> for Sidebar {}
impl Sidebar {
    pub fn new(label: impl Into<String>, items: Vec<Item>, default_value: Option<String>) -> Self {
        let value = default_value.filter(|v| items.iter().any(|i| &i.value == v && !i.disabled));
        Self {
            label: label.into(),
            items,
            value,
            controlled: false,
            disabled: false,
            orientation: Orientation::Vertical,
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
    pub fn orientation(mut self, v: Orientation) -> Self {
        self.orientation = v;
        self
    }
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
    pub fn set_value(&mut self, v: Option<String>, cx: &mut Context<Self>) {
        self.value = v.filter(|value| self.items.iter().any(|i| &i.value == value && !i.disabled));
        cx.notify();
    }
    fn enabled(&self, index: usize) -> bool {
        !self.disabled && self.items.get(index).is_some_and(|item| !item.disabled)
    }
    fn request(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.enabled(index) {
            return;
        }
        let next = Some(self.items[index].value.clone());
        if !self.controlled {
            self.value = next.clone();
            cx.notify();
        }
        cx.emit(ValueChanged(next));
    }
    fn activate_focused(&mut self, _: &Activate, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.item_focus.iter().position(|focus| focus.is_focused(window)) {
            self.request(index, cx);
        }
    }
}
impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let entity = cx.entity();
        while self.item_focus.len() < self.items.len() {
            self.item_focus.push(cx.focus_handle());
        }
        self.item_focus.truncate(self.items.len());
        for (index, focus) in self.item_focus.iter_mut().enumerate() {
            let enabled = !self.disabled && !self.items[index].disabled;
            *focus = focus.clone().tab_stop(enabled).tab_index(if enabled { 0 } else { -1 });
        }
        let row = self.orientation == Orientation::Horizontal;
        let p = palette(&t);
        let hairline = px(t.borders.hairline);
        let mut root = div()
            .id(("mkit-sidebar", cx.entity().entity_id()))
            .key_context(KEY_CONTEXT)
            .role(gpui_pre::accesskit::Role::Navigation)
            .aria_label(self.label.clone())
            .flex()
            .when(row, |e| e.flex_row().border_b(hairline))
            .when(!row, |e| e.flex_col().border_r(hairline))
            .gap(px(t.spacing.xsmall))
            .p(px(t.spacing.small))
            .bg(p.pane)
            .border_color(p.divider)
            .on_action(cx.listener(Self::activate_focused));
        for (index, item) in self.items.iter().enumerate() {
            let current = self.value.as_deref() == Some(item.value.as_str());
            let enabled = self.enabled(index);
            let entity = entity.clone();
            let focus = self.item_focus[index].clone();
            let (active, fill_hover, border) = (p.active, p.fill_hover, t.colors.border);
            let element = div()
                .id(item.value.clone())
                .debug_selector(move || format!("sidebar-item-{index}"))
                .track_focus(&focus)
                .tab_stop(enabled)
                .tab_index(if enabled { 0 } else { -1 })
                .role(gpui_pre::accesskit::Role::Link)
                .aria_label(item.label.clone())
                .when(current, |e| e.aria_description("Current page"))
                .when(!enabled, |e| e.aria_description("Unavailable"))
                .on_click(move |_, _, cx| {
                    entity.update(cx, |sidebar, cx| sidebar.request(index, cx))
                })
                .when(!row, |e| e.w_full())
                .h(px(t.controls.small))
                .px(px(t.spacing.small))
                .flex()
                .items_center()
                .gap(px(t.spacing.small))
                .rounded(px(t.radii.small))
                .border(hairline)
                .border_color(gpui_pre::transparent_black())
                .when(current, |e| e.font_weight(FontWeight::MEDIUM))
                // A disabled current page in the solid-fill themes keeps its
                // marker as a `disabled` outline: disabled text on the solid
                // accent fill would be illegible.
                .when(current && (enabled || fill_hover), |e| e.bg(p.active))
                .when(current && !enabled && !fill_hover, |e| e.border_color(t.colors.disabled))
                .text_color(if !enabled && !fill_hover {
                    t.colors.disabled
                } else if current {
                    p.active_text
                } else {
                    t.colors.text
                })
                .when(!enabled && fill_hover, |e| e.opacity(0.5))
                .when(enabled && !current, |e| {
                    e.hover(move |s| if fill_hover { s.bg(active) } else { s.border_color(border) })
                })
                .text_size(px(t.typography.body))
                .focus_visible(|s| s.border_color(t.colors.focus))
                .child(item.label.clone());
            root = root.child(element);
        }
        root
    }
}
impl Focusable for Sidebar {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.item_focus
            .iter()
            .find(|focus| focus.tab_stop)
            .cloned()
            .or_else(|| self.item_focus.first().cloned())
            .expect("sidebar focus handles initialize during render")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_or_disabled_initial_value_is_not_current() {
        let items =
            vec![Item::new("home", "Home"), Item::new("settings", "Settings").disabled(true)];
        assert_eq!(Sidebar::new("Primary", items.clone(), Some("settings".into())).value(), None);
        assert_eq!(Sidebar::new("Primary", items, Some("missing".into())).value(), None);
    }
    #[test]
    fn controlled_mode_keeps_owner_value_until_set() {
        let items = vec![Item::new("home", "Home"), Item::new("settings", "Settings")];
        let sidebar = Sidebar::controlled("Primary", items, Some("home".into()));
        assert_eq!(sidebar.value(), Some("home"));
        assert!(sidebar.controlled);
    }
}
