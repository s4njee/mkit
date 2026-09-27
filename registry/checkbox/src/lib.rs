//! Compact, theme-driven checkbox with controlled and uncontrolled state.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render, Window,
    actions, div, prelude::*, px,
};
use mkit_core::theme::Theme;
pub const KEY_CONTEXT: &str = "Checkbox";
actions!(checkbox, [Toggle]);
pub fn default_key_bindings() -> [KeyBinding; 1] {
    [KeyBinding::new("space", Toggle, Some(KEY_CONTEXT))]
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChangeRequested(pub bool);
impl EventEmitter<ChangeRequested> for Checkbox {}
pub struct Checkbox {
    label: String,
    checked: bool,
    controlled: bool,
    indeterminate: bool,
    disabled: bool,
    focus: Option<FocusHandle>,
}
impl Checkbox {
    pub fn new(label: impl Into<String>, default_checked: bool) -> Self {
        Self {
            label: label.into(),
            checked: default_checked,
            controlled: false,
            indeterminate: false,
            disabled: false,
            focus: None,
        }
    }
    pub fn controlled(label: impl Into<String>, checked: bool) -> Self {
        Self {
            label: label.into(),
            checked,
            controlled: true,
            indeterminate: false,
            disabled: false,
            focus: None,
        }
    }
    pub fn indeterminate(mut self, value: bool) -> Self {
        self.indeterminate = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn is_checked(&self) -> bool {
        self.checked
    }
    pub fn is_indeterminate(&self) -> bool {
        self.indeterminate
    }
    pub fn set_checked(&mut self, value: bool, cx: &mut Context<Self>) {
        self.checked = value;
        self.indeterminate = false;
        cx.notify();
    }
    fn request_toggle(&mut self, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let value = if self.indeterminate { true } else { !self.checked };
        if !self.controlled {
            self.checked = value;
            self.indeterminate = false;
            cx.notify();
        }
        cx.emit(ChangeRequested(value));
    }
    fn toggle(&mut self, _: &Toggle, _: &mut Window, cx: &mut Context<Self>) {
        self.request_toggle(cx);
    }
}
impl Focusable for Checkbox {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("focus initialized")
    }
}
impl Render for Checkbox {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self
            .focus
            .get_or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(!self.disabled))
            .clone();
        let theme = *cx.global::<Theme>();
        let label = self.label.clone();
        let checked = self.checked;
        let mixed = self.indeterminate;
        let disabled = self.disabled;
        div()
            .id("checkbox")
            .debug_selector(|| "mkit-checkbox".into())
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::toggle))
            .when(!disabled, |d| d.track_focus(&focus))
            .role(gpui_pre::accesskit::Role::CheckBox)
            .aria_label(label.clone())
            .aria_toggled(if mixed {
                gpui_pre::accesskit::Toggled::Mixed
            } else {
                gpui_pre::accesskit::Toggled::from(checked)
            })
            .when(disabled || mixed, |d| {
                d.aria_description(match (disabled, mixed) {
                    (true, true) => "Unavailable; Mixed",
                    (true, false) => "Unavailable",
                    (false, true) => "Mixed",
                    (false, false) => unreachable!(),
                })
            })
            .when(disabled, |d| {
                d.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .when(!disabled, |d| d.on_click(cx.listener(|this, _, _, cx| this.request_toggle(cx))))
            .flex()
            .items_center()
            .gap(px(theme.spacing.small))
            .child(
                div()
                    .w(px(theme.controls.xsmall * 0.62))
                    .h(px(theme.controls.xsmall * 0.62))
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.hairline))
                    .border_color(if checked || mixed {
                        theme.colors.accent
                    } else {
                        theme.colors.border
                    })
                    .bg(if checked || mixed { theme.colors.accent } else { theme.colors.surface })
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.colors.accent_text)
                    .child(if checked {
                        "✓"
                    } else if mixed {
                        "−"
                    } else {
                        ""
                    }),
            )
            .child(
                div()
                    .text_size(px(theme.typography.body))
                    .text_color(if disabled { theme.colors.disabled } else { theme.colors.text })
                    .child(label),
            )
    }
}
