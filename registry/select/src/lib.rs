//! Themed single-value select control.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Anchor, AnchoredPositionMode, Bounds, Context, EventEmitter, FocusHandle, Focusable,
    InteractiveElement, IntoElement, PathBuilder, Pixels, Render, Rgba, ScrollStrategy,
    UniformListScrollHandle, Window, actions, anchored, canvas, deferred, div, point, prelude::*,
    px, uniform_list,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
use std::{cell::RefCell, rc::Rc};

/// Resolved trigger and popup colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    high_contrast: bool,
    background: Rgba,
    trigger_border: Rgba,
    text: Rgba,
    placeholder: Rgba,
    icon: Rgba,
    popup_bg: Rgba,
    popup_border: Rgba,
    active_bg: Rgba,
    active_text: Rgba,
    /// Pointer-hover fill for enabled rows; high contrast keeps rows unchanged.
    hover_bg: Option<Rgba>,
    focus: Rgba,
    ring: Rgba,
    disabled: Rgba,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            high_contrast: true,
            background: c.background,
            trigger_border: c.border,
            text: c.text,
            placeholder: c.text_muted,
            icon: c.text,
            popup_bg: c.background,
            popup_border: c.border,
            active_bg: c.accent,
            active_text: c.accent_text,
            hover_bg: None,
            focus: c.focus,
            ring: c.focus,
            disabled: c.disabled,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    Look {
        high_contrast: false,
        background: c.background,
        trigger_border: if dark { c.text.opacity(0.15) } else { c.border },
        text: c.text,
        placeholder: c.text_muted,
        icon: c.text_muted,
        popup_bg: c.surface,
        popup_border: if dark { c.text.opacity(0.1) } else { c.border },
        active_bg: muted,
        active_text: c.text,
        hover_bg: Some(muted),
        focus: c.focus,
        ring: c.focus.opacity(0.5),
        disabled: c.disabled,
    }
}
/// The web preview's `opacity: .5` applied as one layer: composite over `base`, then mix 50%.
fn dim(color: Rgba, base: Rgba) -> Rgba {
    mix(composite(color, base), base, 0.5)
}
fn box_shadow(shadow: ShadowToken, alpha: f32) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: Rgba { a: shadow.color.a * alpha, ..shadow.color }.into(),
        offset: point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}
