//! Persistent contextual messages with severity-appropriate live-region semantics.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnyElement, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render,
    Window, actions, div, prelude::*, px,
};
use mkit_core::{
    a11y::{AccessibilityExt, LiveRegionPriority},
    theme::Theme,
};
use std::sync::atomic::{AtomicUsize, Ordering};

pub const KEY_CONTEXT: &str = "InlineAlert";
actions!(inline_alert, [ActivateAction, DismissAlert]);
static NEXT_ALERT_ID: AtomicUsize = AtomicUsize::new(1);

pub fn default_key_bindings() -> [KeyBinding; 4] {
    [
        KeyBinding::new("enter", ActivateAction, Some("InlineAlertAction")),
        KeyBinding::new("space", ActivateAction, Some("InlineAlertAction")),
        KeyBinding::new("enter", DismissAlert, Some("InlineAlertDismiss")),
        KeyBinding::new("space", DismissAlert, Some("InlineAlertDismiss")),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionInvoked;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DismissRequested;
impl EventEmitter<ActionInvoked> for InlineAlert {}
impl EventEmitter<DismissRequested> for InlineAlert {}

pub struct InlineAlert {
    id: String,
    severity: Severity,
    title: Option<String>,
    message: String,
    icon: Option<Box<dyn Fn() -> AnyElement>>,
    action_label: Option<String>,
    dismissible: bool,
    disabled: bool,
    dismissed: bool,
    controlled: bool,
    action_focus: Option<FocusHandle>,
    dismiss_focus: Option<FocusHandle>,
    root_focus: Option<FocusHandle>,
}

impl InlineAlert {
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            id: format!("inline-alert-{}", NEXT_ALERT_ID.fetch_add(1, Ordering::Relaxed)),
            severity,
            title: None,
            message: message.into(),
            icon: None,
            action_label: None,
            dismissible: false,
            disabled: false,
            dismissed: false,
            controlled: false,
            action_focus: None,
            dismiss_focus: None,
            root_focus: None,
        }
    }
    pub fn controlled(severity: Severity, message: impl Into<String>, dismissed: bool) -> Self {
        Self { dismissed, controlled: true, ..Self::new(severity, message) }
    }
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    pub fn icon<E>(mut self, icon: impl Fn() -> E + 'static) -> Self
    where
        E: IntoElement + 'static,
    {
        self.icon = Some(Box::new(move || icon().into_any_element()));
        self
    }
    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.action_label = Some(label.into());
        self
    }
    pub fn dismissible(mut self, value: bool) -> Self {
        self.dismissible = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn is_dismissed(&self) -> bool {
        self.dismissed
    }
    pub fn set_dismissed(&mut self, dismissed: bool, cx: &mut Context<Self>) {
        self.dismissed = dismissed;
        cx.notify();
    }

    fn invoke_action(&mut self, cx: &mut Context<Self>) {
        if !self.disabled && !self.dismissed && self.action_label.is_some() {
            cx.emit(ActionInvoked);
        }
    }
    fn dismiss(&mut self, cx: &mut Context<Self>) {
        if self.disabled || self.dismissed || !self.dismissible {
            return;
        }
        if !self.controlled {
            self.dismissed = true;
        }
        cx.emit(DismissRequested);
        cx.notify();
    }
    fn activate(&mut self, _: &ActivateAction, _: &mut Window, cx: &mut Context<Self>) {
        self.invoke_action(cx);
    }
    fn dismiss_action(&mut self, _: &DismissAlert, _: &mut Window, cx: &mut Context<Self>) {
        self.dismiss(cx);
    }
}

impl Focusable for InlineAlert {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.action_focus
            .clone()
            .or_else(|| self.dismiss_focus.clone())
            .or_else(|| self.root_focus.clone())
            .expect("focus handle initializes during render")
    }
}

