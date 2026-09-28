//! Single-selection tabs with manual keyboard activation.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, Render, Window, actions, div,
    prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};

/// Resolved list and trigger colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    list_bg: gpui_pre::Rgba,
    list_border: Option<gpui_pre::Rgba>,
    selected_bg: gpui_pre::Rgba,
    selected_border: gpui_pre::Rgba,
    selected_text: gpui_pre::Rgba,
    ring: gpui_pre::Rgba,
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            list_bg: c.background,
            list_border: Some(c.border),
            selected_bg: c.accent,
            selected_border: c.accent,
            selected_text: c.accent_text,
            ring: c.focus,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let muted = composite(c.text.opacity(if dark { 0.12 } else { 0.04 }), c.background);
    // Selected and focused fills stay opaque: GPUI drop shadows are not clipped to the
    // element's outside, so a translucent fill would let the shadow tint the trigger.
    let (selected_bg, selected_border) = if dark {
        let input = c.text.opacity(0.15);
        (composite(input, muted), input)
    } else {
        (c.background, c.background.opacity(0.))
    };
    Look {
        list_bg: muted,
        list_border: None,
        selected_bg,
        selected_border,
        selected_text: c.text,
        ring: c.focus.opacity(0.5),
    }
}
fn box_shadow(shadow: ShadowToken) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: shadow.color.into(),
        offset: gpui_pre::point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}
