//! Fixed-height, scroll-driven virtual list with selectable rows.
extern crate gpui_pre as gpui;

use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render, Rgba,
    ScrollStrategy, UniformListScrollHandle, Window, actions, div, point, prelude::*, px,
    uniform_list,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::Theme,
};
use std::sync::Arc;

/// Colours derived from theme tokens; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    border: Rgba,
    selected_bg: Rgba,
    selected_text: Rgba,
    /// Pointer-hover fill for unselected rows (shadcn); `None` outlines the row instead.
    hover_bg: Option<Rgba>,
    hover_border: Rgba,
    focus: Rgba,
    ring: Rgba,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            border: c.border,
            selected_bg: c.accent,
            selected_text: c.accent_text,
            hover_bg: None,
            hover_border: c.border,
            focus: c.focus,
            ring: c.focus,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn "accent": text mixed 4% (light) or 12% (dark) into the background.
    let accent = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    Look {
        border: if dark { c.text.opacity(0.1) } else { c.border },
        selected_bg: accent,
        selected_text: c.text,
        hover_bg: Some(accent),
        hover_border: c.border,
        focus: c.focus,
        ring: c.focus.opacity(0.5),
    }
}
/// shadcn/ui focus ring width, drawn outside an empty focused list.
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

pub const KEY_CONTEXT: &str = "MkitVirtualList";
actions!(virtual_list, [Next, Previous, ToggleSelection, First, Last]);