/// shadcn/ui focus ring width, drawn outside the trigger.
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
/// Decorative Lucide icon drawn as a vector stroke so it stays crisp at every scale. Each
/// polyline is a list of points on a 24-unit grid; the stroke is 2 units, Lucide's default.
fn icon(size: f32, lines: &'static [&'static [(f32, f32)]], color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let origin = bounds.origin;
            let mut path = PathBuilder::stroke(unit * 2.0);
            for line in lines {
                for (i, (x, y)) in line.iter().enumerate() {
                    let at = origin + point(unit * *x, unit * *y);
                    if i == 0 { path.move_to(at) } else { path.line_to(at) }
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
}
/// Lucide `chevron-down`.
const CHEVRON_DOWN: &[&[(f32, f32)]] = &[&[(6., 9.), (12., 15.), (18., 9.)]];
/// Lucide `check`.
const CHECK: &[&[(f32, f32)]] = &[&[(20., 6.), (9., 17.), (4., 12.)]];

pub const KEY_CONTEXT: &str = "MkitSelect";
actions!(select, [Next, Previous, First, Last, Toggle, Close, Traverse, TraversePrevious]);
pub fn default_key_bindings() -> [gpui_pre::KeyBinding; 9] {
    [
        gpui_pre::KeyBinding::new("down", Next, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("up", Previous, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("home", First, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("end", Last, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("enter", Toggle, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("space", Toggle, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("escape", Close, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("tab", Traverse, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("shift-tab", TraversePrevious, Some(KEY_CONTEXT)),
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
pub struct ValueChanged(pub Option<String>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenChanged(pub bool);
impl EventEmitter<ValueChanged> for Select {}
impl EventEmitter<OpenChanged> for Select {}
pub struct Select {
    label: String,
    options: Vec<OptionItem>,
    value: Option<String>,
    controlled: bool,
    disabled: bool,
    open: bool,
    active: usize,
    focus: Option<FocusHandle>,
    list_scroll: UniformListScrollHandle,
    trigger_bounds: Rc<RefCell<Option<Bounds<Pixels>>>>,
}
impl Select {
    pub fn new(
        label: impl Into<String>,
        options: Vec<OptionItem>,
        default_value: Option<String>,
    ) -> Self {
        let active = default_value
            .as_ref()
            .and_then(|v| options.iter().position(|o| &o.id == v))
            .unwrap_or(0);
        Self {
            label: label.into(),
            options,
            value: default_value,
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
        value: Option<String>,
    ) -> Self {
        let mut s = Self::new(label, options, value);
        s.controlled = true;
        s
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn set_value(&mut self, v: Option<String>, cx: &mut Context<Self>) {
        self.value = v;
        cx.notify()
    }
    fn visibility(&mut self, v: bool, cx: &mut Context<Self>) {
        if self.open != v {
            self.open = v;
            if v {
                self.list_scroll.scroll_to_item(self.active, ScrollStrategy::Nearest);
            }
            cx.emit(OpenChanged(v));
            cx.notify()
        }
    }
    fn move_to(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.disabled || self.options.is_empty() {
            return;
        }
        let n = self.options.len();
        for step in 1..=n {
            let i = (self.active as isize + delta * step as isize).rem_euclid(n as isize) as usize;
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
        self.move_to(1, cx)
    }
    fn previous(&mut self, _: &Previous, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(-1, cx)
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
        if self.open {
            if let Some(o) = self.options.get(self.active).filter(|o| !o.disabled) {
                let next = Some(o.id.clone());
                if !self.controlled {
                    self.value = next.clone();
                    cx.notify();
                }
                cx.emit(ValueChanged(next));
            }
            self.visibility(false, cx)
        } else {
            self.visibility(true, cx)
        }
    }
    fn close(&mut self, _: &Close, _: &mut Window, cx: &mut Context<Self>) {
        self.visibility(false, cx)
    }
    fn traverse(&mut self, _: &Traverse, window: &mut Window, cx: &mut Context<Self>) {
        self.visibility(false, cx);
        window.focus_next(cx);
    }
    fn traverse_previous(
        &mut self,
        _: &TraversePrevious,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.visibility(false, cx);
        window.focus_prev(cx);
    }
}
impl Focusable for Select {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}
impl Render for Select {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let e = cx.entity();
        let selected_label = self
            .value
            .as_ref()
            .and_then(|v| self.options.iter().find(|o| &o.id == v))
            .map(|o| o.label.clone())
            .unwrap_or_default();
        let title =
            if selected_label.is_empty() { self.label.clone() } else { selected_label.clone() };
        let disabled = self.disabled;
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone().tab_stop(!disabled);
        let look = look(&t);
        // `:focus-visible`: the trigger shows the ring while the Select owns keyboard focus.
        let focus_visible =
            !disabled && focus.is_focused(window) && window.last_input_was_keyboard();
        let (border, text, placeholder, chevron) = if !disabled {
            (look.trigger_border, look.text, look.placeholder, look.icon)
        } else if look.high_contrast {
            (look.disabled, look.disabled, look.disabled, look.disabled)
        } else {
            let bg = look.background;
            (
                dim(look.trigger_border, bg),
                dim(look.text, bg),
                dim(look.placeholder, bg),
                dim(look.icon, bg),
            )
        };
        let trigger = div()
            .id("mkit-select-trigger")
            .debug_selector(|| "mkit-select-trigger".into())
            .h(px(t.controls.medium))
            .px(px(t.spacing.medium))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(t.spacing.small))
            .border(px(t.borders.regular))
            .border_color(if focus_visible { look.focus } else { border })
            .rounded(px(t.radii.medium))
            .bg(look.background)
            .shadow(if focus_visible {
                vec![focus_ring(look.ring)]
            } else if look.high_contrast {
                Vec::new()
            } else {
                vec![box_shadow(t.shadows.small, if disabled { 0.5 } else { 1.0 })]
            })
            .text_size(px(t.typography.body))
            .text_color(if selected_label.is_empty() { placeholder } else { text })
            .child(div().flex_1().min_w_0().truncate().child(title))
            .child(icon(t.spacing.large, CHEVRON_DOWN, chevron));
        let mut root = div()
            .id(("mkit-select", cx.entity().entity_id()))
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .role(gpui_pre::accesskit::Role::ComboBox)
            .aria_label(self.label.clone())
            .aria_value(selected_label)
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
            .on_action(cx.listener(Self::traverse))
            .on_action(cx.listener(Self::traverse_previous))
            .on_click(cx.listener(|s, _, _, cx| {
                if !s.disabled {
                    s.visibility(!s.open, cx);
                }
            }));
        if self.open {
            let rows = self.options.len().min(8) as f32;
            let row_height = t.controls.small;
            let chrome = 2. * (t.spacing.xsmall + t.borders.regular);
            let list = div()
                .id("mkit-select-popup")
                .debug_selector(|| "mkit-select-popup".into())
                .role(gpui_pre::accesskit::Role::ListBox)
                .aria_label(self.label.clone())
                .p(px(t.spacing.xsmall))
                .border(px(t.borders.regular))
                .border_color(look.popup_border)
                .rounded(px(t.radii.medium))
                .bg(look.popup_bg)
                .when(!look.high_contrast, |e| e.shadow(vec![box_shadow(t.shadows.medium, 1.0)]))
                .text_size(px(t.typography.body))
                .on_mouse_down_out(cx.listener(|s, _, _, cx| s.visibility(false, cx)));
            let n = self.options.len();
            let options = self.options.clone();
            let selected = self.value.clone();
            let active = self.active;
            let scroll = self.list_scroll.clone();
            let popup = list.child(
                uniform_list(
                    ("mkit-select-options", cx.entity().entity_id()),
                    n,
                    move |range, _, _| {
                        range
                            .map(|i| {
                                let o = &options[i];
                                let ent = e.clone();
                                let id = o.id.clone();
                                let option_disabled = o.disabled;
                                let is_active = i == active;
                                let is_selected = selected.as_deref() == Some(&o.id);
                                let (fg, check) = if option_disabled {
                                    if look.high_contrast {
                                        (look.disabled, look.disabled)
                                    } else {
                                        let bg = look.popup_bg;
                                        (dim(look.text, bg), dim(look.icon, bg))
                                    }
                                } else if is_active {
                                    let check = if look.high_contrast {
                                        look.active_text
                                    } else {
                                        look.icon
                                    };
                                    (look.active_text, check)
                                } else {
                                    (look.text, look.icon)
                                };
                                let hover_bg = look.hover_bg.filter(|_| !option_disabled);
                                div()
                                    .id(("mkit-select-option", i))
                                    .debug_selector(move || format!("mkit-select-option-{i}"))
                                    .role(gpui_pre::accesskit::Role::ListBoxOption)
                                    .aria_label(o.label.clone())
                                    .aria_selected(is_selected)
                                    .a11y_synthetic_children(move |builder| {
                                        if option_disabled {
                                            builder.parent_node().set_disabled();
                                        }
                                    })
                                    .when(is_active, |row| row.aria_active_descendant())
                                    .h(px(row_height))
                                    .w_full()
                                    .px(px(t.spacing.small))
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap(px(t.spacing.small))
                                    .rounded(px(t.radii.small))
                                    .when(is_active, |row| row.bg(look.active_bg))
                                    .when_some(hover_bg.filter(|_| !is_active), |row, bg| {
                                        row.hover(move |s| s.bg(bg))
                                    })
                                    .text_color(fg)
                                    .child(
                                        div().flex_1().min_w_0().truncate().child(o.label.clone()),
                                    )
                                    .when(is_selected, |row| {
                                        row.child(icon(t.spacing.large, CHECK, check))
                                    })
                                    .on_click(move |_, _, cx| {
                                        cx.stop_propagation();
                                        ent.update(cx, |s, cx| {
                                            if !s.disabled && !s.options[i].disabled {
                                                let next = Some(id.clone());
                                                if !s.controlled {
                                                    s.value = next.clone();
                                                    cx.notify();
                                                }
                                                cx.emit(ValueChanged(next));
                                                s.visibility(false, cx)
                                            }
                                        })
                                    })
                            })
                            .collect::<Vec<_>>()
                    },
                )
                .track_scroll(&scroll)
                .w_full()
                .h(px(row_height * rows)),
            );
            let trigger_bounds = self.trigger_bounds.borrow();
            let popup_height = px(row_height * rows + chrome);
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
    actions!(select_test, [FocusNext]);

    #[test]
    fn disabled_options_are_skipped_by_navigation() {
        let options = [
            OptionItem::new("a", "A").disabled(true),
            OptionItem::new("b", "B"),
            OptionItem::new("c", "C"),
        ];
        let next_enabled = options.iter().position(|o| !o.disabled);
        assert_eq!(next_enabled, Some(1));
        assert_eq!(options.iter().rposition(|o| !o.disabled), Some(2));
    }

    struct SelectHost {
        select: Option<Entity<Select>>,
        options: Vec<OptionItem>,
    }

    impl Render for SelectHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.select.is_none() {
                let options = self.options.clone();
                self.select = Some(cx.new(move |_| Select::new("Options", options, None)));
            }
            div().child(self.select.as_ref().unwrap().clone())
        }
    }

    struct BottomSelectHost {
        select: Option<Entity<Select>>,
        options: Vec<OptionItem>,
    }

    struct TabTraversalHost {
        enabled: Option<Entity<Select>>,
        disabled: Option<Entity<Select>>,
        before: FocusHandle,
        after: FocusHandle,
    }

    impl Render for TabTraversalHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.enabled.is_none() {
                self.enabled = Some(cx.new(|_| {
                    Select::new(
                        "Enabled",
                        vec![OptionItem::new("enabled", "Enabled")],
                        Some("enabled".into()),
                    )
                }));
                self.disabled = Some(cx.new(|_| {
                    Select::new("Disabled", vec![OptionItem::new("disabled", "Disabled")], None)
                        .disabled(true)
                }));
            }
            div()
                .key_context(KEY_CONTEXT)
                .on_action(cx.listener(|_, _: &FocusNext, window, cx| window.focus_next(cx)))
                .child(div().id("before").track_focus(&self.before).tab_stop(true))
                .child(self.enabled.as_ref().unwrap().clone())
                .child(self.disabled.as_ref().unwrap().clone())
                .child(div().id("after").track_focus(&self.after).tab_stop(true))
        }
    }

    impl Render for BottomSelectHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.select.is_none() {
                let options = self.options.clone();
                self.select = Some(cx.new(move |_| Select::new("Options", options, None)));
            }
            div().pt(px(900.)).child(self.select.as_ref().unwrap().clone())
        }
    }

    fn options() -> Vec<OptionItem> {
        (0..120).map(|i| OptionItem::new(format!("option-{i}"), format!("Option {i}"))).collect()
    }

    #[gpui_pre::test]
    fn tab_traversal_reaches_enabled_select_and_skips_disabled_select(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| {
            let mut bindings = default_key_bindings().to_vec();
            bindings.push(gpui_pre::KeyBinding::new("tab", FocusNext, Some(KEY_CONTEXT)));
            app.bind_keys(bindings);
        });
        let (host, visual) = cx.add_window_view(|_, cx| TabTraversalHost {
            enabled: None,
            disabled: None,
            before: cx.focus_handle().tab_stop(true),
            after: cx.focus_handle().tab_stop(true),
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (enabled, disabled, before, after) = host.read_with(visual, |host, cx| {
            (
                host.enabled.as_ref().unwrap().focus_handle(cx),
                host.disabled.as_ref().unwrap().focus_handle(cx),
                host.before.clone(),
                host.after.clone(),
            )
        });

        visual.update(|window, cx| before.focus(window, cx));
        visual.simulate_keystrokes("tab");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.update(|window, _| enabled.is_focused(window)));
        visual.simulate_keystrokes("tab");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.update(|window, _| after.is_focused(window)));
        assert!(!visual.update(|window, _| disabled.is_focused(window)));
    }

    #[gpui_pre::test]
    fn tab_closes_open_select_and_moves_focus_to_next_stop(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, cx| TabTraversalHost {
            enabled: None,
            disabled: None,
            before: cx.focus_handle().tab_stop(true),
            after: cx.focus_handle().tab_stop(true),
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (enabled_select, enabled_focus, after) = host.read_with(visual, |host, cx| {
            let enabled_select = host.enabled.as_ref().unwrap().clone();
            (enabled_select.clone(), enabled_select.focus_handle(cx), host.after.clone())
        });

        visual.update(|window, cx| enabled_focus.focus(window, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("enter");
        assert!(enabled_select.read_with(visual, |select, _| select.is_open()));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("tab");
        visual.update(|window, cx| window.draw(cx).clear(cx));

        assert!(!enabled_select.read_with(visual, |select, _| select.is_open()));
        assert!(visual.update(|window, _| after.is_focused(window)));
        assert_eq!(
            enabled_select.read_with(visual, |select, _| select.value().map(str::to_owned)),
            Some("enabled".into())
        );
    }

    #[gpui_pre::test]
    fn shift_tab_closes_open_select_and_moves_focus_to_previous_stop(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, cx| TabTraversalHost {
            enabled: None,
            disabled: None,
            before: cx.focus_handle().tab_stop(true),
            after: cx.focus_handle().tab_stop(true),
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (enabled_select, enabled_focus, before) = host.read_with(visual, |host, cx| {
            let enabled_select = host.enabled.as_ref().unwrap().clone();
            (enabled_select.clone(), enabled_select.focus_handle(cx), host.before.clone())
        });

        visual.update(|window, cx| enabled_focus.focus(window, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("enter");
        assert!(enabled_select.read_with(visual, |select, _| select.is_open()));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("shift-tab");
        visual.update(|window, cx| window.draw(cx).clear(cx));

        assert!(!enabled_select.read_with(visual, |select, _| select.is_open()));
        assert!(visual.update(|window, _| before.is_focused(window)));
        assert_eq!(
            enabled_select.read_with(visual, |select, _| select.value().map(str::to_owned)),
            Some("enabled".into())
        );
    }

    #[gpui_pre::test]
    fn large_list_virtualizes_and_keyboard_scrolls_last_option_into_view(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) =
            cx.add_window_view(|_, _| SelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        visual.update(|window, cx| select.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("enter end");
        visual.update(|window, cx| window.draw(cx).clear(cx));

        select.read_with(visual, |select, _| {
            assert!(select.open);
            assert_eq!(select.active, 119);
            assert!(select.list_scroll.is_scrollable());
            assert_eq!(select.list_scroll.is_scrolled_to_end(), Some(true));
        });
    }

    #[gpui_pre::test]
    fn large_list_responds_to_pointer_wheel_scroll(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| SelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        select.update(visual, |select, cx| select.visibility(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let popup = visual.debug_bounds("mkit-select-popup").expect("popup bounds");
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
    fn pointer_click_after_virtual_scroll_commits_visible_source_option(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| SelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        select.update(visual, |select, cx| select.visibility(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let popup = visual.debug_bounds("mkit-select-popup").expect("popup bounds");
        visual.simulate_event(gpui_pre::ScrollWheelEvent {
            position: popup.center(),
            delta: gpui_pre::ScrollDelta::Pixels(gpui_pre::point(px(0.), px(-10000.))),
            modifiers: Default::default(),
            touch_phase: gpui_pre::TouchPhase::Moved,
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let row = visual.debug_bounds("mkit-select-option-119").expect("last virtual row").center();
        visual.simulate_mouse_down(row, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_up(row, gpui_pre::MouseButton::Left, Default::default());
        assert_eq!(
            select.read_with(visual, |select, _| select.value().map(str::to_owned)),
            Some("option-119".into())
        );
        assert!(!select.read_with(visual, |select, _| select.is_open()));
    }

    #[gpui_pre::test]
    fn popup_is_anchored_to_trigger_and_clipped_to_window(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| SelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        select.update(visual, |select, cx| select.visibility(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let trigger = visual.debug_bounds("mkit-select-trigger").expect("trigger bounds");
        let popup = visual.debug_bounds("mkit-select-popup").expect("popup bounds");
        let gap = visual.update(|_, cx| cx.global::<Theme>().spacing.small);
        assert!((f32::from(popup.left()) - f32::from(trigger.left())).abs() <= 1.0);
        assert!(f32::from(popup.top()) >= f32::from(trigger.bottom()) + gap - 1.0);
        assert!(f32::from(popup.bottom()) <= 768.0 - gap * 2.0);
    }

    #[gpui_pre::test]
    fn popup_stays_inside_window_when_trigger_is_near_bottom(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| BottomSelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        select.update(visual, |select, cx| select.visibility(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let popup = visual.debug_bounds("mkit-select-popup").expect("popup bounds");
        let trigger = visual.debug_bounds("mkit-select-trigger").expect("trigger bounds");
        let margin = visual.update(|_, cx| cx.global::<Theme>().spacing.medium);
        let window_height = visual.update(|window, _| window.bounds().size.height);
        assert!(f32::from(popup.bottom()) <= f32::from(trigger.top()) - 1.0);
        assert!(f32::from(popup.bottom()) <= f32::from(window_height) - margin + 1.0);
    }

    #[gpui_pre::test]
    fn trigger_and_popup_rows_use_pointer_interaction(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| SelectHost { select: None, options: options() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let select = host.read_with(visual, |host, _| host.select.as_ref().unwrap().clone());
        let trigger = visual.debug_bounds("mkit-select-trigger").expect("trigger bounds").center();
        visual.simulate_mouse_down(trigger, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_up(trigger, gpui_pre::MouseButton::Left, Default::default());
        assert!(select.read_with(visual, |select, _| select.is_open()));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let row =
            visual.debug_bounds("mkit-select-option-1").expect("first option bounds").center();
        visual.simulate_mouse_down(row, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_up(row, gpui_pre::MouseButton::Left, Default::default());
        assert_eq!(
            select.read_with(visual, |select, _| select.value().map(str::to_owned)),
            Some("option-1".into())
        );
        assert!(!select.read_with(visual, |select, _| select.is_open()));
    }

    #[gpui_pre::test]
    fn outside_pointer_dismisses_popup(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) =
            cx.add_window_view(|_, _| SelectHost { select: None, options: options() });
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
