//! In-window application menu bar backed by a GPUI-compatible menu model.
extern crate gpui_pre as gpui;

use gpui_pre::{
    Action, Context, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement, KeyBinding,
    KeyDownEvent, MouseDownEvent, PathBuilder, Render, Rgba, Window, actions, canvas, div, point,
    prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};

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

/// Resolved bar and popup colours; see the spec's "Theme tokens used" table. The popup reuses
/// DropdownMenu's look.
#[derive(Clone, Copy)]
struct Look {
    bar_bg: Rgba,
    bar_border: Rgba,
    header_text: Rgba,
    mnemonic: Rgba,
    /// Open, focused or hovered header fill (shadcn `accent`).
    header_active_bg: Rgba,
    header_active_text: Rgba,
    header_active_mnemonic: Rgba,
    /// Whether pointer hover fills the header (light/dark) or underlines it (high contrast).
    fill_hover: bool,
    disabled_text: Rgba,
    disabled_mnemonic: Rgba,
    disabled_border: Rgba,
    pane: Rgba,
    pane_border: Rgba,
    row_text: Rgba,
    row_muted: Rgba,
    active_bg: Rgba,
    active_text: Rgba,
    active_muted: Rgba,
    row_disabled_text: Rgba,
    row_disabled_muted: Rgba,
    ring: Rgba,
    icon_stroke: IconStroke,
}
#[derive(Clone, Copy)]
enum IconStroke {
    /// Lucide's 2-unit stroke on its 24-unit grid, scaled with the icon.
    Relative,
    Pixels(f32),
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight, ..foreground }, Rgba { a: 1.0, ..base })
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            bar_bg: c.background,
            bar_border: c.border,
            header_text: c.text,
            mnemonic: c.text_muted,
            header_active_bg: c.accent,
            header_active_text: c.accent_text,
            header_active_mnemonic: c.accent_text,
            fill_hover: false,
            disabled_text: c.disabled,
            disabled_mnemonic: c.disabled,
            disabled_border: c.disabled,
            pane: c.background,
            pane_border: c.border,
            row_text: c.text,
            row_muted: c.text_muted,
            active_bg: c.accent,
            active_text: c.accent_text,
            active_muted: c.accent_text,
            row_disabled_text: c.disabled,
            row_disabled_muted: c.disabled,
            ring: c.focus,
            icon_stroke: IconStroke::Pixels(t.borders.regular),
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn "accent": text mixed into the background, as DropdownMenu, Tabs and Sidebar use.
    let accent = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    // The web's translucent dark border (10% text), composited over the opaque fill beneath.
    let bar_border = if dark { mix(c.text, c.background, 0.1) } else { c.border };
    let pane = c.surface;
    Look {
        bar_bg: c.background,
        bar_border,
        header_text: c.text,
        mnemonic: c.text_muted,
        header_active_bg: accent,
        header_active_text: c.text,
        header_active_mnemonic: c.text_muted,
        fill_hover: true,
        // shadcn's disabled `opacity: .5`, flattened over the bar fill part by part.
        disabled_text: mix(c.text, c.background, 0.5),
        disabled_mnemonic: mix(c.text_muted, c.background, 0.5),
        disabled_border: mix(bar_border, c.background, 0.5),
        pane,
        pane_border: if dark { mix(c.text, pane, 0.1) } else { c.border },
        row_text: c.text,
        row_muted: c.text_muted,
        active_bg: accent,
        active_text: c.text,
        active_muted: c.text_muted,
        row_disabled_text: mix(c.text, pane, 0.5),
        row_disabled_muted: mix(c.text_muted, pane, 0.5),
        ring: c.focus.opacity(0.5),
        icon_stroke: IconStroke::Relative,
    }
}
fn box_shadow(shadow: ShadowToken) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: shadow.color.into(),
        offset: point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}
