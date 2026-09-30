//! Grouped disclosures with single-open or multiple-open state.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnyElement, Context, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement, KeyBinding,
    PathBuilder, Render, Rgba, Window, actions, canvas, div, point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::Theme,
};

/// Resolved accordion colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    /// Opaque fill under the trigger so the focus ring never tints it.
    fill: Rgba,
    separator: Rgba,
    text: Rgba,
    /// Chevron colour.
    icon: Rgba,
    disabled_text: Rgba,
    disabled_icon: Rgba,
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
            fill: c.background,
            separator: c.border,
            text: c.text,
            icon: c.text,
            disabled_text: c.disabled,
            disabled_icon: c.disabled,
            ring: c.focus,
            icon_stroke: IconStroke::Pixels(t.borders.regular),
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    Look {
        fill: c.background,
        // The web's translucent dark border (10% text), composited over the background.
        separator: if dark { mix(c.text, c.background, 0.1) } else { c.border },
        text: c.text,
        icon: c.text_muted,
        // shadcn's disabled `opacity: .5`, flattened over the background.
        disabled_text: mix(c.text, c.background, 0.5),
        disabled_icon: mix(c.text_muted, c.background, 0.5),
        ring: c.focus.opacity(0.5),
        icon_stroke: IconStroke::Relative,
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
/// Decorative Lucide `chevron-down` (6,9 → 12,15 → 18,9) while collapsed, or the same chevron
/// turned half a turn (6,15 → 12,9 → 18,15) while expanded, as shadcn rotates it. GPUI cannot
/// rotate elements, so both orientations are vector paths on a 24-unit grid.
fn chevron(expanded: bool, size: f32, stroke: IconStroke, color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let width = match stroke {
                IconStroke::Relative => unit * 2.0,
                IconStroke::Pixels(width) => px(width),
            };
            let (edge, middle) = if expanded { (15.0, 9.0) } else { (9.0, 15.0) };
            let mut path = PathBuilder::stroke(width);
            for (i, (x, y)) in [(6.0, edge), (12.0, middle), (18.0, edge)].into_iter().enumerate() {
                let p = bounds.origin + point(unit * x, unit * y);
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

pub const KEY_CONTEXT: &str = "Accordion";
actions!(accordion, [Toggle, FocusNext, FocusPrevious]);
pub fn default_key_bindings() -> [KeyBinding; 4] {
    [
        KeyBinding::new("space", Toggle, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Toggle, Some(KEY_CONTEXT)),
        KeyBinding::new("tab", FocusNext, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-tab", FocusPrevious, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Single,
    Multiple,
}

pub struct Item {
    pub id: String,
    pub label: String,
    pub expanded: bool,
    pub disabled: bool,
    content: Box<dyn Fn() -> AnyElement>,
}
impl Item {
    pub fn new<E>(
        id: impl Into<String>,
        label: impl Into<String>,
        content: impl Fn() -> E + 'static,
    ) -> Self
    where
        E: IntoElement + 'static,
    {
        Self {
            id: id.into(),
            label: label.into(),
            expanded: false,
            disabled: false,
            content: Box::new(move || content().into_any_element()),
        }
    }
    pub fn expanded(mut self, value: bool) -> Self {
        self.expanded = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpandedChanged(pub Vec<String>);
impl EventEmitter<ExpandedChanged> for Accordion {}

pub struct Accordion {
    items: Vec<Item>,
    mode: Mode,
    open: Vec<String>,
    controlled: bool,
    focus: Vec<FocusHandle>,
}
impl Accordion {
    pub fn new(items: Vec<Item>, mode: Mode) -> Self {
        let open = items.iter().filter(|i| i.expanded).map(|i| i.id.clone()).collect();
        Self { items, mode, open, controlled: false, focus: Vec::new() }
    }
    pub fn controlled(items: Vec<Item>, mode: Mode, expanded_ids: Vec<String>) -> Self {
        Self { items, mode, open: expanded_ids, controlled: true, focus: Vec::new() }
    }
    pub fn expanded_ids(&self) -> &[String] {
        &self.open
    }
    pub fn set_expanded(&mut self, ids: Vec<String>, cx: &mut Context<Self>) {
        self.open = self.normalize(ids);
        cx.notify();
    }
    fn normalize(&self, ids: Vec<String>) -> Vec<String> {
        let mut result = Vec::new();
        for id in ids {
            if self.items.iter().any(|i| i.id == id) && !result.contains(&id) {
                result.push(id);
            }
        }
        if self.mode == Mode::Single {
            result.truncate(1);
        }
        result
    }
    fn request(&mut self, id: &str, cx: &mut Context<Self>) {
        if self.items.iter().find(|i| i.id == id).is_none_or(|i| i.disabled) {
            return;
        }
        let mut next = self.open.clone();
        if next.iter().any(|v| v == id) {
            next.retain(|v| v != id);
        } else {
            if self.mode == Mode::Single {
                next.clear();
            }
            next.push(id.to_owned());
        }
        if next == self.open {
            return;
        }
        if !self.controlled {
            self.open = next.clone();
        }
        cx.emit(ExpandedChanged(next));
        cx.notify();
    }
    fn focus_next(&mut self, _: &FocusNext, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }
    fn focus_previous(&mut self, _: &FocusPrevious, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }
}
impl Focusable for Accordion {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.first().cloned().expect("focus initialized")
    }
}
impl Render for Accordion {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let look = look(&theme);
        while self.focus.len() < self.items.len() {
            let index = self.focus.len();
            self.focus.push(cx.focus_handle().tab_index(0).tab_stop(!self.items[index].disabled));
        }
        let count = self.items.len();
        let mut root = div()
            .id("mkit-accordion")
            .key_context(KEY_CONTEXT)
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label("Accordion")
            .w_full()
            .flex()
            .flex_col();
        for (index, item) in self.items.iter().enumerate() {
            let open = self.open.contains(&item.id);
            let id = item.id.clone();
            let label = item.label.clone();
            let disabled = item.disabled;
            let focus = self.focus[index].clone();
            let click_id = id.clone();
            let (text, icon) = if disabled {
                (look.disabled_text, look.disabled_icon)
            } else {
                (look.text, look.icon)
            };
            // Each item is separated from the next by a bottom border (shadcn `border-b
            // last:border-b-0`); the wrapper has no role, so the accessibility tree is unchanged.
            let mut entry = div()
                .flex()
                .flex_col()
                .when(index + 1 < count, |e| {
                    e.border_b(px(theme.borders.hairline)).border_color(look.separator)
                })
                .child(
                    div()
                        .id(format!("accordion-trigger-{id}"))
                        .key_context(KEY_CONTEXT)
                        .on_action(cx.listener(Self::focus_next))
                        .on_action(cx.listener(Self::focus_previous))
                        .track_focus(&focus)
                        .tab_stop(!disabled)
                        .debug_selector({
                            let s = format!("accordion-trigger-{id}");
                            move || s.clone()
                        })
                        .role(gpui_pre::accesskit::Role::Button)
                        .aria_label(label.clone())
                        .aria_expanded(open)
                        .when(disabled, |el| {
                            el.a11y_synthetic_children(|builder| {
                                builder.parent_node().set_disabled()
                            })
                        })
                        .tab_index(if disabled { -1 } else { 0 })
                        .w_full()
                        .h(px(theme.controls.large + theme.spacing.medium))
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(theme.spacing.large))
                        .rounded(px(theme.radii.medium))
                        .border(px(theme.borders.regular))
                        .border_color(look.fill.opacity(0.))
                        .bg(look.fill)
                        .text_color(text)
                        .text_size(px(theme.typography.body))
                        .font_weight(FontWeight::MEDIUM)
                        .when(!disabled, |e| {
                            e.hover(|s| s.underline()).on_click(cx.listener({
                                let id = click_id.clone();
                                move |this, _, _, cx| this.request(&id, cx)
                            }))
                        })
                        .focus_visible(move |s| {
                            s.border_color(theme.colors.focus).shadow(vec![focus_ring(look.ring)])
                        })
                        .on_action(
                            cx.listener(move |this, _: &Toggle, _, cx| this.request(&click_id, cx)),
                        )
                        .child(
                            div().flex_1().min_w(px(0.)).whitespace_nowrap().child(label.clone()),
                        )
                        .child(chevron(open, theme.spacing.large, look.icon_stroke, icon)),
                );
            if open {
                entry = entry.child(
                    div()
                        .id(format!("accordion-panel-{id}"))
                        .role(gpui_pre::accesskit::Role::Group)
                        .aria_label(label)
                        // Offset by the trigger's reserved focus border so text lines up.
                        .px(px(theme.borders.regular))
                        .pb(px(theme.spacing.large))
                        .text_size(px(theme.typography.body))
                        .text_color(theme.colors.text)
                        .child((item.content)()),
                );
            }
            root = root.child(entry);
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use std::{cell::RefCell, rc::Rc};

    fn items() -> Vec<Item> {
        vec![
            Item::new("one", "One", || div().child("First")),
            Item::new("two", "Two", || div().child("Second")),
        ]
    }

    #[gpui::test]
    fn keyboard_toggle_obeys_mode_and_controlled_contract(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (view, window) = cx.add_window_view(|_, _| Accordion::new(items(), Mode::Single));
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        window.simulate_keystrokes("space");
        assert_eq!(view.read_with(window, |a, _| a.expanded_ids().to_vec()), vec!["one"]);

        let events = Rc::new(RefCell::new(Vec::new()));
        let (controlled, window) =
            cx.add_window_view(|_, _| Accordion::controlled(items(), Mode::Multiple, Vec::new()));
        let out = events.clone();
        let _subscription = window.update(|_, cx| {
            cx.subscribe(&controlled, move |_, event: &ExpandedChanged, _| {
                out.borrow_mut().push(event.0.clone())
            })
        });
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            controlled.focus_handle(cx).focus(w, cx);
        });
        window.simulate_keystrokes("enter");
        assert!(controlled.read_with(window, |a, _| a.expanded_ids().is_empty()));
        assert_eq!(*events.borrow(), vec![vec!["one".to_string()]]);
    }

    #[gpui::test]
    fn tab_to_second_header_and_activate_uses_focused_item(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (view, window) = cx.add_window_view(|_, _| Accordion::new(items(), Mode::Multiple));
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        let (first_focus, second_focus) =
            view.read_with(window, |a, _| (a.focus[0].clone(), a.focus[1].clone()));
        assert!(window.update(|w, _| first_focus.is_focused(w)));
        window.simulate_keystrokes("tab enter");
        assert!(window.update(|w, _| second_focus.is_focused(w)));
        assert_eq!(view.read_with(window, |a, _| a.expanded_ids().to_vec()), vec!["two"]);
    }

    #[gpui::test]
    fn multiple_mode_keeps_independent_expanded_items(cx: &mut TestAppContext) {
        cx.update(|app| mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT));
        let (view, window) = cx.add_window_view(|_, _| Accordion::new(items(), Mode::Multiple));
        window.update(|w, cx| w.draw(cx).clear(cx));
        for selector in ["accordion-trigger-one", "accordion-trigger-two"] {
            let bounds = window.debug_bounds(selector).expect("accordion trigger is rendered");
            window.simulate_click(bounds.center(), gpui::Modifiers::default());
        }
        assert_eq!(view.read_with(window, |a, _| a.expanded_ids().to_vec()), vec!["one", "two"]);
    }
}