impl Render for InlineAlert {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        self.root_focus.get_or_insert_with(|| cx.focus_handle());
        if self.dismissed {
            return div().id(format!("{}-dismissed", self.id)).size_0();
        }
        let action_focus = self.action_label.as_ref().map(|_| {
            self.action_focus
                .get_or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(!self.disabled))
                .clone()
        });
        let dismiss_focus = self.dismissible.then(|| {
            self.dismiss_focus
                .get_or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(!self.disabled))
                .clone()
        });
        let (role, priority, severity_color) = match self.severity {
            Severity::Info => {
                (gpui_pre::accesskit::Role::Status, LiveRegionPriority::Polite, theme.colors.accent)
            }
            Severity::Success => (
                gpui_pre::accesskit::Role::Status,
                LiveRegionPriority::Polite,
                theme.colors.success,
            ),
            Severity::Warning => (
                gpui_pre::accesskit::Role::Alert,
                LiveRegionPriority::Assertive,
                theme.colors.warning,
            ),
            Severity::Error => (
                gpui_pre::accesskit::Role::Alert,
                LiveRegionPriority::Assertive,
                theme.colors.danger,
            ),
        };
        let mut root = div()
            .id(self.id.clone())
            .key_context(KEY_CONTEXT)
            .role(role)
            .aria_label(self.title.as_deref().unwrap_or(&self.message))
            .a11y_live_region(priority)
            .w_full()
            .flex()
            .items_start()
            .gap(px(theme.spacing.medium))
            .p(px(theme.spacing.medium))
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .text_color(theme.colors.text)
            .text_size(px(theme.typography.body));
        if let Some(icon) = self.icon.as_ref() {
            root = root.child(div().text_color(severity_color).child(icon()));
        }
        let mut body = div().flex().flex_col().items_start().gap(px(theme.spacing.small)).flex_1();
        if let Some(title) = self.title.as_ref() {
            body = body.child(
                div()
                    .w_full()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.body_emphasis))
                    .child(title.clone()),
            );
        }
        body = body.child(div().w_full().child(self.message.clone()));
        if let Some(label) = self.action_label.clone() {
            let focus = action_focus.as_ref().expect("action focus initialized").clone();
            body = body.child(
                div()
                    .id(format!("{}-action", self.id))
                    .debug_selector(|| "inline-alert-action".to_string())
                    .key_context("InlineAlertAction")
                    .track_focus(&focus)
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label(label.clone())
                    .tab_stop(!self.disabled)
                    .tab_index(if self.disabled { -1 } else { 0 })
                    .h(px(theme.controls.xsmall))
                    .w(px(label.chars().count() as f32 * theme.typography.body * 0.65
                        + theme.spacing.medium * 2.0))
                    .px(px(theme.spacing.medium))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(theme.radii.small))
                    .bg(theme.colors.accent)
                    .text_color(theme.colors.accent_text)
                    .when(self.disabled, |e| {
                        e.a11y_synthetic_children(|b| b.parent_node().set_disabled())
                    })
                    .when(!self.disabled, |e| {
                        e.on_click(cx.listener(|this, _, _, cx| this.invoke_action(cx)))
                    })
                    .on_action(cx.listener(Self::activate))
                    .self_start()
                    .child(label),
            );
        }
        root = root.child(body);
        if self.dismissible {
            let focus = dismiss_focus.as_ref().expect("dismiss focus initialized").clone();
            root = root.child(
                div()
                    .id(format!("{}-dismiss", self.id))
                    .debug_selector(|| "inline-alert-dismiss".to_string())
                    .key_context("InlineAlertDismiss")
                    .track_focus(&focus)
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label("Dismiss")
                    .tab_stop(!self.disabled)
                    .tab_index(if self.disabled { -1 } else { 0 })
                    .h(px(theme.controls.xsmall))
                    .px(px(theme.spacing.small))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.colors.text_muted)
                    .when(self.disabled, |e| {
                        e.a11y_synthetic_children(|b| b.parent_node().set_disabled())
                    })
                    .when(!self.disabled, |e| {
                        e.on_click(cx.listener(|this, _, _, cx| this.dismiss(cx)))
                    })
                    .on_action(cx.listener(Self::dismiss_action))
                    .child("Dismiss"),
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
    fn action_emits_and_controlled_dismiss_waits_for_owner(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (view, window) = cx.add_window_view(|_, _| {
            InlineAlert::controlled(Severity::Warning, "Check connection", false)
                .title("Warning")
                .action("Retry")
                .dismissible(true)
        });
        window.update(|w, cx| {
            w.draw(cx).clear(cx);
            view.focus_handle(cx).focus(w, cx);
        });
        let action_events = Rc::new(RefCell::new(0));
        let action_out = action_events.clone();
        let dismiss_events = Rc::new(RefCell::new(0));
        let dismiss_out = dismiss_events.clone();
        let _a = window.update(|_, cx| {
            cx.subscribe(&view, move |_, _: &ActionInvoked, _| *action_out.borrow_mut() += 1)
        });
        let _d = window.update(|_, cx| {
            cx.subscribe(&view, move |_, _: &DismissRequested, _| *dismiss_out.borrow_mut() += 1)
        });
        window.simulate_keystrokes("enter");
        assert_eq!(*action_events.borrow(), 1);
        let dismiss =
            window.debug_bounds("inline-alert-dismiss").expect("dismiss control rendered");
        window.simulate_click(dismiss.center(), gpui::Modifiers::default());
        assert!(!view.read_with(window, |alert, _| alert.is_dismissed()));
        assert_eq!(*dismiss_events.borrow(), 1);
    }

    #[gpui::test]
    fn uncontrolled_dismiss_hides_alert(cx: &mut TestAppContext) {
        cx.update(|app| mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT));
        let (view, window) = cx
            .add_window_view(|_, _| InlineAlert::new(Severity::Error, "Failed").dismissible(true));
        window.update(|w, cx| w.draw(cx).clear(cx));
        let bounds = window.debug_bounds("inline-alert-dismiss").unwrap();
        window.simulate_click(bounds.center(), gpui::Modifiers::default());
        assert!(view.read_with(window, |alert, _| alert.is_dismissed()));
    }

    #[gpui::test]
    fn disabled_alert_ignores_dismiss_click(cx: &mut TestAppContext) {
        cx.update(|app| mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT));
        let (view, window) = cx.add_window_view(|_, _| {
            InlineAlert::new(Severity::Error, "Failed").dismissible(true).disabled(true)
        });
        window.update(|w, cx| w.draw(cx).clear(cx));
        let bounds =
            window.debug_bounds("inline-alert-dismiss").expect("disabled dismiss control rendered");
        window.simulate_click(bounds.center(), gpui::Modifiers::default());
        assert!(!view.read_with(window, |alert, _| alert.is_dismissed()));
    }

    #[test]
    fn alert_instances_have_distinct_default_ids() {
        let first = InlineAlert::new(Severity::Info, "First");
        let second = InlineAlert::new(Severity::Info, "Second");
        assert_ne!(first.id, second.id);
    }
}