/// shadcn/ui focus ring width, drawn outside the focused header.
const FOCUS_RING_WIDTH: f32 = 3.0;
fn focus_ring(color: Rgba) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: color.into(),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(FOCUS_RING_WIDTH),
        inset: false,
    }
}
#[derive(Clone, Copy)]
enum Icon {
    Check,
    ChevronRight,
}
/// Decorative Lucide `check` (20,6 → 9,17 → 4,12) or `chevron-right` (9,6 → 15,12 → 9,18)
/// drawn as a vector path on a 24-unit grid, matching DropdownMenu.
fn icon(icon: Icon, size: f32, stroke: IconStroke, color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let width = match stroke {
                IconStroke::Relative => unit * 2.0,
                IconStroke::Pixels(width) => px(width),
            };
            let points: &[(f32, f32)] = match icon {
                Icon::Check => &[(20.0, 6.0), (9.0, 17.0), (4.0, 12.0)],
                Icon::ChevronRight => &[(9.0, 6.0), (15.0, 12.0), (9.0, 18.0)],
            };
            let mut path = PathBuilder::stroke(width);
            for (i, (x, y)) in points.iter().enumerate() {
                let p = bounds.origin + point(unit * *x, unit * *y);
                if i == 0 { path.move_to(p) } else { path.line_to(p) }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
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
        let look = look(&theme);
        let bar_focused = self.focus.is_focused(window);
        let keyboard = window.last_input_was_keyboard();
        let focus = self.focus.clone();
        let disabled = self.disabled;
        let stacked = self.model.menus.iter().any(|menu| menu.mnemonic.is_some());
        let icon_size = theme.spacing.large;
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
            .when(self.open_menu.is_some(), |el| {
                el.on_mouse_down_out(cx.listener(Self::outside_down))
            });
        let mut headers_row = div()
            .id("menu-bar-headers")
            .w_full()
            .flex()
            .items_center()
            .gap(px(theme.spacing.xsmall))
            .p(px(theme.spacing.xsmall))
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.regular))
            .border_color(if disabled { look.disabled_border } else { look.bar_border })
            .bg(look.bar_bg)
            .when(!disabled, |el| el.shadow(vec![box_shadow(theme.shadows.small)]));
        let headers = self
            .model
            .menus
            .iter()
            .enumerate()
            .map(|(index, menu)| {
                let open = self.open_menu == Some(index);
                let selected =
                    (bar_focused || self.open_menu.is_some()) && self.focused_menu == index;
                let unavailable = menu.disabled || disabled;
                // Keyboard focus on a closed bar draws the focus ring on the active header.
                let ring = selected && bar_focused && keyboard && self.open_menu.is_none();
                let (text, mnemonic, bg) = if unavailable {
                    (look.disabled_text, look.disabled_mnemonic, look.bar_bg)
                } else if selected || open {
                    (look.header_active_text, look.header_active_mnemonic, look.header_active_bg)
                } else {
                    (look.header_text, look.mnemonic, look.bar_bg)
                };
                let mut item = div()
                    .id(format!("menu-header-{}", menu.id))
                    .debug_selector(move || format!("menu-header-{index}"))
                    .role(gpui_pre::accesskit::Role::MenuItem)
                    .aria_label(menu.label.clone())
                    .aria_expanded(open)
                    .when(unavailable, |e| {
                        e.a11y_synthetic_children(|b| {
                            b.parent_node().set_disabled();
                        })
                    })
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .when(stacked, |e| {
                        e.h(px(theme.controls.large + theme.spacing.small))
                            .min_w(px(theme.controls.large))
                    })
                    .when(!stacked, |e| e.h(px(theme.controls.xsmall)))
                    .px(px(theme.spacing.small))
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.regular))
                    .border_color(if ring { theme.colors.focus } else { bg.opacity(0.) })
                    .bg(bg)
                    .when(ring, |e| e.shadow(vec![focus_ring(look.ring)]))
                    .text_size(px(theme.typography.body))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(text)
                    .when(selected, |e| e.aria_active_descendant());
                if !unavailable {
                    item = item.hover(move |s| {
                        if look.fill_hover {
                            s.bg(look.header_active_bg).text_color(look.header_active_text)
                        } else {
                            s.underline()
                        }
                    });
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
                if let Some(value) = menu.mnemonic {
                    item = item.child(
                        div()
                            .text_size(px(theme.typography.caption))
                            .font_weight(FontWeight::NORMAL)
                            .text_color(mnemonic)
                            .child(format!("({value})")),
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
                    // shadcn `-mx-1 my-1 h-px`: the rule spans the pane's padding.
                    MenuEntry::Separator => div()
                        .id(format!("menu-separator-{index}"))
                        .h(px(theme.borders.hairline))
                        .mx(px(-theme.spacing.xsmall))
                        .my(px(theme.spacing.xsmall))
                        .bg(look.pane_border),
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
                        let (fg, muted) = if *unavailable {
                            (look.row_disabled_text, look.row_disabled_muted)
                        } else if active {
                            (look.active_text, look.active_muted)
                        } else {
                            (look.row_text, look.row_muted)
                        };
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
                            .when(active, |e| e.aria_active_descendant().bg(look.active_bg))
                            .h(px(theme.controls.small))
                            .px(px(theme.spacing.small))
                            .rounded(px(theme.radii.small))
                            .flex()
                            .items_center()
                            .gap(px(theme.spacing.small))
                            .text_size(px(theme.typography.body))
                            .text_color(fg)
                            .when_some(*checked, |e, checked| {
                                e.child(div().size(px(icon_size)).flex_none().when(
                                    checked,
                                    |slot| {
                                        slot.child(icon(
                                            Icon::Check,
                                            icon_size,
                                            look.icon_stroke,
                                            muted,
                                        ))
                                    },
                                ))
                            })
                            .child(div().flex_grow(1.0).whitespace_nowrap().child(label.clone()));
                        if let Some(shortcut) = shortcut {
                            row = row.child(
                                div()
                                    .ml_auto()
                                    .flex_none()
                                    .whitespace_nowrap()
                                    .text_size(px(theme.typography.caption))
                                    .text_color(muted)
                                    .child(shortcut.clone()),
                            );
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
                        let (fg, muted) = if *unavailable {
                            (look.row_disabled_text, look.row_disabled_muted)
                        } else if active {
                            (look.active_text, look.active_muted)
                        } else {
                            (look.row_text, look.row_muted)
                        };
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
                            .when(active, |e| e.aria_active_descendant().bg(look.active_bg))
                            .h(px(theme.controls.small))
                            .px(px(theme.spacing.small))
                            .rounded(px(theme.radii.small))
                            .flex()
                            .items_center()
                            .gap(px(theme.spacing.small))
                            .text_size(px(theme.typography.body))
                            .text_color(fg)
                            .child(div().flex_grow(1.0).whitespace_nowrap().child(label.clone()))
                            .child(div().ml_auto().flex_none().child(icon(
                                Icon::ChevronRight,
                                icon_size,
                                look.icon_stroke,
                                muted,
                            )));
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
                    .border_color(look.pane_border)
                    .bg(look.pane)
                    .shadow(vec![box_shadow(theme.shadows.medium)])
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
