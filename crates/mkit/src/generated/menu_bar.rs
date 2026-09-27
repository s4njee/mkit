//! In-window application menu bar backed by a GPUI-compatible menu model.
extern crate gpui_pre as gpui;

use gpui_pre::{
    Action, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, KeyDownEvent,
    MouseDownEvent, Render, Window, actions, div, prelude::*, px,
};
use mkit_core::theme::Theme;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandInvoked(pub String);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenMenuChanged(pub Option<String>);

pub struct MenuBarModel {
    pub menus: Vec<MenuDefinition>,
}

impl MenuBarModel {
    pub fn new(menus: impl IntoIterator<Item = MenuDefinition>) -> Self {
        Self { menus: menus.into_iter().collect() }
    }

    /// Convert this model to GPUI's native app-menu representation.
    pub fn to_gpui_menus(&self) -> Vec<gpui_pre::Menu> {
        self.menus.iter().map(MenuDefinition::to_gpui).collect()
    }
}

pub struct MenuDefinition {
    pub id: String,
    pub label: String,
    pub mnemonic: Option<char>,
    pub disabled: bool,
    pub items: Vec<MenuEntry>,
}

impl MenuDefinition {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            mnemonic: None,
            disabled: false,
            items: Vec::new(),
        }
    }
    pub fn mnemonic(mut self, value: char) -> Self {
        self.mnemonic = Some(value.to_ascii_lowercase());
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn items(mut self, values: impl IntoIterator<Item = MenuEntry>) -> Self {
        self.items = values.into_iter().collect();
        self
    }
    fn to_gpui(&self) -> gpui_pre::Menu {
        gpui_pre::Menu::new(self.label.clone())
            .disabled(self.disabled)
            .items(self.items.iter().map(MenuEntry::to_gpui))
    }
}

pub enum MenuEntry {
    Separator,
    Command {
        id: String,
        label: String,
        shortcut: Option<String>,
        disabled: bool,
        checked: Option<bool>,
        action: Box<dyn Action>,
    },
    Submenu {
        id: String,
        label: String,
        disabled: bool,
        items: Vec<MenuEntry>,
    },
}

impl MenuEntry {
    pub fn separator() -> Self {
        Self::Separator
    }
    pub fn command(id: impl Into<String>, label: impl Into<String>, action: impl Action) -> Self {
        Self::Command {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            disabled: false,
            checked: None,
            action: Box::new(action),
        }
    }
    pub fn submenu(
        id: impl Into<String>,
        label: impl Into<String>,
        items: impl IntoIterator<Item = MenuEntry>,
    ) -> Self {
        Self::Submenu {
            id: id.into(),
            label: label.into(),
            disabled: false,
            items: items.into_iter().collect(),
        }
    }
    pub fn shortcut(mut self, value: impl Into<String>) -> Self {
        if let Self::Command { shortcut, .. } = &mut self {
            *shortcut = Some(value.into());
        }
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        match &mut self {
            Self::Command { disabled, .. } | Self::Submenu { disabled, .. } => *disabled = value,
            Self::Separator => {}
        }
        self
    }
    pub fn checked(mut self, value: bool) -> Self {
        if let Self::Command { checked, .. } = &mut self {
            *checked = Some(value);
        }
        self
    }
    fn to_gpui(&self) -> gpui_pre::MenuItem {
        match self {
            Self::Separator => gpui_pre::MenuItem::separator(),
            Self::Submenu { label, disabled, items, .. } => gpui_pre::MenuItem::submenu(
                gpui_pre::Menu::new(label.clone())
                    .disabled(*disabled)
                    .items(items.iter().map(Self::to_gpui)),
            ),
            Self::Command { label, disabled, checked, action, .. } => gpui_pre::MenuItem::Action {
                name: label.clone().into(),
                action: action.boxed_clone(),
                os_action: None,
                checked: checked.unwrap_or(false),
                disabled: *disabled,
            },
        }
    }
}

pub const KEY_CONTEXT: &str = "MenuBar";
actions!(menu_bar, [MoveNext, MovePrevious, MoveRight, MoveLeft, Activate, Dismiss, First, Last]);

