//! Persistent contextual messages with severity-appropriate live-region semantics.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnyElement, Context, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement, KeyBinding,
    Render, Rgba, Window, actions, div, point, prelude::*, px,
};
use mkit_core::{
    a11y::{AccessibilityExt, LiveRegionPriority},
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
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

/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}

/// shadcn's `opacity: .5` applied as one layer: composite over `base`, then mix 50%.
fn dim(color: Rgba, base: Rgba) -> Rgba {
    mix(composite(color, base), base, 0.5)
}

/// Colours for one button: fill, border, text, hover fill, hover text.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ControlLook {
    bg: Rgba,
    border: Rgba,
    fg: Rgba,
    hover: Option<(Rgba, Rgba)>,
    /// Opaque fill used while the focus ring is drawn.
    focus_bg: Rgba,
}

/// Resolved colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Look {
    border: Rgba,
    icon: Rgba,
    title: Rgba,
    message: Rgba,
    action: ControlLook,
    dismiss: ControlLook,
    ring: Rgba,
    shadow_alpha: f32,
}

fn look(t: &Theme, severity: Severity, disabled: bool) -> Look {
    let c = t.colors;
    let transparent = c.background.opacity(0.);
    let icon = match severity {
        Severity::Info => c.text,
        Severity::Success => c.success,
        Severity::Warning => c.warning,
        Severity::Error => c.danger,
    };
    if t.name == "high-contrast" {
        let fg = if disabled { c.disabled } else { c.text };
        let border = if disabled { c.disabled } else { c.border };
        return Look {
            border: c.border,
            icon,
            title: c.text,
            message: c.text,
            action: ControlLook {
                bg: c.background,
                border,
                fg,
                hover: None,
                focus_bg: c.background,
            },
            dismiss: ControlLook {
                bg: transparent,
                border: transparent,
                fg,
                hover: None,
                focus_bg: c.background,
            },
            ring: c.focus,
            shadow_alpha: 0.0,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    let border = if dark { mix(c.text, c.background, 0.1) } else { c.border };
    let error = severity == Severity::Error;
    let mut action = ControlLook {
        bg: c.background,
        border,
        fg: c.text,
        hover: Some((muted, c.text)),
        focus_bg: c.background,
    };
    let mut dismiss = ControlLook {
        bg: transparent,
        border: transparent,
        fg: c.text_muted,
        hover: Some((muted, c.text)),
        focus_bg: c.background,
    };
    if disabled {
        for control in [&mut action, &mut dismiss] {
            let bg = if control.bg.a > 0.0 { dim(control.bg, c.background) } else { control.bg };
            let border = if control.border.a > 0.0 {
                dim(control.border, c.background)
            } else {
                control.border
            };
            *control = ControlLook {
                bg,
                border,
                fg: dim(control.fg, c.background),
                hover: None,
                focus_bg: control.focus_bg,
            };
        }
    }
    Look {
        border: match severity {
            Severity::Warning | Severity::Error => mix(icon, border, 0.45),
            _ => border,
        },
        icon,
        title: if error { c.danger } else { c.text },
        message: if error { mix(c.danger, c.background, 0.9) } else { c.text_muted },
        action,
        dismiss,
        ring: c.focus.opacity(0.5),
        shadow_alpha: if disabled { 0.5 } else { 1.0 },
    }
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

/// shadcn/ui focus ring width, drawn outside the focused button.
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
        let (role, priority) = match self.severity {
            Severity::Info | Severity::Success => {
                (gpui_pre::accesskit::Role::Status, LiveRegionPriority::Polite)
            }
            Severity::Warning | Severity::Error => {
                (gpui_pre::accesskit::Role::Alert, LiveRegionPriority::Assertive)
            }
        };
        let look = look(&theme, self.severity, self.disabled);
        let line = theme.spacing.large + theme.spacing.xsmall;
        let enabled = !self.disabled;
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
            .rounded(px(theme.radii.large))
            .border(px(theme.borders.regular))
            .border_color(look.border)
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .text_size(px(theme.typography.body))
            .line_height(px(line));
        if let Some(icon) = self.icon.as_ref() {
            root = root.child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .w(px(theme.spacing.large))
                    .h(px(line))
                    .text_color(look.icon)
                    .child(icon()),
            );
        }
        let mut body =
            div().flex().flex_col().items_start().gap(px(theme.spacing.xsmall)).flex_1().min_w_0();
        if let Some(title) = self.title.as_ref() {
            body = body.child(
                div()
                    .w_full()
                    .text_color(look.title)
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title.clone()),
            );
        }
        body = body.child(div().w_full().text_color(look.message).child(self.message.clone()));
        if let Some(label) = self.action_label.clone() {
            let focus = action_focus.as_ref().expect("action focus initialized").clone();
            let control = look.action;
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
                    .mt(px(theme.spacing.small))
                    .h(px(theme.controls.small))
                    .px(px(theme.spacing.medium))
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(control.border)
                    .bg(control.bg)
                    .shadow(vec![box_shadow(theme.shadows.small, look.shadow_alpha)])
                    .text_color(control.fg)
                    .font_weight(FontWeight::MEDIUM)
                    .whitespace_nowrap()
                    .when_some(control.hover.filter(|_| enabled), |e, (bg, fg)| {
                        e.hover(move |s| s.bg(bg).text_color(fg))
                    })
                    .focus_visible(move |s| {
                        s.border_color(theme.colors.focus)
                            .bg(control.focus_bg)
                            .shadow(vec![focus_ring(look.ring)])
                    })
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
            let control = look.dismiss;
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
                    // Centre the button on the title line without growing the card.
                    .my(px(-(theme.controls.xsmall - line) / 2.0))
                    .px(px(theme.spacing.small))
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(control.border)
                    .bg(control.bg)
                    .text_color(control.fg)
                    .font_weight(FontWeight::MEDIUM)
                    .when_some(control.hover.filter(|_| enabled), |e, (bg, fg)| {
                        e.hover(move |s| s.bg(bg).text_color(fg))
                    })
                    .focus_visible(move |s| {
                        s.border_color(theme.colors.focus)
                            .bg(control.focus_bg)
                            .shadow(vec![focus_ring(look.ring)])
                    })
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
    fn disabled_controls_dim_and_high_contrast_stays_solid() {
        use mkit_core::theme::{HIGH_CONTRAST, SHADCN_DARK};
        let resting = look(&SHADCN_DARK, Severity::Info, false);
        let disabled = look(&SHADCN_DARK, Severity::Info, true);
        let bg = SHADCN_DARK.colors.background;
        assert_eq!(disabled.action.fg, dim(resting.action.fg, bg));
        assert_eq!(disabled.action.hover, None);
        assert_eq!(disabled.message, resting.message, "the message is not dimmed");
        let hc = look(&HIGH_CONTRAST, Severity::Error, true);
        assert_eq!(hc.action.fg, HIGH_CONTRAST.colors.disabled);
        assert_eq!(hc.ring, HIGH_CONTRAST.colors.focus);
    }

    #[test]
    fn alert_instances_have_distinct_default_ids() {
        let first = InlineAlert::new(Severity::Info, "First");
        let second = InlineAlert::new(Severity::Info, "Second");
        assert_ne!(first.id, second.id);
    }
}
