//! Stateful toast primitive. Window-level placement and focus management remain host-owned.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render, Window,
    actions, div, prelude::*, px,
};
use mkit_core::{
    a11y::{AccessibilityExt, LiveRegionPriority},
    theme::Theme,
};
use std::time::Duration;

const DISMISS_AFTER: Duration = Duration::from_secs(5);

pub const KEY_CONTEXT: &str = "Toast";
actions!(toast, [Dismiss]);
pub fn default_key_bindings() -> [KeyBinding; 1] {
    [KeyBinding::new("escape", Dismiss, Some(KEY_CONTEXT))]
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenChanged(pub bool);
impl EventEmitter<OpenChanged> for Toast {}
pub struct Toast {
    title: String,
    content: String,
    open: bool,
    controlled: bool,
    busy: bool,
    focus: Option<FocusHandle>,
    timer_generation: u64,
    scheduled_generation: Option<u64>,
    dismiss_requested: bool,
    alert: bool,
}
impl Toast {
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            open: true,
            controlled: false,
            busy: false,
            focus: None,
            timer_generation: 0,
            scheduled_generation: None,
            dismiss_requested: false,
            alert: false,
        }
    }
    pub fn controlled(title: impl Into<String>, content: impl Into<String>, open: bool) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            open,
            controlled: true,
            busy: false,
            focus: None,
            timer_generation: 0,
            scheduled_generation: None,
            dismiss_requested: false,
            alert: false,
        }
    }
    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.open = open;
        self.dismiss_requested = false;
        self.reset_timer();
        cx.notify();
    }
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }
    pub fn set_busy(&mut self, busy: bool, cx: &mut Context<Self>) {
        if self.busy == busy {
            return;
        }
        self.busy = busy;
        self.reset_timer();
        cx.notify();
    }
    /// Announce this toast as urgent (assertive) instead of a polite status message.
    pub fn alert(mut self, alert: bool) -> Self {
        self.alert = alert;
        self
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    fn reset_timer(&mut self) {
        self.timer_generation = self.timer_generation.wrapping_add(1);
        self.scheduled_generation = None;
    }
    fn request_dismiss(&mut self, cx: &mut Context<Self>) {
        if !self.open || self.busy || self.dismiss_requested {
            return;
        }
        self.dismiss_requested = true;
        if !self.controlled {
            self.open = false;
            self.reset_timer();
        }
        cx.emit(OpenChanged(false));
        cx.notify();
    }
    fn dismiss(&mut self, _: &Dismiss, _: &mut Window, cx: &mut Context<Self>) {
        self.request_dismiss(cx);
    }
}
impl Focusable for Toast {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}
impl Render for Toast {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.open
            && !self.busy
            && !self.dismiss_requested
            && self.scheduled_generation != Some(self.timer_generation)
        {
            let generation = self.timer_generation;
            self.scheduled_generation = Some(generation);
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(DISMISS_AFTER).await;
                let _ = this.update(cx, |this, cx| {
                    if this.timer_generation == generation {
                        this.request_dismiss(cx);
                    }
                });
            })
            .detach();
        }
        let t = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        let mut root = div()
            .id(("mkit-toast", cx.entity().entity_id()))
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::dismiss))
            .when(self.open, |d| d.track_focus(&focus))
            .when(self.open, |d| {
                d.role(if self.alert {
                    gpui_pre::accesskit::Role::Alert
                } else {
                    gpui_pre::accesskit::Role::Status
                })
                .aria_label(format!("{}: {}", self.title, self.content))
                .a11y_live_region(if self.alert {
                    LiveRegionPriority::Assertive
                } else {
                    LiveRegionPriority::Polite
                })
            })
            .when(self.open, |d| {
                d.flex()
                    .flex_col()
                    .gap(px(t.spacing.medium))
                    .p(px(t.spacing.large))
                    .rounded(px(t.radii.large))
                    .border(px(t.borders.regular))
                    .border_color(t.colors.border)
                    .bg(t.colors.elevated_surface)
                    .text_color(t.colors.text)
                    .text_size(px(t.typography.body))
                    .child(
                        div().text_size(px(t.typography.heading_small)).child(self.title.clone()),
                    )
                    .child(div().child(self.content.clone()))
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
    use gpui_pre::{AppContext, Entity, ParentElement, TestAppContext};
    use std::{cell::RefCell, rc::Rc};

    struct ToastHost {
        toast: Option<Entity<Toast>>,
        events: Rc<RefCell<Vec<bool>>>,
        subscription: Option<gpui_pre::Subscription>,
        controlled: bool,
        busy: bool,
    }

    impl ToastHost {
        fn new(controlled: bool, busy: bool) -> Self {
            Self {
                toast: None,
                events: Rc::new(RefCell::new(Vec::new())),
                subscription: None,
                controlled,
                busy,
            }
        }
    }

    impl Render for ToastHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.toast.is_none() {
                let toast = if self.controlled {
                    Toast::controlled("Saved", "Your changes are saved", true)
                } else {
                    Toast::new("Saved", "Your changes are saved")
                }
                .busy(self.busy);
                let toast = cx.new(|_| toast);
                let events = self.events.clone();
                self.subscription =
                    Some(cx.subscribe(&toast, move |_, _, event: &OpenChanged, _| {
                        events.borrow_mut().push(event.0);
                    }));
                self.toast = Some(toast);
            }
            div().child(self.toast.as_ref().expect("toast initialized").clone())
        }
    }

    #[gpui_pre::test]
    fn timeout_restarts_after_reopen_and_ignores_stale_timer(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) = cx.add_window_view(|_, _| ToastHost::new(false, false));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.run_until_parked();
        let toast = host.read_with(visual, |host, _| host.toast.as_ref().unwrap().clone());

        visual.executor().advance_clock(Duration::from_secs(2));
        visual.run_until_parked();
        toast.update(visual, |toast, cx| toast.set_open(false, cx));
        toast.update(visual, |toast, cx| toast.set_open(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.run_until_parked();

        visual.executor().advance_clock(Duration::from_secs(3));
        visual.run_until_parked();
        assert!(toast.read_with(visual, |toast, _| toast.is_open()));
        host.read_with(visual, |host, _| assert!(host.events.borrow().is_empty()));

        visual.executor().advance_clock(Duration::from_secs(2));
        visual.run_until_parked();
        assert!(!toast.read_with(visual, |toast, _| toast.is_open()));
        host.read_with(visual, |host, _| assert_eq!(*host.events.borrow(), vec![false]));
    }

    #[gpui_pre::test]
    fn controlled_and_busy_timeout_requests_once_until_rearmed(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) = cx.add_window_view(|_, _| ToastHost::new(true, true));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.run_until_parked();
        let toast = host.read_with(visual, |host, _| host.toast.as_ref().unwrap().clone());
        visual.executor().advance_clock(DISMISS_AFTER);
        visual.run_until_parked();
        host.read_with(visual, |host, _| assert!(host.events.borrow().is_empty()));

        toast.update(visual, |toast, cx| toast.set_busy(false, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.run_until_parked();
        visual.executor().advance_clock(DISMISS_AFTER);
        visual.run_until_parked();
        assert!(toast.read_with(visual, |toast, _| toast.is_open()));
        host.read_with(visual, |host, _| assert_eq!(*host.events.borrow(), vec![false]));

        visual.executor().advance_clock(DISMISS_AFTER);
        visual.run_until_parked();
        host.read_with(visual, |host, _| assert_eq!(*host.events.borrow(), vec![false]));

        toast.update(visual, |toast, cx| toast.set_open(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.run_until_parked();
        visual.executor().advance_clock(DISMISS_AFTER);
        visual.run_until_parked();
        host.read_with(visual, |host, _| assert_eq!(*host.events.borrow(), vec![false, false]));
    }

    #[test]
    fn controlled_mode_remains_authoritative() {
        let c = Toast::controlled("title", "body", false);
        assert!(c.controlled);
        assert!(!c.open);
    }
    #[test]
    fn escape_is_rebindable() {
        assert_eq!(default_key_bindings().len(), 1);
    }
}