pub fn default_key_bindings() -> [KeyBinding; 9] {
    [
        KeyBinding::new("right", MoveRight, Some(KEY_CONTEXT)),
        KeyBinding::new("left", MoveLeft, Some(KEY_CONTEXT)),
        KeyBinding::new("down", MoveNext, Some(KEY_CONTEXT)),
        KeyBinding::new("up", MovePrevious, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Activate, Some(KEY_CONTEXT)),
        KeyBinding::new("space", Activate, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", Dismiss, Some(KEY_CONTEXT)),
        KeyBinding::new("home", First, Some(KEY_CONTEXT)),
        KeyBinding::new("end", Last, Some(KEY_CONTEXT)),
    ]
}

fn active_row_colors(theme: Theme) -> (gpui_pre::Rgba, gpui_pre::Rgba) {
    let weight = match theme.name {
        "shadcn-light" => 0.04,
        "shadcn-dark" => 0.12,
        _ => return (theme.colors.accent, theme.colors.accent_text),
    };
    let foreground = theme.colors.text;
    let background = theme.colors.elevated_surface;
    let mix = |front: f32, base: f32| front * weight + base * (1.0 - weight);
    (
        gpui_pre::Rgba {
            r: mix(foreground.r, background.r),
            g: mix(foreground.g, background.g),
            b: mix(foreground.b, background.b),
            a: 1.0,
        },
        foreground,
    )
}

/// A keyboard navigable, in-window rendering of the application's menu model.
pub struct MenuBar {
    model: MenuBarModel,
    controlled: bool,
    open_menu: Option<usize>,
    focused_menu: usize,
    active_item: usize,
    submenu_path: Vec<usize>,
    disabled: bool,
    focus: FocusHandle,
}

impl EventEmitter<CommandInvoked> for MenuBar {}
impl EventEmitter<OpenMenuChanged> for MenuBar {}

impl MenuBar {
    pub fn new(model: MenuBarModel, cx: &mut Context<Self>) -> Self {
        Self::build(model, false, None, cx)
    }
    pub fn controlled(
        model: MenuBarModel,
        open_menu: Option<String>,
        cx: &mut Context<Self>,
    ) -> Self {
        let index =
            open_menu.as_ref().and_then(|id| model.menus.iter().position(|menu| &menu.id == id));
        Self::build(model, true, index, cx)
    }
    fn build(
        model: MenuBarModel,
        controlled: bool,
        open_menu: Option<usize>,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus = cx.focus_handle();
        let focused_menu = model.menus.iter().position(|menu| !menu.disabled).unwrap_or(0);
        Self {
            model,
            controlled,
            open_menu,
            focused_menu,
            active_item: 0,
            submenu_path: Vec::new(),
            disabled: false,
            focus,
        }
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn is_open(&self) -> bool {
        self.open_menu.is_some()
    }
    pub fn open_menu_id(&self) -> Option<&str> {
        self.open_menu.map(|i| self.model.menus[i].id.as_str())
    }
    pub fn set_open_menu(&mut self, id: Option<&str>, cx: &mut Context<Self>) {
        self.open_menu = id.and_then(|id| self.model.menus.iter().position(|menu| menu.id == id));
        self.submenu_path.clear();
        self.active_item = 0;
        cx.notify();
    }
    fn request_open(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        if index.is_some_and(|i| self.model.menus[i].disabled) {
            return;
        }
        let id = index.map(|i| self.model.menus[i].id.clone());
        if !self.controlled {
            self.open_menu = index;
        }
        self.submenu_path.clear();
        self.active_item = 0;
        cx.emit(OpenMenuChanged(id));
        cx.notify();
    }
    fn outside_down(&mut self, _: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.open_menu.is_some() {
            self.request_open(None, cx);
        }
    }
    fn open_submenu_at(&mut self, depth: usize, index: usize, cx: &mut Context<Self>) {
        self.active_item = index;
        self.submenu_path.truncate(depth);
        self.submenu_path.push(index);
        cx.notify();
    }
    fn entries(&self) -> Option<&[MenuEntry]> {
        let menu = self.model.menus.get(self.open_menu?)?;
        let mut entries = menu.items.as_slice();
        for &index in &self.submenu_path {
            let MenuEntry::Submenu { items, .. } = entries.get(index)? else {
                return None;
            };
            entries = items;
        }
        Some(entries)
    }
    fn move_header(&mut self, forward: bool) {
        if self.model.menus.is_empty() {
            return;
        }
        for _ in 0..self.model.menus.len() {
            self.focused_menu = if forward {
                (self.focused_menu + 1) % self.model.menus.len()
            } else {
                (self.focused_menu + self.model.menus.len() - 1) % self.model.menus.len()
            };
            if !self.model.menus[self.focused_menu].disabled {
                break;
            }
        }
    }
    fn move_item(&mut self, forward: bool) {
        let Some(entries) = self.entries() else {
            return;
        };
        if entries.is_empty() {
            return;
        }
        let enabled = entries.iter().map(|entry| !entry_disabled(entry)).collect::<Vec<_>>();
        let len = enabled.len();
        for _ in 0..len {
            self.active_item = if forward {
                (self.active_item + 1) % len
            } else {
                (self.active_item + len - 1) % len
            };
            if enabled[self.active_item] {
                break;
            }
        }
    }
    fn activate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(entry) = self.entries().and_then(|items| items.get(self.active_item)) else {
            return;
        };
        match entry {
            MenuEntry::Submenu { disabled: false, .. } => {
                self.submenu_path.push(self.active_item);
                self.active_item = 0;
                cx.notify();
            }
            MenuEntry::Command { id, disabled: false, action, .. } => {
                let id = id.clone();
                let action = action.boxed_clone();
                window.dispatch_action(action, cx);
                cx.emit(CommandInvoked(id));
                self.request_open(None, cx);
            }
            _ => {}
        }
    }
    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let key = event.keystroke.key.to_ascii_lowercase();
        let alt = event.keystroke.modifiers.alt;
        if alt && key.len() == 1 {
            if let Some(i) = self
                .model
                .menus
                .iter()
                .position(|m| !m.disabled && m.mnemonic == key.chars().next())
            {
                self.focused_menu = i;
                self.request_open(Some(i), cx);
            }
            return;
        }
        if key == "tab" && self.open_menu.is_some() {
            self.request_open(None, cx);
            return;
        }
        if key == "alt" {
            self.focused_menu = self.model.menus.iter().position(|m| !m.disabled).unwrap_or(0);
            window.focus(&self.focus, cx);
            cx.notify();
        }
    }

    fn navigate_next(&mut self, _: &MoveNext, _: &mut Window, cx: &mut Context<Self>) {
        if self.open_menu.is_some() {
            self.move_item(true);
        } else {
            self.request_open(Some(self.focused_menu), cx);
        }
        cx.notify();
    }
    fn navigate_previous(&mut self, _: &MovePrevious, _: &mut Window, cx: &mut Context<Self>) {
        if self.open_menu.is_some() {
            self.move_item(false);
        } else {
            self.request_open(Some(self.focused_menu), cx);
            if let Some(items) = self.entries() {
                self.active_item =
                    items.iter().rposition(|item| !entry_disabled(item)).unwrap_or(0);
            }
        }
        cx.notify();
    }
    fn navigate_right(&mut self, _: &MoveRight, _: &mut Window, cx: &mut Context<Self>) {
        if self.open_menu.is_some()
            && matches!(
                self.entries().and_then(|e| e.get(self.active_item)),
                Some(MenuEntry::Submenu { disabled: false, .. })
            )
        {
            self.submenu_path.push(self.active_item);
            self.active_item = 0;
        } else if self.open_menu.is_some() && self.submenu_path.is_empty() {
            self.move_header(true);
            self.request_open(Some(self.focused_menu), cx);
        } else {
            self.move_header(true);
        }
        cx.notify();
    }
    fn navigate_left(&mut self, _: &MoveLeft, _: &mut Window, cx: &mut Context<Self>) {
        if !self.submenu_path.is_empty() {
            self.active_item = self.submenu_path.pop().unwrap_or(0);
        } else {
            self.move_header(false);
            if self.open_menu.is_some() {
                self.request_open(Some(self.focused_menu), cx);
            }
        }
        cx.notify();
    }
    fn activate_action(&mut self, _: &Activate, window: &mut Window, cx: &mut Context<Self>) {
        if self.open_menu.is_some() {
            self.activate(window, cx);
        } else {
            self.request_open(Some(self.focused_menu), cx);
        }
    }
    fn dismiss_action(&mut self, _: &Dismiss, window: &mut Window, cx: &mut Context<Self>) {
        if self.open_menu.is_some() {
            self.request_open(None, cx);
            window.focus(&self.focus, cx);
        }
    }
    fn first_action(&mut self, _: &First, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(items) = self.entries() {
            self.active_item = items.iter().position(|item| !entry_disabled(item)).unwrap_or(0);
            cx.notify();
        }
    }
    fn last_action(&mut self, _: &Last, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(items) = self.entries() {
            self.active_item = items.iter().rposition(|item| !entry_disabled(item)).unwrap_or(0);
            cx.notify();
        }
    }
}

