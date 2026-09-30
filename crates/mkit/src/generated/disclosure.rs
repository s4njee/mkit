//! Single collapsible region following the WAI-ARIA Disclosure pattern.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnimationExt, AnyElement, App, Context, EventEmitter, FocusHandle, Focusable, FontWeight,
    IntoElement, KeyBinding, PathBuilder, Render, RenderOnce, Rgba, StyleRefinement, Window,
    actions, canvas, div, point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    motion::{TransitionKind, transition_animation, transition_duration},
    theme::Theme,
};
use std::rc::Rc;

/// Resolved disclosure colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    /// Card fill (shadcn `card`), also the trigger fill so the focus ring sits on an opaque fill.
    card: Rgba,
    card_border: Rgba,
    text: Rgba,
    /// Trigger label colour while hovered; `None` underlines instead (high contrast).
    hover_text: Option<Rgba>,
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
            card: c.background,
            card_border: c.border,
            text: c.text,
            hover_text: None,
            icon: c.text,
            disabled_text: c.disabled,
            disabled_icon: c.disabled,
            ring: c.focus,
            icon_stroke: IconStroke::Pixels(t.borders.regular),
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn "card" is `surface`; the dark border is the web's translucent 10% text, composited.
    let card = c.surface;
    Look {
        card,
        card_border: if dark { mix(c.text, card, 0.1) } else { c.border },
        text: c.text,
        hover_text: Some(c.text_muted),
        icon: c.text_muted,
        // shadcn's disabled `opacity: .5`, flattened over the opaque card fill.
        disabled_text: mix(c.text, card, 0.5),
        disabled_icon: mix(c.text_muted, card, 0.5),
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

pub const KEY_CONTEXT: &str = "Disclosure";
actions!(disclosure, [Toggle]);

pub fn default_key_bindings() -> [KeyBinding; 2] {
    [
        KeyBinding::new("space", Toggle, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Toggle, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpandedChanged(pub bool);
impl EventEmitter<ExpandedChanged> for Disclosure {}

type ToggleHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// Stateless disclosure trigger: a button with expanded state, the `Disclosure` key context, and
/// the rebindable [`Toggle`] action. The owner supplies state and decides what a toggle means.
#[derive(IntoElement)]
pub struct DisclosureTrigger {
    id: String,
    label: String,
    expanded: bool,
    disabled: bool,
    focus: Option<FocusHandle>,
    on_toggle: Option<ToggleHandler>,
    style: StyleRefinement,
}

impl DisclosureTrigger {
    /// `id` is used as the element ID and debug selector.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            expanded: false,
            disabled: false,
            focus: None,
            on_toggle: None,
            style: StyleRefinement::default(),
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
    /// Use an owner-held focus handle instead of an element-owned one.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }
    /// Called for pointer clicks and the `Toggle` action while enabled.
    pub fn on_toggle(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

impl Styled for DisclosureTrigger {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for DisclosureTrigger {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let look = look(&theme);
        let Self { id, label, expanded, disabled, focus, on_toggle, style } = self;
        let selector = id.clone();
        let (text, icon) = if disabled {
            (look.disabled_text, look.disabled_icon)
        } else {
            (look.text, look.icon)
        };
        let mut trigger = div()
            .id(id)
            .key_context(KEY_CONTEXT)
            .debug_selector(move || selector.clone())
            .role(gpui_pre::accesskit::Role::Button)
            .aria_label(label.clone())
            .aria_expanded(expanded)
            .when(disabled, |e| {
                e.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .when_some(focus, |e, focus| e.track_focus(&focus))
            .tab_index(if disabled { -1 } else { 0 })
            .w_full()
            .h(px(theme.controls.large + theme.spacing.xsmall))
            // The reserved focus border is part of the inset, so the label lines up with the panel.
            .px(px(theme.spacing.large - theme.borders.regular))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(theme.spacing.large))
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.regular))
            .border_color(look.card.opacity(0.))
            .text_color(text)
            .text_size(px(theme.typography.body))
            .font_weight(FontWeight::MEDIUM)
            .bg(look.card)
            .when(!disabled, |e| {
                e.hover(move |s| match look.hover_text {
                    Some(color) => s.text_color(color),
                    None => s.underline(),
                })
            })
            .focus_visible(move |s| {
                s.border_color(theme.colors.focus).shadow(vec![focus_ring(look.ring)])
            })
            .when_some(on_toggle.filter(|_| !disabled), |e, handler| {
                let on_action = handler.clone();
                e.on_action(move |_: &Toggle, window, cx| on_action(window, cx))
                    .on_click(move |_, window, cx| handler(window, cx))
            })
            .child(div().flex_1().min_w(px(0.)).whitespace_nowrap().child(label))
            .child(chevron(expanded, theme.spacing.large, look.icon_stroke, icon));
        trigger.style().refine(&style);
        trigger
    }
}

/// Stateless disclosure panel: a labelled group with a stable ID and optional reveal motion.
/// Render it only while expanded; GPUI drops the reveal state while the panel is omitted, so each
/// expansion starts a new reveal.
#[derive(IntoElement)]
pub struct DisclosurePanel {
    id: String,
    label: String,
    motion: bool,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl DisclosurePanel {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            motion: false,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
    /// Fade the panel in over the theme's open duration unless motion is reduced.
    pub fn motion(mut self, value: bool) -> Self {
        self.motion = value;
        self
    }
}

impl ParentElement for DisclosurePanel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for DisclosurePanel {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// Whether an opted-in reveal should animate. Reduced motion or a zero theme token disables it.
fn reveal_animates(theme: &Theme, motion: bool, reduce_motion: bool) -> bool {
    motion && !transition_duration(theme, TransitionKind::Open, reduce_motion).is_zero()
}

impl RenderOnce for DisclosurePanel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let Self { id, label, motion, children, style } = self;
        let animation_id = format!("{id}-reveal");
        let mut panel = div()
            .id(id)
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(label)
            .px(px(theme.spacing.large))
            .pb(px(theme.spacing.large))
            .text_size(px(theme.typography.body))
            .text_color(theme.colors.text)
            .children(children);
        panel.style().refine(&style);
        if reveal_animates(&theme, motion, cx.reduce_motion()) {
            panel
                .with_animation(
                    animation_id,
                    transition_animation(&theme, TransitionKind::Open),
                    |panel, delta| panel.opacity(delta),
                )
                .into_any_element()
        } else {
            panel.into_any_element()
        }
    }
}

pub struct Disclosure {
    id: String,
    label: String,
    expanded: bool,
    controlled: bool,
    disabled: bool,
    motion: bool,
    content: Box<dyn Fn() -> AnyElement>,
    focus: Option<FocusHandle>,
}

impl Disclosure {
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
            controlled: false,
            disabled: false,
            motion: false,
            content: Box::new(move || content().into_any_element()),
            focus: None,
        }
    }

    pub fn controlled<E>(
        id: impl Into<String>,
        label: impl Into<String>,
        expanded: bool,
        content: impl Fn() -> E + 'static,
    ) -> Self
    where
        E: IntoElement + 'static,
    {
        let mut this = Self::new(id, label, content);
        this.expanded = expanded;
        this.controlled = true;
        this
    }

    pub fn expanded(mut self, value: bool) -> Self {
        self.expanded = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    /// Opt in to the panel reveal transition. Off by default; reduced motion disables it.
    pub fn motion(mut self, value: bool) -> Self {
        self.motion = value;
        self
    }
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }
    pub fn set_expanded(&mut self, value: bool, cx: &mut Context<Self>) {
        self.expanded = value;
        cx.notify();
    }

    fn request_toggle(&mut self, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let next = !self.expanded;
        if !self.controlled {
            self.expanded = next;
        }
        cx.emit(ExpandedChanged(next));
        cx.notify();
    }
}

impl Focusable for Disclosure {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("focus initialized during render")
    }
}

impl Render for Disclosure {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self
            .focus
            .get_or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(!self.disabled))
            .clone();
        let id = self.id.clone();
        let label = self.label.clone();
        let entity = cx.entity().downgrade();
        let theme = *cx.global::<Theme>();
        let look = look(&theme);
        let mut root = div()
            .id(format!("disclosure-{id}"))
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(theme.radii.large))
            .border(px(theme.borders.regular))
            .border_color(look.card_border)
            .bg(look.card)
            .child(
                DisclosureTrigger::new(format!("disclosure-trigger-{id}"), label.clone())
                    .expanded(self.expanded)
                    .disabled(self.disabled)
                    .track_focus(&focus)
                    .on_toggle(move |_, cx| {
                        entity.update(cx, |this, cx| this.request_toggle(cx)).ok();
                    }),
            );
        if self.expanded {
            root = root.child(
                DisclosurePanel::new(format!("disclosure-panel-{id}"), label)
                    .motion(self.motion)
                    .child((self.content)()),
            );
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use std::{cell::RefCell, rc::Rc};

    #[gpui::test]
    fn keyboard_toggles_and_controlled_mode_waits_for_owner(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (view, window) =
            cx.add_window_view(|_, _| Disclosure::new("notes", "Notes", || div().child("Details")));
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        window.simulate_keystrokes("space");
        assert!(view.read_with(window, |d, _| d.is_expanded()));

        let events = Rc::new(RefCell::new(Vec::new()));
        let (controlled, window) = cx.add_window_view(|_, _| {
            Disclosure::controlled("controlled", "Controlled", false, || div())
        });
        let out = events.clone();
        let _subscription = window.update(|_, cx| {
            cx.subscribe(&controlled, move |_, event: &ExpandedChanged, _| {
                out.borrow_mut().push(event.0)
            })
        });
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            controlled.focus_handle(cx).focus(w, cx);
        });
        window.simulate_keystrokes("enter");
        assert!(!controlled.read_with(window, |d, _| d.is_expanded()));
        assert_eq!(*events.borrow(), vec![true]);
    }

    #[gpui::test]
    fn panel_content_keys_do_not_toggle_the_disclosure(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (view, window) = cx.add_window_view(|_, _| {
            Disclosure::new("nested", "Nested", || {
                div().id("inner-control").debug_selector(|| "inner-control".into()).tab_index(0)
            })
            .expanded(true)
        });
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        window.simulate_keystrokes("tab");
        window.simulate_keystrokes("space");
        window.simulate_keystrokes("enter");
        assert!(view.read_with(window, |d, _| d.is_expanded()));
    }

    struct OwnedTrigger {
        expanded: bool,
        focus: FocusHandle,
    }

    impl Render for OwnedTrigger {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let entity = cx.entity().downgrade();
            div().child(
                DisclosureTrigger::new("owned", "Owned")
                    .expanded(self.expanded)
                    .track_focus(&self.focus)
                    .on_toggle(move |_, cx| {
                        entity
                            .update(cx, |this, cx| {
                                this.expanded = !this.expanded;
                                cx.notify();
                            })
                            .ok();
                    }),
            )
        }
    }

    #[gpui::test]
    fn trigger_part_routes_rebindable_toggle_and_click_to_owner(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys([KeyBinding::new("t", Toggle, Some(KEY_CONTEXT))]);
        });
        let (view, window) =
            cx.add_window_view(|_, cx| OwnedTrigger { expanded: false, focus: cx.focus_handle() });
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            let focus = view.read(cx).focus.clone();
            focus.focus(w, cx);
        });
        window.simulate_keystrokes("t");
        assert!(view.read_with(window, |v, _| v.expanded));
        let bounds = window.debug_bounds("owned").expect("trigger rendered");
        window.simulate_click(bounds.center(), gpui_pre::Modifiers::none());
        assert!(!view.read_with(window, |v, _| v.expanded));
    }

    #[test]
    fn reveal_motion_is_opt_in_and_honours_reduced_motion_and_zero_tokens() {
        use mkit_core::theme::{HIGH_CONTRAST, SHADCN_LIGHT};
        assert!(!reveal_animates(&SHADCN_LIGHT, false, false));
        assert!(reveal_animates(&SHADCN_LIGHT, true, false));
        assert!(!reveal_animates(&SHADCN_LIGHT, true, true));
        assert!(!reveal_animates(&HIGH_CONTRAST, true, false));
    }

    #[gpui::test]
    fn motion_panel_renders_immediately_with_and_without_reduced_motion(cx: &mut TestAppContext) {
        for reduce in [false, true] {
            cx.update(|app| {
                mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
                app.set_reduce_motion(reduce);
                app.bind_keys(default_key_bindings());
            });
            let (view, window) = cx.add_window_view(|_, _| {
                Disclosure::new("animated", "Animated", || {
                    div()
                        .id("animated-body")
                        .debug_selector(|| "animated-body".into())
                        .child("Body")
                })
                .motion(true)
            });
            window.update(|w, cx| {
                w.draw(cx).clear(cx);
                view.focus_handle(cx).focus(w, cx);
            });
            window.simulate_keystrokes("enter");
            window.run_until_parked();
            assert!(view.read_with(window, |d, _| d.is_expanded()));
            assert!(window.debug_bounds("animated-body").is_some(), "panel present at once");
        }
    }

    #[gpui::test]
    fn disabled_disclosure_ignores_activation(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (view, window) =
            cx.add_window_view(|_, _| Disclosure::new("locked", "Locked", || div()).disabled(true));
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        window.simulate_keystrokes("space");
        assert!(!view.read_with(window, |d, _| d.is_expanded()));
    }
}
