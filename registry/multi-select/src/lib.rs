//! Themed multi-value select control with a bounded visible option window.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Anchor, AnchoredPositionMode, Bounds, Context, EventEmitter, FocusHandle, Focusable,
    InteractiveElement, IntoElement, Pixels, Render, ScrollStrategy, UniformListScrollHandle,
    Window, actions, anchored, deferred, div, point, prelude::*, px, uniform_list,
};
use mkit_core::theme::Theme;
use std::{cell::RefCell, rc::Rc};
pub const KEY_CONTEXT: &str = "MkitMultiSelect";
actions!(multi_select, [Next, Previous, First, Last, Toggle, Close, TabForward, TabBackward]);
pub fn default_key_bindings() -> [gpui_pre::KeyBinding; 9] {
    [
        gpui_pre::KeyBinding::new("down", Next, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("up", Previous, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("home", First, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("end", Last, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("space", Toggle, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("enter", Toggle, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("escape", Close, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("tab", TabForward, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("shift-tab", TabBackward, Some(KEY_CONTEXT)),
    ]
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionItem {
    pub id: String,
    pub label: String,
    pub disabled: bool,
}
impl OptionItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), disabled: false }
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValuesChanged(pub Vec<String>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenChanged(pub bool);
impl EventEmitter<ValuesChanged> for MultiSelect {}
impl EventEmitter<OpenChanged> for MultiSelect {}
pub struct MultiSelect {
    label: String,
    options: Vec<OptionItem>,
    values: Vec<String>,
    controlled: bool,
    disabled: bool,
    open: bool,
    active: usize,
    focus: Option<FocusHandle>,
    list_scroll: UniformListScrollHandle,
    trigger_bounds: Rc<RefCell<Option<Bounds<Pixels>>>>,
}
impl MultiSelect {
    pub fn new(
        label: impl Into<String>,
        options: Vec<OptionItem>,
        default_values: Vec<String>,
    ) -> Self {
        let active = default_values
            .first()
            .and_then(|v| options.iter().position(|o| &o.id == v))
            .unwrap_or(0);
        Self {
            label: label.into(),
            options,
            values: default_values,
            controlled: false,
            disabled: false,
            open: false,
            active,
            focus: None,
            list_scroll: UniformListScrollHandle::new(),
            trigger_bounds: Rc::new(RefCell::new(None)),
        }
    }
    pub fn controlled(
        label: impl Into<String>,
        options: Vec<OptionItem>,
        values: Vec<String>,
    ) -> Self {
        let mut s = Self::new(label, options, values);
        s.controlled = true;
        s
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn values(&self) -> &[String] {
        &self.values
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn active_option_id(&self) -> Option<&str> {
        self.options
            .get(self.active)
            .filter(|option| !option.disabled)
            .map(|option| option.id.as_str())
    }
    pub fn set_values(&mut self, v: Vec<String>, cx: &mut Context<Self>) {
        self.values = v;
        cx.notify()
    }
    fn visibility(&mut self, v: bool, cx: &mut Context<Self>) {
        if v && (self.disabled || self.options.iter().all(|option| option.disabled)) {
            return;
        }
        if self.open != v {
            self.open = v;
            if v {
                self.list_scroll.scroll_to_item(self.active, ScrollStrategy::Nearest);
            }
            cx.emit(OpenChanged(v));
            cx.notify()
        }
    }
    fn move_by(&mut self, d: isize, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let n = self.options.len();
        if n == 0 {
            return;
        }
        for step in 1..=n {
            let i = (self.active as isize + d * step as isize).rem_euclid(n as isize) as usize;
            if !self.options[i].disabled {
                self.active = i;
                self.list_scroll.scroll_to_item(i, ScrollStrategy::Nearest);
                cx.notify();
                break;
            }
        }
        self.visibility(true, cx)
    }
    fn next(&mut self, _: &Next, _: &mut Window, cx: &mut Context<Self>) {
        self.move_by(1, cx)
    }
    fn previous(&mut self, _: &Previous, _: &mut Window, cx: &mut Context<Self>) {
        self.move_by(-1, cx)
    }
    fn first(&mut self, _: &First, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if let Some(i) = self.options.iter().position(|o| !o.disabled) {
            self.active = i;
            self.list_scroll.scroll_to_item(i, ScrollStrategy::Nearest);
            cx.notify();
            self.visibility(true, cx)
        }
    }
    fn last(&mut self, _: &Last, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if let Some(i) = self.options.iter().rposition(|o| !o.disabled) {
            self.active = i;
            self.list_scroll.scroll_to_item(i, ScrollStrategy::Nearest);
            cx.notify();
            self.visibility(true, cx)
        }
    }
    fn toggle(&mut self, _: &Toggle, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if !self.open {
            self.visibility(true, cx);
            return;
        }
        if let Some(o) = self.options.get(self.active).filter(|o| !o.disabled) {
            let id = o.id.clone();
            let mut next = self.values.clone();
            if let Some(i) = next.iter().position(|v| v == &id) {
                next.remove(i);
            } else {
                next.push(id.clone());
            }
            if !self.controlled {
                self.values = next.clone();
                cx.notify();
            }
            cx.emit(ValuesChanged(next));
        }
    }
    fn close(&mut self, _: &Close, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.visibility(false, cx)
    }
    fn tab_next(&mut self, _: &TabForward, window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled {
            window.focus_next(cx);
            self.visibility(false, cx);
        }
    }
    fn tab_previous(&mut self, _: &TabBackward, window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled {
            window.focus_prev(cx);
            self.visibility(false, cx);
        }
    }
}
impl Focusable for MultiSelect {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}
impl Render for MultiSelect {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let e = cx.entity();
        let summary = self
            .options
            .iter()
            .filter(|o| self.values.contains(&o.id))
            .map(|o| o.label.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let disabled = self.disabled;
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_stop(!disabled)).clone();
        let trigger = div()
            .id("mkit-multi-select-trigger")
            .debug_selector(|| "mkit-multi-select-trigger".into())
            .h(px(t.controls.medium))
            .px(px(t.spacing.small))
            .flex()
            .items_center()
            .border(px(t.borders.regular))
            .border_color(t.colors.border)
            .rounded(px(t.radii.small))
            .bg(t.colors.surface)
            .text_color(if self.disabled { t.colors.disabled } else { t.colors.text })
            .child(if summary.is_empty() { self.label.clone() } else { summary.clone() });
        let mut root = div()
            .id(("mkit-multi-select", cx.entity().entity_id()))
            .debug_selector(|| "mkit-multi-select".into())
            .key_context(KEY_CONTEXT)
            .when(!disabled, |root| root.track_focus(&focus))
            .role(gpui_pre::accesskit::Role::ComboBox)
            .aria_label(self.label.clone())
            .aria_value(summary)
            .aria_expanded(self.open)
            .a11y_synthetic_children(move |builder| {
                let node = builder.parent_node();
                node.set_has_popup(gpui_pre::accesskit::HasPopup::Listbox);
                if disabled {
                    node.set_disabled();
                }
            })
            .flex()
            .flex_col()
            .gap(px(t.spacing.xsmall))
            .on_action(cx.listener(Self::next))
            .on_action(cx.listener(Self::previous))
            .on_action(cx.listener(Self::first))
            .on_action(cx.listener(Self::last))
            .on_action(cx.listener(Self::toggle))
            .on_action(cx.listener(Self::close))
            .on_action(cx.listener(Self::tab_next))
            .on_action(cx.listener(Self::tab_previous))
            .on_click(cx.listener(|s, _, _, cx| {
                if !s.disabled {
                    s.visibility(!s.open, cx);
                }
            }));
        if self.open {
            let list = div()
                .id("mkit-multi-select-popup")
                .debug_selector(|| "mkit-multi-select-popup".into())
                .role(gpui_pre::accesskit::Role::ListBox)
                .aria_label(self.label.clone())
                .a11y_synthetic_children(|builder| {
                    builder.parent_node().set_multiselectable();
                })
                .border(px(t.borders.regular))
                .border_color(t.colors.border)
                .bg(t.colors.elevated_surface)
                .on_mouse_down_out(cx.listener(|s, _, _, cx| s.visibility(false, cx)));
            let n = self.options.len();
            let options = self.options.clone();
            let values = self.values.clone();
            let active = self.active;
            let scroll = self.list_scroll.clone();
            let popup = list.child(
                uniform_list(
                    ("mkit-multi-select-options", cx.entity().entity_id()),
                    n,
                    move |range, _, _| {
                        range
                            .map(|i| {
                                let o = &options[i];
                                let ent = e.clone();
                                let id = o.id.clone();
                                let selector_id = id.clone();
                                let is_active = i == active;
                                let is_selected = values.contains(&o.id);
                                div()
                                    .id(("mkit-multi-select-option", i))
                                    .debug_selector(move || {
                                        format!("mkit-multi-select-option-{selector_id}")
                                    })
                                    .role(gpui_pre::accesskit::Role::ListBoxOption)
                                    .aria_label(o.label.clone())
                                    .aria_selected(is_selected)
                                    .when(o.disabled, |row| {
                                        row.a11y_synthetic_children(|builder| {
                                            builder.parent_node().set_disabled()
                                        })
                                    })
                                    .when(is_active, |row| row.aria_active_descendant())
                                    .h(px(t.controls.medium))
                                    .px(px(t.spacing.small))
                                    .bg(if is_active {
                                        t.colors.accent
                                    } else {
                                        t.colors.elevated_surface
                                    })
                                    .text_color(if o.disabled {
                                        t.colors.disabled
                                    } else if is_active {
                                        t.colors.accent_text
                                    } else {
                                        t.colors.text
                                    })
                                    .child(format!(
                                        "{}{}",
                                        if is_selected { "✓ " } else { "" },
                                        o.label
                                    ))
                                    .on_click(move |_, _, cx| {
                                        cx.stop_propagation();
                                        ent.update(cx, |s, cx| {
                                            if s.disabled || s.options[i].disabled {
                                                return;
                                            }
                                            let mut next = s.values.clone();
                                            if let Some(i) = next.iter().position(|v| v == &id) {
                                                next.remove(i);
                                            } else {
                                                next.push(id.clone());
                                            }
                                            if !s.controlled {
                                                s.values = next.clone();
                                                cx.notify();
                                            }
                                            cx.emit(ValuesChanged(next));
                                        })
                                    })
                            })
                            .collect::<Vec<_>>()
                    },
                )
                .track_scroll(&scroll)
                .w_full()
                .h(px(t.controls.medium * n.min(8) as f32)),
            );
            let trigger_bounds = self.trigger_bounds.borrow();
            let popup_height = px(t.controls.medium * n.min(8) as f32);
            let margin = px(t.spacing.medium);
            let gap = px(t.spacing.small);
            let window_height = window.bounds().size.height;
            let (anchor, position) = trigger_bounds
                .as_ref()
                .map(|b| {
                    let below = window_height - margin - b.bottom() - gap;
                    let above = b.top() - margin - gap;
                    if popup_height > below && above > below {
                        (Anchor::BottomLeft, point(b.left(), b.top() - gap))
                    } else {
                        (Anchor::TopLeft, point(b.left(), b.bottom() + gap))
                    }
                })
                .unwrap_or((Anchor::TopLeft, point(px(0.), px(0.))));
            let popup_width = trigger_bounds.as_ref().map(|b| b.size.width).unwrap_or(px(240.));
            let popup = popup.w(popup_width);
            root = root.child(
                deferred(
                    anchored()
                        .anchor(anchor)
                        .position_mode(AnchoredPositionMode::Window)
                        .position(position)
                        .snap_to_window_with_margin(margin)
                        .child(popup),
                )
                .with_priority(mkit_core::overlay::layer::POPOVER),
            );
        }
        let bounds = Rc::clone(&self.trigger_bounds);
        let trigger = div()
            .on_children_prepainted(move |children, _, _| {
                if let Some(b) = children.first() {
                    *bounds.borrow_mut() = Some(*b);
                }
            })
            .child(trigger);
        root.child(trigger)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{AppContext, Entity, ParentElement, TestAppContext, div};
    actions!(multi_select_test, [FocusNext, FocusPrevious]);

    #[test]
    fn membership_toggle_adds_and_removes_without_duplicates() {
        let mut values = vec!["a".to_owned()];
        if let Some(i) = values.iter().position(|v| v == "b") {
            values.remove(i);
        } else {
            values.push("b".into())
        }
        assert_eq!(values, ["a", "b"]);
        if let Some(i) = values.iter().position(|v| v == "b") {
            values.remove(i);
        } else {
            values.push("b".into())
        }
        assert_eq!(values, ["a"]);
    }

    struct MultiSelectHost {
        select: Option<Entity<MultiSelect>>,
        options: Vec<OptionItem>,
    }

    impl Render for MultiSelectHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.select.is_none() {
                let options = self.options.clone();
                self.select = Some(cx.new(move |_| MultiSelect::new("Options", options, vec![])));
            }
            div().child(self.select.as_ref().unwrap().clone())
        }
    }

    struct BottomMultiSelectHost {
        select: Option<Entity<MultiSelect>>,
        options: Vec<OptionItem>,
    }

    struct TabMultiSelectHost {
        select: Option<Entity<MultiSelect>>,
        before: FocusHandle,
        after: FocusHandle,
    }

    impl Render for TabMultiSelectHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.select.is_none() {
                self.select = Some(cx.new(|_| {
                    MultiSelect::new("Options", vec![OptionItem::new("a", "A")], vec!["a".into()])
                }));
            }
            div()
                .key_context("MultiSelectTestHost")
                .on_action(cx.listener(|_, _: &FocusNext, window, cx| window.focus_next(cx)))
                .on_action(cx.listener(|_, _: &FocusPrevious, window, cx| window.focus_prev(cx)))
                .child(div().id("before").track_focus(&self.before).tab_stop(true))
                .child(self.select.as_ref().unwrap().clone())
                .child(div().id("after").track_focus(&self.after).tab_stop(true))
        }
    }

    impl Render for BottomMultiSelectHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.select.is_none() {
                let options = self.options.clone();
                self.select = Some(cx.new(move |_| MultiSelect::new("Options", options, vec![])));
            }
            div().pt(px(900.)).child(self.select.as_ref().unwrap().clone())
        }
    }

    fn options() -> Vec<OptionItem> {
        (0..120).map(|i| OptionItem::new(format!("option-{i}"), format!("Option {i}"))).collect()
    }

    #[gpui_pre::test]
    fn large_list_virtualizes_and_keyboard_scrolls_last_option_into_view(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) =
            cx.add_window_view(|_, _| MultiSelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        visual.update(|window, cx| select.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("end");
        visual.update(|window, cx| window.draw(cx).clear(cx));

        select.read_with(visual, |select, _| {
            assert!(select.open);
            assert_eq!(select.active, 119);
            assert!(select.list_scroll.is_scrollable());
            assert_eq!(select.list_scroll.is_scrolled_to_end(), Some(true));
        });
    }

    #[gpui_pre::test]
    fn tab_leaves_multiselect_closes_popup_and_preserves_values(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| {
            app.bind_keys(default_key_bindings());
            app.bind_keys([
                gpui_pre::KeyBinding::new("tab", FocusNext, Some("MultiSelectTestHost")),
                gpui_pre::KeyBinding::new("shift-tab", FocusPrevious, Some("MultiSelectTestHost")),
            ]);
        });
        let (host, visual) = cx.add_window_view(|_, cx| TabMultiSelectHost {
            select: None,
            before: cx.focus_handle().tab_stop(true),
            after: cx.focus_handle().tab_stop(true),
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (select, before, after) = host.read_with(visual, |host, _| {
            (host.select.as_ref().unwrap().clone(), host.before.clone(), host.after.clone())
        });
        visual.update(|window, cx| before.focus(window, cx));
        visual.simulate_keystrokes("tab");
        let trigger_focus = select.read_with(visual, |select, cx| select.focus_handle(cx));
        assert!(visual.update(|window, _| trigger_focus.is_focused(window)));
        select.update(visual, |select, cx| select.visibility(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(select.read_with(visual, |select, _| select.is_open()));
        visual.simulate_keystrokes("tab");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.update(|window, _| after.is_focused(window)));
        select.read_with(visual, |select, _| {
            assert!(!select.is_open());
            assert_eq!(select.values(), ["a"]);
        });

        visual.update(|window, cx| trigger_focus.focus(window, cx));
        select.update(visual, |select, cx| select.visibility(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(select.read_with(visual, |select, _| select.is_open()));
        visual.simulate_keystrokes("shift-tab");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.update(|window, _| before.is_focused(window)));
        select.read_with(visual, |select, _| {
            assert!(!select.is_open());
            assert_eq!(select.values(), ["a"]);
        });
    }

    #[gpui_pre::test]
    fn large_list_responds_to_pointer_wheel_scroll(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| MultiSelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        select.update(visual, |select, cx| select.visibility(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let popup = visual.debug_bounds("mkit-multi-select-popup").expect("popup bounds");
        visual.simulate_event(gpui_pre::ScrollWheelEvent {
            position: popup.center(),
            delta: gpui_pre::ScrollDelta::Pixels(gpui_pre::point(px(0.), px(-10000.))),
            modifiers: Default::default(),
            touch_phase: gpui_pre::TouchPhase::Moved,
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));

        select.read_with(visual, |select, _| {
            assert!(select.list_scroll.is_scrollable());
            assert_eq!(select.list_scroll.is_scrolled_to_end(), Some(true));
        });
    }

    #[gpui_pre::test]
    fn pointer_click_after_virtual_scroll_toggles_visible_source_option(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| MultiSelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        select.update(visual, |select, cx| select.visibility(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let popup = visual.debug_bounds("mkit-multi-select-popup").expect("popup bounds");
        visual.simulate_event(gpui_pre::ScrollWheelEvent {
            position: popup.center(),
            delta: gpui_pre::ScrollDelta::Pixels(gpui_pre::point(px(0.), px(-10000.))),
            modifiers: Default::default(),
            touch_phase: gpui_pre::TouchPhase::Moved,
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let row = visual
            .debug_bounds("mkit-multi-select-option-option-119")
            .expect("last virtual row")
            .center();
        visual.simulate_mouse_down(row, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_up(row, gpui_pre::MouseButton::Left, Default::default());
        assert_eq!(select.read_with(visual, |select, _| select.values().to_vec()), ["option-119"]);
        assert!(select.read_with(visual, |select, _| select.is_open()));
    }

    #[gpui_pre::test]
    fn popup_is_anchored_to_trigger_and_clipped_to_window(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| MultiSelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        select.update(visual, |select, cx| select.visibility(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let trigger = visual.debug_bounds("mkit-multi-select-trigger").expect("trigger bounds");
        let popup = visual.debug_bounds("mkit-multi-select-popup").expect("popup bounds");
        let gap = visual.update(|_, cx| cx.global::<Theme>().spacing.small);
        assert!((f32::from(popup.left()) - f32::from(trigger.left())).abs() <= 1.0);
        assert!(f32::from(popup.top()) >= f32::from(trigger.bottom()) + gap - 1.0);
        assert!(f32::from(popup.bottom()) <= 768.0 - gap * 2.0);
    }

    #[gpui_pre::test]
    fn popup_stays_inside_window_when_trigger_is_near_bottom(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| BottomMultiSelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        select.update(visual, |select, cx| select.visibility(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let popup = visual.debug_bounds("mkit-multi-select-popup").expect("popup bounds");
        let trigger = visual.debug_bounds("mkit-multi-select-trigger").expect("trigger bounds");
        let margin = visual.update(|_, cx| cx.global::<Theme>().spacing.medium);
        let window_height = visual.update(|window, _| window.bounds().size.height);
        assert!(f32::from(popup.bottom()) <= f32::from(trigger.top()) - 1.0);
        assert!(f32::from(popup.bottom()) <= f32::from(window_height) - margin + 1.0);
    }

    #[gpui_pre::test]
    fn trigger_and_popup_rows_use_pointer_interaction(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| MultiSelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        let trigger =
            visual.debug_bounds("mkit-multi-select-trigger").expect("trigger bounds").center();
        visual.simulate_mouse_down(trigger, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_up(trigger, gpui_pre::MouseButton::Left, Default::default());
        assert!(select.read_with(visual, |select, _| select.is_open()));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let row = visual
            .debug_bounds("mkit-multi-select-option-option-1")
            .expect("first option bounds")
            .center();
        visual.simulate_mouse_down(row, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_up(row, gpui_pre::MouseButton::Left, Default::default());
        assert_eq!(select.read_with(visual, |select, _| select.values().to_vec()), ["option-1"]);
        assert!(select.read_with(visual, |select, _| select.is_open()));
    }

    #[gpui_pre::test]
    fn outside_pointer_dismisses_popup(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| MultiSelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        select.update(visual, |select, cx| select.visibility(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_mouse_down(
            gpui_pre::point(px(900.), px(700.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );
        assert!(!select.read_with(visual, |select, _| select.open));
    }
}