fn entry_disabled(entry: &MenuEntry) -> bool {
    match entry {
        MenuEntry::Command { disabled, .. } | MenuEntry::Submenu { disabled, .. } => *disabled,
        MenuEntry::Separator => true,
    }
}

impl Focusable for MenuBar {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for MenuBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let (active_bg, active_fg) = active_row_colors(theme);
        let bar_focused = self.focus.is_focused(window);
        let focus = self.focus.clone();
        let disabled = self.disabled;
        let mut root = div()
            .id("menu-bar")
            .debug_selector(|| "menu-bar".to_owned())
            .role(gpui_pre::accesskit::Role::MenuBar)
            .track_focus(&focus)
            .key_context(KEY_CONTEXT)
            .on_key_down(cx.listener(Self::key_down))
            .on_action(cx.listener(Self::navigate_next))
            .on_action(cx.listener(Self::navigate_previous))
            .on_action(cx.listener(Self::navigate_right))
            .on_action(cx.listener(Self::navigate_left))
            .on_action(cx.listener(Self::activate_action))
            .on_action(cx.listener(Self::dismiss_action))
            .on_action(cx.listener(Self::first_action))
            .on_action(cx.listener(Self::last_action))
            .w_full()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(theme.spacing.xsmall))
            .when(disabled, |el| el.opacity(0.6))
            .when(self.open_menu.is_some(), |el| {
                el.on_mouse_down_out(cx.listener(Self::outside_down))
            });
        let mut headers_row = div()
            .id("menu-bar-headers")
            .flex()
            .items_center()
            .gap(px(theme.spacing.xsmall))
            .px(px(theme.spacing.small))
            .py(px(theme.spacing.xsmall))
            .bg(theme.colors.surface)
            .text_color(theme.colors.text);
        let headers = self
            .model
            .menus
            .iter()
            .enumerate()
            .map(|(index, menu)| {
                let open = self.open_menu == Some(index);
                let selected =
                    (bar_focused || self.open_menu.is_some()) && self.focused_menu == index;
                let mut item = div()
                    .id(format!("menu-header-{}", menu.id))
                    .debug_selector(move || format!("menu-header-{index}"))
                    .role(gpui_pre::accesskit::Role::MenuItem)
                    .aria_label(menu.label.clone())
                    .aria_expanded(open)
                    .when(menu.disabled || disabled, |e| {
                        e.a11y_synthetic_children(|b| {
                            b.parent_node().set_disabled();
                        })
                    })
                    .px(px(theme.spacing.small))
                    .py(px(theme.spacing.xsmall))
                    .rounded(px(theme.radii.small))
                    .text_size(px(theme.typography.body))
                    .text_color(if menu.disabled {
                        theme.colors.disabled
                    } else {
                        theme.colors.text
                    })
                    .when(selected, |e| {
                        e.aria_active_descendant().bg(active_bg).text_color(active_fg)
                    });
                if !menu.disabled && !disabled {
                    let ent = cx.entity().clone();
                    item = item.on_click(move |_, window, cx| {
                        ent.update(cx, |this, cx| {
                            this.focused_menu = index;
                            let next =
                                if this.open_menu == Some(index) { None } else { Some(index) };
                            this.request_open(next, cx);
                            window.focus(&this.focus, cx);
                        })
                    });
                    let ent = cx.entity().clone();
                    let is_disabled = menu.disabled;
                    item = item.on_mouse_move(move |_, _, cx| {
                        ent.update(cx, |this, cx| {
                            if this.open_menu.is_some() && !is_disabled {
                                this.focused_menu = index;
                                this.request_open(Some(index), cx);
                            }
                        });
                    });
                }
                if let Some(mnemonic) = menu.mnemonic {
                    item = item.child(
                        div().text_color(theme.colors.text_muted).child(format!("({mnemonic})")),
                    );
                    item = item.child(div().child(menu.label.clone()));
                } else {
                    item = item.child(menu.label.clone());
                }
                item
            })
            .collect::<Vec<_>>();
        headers_row = headers_row.children(headers);
        root = root.child(headers_row);
        if let Some(items) = self.entries() {
            let submenu_depth = self.submenu_path.len();
            let rows = items
                .iter()
                .enumerate()
                .map(|(index, entry)| match entry {
                    MenuEntry::Separator => div()
                        .id(format!("menu-separator-{index}"))
                        .h(px(theme.borders.hairline))
                        .my(px(theme.spacing.xsmall))
                        .bg(theme.colors.border),
                    MenuEntry::Command {
                        id,
                        label,
                        shortcut,
                        disabled: unavailable,
                        checked,
                        action,
                    } => {
                        let id = id.clone();
                        let command_id = id.clone();
                        let action = action.boxed_clone();
                        let active = index == self.active_item;
                        let muted = theme.colors.text_muted;
                        let mut row = div()
                            .id(format!("menu-command-{id}"))
                            .debug_selector(move || format!("menu-command-{command_id}"))
                            .role(if checked.is_some() {
                                gpui_pre::accesskit::Role::MenuItemCheckBox
                            } else {
                                gpui_pre::accesskit::Role::MenuItem
                            })
                            .aria_label(label.clone())
                            .aria_toggled(checked.unwrap_or(false).into())
                            .when(*unavailable, |e| {
                                e.a11y_synthetic_children(|b| {
                                    b.parent_node().set_disabled();
                                })
                            })
                            .when(active, |e| {
                                e.aria_active_descendant().bg(active_bg).text_color(active_fg)
                            })
                            .h(px(theme.controls.medium))
                            .px(px(theme.spacing.small))
                            .rounded(px(theme.radii.small))
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(theme.spacing.large))
                            .text_color(if *unavailable {
                                theme.colors.disabled
                            } else {
                                theme.colors.text
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(theme.spacing.xsmall))
                                    .child(if checked.unwrap_or(false) { "✓" } else { "" })
                                    .child(label.clone()),
                            );
                        if let Some(shortcut) = shortcut {
                            row = row.child(div().text_color(muted).child(shortcut.clone()));
                        }
                        if !*unavailable {
                            let ent = cx.entity().clone();
                            row = row
                                .on_mouse_move({
                                    let ent = ent.clone();
                                    move |_, _, cx| {
                                        ent.update(cx, |this, cx| {
                                            this.active_item = index;
                                            cx.notify();
                                        });
                                    }
                                })
                                .on_click(move |_, window, cx| {
                                    let action = action.boxed_clone();
                                    ent.update(cx, |this, cx| {
                                        window.dispatch_action(action, cx);
                                        cx.emit(CommandInvoked(id.clone()));
                                        this.request_open(None, cx);
                                    })
                                });
                        }
                        row
                    }
                    MenuEntry::Submenu { id, label, disabled: unavailable, .. } => {
                        let active = index == self.active_item;
                        let mut row = div()
                            .id(format!("menu-submenu-{id}"))
                            .role(gpui_pre::accesskit::Role::MenuItem)
                            .aria_label(label.clone())
                            .aria_expanded(self.submenu_path.last() == Some(&index))
                            .when(*unavailable, |e| {
                                e.a11y_synthetic_children(|b| {
                                    b.parent_node().set_disabled();
                                })
                            })
                            .when(active, |e| {
                                e.aria_active_descendant().bg(active_bg).text_color(active_fg)
                            })
                            .h(px(theme.controls.medium))
                            .px(px(theme.spacing.small))
                            .rounded(px(theme.radii.small))
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_color(if *unavailable {
                                theme.colors.disabled
                            } else {
                                theme.colors.text
                            })
                            .child(label.clone())
                            .child("›");
                        if !*unavailable {
                            let ent = cx.entity().clone();
                            row = row
                                .on_mouse_move({
                                    let ent = ent.clone();
                                    move |_, _, cx| {
                                        ent.update(cx, |this, cx| {
                                            if this.open_menu.is_some() {
                                                this.open_submenu_at(submenu_depth, index, cx);
                                            }
                                        })
                                    }
                                })
                                .on_click(move |_, _, cx| {
                                    ent.update(cx, |this, cx| {
                                        if this.open_menu.is_some() {
                                            this.open_submenu_at(submenu_depth, index, cx);
                                        }
                                    })
                                });
                        }
                        row
                    }
                })
                .collect::<Vec<_>>();
            root = root.child(
                div()
                    .id("menu-popup")
                    .debug_selector(|| "menu-popup".to_owned())
                    .role(gpui_pre::accesskit::Role::Menu)
                    .p(px(theme.spacing.xsmall))
                    .mt(px(theme.spacing.xsmall))
                    .min_w(px(theme.controls.large * 5.0))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.elevated_surface)
                    .flex()
                    .flex_col()
                    .children(rows),
            );
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Entity, TestAppContext, actions};

    actions!(menu_bar_tests, [OpenDocument, SaveDocument]);

    fn model() -> MenuBarModel {
        MenuBarModel::new([
            MenuDefinition::new("file", "File").mnemonic('f').items([
                MenuEntry::command("new", "New", OpenDocument),
                MenuEntry::command("save", "Save", SaveDocument).disabled(true),
                MenuEntry::separator(),
                MenuEntry::submenu(
                    "recent",
                    "Recent",
                    [MenuEntry::command("recent-one", "One", OpenDocument)],
                ),
            ]),
            MenuDefinition::new("edit", "Edit").mnemonic('e').items([MenuEntry::command(
                "copy",
                "Copy",
                OpenDocument,
            )]),
        ])
    }

    struct Host {
        bar: Option<Entity<MenuBar>>,
        controlled: bool,
    }
    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.bar.is_none() {
                let controlled = self.controlled;
                let bar = cx.new(|cx| {
                    if controlled {
                        MenuBar::controlled(model(), None, cx)
                    } else {
                        MenuBar::new(model(), cx)
                    }
                });
                self.bar = Some(bar);
            }
            div().child(self.bar.as_ref().unwrap().clone())
        }
    }

    fn with_bar(
        cx: &mut TestAppContext,
        controlled: bool,
        test: impl FnOnce(Entity<MenuBar>, &mut gpui_pre::VisualTestContext),
    ) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, _| Host { bar: None, controlled });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let bar = host.read_with(&*visual, |host, _| host.bar.as_ref().unwrap().clone());
        visual.update(|window, cx| {
            let focus = bar.read(cx).focus_handle(cx);
            window.focus(&focus, cx);
        });
        test(bar, visual);
    }

    fn key(key: &str, alt: bool) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: gpui_pre::Keystroke {
                modifiers: gpui_pre::Modifiers { alt, ..Default::default() },
                key: key.to_owned(),
                key_char: Some(key.to_owned()),
            },
            is_held: false,
            prefer_character_input: false,
        }
    }

    #[gpui_pre::test]
    fn down_opens_header_and_navigation_skips_disabled_items_and_separators(
        cx: &mut TestAppContext,
    ) {
        with_bar(cx, false, |bar, visual| {
            visual.update(|window, app| window.dispatch_action(Box::new(MoveNext), app));
            assert!(bar.read_with(&*visual, |bar, _| bar.is_open()));
            visual.update(|window, app| window.dispatch_action(Box::new(MoveNext), app));
            assert_eq!(bar.read_with(&*visual, |bar, _| bar.active_item), 3);
            visual.update(|window, app| window.dispatch_action(Box::new(Dismiss), app));
            assert!(!bar.read_with(&*visual, |bar, _| bar.is_open()));
        });
    }

    #[gpui_pre::test]
    fn alt_mnemonic_opens_matching_header(cx: &mut TestAppContext) {
        with_bar(cx, false, |bar, visual| {
            visual.update(|window, app| {
                bar.update(app, |bar, cx| bar.key_down(&key("e", true), window, cx))
            });
            assert_eq!(
                bar.read_with(&*visual, |bar, _| bar.open_menu_id().map(str::to_owned)),
                Some("edit".to_owned())
            );
        });
    }

    #[gpui_pre::test]
    fn controlled_open_request_waits_for_owner(cx: &mut TestAppContext) {
        with_bar(cx, true, |bar, visual| {
            visual.update(|window, app| window.dispatch_action(Box::new(MoveNext), app));
            assert!(!bar.read_with(&*visual, |bar, _| bar.is_open()));
        });
    }

    #[test]
    fn native_conversion_preserves_menu_tree_actions_and_disabled_state() {
        let native = model().to_gpui_menus();
        assert_eq!(native.len(), 2);
        assert_eq!(native[0].name.as_ref(), "File");
        assert_eq!(native[0].items.len(), 4);
        assert!(matches!(&native[0].items[1], gpui_pre::MenuItem::Action { disabled: true, .. }));
        assert!(matches!(&native[0].items[2], gpui_pre::MenuItem::Separator));
        assert!(matches!(&native[0].items[3], gpui_pre::MenuItem::Submenu(_)));
    }
}
