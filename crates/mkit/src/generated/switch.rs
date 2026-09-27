//! Compact shadcn-inspired switch.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render, Window,
    actions, div, prelude::*, px,
};
use mkit_core::theme::Theme;
pub const KEY_CONTEXT: &str = "Switch";
actions!(switch, [Toggle]);
pub fn default_key_bindings() -> [KeyBinding; 1] {
    [KeyBinding::new("space", Toggle, Some(KEY_CONTEXT))]
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChangeRequested(pub bool);
impl EventEmitter<ChangeRequested> for Switch {}
pub struct Switch {
    label: String,
    checked: bool,
    controlled: bool,
    disabled: bool,
    focus: Option<FocusHandle>,
}
impl Switch {
    pub fn new(label: impl Into<String>, default_checked: bool) -> Self {
        Self {
            label: label.into(),
            checked: default_checked,
            controlled: false,
            disabled: false,
            focus: None,
        }
    }
    pub fn controlled(label: impl Into<String>, checked: bool) -> Self {
        Self { label: label.into(), checked, controlled: true, disabled: false, focus: None }
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn set_checked(&mut self, v: bool) {
        self.checked = v;
    }
    pub fn is_checked(&self) -> bool {
        self.checked
    }
    fn request(&mut self, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let v = !self.checked;
        if !self.controlled {
            self.checked = v;
            cx.notify();
        }
        cx.emit(ChangeRequested(v));
    }
    fn toggle(&mut self, _: &Toggle, _: &mut Window, cx: &mut Context<Self>) {
        self.request(cx);
    }
}
impl Focusable for Switch {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("focus initialized")
    }
}
impl Render for Switch {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let f = self
            .focus
            .get_or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(!self.disabled))
            .clone();
        let t = *cx.global::<Theme>();
        let label = self.label.clone();
        let on = self.checked;
        let disabled = self.disabled;
        div()
            .id("switch")
            .debug_selector(|| "mkit-switch".into())
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::toggle))
            .when(!disabled, |d| d.track_focus(&f))
            .role(gpui_pre::accesskit::Role::Switch)
            .aria_label(label.clone())
            .aria_toggled(gpui_pre::accesskit::Toggled::from(on))
            .when(disabled, |d| d.aria_description("Unavailable"))
            .when(disabled, |d| {
                d.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .when(!disabled, |d| d.on_click(cx.listener(|s, _, _, cx| s.request(cx))))
            .flex()
            .items_center()
            .gap(px(t.spacing.small))
            .child(
                div()
                    .w(px(t.controls.medium))
                    .h(px(t.controls.xsmall * 0.64))
                    .rounded(px(t.radii.pill))
                    .bg(if on { t.colors.accent } else { t.colors.border })
                    .p(px(t.spacing.xsmall * 0.25))
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .size(px(t.controls.xsmall * 0.48))
                            .rounded(px(t.radii.pill))
                            .bg(t.colors.surface)
                            .when(on, |d| d.ml(px(t.controls.xsmall * 0.55))),
                    ),
            )
            .child(
                div()
                    .text_size(px(t.typography.body))
                    .text_color(if disabled { t.colors.disabled } else { t.colors.text })
                    .child(label),
            )
    }
}
