//! Single collapsible region following the WAI-ARIA Disclosure pattern.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnimationExt, AnyElement, App, Context, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyBinding, Render, RenderOnce, StyleRefinement, Window, actions, div, prelude::*, px,
};
use mkit_core::{
    motion::{TransitionKind, transition_animation, transition_duration},
    theme::Theme,
};
use std::rc::Rc;

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
        let Self { id, label, expanded, disabled, focus, on_toggle, style } = self;
        let selector = id.clone();
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
            .h(px(theme.controls.small))
            .px(px(theme.spacing.medium))
            .flex()
            .items_center()
            .justify_between()
            .text_color(if disabled { theme.colors.disabled } else { theme.colors.text })
            .text_size(px(theme.typography.body_emphasis))
            .bg(theme.colors.surface)
            .when_some(on_toggle.filter(|_| !disabled), |e, handler| {
                let on_action = handler.clone();
                e.on_action(move |_: &Toggle, window, cx| on_action(window, cx))
                    .on_click(move |_, window, cx| handler(window, cx))
            })
            .child(label)
            .child(if expanded { "⌄" } else { "›" });
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
            .pl(px(theme.spacing.medium))
            .py(px(theme.spacing.small))
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
        let mut root = div().id(format!("disclosure-{id}")).flex().flex_col().w_full().child(
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
