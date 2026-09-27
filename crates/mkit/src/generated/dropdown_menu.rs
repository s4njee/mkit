//! Stateful command menu with checkable items and visible shortcut labels.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Anchor, AnchoredPositionMode, Context, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyBinding, MouseDownEvent, Render, WeakFocusHandle, Window, actions, anchored, deferred, div,
    point, prelude::*, px,
};
use mkit_core::theme::Theme;
fn active_row_colors(theme: Theme) -> (gpui_pre::Rgba, gpui_pre::Rgba) {
    let weight = match theme.name {
        "shadcn-light" => 0.04,
        "shadcn-dark" => 0.12,
        _ => return (theme.colors.accent, theme.colors.accent_text),
    };
    let text = theme.colors.text;
    let background = theme.colors.background;
    let mix = |foreground: f32, base: f32| foreground * weight + base * (1.0 - weight);
    (
        gpui_pre::Rgba {
            r: mix(text.r, background.r),
            g: mix(text.g, background.g),
            b: mix(text.b, background.b),
            a: 1.0,
        },
        text,
    )
}
pub const KEY_CONTEXT: &str = "DropdownMenu";
actions!(
    dropdown_menu,
    [MoveNext, MovePrevious, Activate, Dismiss, OpenSubmenu, CloseSubmenu, First, Last]
);
pub fn default_key_bindings() -> [KeyBinding; 9] {
    [
        KeyBinding::new("down", MoveNext, Some(KEY_CONTEXT)),
        KeyBinding::new("up", MovePrevious, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Activate, Some(KEY_CONTEXT)),
        KeyBinding::new("space", Activate, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", Dismiss, Some(KEY_CONTEXT)),
        KeyBinding::new("right", OpenSubmenu, Some(KEY_CONTEXT)),
        KeyBinding::new("left", CloseSubmenu, Some(KEY_CONTEXT)),
        KeyBinding::new("home", First, Some(KEY_CONTEXT)),
        KeyBinding::new("end", Last, Some(KEY_CONTEXT)),
    ]
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub shortcut: Option<String>,
    pub disabled: bool,
    pub checked: Option<bool>,
    pub children: Vec<MenuItem>,
}
impl MenuItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            disabled: false,
            checked: None,
            children: vec![],
        }
    }
    pub fn shortcut(mut self, s: impl Into<String>) -> Self {
        self.shortcut = Some(s.into());
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn checked(mut self, v: bool) -> Self {
        self.checked = Some(v);
        self
    }
    pub fn submenu(mut self, items: Vec<MenuItem>) -> Self {
        self.children = items;
        self
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenChanged(pub bool);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemSelected(pub String);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedChanged(pub String, pub bool);
impl EventEmitter<OpenChanged> for DropdownMenu {}
impl EventEmitter<ItemSelected> for DropdownMenu {}
impl EventEmitter<CheckedChanged> for DropdownMenu {}
pub struct DropdownMenu {
    items: Vec<MenuItem>,
    open: bool,
    controlled: bool,
    // `path[d]` is the parent row opening pane d + 1; `active[d]` is active row in pane d.
    path: Vec<usize>,
    active: Vec<usize>,
    focus: Option<FocusHandle>,
    anchor: Option<gpui_pre::Point<gpui_pre::Pixels>>,
    return_focus_to: Option<WeakFocusHandle>,
    last_open: bool,
    captured_focus: Option<WeakFocusHandle>,
}
impl DropdownMenu {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            open: false,
            controlled: false,
            path: vec![],
            active: vec![0],
            focus: None,
            anchor: None,
            return_focus_to: None,
            last_open: false,
            captured_focus: None,
        }
    }
    pub fn controlled(items: Vec<MenuItem>, open: bool) -> Self {
        Self {
            items,
            open,
            controlled: true,
            path: vec![],
            active: vec![0],
            focus: None,
            anchor: None,
            return_focus_to: None,
            last_open: false,
            captured_focus: None,
        }
    }
    pub fn set_open(&mut self, v: bool, cx: &mut Context<Self>) {
        self.open = v;
        if v {
            self.reset_navigation();
        }
        cx.notify();
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    /// Set the window-coordinate point used by the deferred menu surface.
    pub fn anchor_at(mut self, position: gpui_pre::Point<gpui_pre::Pixels>) -> Self {
        self.anchor = Some(position);
        self
    }
    /// Update the window-coordinate point before opening or repositioning the menu.
    pub fn set_anchor(
        &mut self,
        position: gpui_pre::Point<gpui_pre::Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.anchor = Some(position);
        cx.notify();
    }
    /// Return focus to this host trigger or invoking element after the menu closes.
    pub fn return_focus_to(mut self, handle: &FocusHandle) -> Self {
        self.return_focus_to = Some(handle.downgrade());
        self
    }
    fn outside_down(&mut self, _: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.set_visibility(false, cx);
        }
    }
    fn restore_focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(handle) = self.return_focus_to.as_ref().and_then(WeakFocusHandle::upgrade) {
            handle.focus(window, cx);
        } else if let Some(handle) = self.captured_focus.as_ref().and_then(WeakFocusHandle::upgrade)
        {
            handle.focus(window, cx);
        }
    }
    fn reset_navigation(&mut self) {
        self.path.clear();
        self.active.clear();
        let first = Self::enabled_index(&self.items, 0, true).unwrap_or(0);
        self.active.push(first);
    }
    fn pane(&self, depth: usize) -> &[MenuItem] {
        let mut items = self.items.as_slice();
        for parent in self.path.iter().take(depth) {
            let Some(item) = items.get(*parent) else { return &[] };
            items = &item.children;
        }
        items
    }
    fn pane_mut(&mut self, depth: usize) -> &mut [MenuItem] {
        fn descend<'a>(items: &'a mut [MenuItem], path: &[usize]) -> &'a mut [MenuItem] {
            if path.is_empty() {
                return items;
            }
            let index = path[0];
            if index >= items.len() {
                return &mut [];
            }
            descend(&mut items[index].children, &path[1..])
        }
        descend(&mut self.items, &self.path[..depth.min(self.path.len())])
    }
    fn active_index(&self, depth: usize) -> usize {
        *self.active.get(depth).unwrap_or(&0)
    }
    fn set_active(&mut self, depth: usize, index: usize) {
        while self.active.len() <= depth {
            self.active.push(0);
        }
        self.active[depth] = index;
        self.path.truncate(depth);
        self.active.truncate(depth + 1);
    }
    fn enabled_index(items: &[MenuItem], start: usize, forward: bool) -> Option<usize> {
        if items.is_empty() {
            return None;
        }
        (0..items.len())
            .map(|step| {
                if forward {
                    (start + step) % items.len()
                } else {
                    (start + items.len() - step % items.len()) % items.len()
                }
            })
            .find(|i| !items[*i].disabled)
    }
    fn move_active(&mut self, forward: bool, cx: &mut Context<Self>) {
        let depth = self.path.len();
        let items = self.pane(depth);
        if items.is_empty() {
            return;
        }
        let current = self.active_index(depth);
        let start = if forward {
            (current + 1) % items.len()
        } else {
            (current + items.len() - 1) % items.len()
        };
        if let Some(next) = Self::enabled_index(items, start, forward) {
            self.set_active(depth, next);
            cx.notify();
        }
    }
    fn jump(&mut self, last: bool, cx: &mut Context<Self>) {
        let depth = self.path.len();
        if let Some(index) = Self::enabled_index(
            self.pane(depth),
            if last { self.pane(depth).len().saturating_sub(1) } else { 0 },
            !last,
        ) {
            self.set_active(depth, index);
            cx.notify();
        }
    }
    fn set_visibility(&mut self, v: bool, cx: &mut Context<Self>) {
        if !self.controlled {
            self.open = v;
        }
        if !v {
            self.reset_navigation();
        }
        cx.emit(OpenChanged(v));
        cx.notify();
    }
    fn next(&mut self, _: &MoveNext, _: &mut Window, cx: &mut Context<Self>) {
        self.move_active(true, cx);
    }
    fn previous(&mut self, _: &MovePrevious, _: &mut Window, cx: &mut Context<Self>) {
        self.move_active(false, cx);
    }
    fn first(&mut self, _: &First, _: &mut Window, cx: &mut Context<Self>) {
        self.jump(false, cx);
    }
    fn last(&mut self, _: &Last, _: &mut Window, cx: &mut Context<Self>) {
        self.jump(true, cx);
    }
    fn open_submenu(&mut self, _: &OpenSubmenu, _: &mut Window, cx: &mut Context<Self>) {
        self.open_active_submenu(cx);
    }
    fn close_submenu(&mut self, _: &CloseSubmenu, _: &mut Window, cx: &mut Context<Self>) {
        self.close_active_submenu(cx);
    }
    fn open_active_submenu(&mut self, cx: &mut Context<Self>) {
        let depth = self.path.len();
        let index = self.active_index(depth);
        if self
            .pane(depth)
            .get(index)
            .is_some_and(|item| !item.disabled && !item.children.is_empty())
        {
            self.path.push(index);
            let children = self.pane(depth + 1);
            let first = Self::enabled_index(children, 0, true).unwrap_or(0);
            self.active.push(first);
            cx.notify();
        }
    }
    fn close_active_submenu(&mut self, cx: &mut Context<Self>) {
        if !self.path.is_empty() {
            self.path.pop();
            self.active.pop();
            cx.notify();
        }
    }
    fn activate_at(&mut self, depth: usize, index: usize, cx: &mut Context<Self>) {
        self.set_active(depth, index);
        let Some(item) = self.pane(depth).get(index) else { return };
        if item.disabled {
            return;
        }
        if !item.children.is_empty() {
            self.open_active_submenu(cx);
            return;
        }
        let id = item.id.clone();
        let checked = item.checked;
        if let Some(value) = checked {
            let item = &mut self.pane_mut(depth)[index];
            item.checked = Some(!value);
            cx.emit(CheckedChanged(id, !value));
            cx.notify();
        } else {
            cx.emit(ItemSelected(id));
            self.set_visibility(false, cx);
        }
    }
    fn activate(&mut self, _: &Activate, _: &mut Window, cx: &mut Context<Self>) {
        let depth = self.path.len();
        let index = self.active_index(depth);
        self.activate_at(depth, index, cx);
    }
    fn dismiss(&mut self, _: &Dismiss, _: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.set_visibility(false, cx);
        }
    }
    fn pointer_move(
        &mut self,
        depth: usize,
        index: usize,
        _: &gpui_pre::MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.pane(depth).get(index).is_some_and(|item| item.disabled) {
            self.set_active(depth, index);
            if self.pane(depth).get(index).is_some_and(|item| item.children.is_empty()) {
                self.path.truncate(depth);
                self.active.truncate(depth + 1);
            }
            cx.notify();
        }
    }
}
impl Focusable for DropdownMenu {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}
impl Render for DropdownMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        if self.open && !self.last_open {
            self.captured_focus = None;
            if self.return_focus_to.is_none()
                && let Some(previous) = window.focused(cx)
                && previous != focus
            {
                self.captured_focus = Some(previous.downgrade());
            }
        } else if !self.open && self.last_open {
            self.restore_focus(window, cx);
        }
        self.last_open = self.open;
        let mut root = div()
            .id(("mkit-menu", cx.entity().entity_id()))
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::next))
            .on_action(cx.listener(Self::previous))
            .on_action(cx.listener(Self::activate))
            .on_action(cx.listener(Self::dismiss))
            .on_action(cx.listener(Self::open_submenu))
            .on_action(cx.listener(Self::close_submenu))
            .on_action(cx.listener(Self::first))
            .on_action(cx.listener(Self::last))
            .when(self.open, |d| {
                d.track_focus(&focus).on_mouse_down_out(cx.listener(Self::outside_down))
            });
        if self.open {
            let (active_bg, active_fg) = active_row_colors(t);
            let pane_bg = if t.name.starts_with("shadcn-") {
                t.colors.surface
            } else {
                t.colors.elevated_surface
            };
            root = root.flex().items_start().gap(px(t.spacing.xsmall));
            for depth in 0..=self.path.len() {
                let items = self.pane(depth);
                let mut pane = div()
                    .id(("menu-pane", depth))
                    .role(gpui_pre::accesskit::Role::Menu)
                    .flex()
                    .flex_col()
                    .p(px(t.spacing.xsmall))
                    .rounded(px(t.radii.medium))
                    .border(px(t.borders.regular))
                    .border_color(t.colors.border)
                    .bg(pane_bg);
                for (index, item) in items.iter().enumerate() {
                    let mut row = div()
                        .id(format!("menu-item-{depth}-{index}"))
                        .debug_selector(|| format!("menu-item-{depth}-{index}"))
                        .on_mouse_move(cx.listener(move |this, event, window, cx| {
                            this.pointer_move(depth, index, event, window, cx)
                        }))
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.activate_at(depth, index, cx)),
                        )
                        .role(if item.checked.is_some() {
                            gpui_pre::accesskit::Role::MenuItemCheckBox
                        } else {
                            gpui_pre::accesskit::Role::MenuItem
                        })
                        .aria_label(item.label.clone())
                        .when(item.disabled, |row| {
                            row.a11y_synthetic_children(|builder| {
                                builder.parent_node().set_disabled()
                            })
                        })
                        .when(item.checked.is_some(), |row| {
                            row.aria_toggled(item.checked.unwrap_or(false).into())
                        })
                        .aria_expanded(self.path.get(depth) == Some(&index))
                        .when(index == self.active_index(depth), |d| {
                            d.aria_active_descendant().bg(active_bg).text_color(active_fg)
                        })
                        .when(item.disabled, |d| d.text_color(t.colors.disabled))
                        .when(!item.disabled && index != self.active_index(depth), |d| {
                            d.text_color(t.colors.text)
                        })
                        .flex()
                        .rounded(px(t.radii.small))
                        .text_size(px(t.typography.body))
                        .items_center()
                        .justify_between()
                        .gap(px(t.spacing.large))
                        .px(px(t.spacing.small))
                        .h(px(t.controls.small));
                    let check = if item.checked == Some(true) {
                        "✓ "
                    } else if item.checked.is_some() {
                        "  "
                    } else {
                        ""
                    };
                    row = row.child(div().flex_1().child(format!("{}{}", check, item.label)));
                    if index == self.active_index(depth) {
                        row = row.child(
                            div()
                                .debug_selector(|| format!("menu-active-{depth}-{index}"))
                                .size(px(0.0)),
                        );
                    }
                    if let Some(shortcut) = &item.shortcut {
                        row = row.child(
                            div()
                                .debug_selector(|| format!("menu-shortcut-{depth}-{index}"))
                                .text_color(t.colors.text_muted)
                                .child(shortcut.clone()),
                        );
                    }
                    if !item.children.is_empty() {
                        row = row.child(div().text_color(t.colors.text_muted).child("›"));
                    }
                    pane = pane.child(row);
                }
                root = root.child(pane);
            }
        }
        if self.open {
            let position = self.anchor.unwrap_or_else(|| point(px(0.), px(0.)));
            div().id(("mkit-menu-host", cx.entity().entity_id())).child(
                deferred(
                    anchored()
                        .anchor(Anchor::TopLeft)
                        .position_mode(AnchoredPositionMode::Window)
                        .position(position)
                        .snap_to_window_with_margin(px(t.spacing.medium))
                        .child(root),
                )
                .with_priority(mkit_core::overlay::layer::POPOVER),
            )
        } else {
            div().id(("mkit-menu-host", cx.entity().entity_id())).size(px(0.0))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn item_builder_preserves_shortcut_check_and_nested_items() {
        let child = MenuItem::new("save", "Save").shortcut("⌘S");
        let checked = MenuItem::new("pin", "Pin").checked(true);
        let parent = MenuItem::new("more", "More").submenu(vec![child.clone()]);
        assert_eq!(child.shortcut.as_deref(), Some("⌘S"));
        assert_eq!(checked.checked, Some(true));
        assert_eq!(parent.children, vec![child]);
    }
    #[test]
    fn keyboard_actions_cover_navigation_and_are_rebindable() {
        assert_eq!(default_key_bindings().len(), 9);
    }
    #[test]
    fn pane_path_tracks_nested_selection_and_returns_to_parent() {
        let items = vec![MenuItem::new("more", "More").submenu(vec![
            MenuItem::new("nested", "Nested").submenu(vec![MenuItem::new("leaf", "Leaf")]),
        ])];
        let mut menu = DropdownMenu::new(items);
        assert!(menu.pane(0).len() == 1);
        menu.path.push(0);
        menu.active.push(0);
        assert_eq!(menu.pane(1)[0].id, "nested");
        menu.path.push(0);
        menu.active.push(0);
        assert_eq!(menu.pane(2)[0].id, "leaf");
        menu.path.pop();
        menu.active.pop();
        assert_eq!(menu.path.len(), 1);
    }
    #[test]
    fn next_enabled_item_skips_disabled_items() {
        let items = [
            MenuItem::new("a", "A").disabled(true),
            MenuItem::new("b", "B"),
            MenuItem::new("c", "C"),
        ];
        assert_eq!(DropdownMenu::enabled_index(&items, 0, true), Some(1));
        assert_eq!(DropdownMenu::enabled_index(&items, 2, true), Some(2));
        assert_eq!(DropdownMenu::enabled_index(&items, 0, false), Some(2));
    }
}
