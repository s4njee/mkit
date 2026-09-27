//! Stateful modal sheet surface with focus trapping and a blocking backdrop.
extern crate gpui_pre as gpui;
use gpui_pre::{
    AnyElement, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding,
    MouseDownEvent, Render, Window, actions, div, prelude::*, px,
};
use mkit_core::focus::{FocusDirection, FocusScope};
use mkit_core::theme::Theme;

pub const KEY_CONTEXT: &str = "Sheet";
actions!(sheet, [Dismiss, FocusForward, FocusBackward]);
pub fn default_key_bindings() -> [KeyBinding; 3] {
    [
        KeyBinding::new("escape", Dismiss, Some(KEY_CONTEXT)),
        KeyBinding::new("tab", FocusForward, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-tab", FocusBackward, Some(KEY_CONTEXT)),
    ]
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenChanged(pub bool);
impl EventEmitter<OpenChanged> for Sheet {}
pub struct Sheet {
    title: String,
    content: String,
    content_builder: Option<Box<dyn Fn() -> AnyElement>>,
    focus_stops: Vec<FocusHandle>,
    open: bool,
    controlled: bool,
    busy: bool,
    focus: Option<FocusHandle>,
    return_focus_to: Option<gpui_pre::WeakFocusHandle>,
    last_open: bool,
}
impl Sheet {
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            content_builder: None,
            focus_stops: Vec::new(),
            open: true,
            controlled: false,
            busy: false,
            focus: None,
            return_focus_to: None,
            last_open: false,
        }
    }
    pub fn controlled(title: impl Into<String>, content: impl Into<String>, open: bool) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            content_builder: None,
            focus_stops: Vec::new(),
            open,
            controlled: true,
            busy: false,
            focus: None,
            return_focus_to: None,
            last_open: false,
        }
    }
    /// Create a sheet whose content is rebuilt for each render.
    pub fn with_content<E>(title: impl Into<String>, content: impl Fn() -> E + 'static) -> Self
    where
        E: IntoElement + 'static,
    {
        let mut sheet = Self::new(title, "");
        sheet.content_builder = Some(Box::new(move || content().into_any_element()));
        sheet
    }
    /// Supply enabled content focus handles in visual order. The content factory must track
    /// each handle on its corresponding control.
    pub fn focus_stops(mut self, handles: Vec<FocusHandle>) -> Self {
        self.focus_stops = handles;
        self
    }
    /// Set the focus handle to restore when this surface closes.
    pub fn return_focus_to(mut self, handle: &FocusHandle) -> Self {
        self.return_focus_to = Some(handle.downgrade());
        self
    }
    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.open = open;
        cx.notify();
    }
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    fn dismiss(&mut self, _: &Dismiss, window: &mut Window, cx: &mut Context<Self>) {
        self.request_dismiss(window, cx);
    }
    fn outside_down(&mut self, _: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.request_dismiss(window, cx);
    }
    fn request_dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open || self.busy {
            return;
        }
        if !self.controlled {
            self.open = false;
            self.restore_focus(window, cx);
        }
        cx.emit(OpenChanged(false));
        cx.notify();
    }
    fn restore_focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(handle) =
            self.return_focus_to.as_ref().and_then(gpui_pre::WeakFocusHandle::upgrade)
        {
            handle.focus(window, cx);
        }
    }
    fn tab_forward(&mut self, _: &FocusForward, window: &mut Window, cx: &mut Context<Self>) {
        if self.open
            && let Some(handle) = &self.focus
        {
            FocusScope::new(self.scope_stops(handle.clone())).move_focus(
                FocusDirection::Forward,
                window,
                cx,
            );
        }
    }
    fn tab_backward(&mut self, _: &FocusBackward, window: &mut Window, cx: &mut Context<Self>) {
        if self.open
            && let Some(handle) = &self.focus
        {
            FocusScope::new(self.scope_stops(handle.clone())).move_focus(
                FocusDirection::Backward,
                window,
                cx,
            );
        }
    }
    fn scope_stops(&self, fallback: FocusHandle) -> Vec<FocusHandle> {
        if self.focus_stops.is_empty() { vec![fallback] } else { self.focus_stops.clone() }
    }
}
impl Focusable for Sheet {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}
impl Render for Sheet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_stop(true)).clone();
        if self.open && !self.last_open {
            if let Some(previous) = window.focused(cx)
                && previous != focus
            {
                self.return_focus_to = Some(previous.downgrade());
            }
            FocusScope::new(self.scope_stops(focus.clone())).focus_first(window, cx);
        } else if !self.open && self.last_open {
            self.restore_focus(window, cx);
        }
        self.last_open = self.open;
        let mut root = div()
            .id(("mkit-sheet-overlay", cx.entity().entity_id()))
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::dismiss))
            .on_action(cx.listener(Self::tab_forward))
            .on_action(cx.listener(Self::tab_backward))
            .when(self.open, |element| {
                element.absolute().inset_0().flex().items_end().justify_center().occlude()
            })
            .when(self.open, |element| {
                element.child(div().absolute().inset_0().bg(theme.colors.text.opacity(0.42))).child(
                    div()
                        .id(("mkit-sheet", cx.entity().entity_id()))
                        .debug_selector(|| "mkit-sheet-surface".to_owned())
                        .key_context(KEY_CONTEXT)
                        .track_focus(&focus)
                        .when(self.focus_stops.is_empty(), |element| element.tab_stop(true))
                        .role(gpui_pre::accesskit::Role::Dialog)
                        .aria_label(self.title.clone())
                        .on_mouse_down_out(cx.listener(Self::outside_down))
                        .flex()
                        .flex_col()
                        .gap(px(theme.spacing.medium))
                        .p(px(theme.spacing.large))
                        .rounded(px(theme.radii.large))
                        .border(px(theme.borders.regular))
                        .border_color(theme.colors.border)
                        .bg(theme.colors.elevated_surface)
                        .text_color(theme.colors.text)
                        .text_size(px(theme.typography.body))
                        .w_full()
                        .child(
                            div()
                                .text_size(px(theme.typography.heading_small))
                                .child(self.title.clone()),
                        )
                        .child(if let Some(build) = &self.content_builder {
                            build()
                        } else {
                            div().child(self.content.clone()).into_any_element()
                        })
                        .when(self.busy, |surface| {
                            surface.child(
                                div()
                                    .id("mkit-sheet-busy-status")
                                    .debug_selector(|| "mkit-sheet-busy-status".to_owned())
                                    .role(gpui_pre::accesskit::Role::Status)
                                    .aria_label("Work in progress")
                                    .flex()
                                    .items_center()
                                    .gap(px(theme.spacing.small))
                                    .p(px(theme.spacing.small))
                                    .rounded(px(theme.radii.medium))
                                    .border(px(theme.borders.regular))
                                    .border_color(theme.colors.border)
                                    .bg(theme.colors.surface)
                                    .text_color(theme.colors.text_muted)
                                    .child(
                                        div()
                                            .size(px(theme.spacing.xsmall))
                                            .rounded(px(theme.radii.pill))
                                            .bg(theme.colors.accent),
                                    )
                                    .child("Dismissal is paused until this work finishes."),
                            )
                        }),
                )
            });
        if !self.open {
            root = root.size(px(0.0));
        }
        root
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Context, IntoElement, ParentElement, Render, TestAppContext, div};

    struct Host {
        surface: Option<gpui_pre::Entity<Sheet>>,
        opener: FocusHandle,
        outside_hits: usize,
        busy: bool,
    }
    struct InteractiveHost {
        surface: Option<gpui_pre::Entity<Sheet>>,
        first: FocusHandle,
        second: FocusHandle,
    }
    impl Render for InteractiveHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.surface.is_none() {
                let first = self.first.clone();
                let second = self.second.clone();
                self.surface = Some(cx.new(move |_| {
                    let content_first = first.clone();
                    let content_second = second.clone();
                    Sheet::with_content("title", move || {
                        div()
                            .child(div().track_focus(&content_first).tab_stop(true).child("first"))
                            .child(
                                div().track_focus(&content_second).tab_stop(true).child("second"),
                            )
                    })
                    .focus_stops(vec![first, second])
                }));
            }
            div().child(self.surface.as_ref().unwrap().clone())
        }
    }
    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.surface.is_none() {
                let opener = self.opener.clone();
                let busy = self.busy;
                self.surface =
                    Some(cx.new(move |_| {
                        Sheet::new("title", "body").return_focus_to(&opener).busy(busy)
                    }));
            }
            div()
                .child(
                    div()
                        .id("underlying-control")
                        .debug_selector(|| "underlying-control".to_owned())
                        .w(px(100.))
                        .h(px(100.))
                        .track_focus(&self.opener)
                        .tab_stop(true)
                        .on_mouse_down(
                            gpui_pre::MouseButton::Left,
                            cx.listener(|host, _, _, cx| {
                                host.outside_hits += 1;
                                cx.notify();
                            }),
                        ),
                )
                .child(self.surface.as_ref().unwrap().clone())
        }
    }

    #[test]
    fn controlled_mode_remains_authoritative() {
        let instance = Sheet::controlled("title", "body", false);
        assert!(instance.controlled);
        assert!(!instance.open);
    }

    #[gpui_pre::test]
    fn opening_focuses_surface_and_tab_stays_inside(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, cx| Host {
            surface: None,
            opener: cx.focus_handle().tab_stop(true),
            outside_hits: 0,
            busy: false,
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (surface, opener) = host.read_with(visual, |host, _| {
            (host.surface.as_ref().unwrap().clone(), host.opener.clone())
        });
        surface.update(visual, |surface, cx| surface.set_open(false, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| opener.focus(window, cx));
        surface.update(visual, |surface, cx| surface.set_open(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let focus = surface.read_with(visual, |surface, cx| surface.focus_handle(cx));
        assert!(visual.update(|window, _| focus.is_focused(window)));
        visual.simulate_keystrokes("tab shift-tab");
        assert!(visual.update(|window, _| focus.is_focused(window)));
    }

    #[gpui_pre::test]
    fn content_focus_stops_cycle_in_visual_order(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, cx| InteractiveHost {
            surface: None,
            first: cx.focus_handle().tab_stop(true),
            second: cx.focus_handle().tab_stop(true),
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (first, second) =
            host.read_with(visual, |host, _| (host.first.clone(), host.second.clone()));
        assert!(visual.update(|window, _| first.is_focused(window)));
        visual.simulate_keystrokes("tab");
        assert!(visual.update(|window, _| second.is_focused(window)));
        visual.simulate_keystrokes("tab");
        assert!(visual.update(|window, _| first.is_focused(window)));
        visual.simulate_keystrokes("shift-tab");
        assert!(visual.update(|window, _| second.is_focused(window)));
    }

    #[gpui_pre::test]
    fn escape_and_outside_pointer_dismiss_and_restore_focus(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, cx| Host {
            surface: None,
            opener: cx.focus_handle().tab_stop(true),
            outside_hits: 0,
            busy: false,
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (surface, opener) = host.read_with(visual, |host, _| {
            (host.surface.as_ref().unwrap().clone(), host.opener.clone())
        });
        surface.update(visual, |surface, cx| surface.set_open(false, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| opener.focus(window, cx));
        surface.update(visual, |surface, cx| surface.set_open(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("escape");
        assert!(!surface.read_with(visual, |surface, _| surface.is_open()));
        assert!(visual.update(|window, _| opener.is_focused(window)));

        surface.update(visual, |surface, cx| surface.set_open(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_mouse_down(
            gpui_pre::point(px(900.), px(700.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );
        assert!(!surface.read_with(visual, |surface, _| surface.is_open()));
    }

    #[gpui_pre::test]
    fn modal_blocks_pointer_input_to_host_and_anchors_panel_at_viewport_bottom(
        cx: &mut TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) = cx.add_window_view(|_, cx| Host {
            surface: None,
            opener: cx.focus_handle().tab_stop(true),
            outside_hits: 0,
            busy: false,
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let surface = host.read_with(visual, |host, _| host.surface.as_ref().unwrap().clone());
        let control = visual.debug_bounds("underlying-control").expect("host control bounds");
        let panel = visual.debug_bounds("mkit-sheet-surface").expect("sheet bounds");
        let viewport = visual.update(|window, _| window.bounds());
        assert!((panel.bottom() - viewport.bottom()).abs() < px(1.));

        visual.simulate_mouse_down(
            control.center(),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );
        assert_eq!(host.read_with(visual, |host, _| host.outside_hits), 0);
        assert!(!surface.read_with(visual, |surface, _| surface.is_open()));
    }

    #[gpui_pre::test]
    fn busy_status_is_visible_and_dismissal_stays_blocked(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, cx| Host {
            surface: None,
            opener: cx.focus_handle().tab_stop(true),
            outside_hits: 0,
            busy: true,
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let surface = host.read_with(visual, |host, _| host.surface.as_ref().unwrap().clone());
        let status =
            visual.debug_bounds("mkit-sheet-busy-status").expect("busy status is rendered");
        let panel = visual.debug_bounds("mkit-sheet-surface").expect("sheet is rendered");
        assert!(status.size.width > px(0.));
        assert!(status.size.height > px(0.));
        assert!(panel.contains(&status.center()));

        visual.simulate_keystrokes("escape");
        assert!(surface.read_with(visual, |surface, _| surface.is_open()));
        visual.simulate_mouse_down(
            gpui_pre::point(px(900.), px(700.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );
        assert!(surface.read_with(visual, |surface, _| surface.is_open()));
    }
}