/// shadcn/ui focus ring width, drawn outside the trigger inside the list padding.
const FOCUS_RING_WIDTH: f32 = 3.0;
fn focus_ring(color: gpui_pre::Rgba) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: color.into(),
        offset: gpui_pre::point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(FOCUS_RING_WIDTH),
        inset: false,
    }
}
pub const KEY_CONTEXT: &str = "MkitTabs";
actions!(tabs, [Next, Previous, NextVertical, PreviousVertical, First, Last, Activate]);
pub fn default_key_bindings() -> [gpui_pre::KeyBinding; 8] {
    [
        gpui_pre::KeyBinding::new("right", Next, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("left", Previous, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("down", NextVertical, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("up", PreviousVertical, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("home", First, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("end", Last, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("enter", Activate, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("space", Activate, Some(KEY_CONTEXT)),
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
pub struct Tabs {
    label: String,
    items: Vec<Item>,
    value: Option<String>,
    controlled: bool,
    disabled: bool,
    orientation: Orientation,
    active: usize,
    item_focus: Vec<FocusHandle>,
}
impl EventEmitter<ValueChanged> for Tabs {}
impl Tabs {
    pub fn new(label: impl Into<String>, items: Vec<Item>, default_value: Option<String>) -> Self {
        let active = default_value
            .as_ref()
            .and_then(|v| items.iter().position(|i| &i.value == v && !i.disabled))
            .or_else(|| items.iter().position(|i| !i.disabled))
            .unwrap_or(0);
        let value = default_value.filter(|v| items.iter().any(|i| &i.value == v && !i.disabled));
        Self {
            label: label.into(),
            items,
            value,
            controlled: false,
            disabled: false,
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
    pub fn orientation(mut self, v: Orientation) -> Self {
        self.orientation = v;
        self
    }
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
    pub fn focused_value(&self) -> Option<&str> {
        self.items.get(self.active).map(|i| i.value.as_str())
    }
    pub fn set_value(&mut self, v: Option<String>, cx: &mut Context<Self>) {
        self.value = v;
        if let Some(i) = self
            .value
            .as_ref()
            .and_then(|v| self.items.iter().position(|item| &item.value == v && !item.disabled))
        {
            self.active = i;
        }
        cx.notify();
    }
    fn enabled(&self, i: usize) -> bool {
        !self.disabled && self.items.get(i).is_some_and(|item| !item.disabled)
    }
    fn request(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.enabled(index) {
            return;
        }
        let next = Some(self.items[index].value.clone());
        if self.value == next {
            return;
        }
        if !self.controlled {
            self.value = next.clone();
            cx.notify();
        }
        cx.emit(ValueChanged(next));
    }
    fn focus_index(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled(index) {
            return;
        }
        self.active = index;
        cx.notify();
        if let Some(focus) = self.item_focus.get(index) {
            window.focus(focus, cx);
        }
    }
    fn move_by(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.items.is_empty() {
            return;
        }
        let n = self.items.len();
        for step in 1..=n {
            let i = (self.active as isize + delta * step as isize).rem_euclid(n as isize) as usize;
            if self.enabled(i) {
                self.focus_index(i, window, cx);
                return;
            }
        }
    }
    fn next(&mut self, _: &Next, window: &mut Window, cx: &mut Context<Self>) {
        if self.orientation == Orientation::Horizontal {
            self.move_by(1, window, cx);
        }
    }
    fn previous(&mut self, _: &Previous, window: &mut Window, cx: &mut Context<Self>) {
        if self.orientation == Orientation::Horizontal {
            self.move_by(-1, window, cx);
        }
    }
    fn next_vertical(&mut self, _: &NextVertical, window: &mut Window, cx: &mut Context<Self>) {
        if self.orientation == Orientation::Vertical {
            self.move_by(1, window, cx);
        }
    }
    fn previous_vertical(
        &mut self,
        _: &PreviousVertical,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.orientation == Orientation::Vertical {
            self.move_by(-1, window, cx);
        }
    }
    fn first(&mut self, _: &First, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(i) = self.items.iter().position(|item| !item.disabled) {
            self.focus_index(i, window, cx);
        }
    }
    fn last(&mut self, _: &Last, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(i) = self.items.iter().rposition(|item| !item.disabled) {
            self.focus_index(i, window, cx);
        }
    }
    fn activate(&mut self, _: &Activate, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.active, cx);
    }
}
impl Render for Tabs {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let entity = cx.entity();
        while self.item_focus.len() < self.items.len() {
            self.item_focus.push(cx.focus_handle());
        }
        self.item_focus.truncate(self.items.len());
        for (i, focus) in self.item_focus.iter_mut().enumerate() {
            let enabled = !self.disabled && !self.items[i].disabled;
            *focus = focus
                .clone()
                .tab_stop(enabled && i == self.active)
                .tab_index(if enabled && i == self.active { 0 } else { -1 });
        }
        let row = self.orientation == Orientation::Horizontal;
        let look = look(&t);
        let trigger_height = t.controls.medium - 2. * t.spacing.xsmall;
        let transparent = t.colors.background.opacity(0.);
        let mut root = div()
            .id(("mkit-tabs", cx.entity().entity_id()))
            .key_context(KEY_CONTEXT)
            .role(gpui_pre::accesskit::Role::TabList)
            .aria_label(self.label.clone())
            .aria_orientation(if row {
                gpui_pre::accesskit::Orientation::Horizontal
            } else {
                gpui_pre::accesskit::Orientation::Vertical
            })
            .flex()
            .when(row, |e| e.flex_row())
            .when(!row, |e| e.flex_col())
            .p(px(t.spacing.xsmall))
            .rounded(px(t.radii.large))
            .bg(look.list_bg)
            .when_some(look.list_border, |e, color| {
                e.border(px(t.borders.hairline)).border_color(color)
            })
            .on_action(cx.listener(Self::next))
            .on_action(cx.listener(Self::previous))
            .on_action(cx.listener(Self::next_vertical))
            .on_action(cx.listener(Self::previous_vertical))
            .on_action(cx.listener(Self::first))
            .on_action(cx.listener(Self::last))
            .on_action(cx.listener(Self::activate));
        for (i, item) in self.items.iter().enumerate() {
            let selected = self.value.as_deref() == Some(item.value.as_str());
            let enabled = self.enabled(i);
            let entity = entity.clone();
            let focus = self.item_focus[i].clone();
            let item_index = i;
            let element = div()
                .id(item.value.clone())
                .debug_selector(move || format!("tabs-item-{item_index}"))
                .track_focus(&focus)
                .tab_stop(enabled && i == self.active)
                .tab_index(if enabled && i == self.active { 0 } else { -1 })
                .role(gpui_pre::accesskit::Role::Tab)
                .aria_label(item.label.clone())
                .aria_selected(selected)
                .when(!enabled, |e| e.aria_description("Unavailable"))
                .on_click(move |_, window, cx| {
                    entity.update(cx, |s, cx| {
                        s.focus_index(i, window, cx);
                        s.request(i, cx);
                    })
                })
                .h(px(trigger_height))
                .when(row, |e| e.flex_1())
                .px(px(t.spacing.small))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(t.radii.medium))
                .border(px(t.borders.hairline))
                .border_color(if selected { look.selected_border } else { transparent })
                .when(selected, |e| {
                    e.bg(look.selected_bg).shadow(vec![box_shadow(t.shadows.small)])
                })
                .text_color(if selected { look.selected_text } else { t.colors.text_muted })
                .text_size(px(t.typography.body))
                .font_weight(gpui_pre::FontWeight::MEDIUM)
                .whitespace_nowrap()
                .when(!enabled, |e| e.opacity(0.5))
                .focus_visible(|s| {
                    s.border_color(t.colors.focus)
                        .bg(if selected { look.selected_bg } else { look.list_bg })
                        .shadow(vec![focus_ring(look.ring)])
                })
                .child(item.label.clone());
            root = root.child(element);
        }
        root
    }
}
impl Focusable for Tabs {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.item_focus
            .get(self.active)
            .cloned()
            .expect("Tabs focus handles initialize during render")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use std::{cell::RefCell, rc::Rc};
    fn items() -> Vec<Item> {
        vec![
            Item::new("one", "One"),
            Item::new("skip", "Skip").disabled(true),
            Item::new("three", "Three"),
        ]
    }
    #[gpui::test]
    fn keyboard_moves_focus_without_selecting_until_activated(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (tabs, visual) =
            cx.add_window_view(|_, _| Tabs::new("Views", items(), Some("one".into())));
        visual.update(|w, cx| {
            w.draw(cx).clear(cx);
            tabs.focus_handle(cx).focus(w, cx);
        });
        visual.simulate_keystrokes("right");
        assert_eq!(
            tabs.read_with(visual, |t, _| (
                t.value().map(str::to_owned),
                t.focused_value().map(str::to_owned)
            )),
            (Some("one".into()), Some("three".into()))
        );
        visual.simulate_keystrokes("enter");
        assert_eq!(
            tabs.read_with(visual, |t, _| t.value().map(str::to_owned)),
            Some("three".into())
        );
    }
    #[gpui::test]
    fn pointer_selection_emits_and_controlled_value_waits_for_owner(cx: &mut TestAppContext) {
        cx.update(|app| mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT));
        let events = Rc::new(RefCell::new(Vec::new()));
        let (tabs, visual) =
            cx.add_window_view(|_, _| Tabs::controlled("Views", items(), Some("one".into())));
        let out = events.clone();
        let _subscription = visual.update(|_, cx| {
            cx.subscribe(&tabs, move |_, e: &ValueChanged, _| out.borrow_mut().push(e.0.clone()))
        });
        visual.update(|w, cx| w.draw(cx).clear(cx));
        let bounds = visual.debug_bounds("tabs-item-2").expect("third tab is rendered");
        visual.simulate_click(bounds.center(), gpui::Modifiers::default());
        assert_eq!(tabs.read_with(visual, |t, _| t.value().map(str::to_owned)), Some("one".into()));
        assert_eq!(*events.borrow(), vec![Some("three".into())]);
    }
    #[gpui::test]
    fn disabled_tabs_ignore_keyboard_and_pointer(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (tabs, visual) = cx
            .add_window_view(|_, _| Tabs::new("Views", items(), Some("one".into())).disabled(true));
        visual.update(|w, cx| {
            w.draw(cx).clear(cx);
            tabs.focus_handle(cx).focus(w, cx);
        });
        visual.simulate_keystrokes("right");
        let bounds = visual.debug_bounds("tabs-item-2").expect("third tab is rendered");
        visual.simulate_click(bounds.center(), gpui::Modifiers::default());
        assert_eq!(
            tabs.read_with(visual, |t, _| (
                t.value().map(str::to_owned),
                t.focused_value().map(str::to_owned)
            )),
            (Some("one".into()), Some("one".into()))
        );
    }
}