pub fn default_key_bindings() -> [KeyBinding; 5] {
    [
        KeyBinding::new("down", Next, Some(KEY_CONTEXT)),
        KeyBinding::new("up", Previous, Some(KEY_CONTEXT)),
        KeyBinding::new("space", ToggleSelection, Some(KEY_CONTEXT)),
        KeyBinding::new("home", First, Some(KEY_CONTEXT)),
        KeyBinding::new("end", Last, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem {
    pub id: String,
    pub label: String,
}
impl ListItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into() }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveChanged(pub Option<String>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionChanged(pub Vec<String>);
impl EventEmitter<ActiveChanged> for VirtualList {}
impl EventEmitter<SelectionChanged> for VirtualList {}

/// Stateful selectable list. GPUI renders only rows in the scroll viewport.
pub struct VirtualList {
    label: String,
    items: Arc<Vec<ListItem>>,
    active: Option<usize>,
    selected: Vec<String>,
    controlled: bool,
    multi: bool,
    row_height: f32,
    viewport_height: f32,
    list_scroll: UniformListScrollHandle,
    focus: Option<FocusHandle>,
}
impl VirtualList {
    pub fn new(label: impl Into<String>, items: Vec<ListItem>) -> Self {
        let active = (!items.is_empty()).then_some(0);
        Self {
            label: label.into(),
            items: Arc::new(items),
            active,
            selected: Vec::new(),
            controlled: false,
            multi: true,
            row_height: 32.0,
            viewport_height: 320.0,
            list_scroll: UniformListScrollHandle::new(),
            focus: None,
        }
    }
    pub fn controlled(
        label: impl Into<String>,
        items: Vec<ListItem>,
        selected: Vec<String>,
    ) -> Self {
        let mut list = Self::new(label, items);
        list.selected = selected;
        list.controlled = true;
        list
    }
    pub fn single_selection(mut self) -> Self {
        self.multi = false;
        self
    }
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height.max(1.0);
        self
    }
    pub fn viewport_height(mut self, height: f32) -> Self {
        self.viewport_height = height.max(1.0);
        self
    }
    pub fn selected(&self) -> &[String] {
        &self.selected
    }
    pub fn active(&self) -> Option<&str> {
        self.active.and_then(|i| self.items.get(i)).map(|v| v.id.as_str())
    }
    pub fn visible_range(&self, start: usize, count: usize) -> std::ops::Range<usize> {
        let start = start.min(self.items.len());
        start..start.saturating_add(count).min(self.items.len())
    }
    /// Legacy compatibility shim. GPUI now derives the rendered range from scroll position.
    #[deprecated(note = "visible rows are derived from the GPUI scroll position")]
    pub fn set_visible_window(&mut self, start: usize, _count: usize, cx: &mut Context<Self>) {
        self.list_scroll
            .scroll_to_item(start.min(self.items.len().saturating_sub(1)), ScrollStrategy::Top);
        cx.notify();
    }
    pub fn set_items(&mut self, items: Vec<ListItem>, cx: &mut Context<Self>) {
        self.items = Arc::new(items);
        self.selected.retain(|id| self.items.iter().any(|i| &i.id == id));
        self.active = self
            .active
            .filter(|i| *i < self.items.len())
            .or_else(|| (!self.items.is_empty()).then_some(0));
        cx.notify();
    }
    pub fn set_selection(&mut self, selected: Vec<String>, cx: &mut Context<Self>) {
        self.selected = selected;
        cx.notify();
    }
    fn move_active(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            return;
        }
        let next = self.active.map_or(if delta > 0 { 0 } else { self.items.len() - 1 }, |i| {
            (i as isize + delta).clamp(0, self.items.len() as isize - 1) as usize
        });
        self.active = Some(next);
        cx.emit(ActiveChanged(self.active().map(str::to_owned)));
        self.list_scroll.scroll_to_item(next, ScrollStrategy::Nearest);
        cx.notify();
    }
    fn request_toggle(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.active().map(str::to_owned) else {
            return;
        };
        let mut next = self.selected.clone();
        if next.contains(&id) {
            next.retain(|v| v != &id);
        } else {
            if !self.multi {
                next.clear();
            }
            next.push(id);
        }
        if !self.controlled {
            self.selected = next.clone();
        }
        cx.emit(SelectionChanged(next));
        cx.notify();
    }
    fn click_row(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.items.len() {
            return;
        }
        if self.active != Some(index) {
            self.active = Some(index);
            cx.emit(ActiveChanged(self.active().map(str::to_owned)));
        }
        self.request_toggle(cx);
    }
}
impl Focusable for VirtualList {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}

impl Render for VirtualList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let look = look(&theme);
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        // `:focus-visible`: the active row outline shows for keyboard focus only.
        let focused = focus.is_focused(window) && window.last_input_was_keyboard();
        let items = self.items.clone();
        let entity = cx.entity();
        let selected = self.selected.clone();
        let active = self.active;
        let count = items.len();
        let row_height = self.row_height;
        let hairline = px(theme.borders.hairline);
        let transparent = theme.colors.background.opacity(0.);
        let list = uniform_list(
            ("mkit-virtual-list-items", cx.entity().entity_id()),
            count,
            move |range, _, _| {
                range
                    .map(|i| {
                        let item = &items[i];
                        let entity = entity.clone();
                        let is_active = active == Some(i);
                        let is_selected = selected.contains(&item.id);
                        div()
                            .id(item.id.clone())
                            .role(gpui_pre::accesskit::Role::ListBoxOption)
                            .aria_label(item.label.clone())
                            .aria_selected(is_selected)
                            .when(is_active, |row| row.aria_active_descendant())
                            .aria_position_in_set(i + 1)
                            .aria_size_of_set(items.len())
                            .on_click(move |_, _, cx| {
                                entity.update(cx, |list, cx| list.click_row(i, cx));
                            })
                            .w_full()
                            .h(px(row_height))
                            .px(px(theme.spacing.small))
                            .flex()
                            .items_center()
                            .rounded(px(theme.radii.small))
                            .border(hairline)
                            .border_color(if focused && is_active {
                                look.focus
                            } else {
                                transparent
                            })
                            .text_size(px(theme.typography.body))
                            .when(is_selected, |row| {
                                row.bg(look.selected_bg).text_color(look.selected_text)
                            })
                            .when(!is_selected, |row| row.text_color(theme.colors.text))
                            .when(!is_selected && !(focused && is_active), |row| {
                                row.hover(move |style| match look.hover_bg {
                                    Some(fill) => style.bg(fill),
                                    None => style.border_color(look.hover_border),
                                })
                            })
                            .child(item.label.clone())
                    })
                    .collect::<Vec<_>>()
            },
        )
        .track_scroll(&self.list_scroll)
        .w_full()
        .h(px(self.viewport_height))
        .p(px(theme.spacing.xsmall));

        div()
            .id(self.label.clone())
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .tab_index(0)
            .role(gpui_pre::accesskit::Role::ListBox)
            .aria_label(self.label.clone())
            .border(hairline)
            .border_color(look.border)
            .rounded(px(theme.radii.large))
            .overflow_hidden()
            .bg(theme.colors.surface)
            // With no rows there is no active-row outline, so the list itself shows focus.
            .when(focused && count == 0, |list| {
                list.border_color(look.focus).shadow(vec![focus_ring(look.ring)])
            })
            .on_action(cx.listener(|this, _: &Next, _, cx| this.move_active(1, cx)))
            .on_action(cx.listener(|this, _: &Previous, _, cx| this.move_active(-1, cx)))
            .on_action(cx.listener(|this, _: &ToggleSelection, _, cx| this.request_toggle(cx)))
            .on_action(cx.listener(|this, _: &First, _, cx| {
                if !this.items.is_empty() {
                    this.active = Some(0);
                    this.list_scroll.scroll_to_item(0, ScrollStrategy::Nearest);
                    cx.emit(ActiveChanged(this.active().map(str::to_owned)));
                    cx.notify();
                }
            }))
            .on_action(cx.listener(|this, _: &Last, _, cx| {
                if !this.items.is_empty() {
                    let last = this.items.len() - 1;
                    this.active = Some(last);
                    this.list_scroll.scroll_to_item(last, ScrollStrategy::Nearest);
                    cx.emit(ActiveChanged(this.active().map(str::to_owned)));
                    cx.notify();
                }
            }))
            .child(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visible_window_is_clamped_and_bounded() {
        let list = VirtualList::new(
            "items",
            (0..10).map(|i| ListItem::new(i.to_string(), "row")).collect(),
        );
        assert_eq!(list.visible_range(4, 3), 4..7);
        assert_eq!(list.visible_range(9, 8), 9..10);
        assert_eq!(list.visible_range(99, 8), 10..10);
    }
}

#[cfg(test)]
mod gpui_tests {
    use super::*;
    use gpui_pre::{AppContext, Entity, Modifiers, ParentElement, TestAppContext, point};
    use std::{cell::RefCell, rc::Rc};

    struct Host {
        list: Option<Entity<VirtualList>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let list = self.list.get_or_insert_with(|| {
                cx.new(|_| {
                    VirtualList::new(
                        "large-list",
                        (0..10_000)
                            .map(|i| ListItem::new(format!("item-{i}"), format!("Item {i}")))
                            .collect(),
                    )
                })
            });
            div().child(list.clone())
        }
    }

    #[gpui_pre::test]
    fn large_list_virtualizes_and_keyboard_navigation_scrolls_active_row(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, _| Host { list: None });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let list = host.read_with(visual, |host, _| host.list.as_ref().unwrap().clone());
        visual.update(|window, cx| list.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("end");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        list.read_with(visual, |list, _| {
            assert_eq!(list.active, Some(9_999));
            assert!(list.list_scroll.is_scrollable());
            assert_eq!(list.list_scroll.is_scrolled_to_end(), Some(true));
        });
    }

    #[gpui_pre::test]
    fn large_list_responds_to_pointer_wheel_scroll(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) = cx.add_window_view(|_, _| Host { list: None });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let list = host.read_with(visual, |host, _| host.list.as_ref().unwrap().clone());
        visual.simulate_mouse_move(gpui_pre::point(px(10.), px(100.)), None, Default::default());
        visual.simulate_event(gpui_pre::ScrollWheelEvent {
            position: gpui_pre::point(px(10.), px(100.)),
            delta: gpui_pre::ScrollDelta::Pixels(gpui_pre::point(px(0.), px(-10_000.))),
            modifiers: Default::default(),
            touch_phase: gpui_pre::TouchPhase::Moved,
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        list.read_with(visual, |list, _| {
            assert!(list.list_scroll.is_scrollable());
            assert!(!list.list_scroll.is_scrolled_to_end().unwrap_or(false));
            assert!(f32::from(list.list_scroll.0.borrow().base_handle.offset().y) < 0.0);
        });
    }

    #[gpui_pre::test]
    fn row_click_activates_and_toggles_selection(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (list, visual) = cx.add_window_view(|_, _| {
            VirtualList::new("files", vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Beta")])
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let row = point(px(12.), px(48.));
        visual.simulate_click(row, Modifiers::default());
        list.read_with(visual, |list, _| {
            assert_eq!(list.active(), Some("b"));
            assert_eq!(list.selected(), &["b".to_owned()]);
        });
        visual.simulate_click(row, Modifiers::default());
        list.read_with(visual, |list, _| {
            assert_eq!(list.active(), Some("b"));
            assert!(list.selected().is_empty());
        });
    }

    #[gpui_pre::test]
    fn controlled_row_click_waits_for_owner_selection(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (list, visual) = cx.add_window_view(|_, _| {
            VirtualList::controlled(
                "files",
                vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Beta")],
                vec!["a".to_owned()],
            )
            .single_selection()
        });
        let requests = Rc::new(RefCell::new(Vec::new()));
        let request_log = requests.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&list, move |_, event: &SelectionChanged, _| {
                request_log.borrow_mut().push(event.0.clone());
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let row = point(px(12.), px(48.));
        visual.simulate_click(row, Modifiers::default());
        list.read_with(visual, |list, _| {
            assert_eq!(list.active(), Some("b"));
            assert_eq!(list.selected(), &["a".to_owned()]);
        });
        assert_eq!(*requests.borrow(), vec![vec!["b".to_owned()]]);
        list.update(visual, |list, cx| list.set_selection(vec!["b".to_owned()], cx));
        list.read_with(visual, |list, _| assert_eq!(list.selected(), &["b".to_owned()]));
    }
}
