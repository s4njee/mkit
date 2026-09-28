//! Stateful command menu with checkable items and visible shortcut labels.
extern crate gpui_pre as gpui;
#[cfg(all(test, feature = "mkit-mirror"))]
use crate::key_hint::shortcut_label;
#[cfg(feature = "mkit-mirror")]
use crate::key_hint::{KeyChord, KeyHint};
use gpui_pre::{
    Anchor, AnchoredPositionMode, Context, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyBinding, MouseDownEvent, PathBuilder, Render, Rgba, WeakFocusHandle, Window, actions,
    anchored, canvas, deferred, div, point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
#[cfg(all(test, not(feature = "mkit-mirror")))]
use mkit_registry_key_hint::shortcut_label;
#[cfg(not(feature = "mkit-mirror"))]
use mkit_registry_key_hint::{KeyChord, KeyHint};
/// Resolved menu colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    pane: Rgba,
    border: Rgba,
    text: Rgba,
    /// Shortcut labels, check marks and submenu chevrons.
    muted: Rgba,
    active_bg: Rgba,
    active_text: Rgba,
    active_muted: Rgba,
    disabled_text: Rgba,
    disabled_muted: Rgba,
    /// Stroke width of the check and chevron icons, as a fraction of the icon size or in pixels.
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
            pane: c.background,
            border: c.border,
            text: c.text,
            muted: c.text_muted,
            active_bg: c.accent,
            active_text: c.accent_text,
            active_muted: c.accent_text,
            disabled_text: c.disabled,
            disabled_muted: c.disabled,
            icon_stroke: IconStroke::Pixels(t.borders.regular),
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn "popover" is `surface`; GPUI fills inside drop shadows, so the pane and its
    // border stay opaque (the dark border is the web's translucent 10% text, composited).
    let pane = c.surface;
    Look {
        pane,
        border: if dark { mix(c.text, pane, 0.1) } else { c.border },
        text: c.text,
        muted: c.text_muted,
        // shadcn "accent": text mixed into the background.
        active_bg: mix(c.text, c.background, if dark { 0.12 } else { 0.04 }),
        active_text: c.text,
        active_muted: c.text_muted,
        // The web preview's `opacity: .5`, flattened over the opaque pane.
        disabled_text: mix(c.text, pane, 0.5),
        disabled_muted: mix(c.text_muted, pane, 0.5),
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
#[derive(Clone, Copy)]
enum Icon {
    Check,
    ChevronRight,
}
/// Decorative Lucide `check` (20,6 → 9,17 → 4,12) or `chevron-right` (9,6 → 15,12 → 9,18)
/// drawn as a vector path on a 24-unit grid inside a square `size` box, so it stays crisp at
/// every scale without icon assets.
fn icon(icon: Icon, size: f32, stroke: IconStroke, color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let origin = bounds.origin;
            let width = match stroke {
                IconStroke::Relative => unit * 2.0,
                IconStroke::Pixels(width) => px(width),
            };
            let mut path = PathBuilder::stroke(width);
            let points: &[(f32, f32)] = match icon {
                Icon::Check => &[(20.0, 6.0), (9.0, 17.0), (4.0, 12.0)],
                Icon::ChevronRight => &[(9.0, 6.0), (15.0, 12.0), (9.0, 18.0)],
            };
            for (i, (x, y)) in points.iter().enumerate() {
                let p = origin + point(unit * *x, unit * *y);
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
pub const KEY_CONTEXT: &str = "ContextMenu";
actions!(
    context_menu,
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
impl EventEmitter<OpenChanged> for ContextMenu {}
impl EventEmitter<ItemSelected> for ContextMenu {}
impl EventEmitter<CheckedChanged> for ContextMenu {}
pub struct ContextMenu {
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
impl ContextMenu {
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
impl Focusable for ContextMenu {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}
impl Render for ContextMenu {
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
            let look = look(&t);
            let icon_size = t.spacing.large;
            root = root.flex().items_start().gap(px(t.spacing.xsmall));
            for depth in 0..=self.path.len() {
                let items = self.pane(depth);
                let active_index = self.active_index(depth);
                let mut pane = div()
                    .id(("menu-pane", depth))
                    .role(gpui_pre::accesskit::Role::Menu)
                    .flex()
                    .flex_col()
                    .min_w(px(t.spacing.xxlarge * 4.0))
                    .p(px(t.spacing.xsmall))
                    .rounded(px(t.radii.medium))
                    .border(px(t.borders.regular))
                    .border_color(look.border)
                    .bg(look.pane)
                    .shadow(vec![box_shadow(t.shadows.medium)]);
                for (index, item) in items.iter().enumerate() {
                    let active = index == active_index;
                    let (fg, muted) = if item.disabled {
                        (look.disabled_text, look.disabled_muted)
                    } else if active {
                        (look.active_text, look.active_muted)
                    } else {
                        (look.text, look.muted)
                    };
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
                        .when(active, |d| d.aria_active_descendant().bg(look.active_bg))
                        .text_color(fg)
                        .flex()
                        .items_center()
                        .gap(px(t.spacing.small))
                        .rounded(px(t.radii.small))
                        .text_size(px(t.typography.body))
                        .px(px(t.spacing.small))
                        .h(px(t.controls.small));
                    if let Some(checked) = item.checked {
                        row = row.child(div().size(px(icon_size)).flex_none().when(
                            checked,
                            |slot| {
                                slot.child(icon(Icon::Check, icon_size, look.icon_stroke, muted))
                            },
                        ));
                    }
                    row = row
                        .child(div().flex_grow(1.0).whitespace_nowrap().child(item.label.clone()));
                    if active {
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
                                .ml_auto()
                                .flex_none()
                                .whitespace_nowrap()
                                .text_size(px(t.typography.caption))
                                .text_color(muted)
                                .child(shortcut_hint(shortcut)),
                        );
                    }
                    if !item.children.is_empty() {
                        row = row.child(div().ml_auto().flex_none().child(icon(
                            Icon::ChevronRight,
                            icon_size,
                            look.icon_stroke,
                            muted,
                        )));
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
/// Renders a display-only shortcut through KeyHint's shared platform
/// formatter, keeping text KeyHint cannot parse verbatim.
fn shortcut_hint(shortcut: &str) -> gpui_pre::AnyElement {
    match KeyChord::parse(shortcut) {
        Some(chord) => KeyHint::new(chord).inline().into_any_element(),
        None => shortcut.to_owned().into_any_element(),
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
    #[gpui_pre::test]
    fn shortcut_labels_render_through_the_shared_key_hint(cx: &mut gpui_pre::TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (_, visual) = cx.add_window_view(|_, _| {
            ContextMenu::controlled(
                vec![MenuItem::new("save", "Save").shortcut("cmd-shift-s")],
                true,
            )
        });
        visual.run_until_parked();
        assert!(visual.debug_bounds("menu-shortcut-0-0").is_some());
        assert!(visual.debug_bounds("mkit-key-hint").is_some(), "parsed shortcut uses KeyHint");
        #[cfg(target_os = "macos")]
        assert_eq!(shortcut_label("cmd-shift-s"), "⇧⌘S");
    }
    #[gpui_pre::test]
    fn unparseable_shortcut_labels_are_shown_verbatim(cx: &mut gpui_pre::TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (_, visual) = cx.add_window_view(|_, _| {
            ContextMenu::controlled(
                vec![MenuItem::new("next", "Next").shortcut("cmd-k cmd-n")],
                true,
            )
        });
        visual.run_until_parked();
        assert!(visual.debug_bounds("menu-shortcut-0-0").is_some());
        assert!(visual.debug_bounds("mkit-key-hint").is_none());
        assert_eq!(shortcut_label("cmd-k cmd-n"), "cmd-k cmd-n");
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
        let mut menu = ContextMenu::new(items);
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
        assert_eq!(ContextMenu::enabled_index(&items, 0, true), Some(1));
        assert_eq!(ContextMenu::enabled_index(&items, 2, true), Some(2));
        assert_eq!(ContextMenu::enabled_index(&items, 0, false), Some(2));
    }
}
