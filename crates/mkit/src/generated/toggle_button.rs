//! Stateful, controlled-or-uncontrolled toggle button.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, Render, Window, actions, div,
    prelude::*, px,
};
use mkit_core::theme::Theme;
pub const KEY_CONTEXT: &str = "MkitToggleButton";
actions!(toggle_button, [Toggle]);
pub fn default_key_bindings() -> [gpui_pre::KeyBinding; 2] {
    [
        gpui_pre::KeyBinding::new("enter", Toggle, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("space", Toggle, Some(KEY_CONTEXT)),
    ]
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueChanged(pub bool);
pub struct ToggleButton {
    label: String,
    pressed: bool,
    controlled: bool,
    disabled: bool,
    focus: Option<FocusHandle>,
}
impl EventEmitter<ValueChanged> for ToggleButton {}
impl ToggleButton {
    pub fn new(label: impl Into<String>, default_pressed: bool) -> Self {
        Self {
            label: label.into(),
            pressed: default_pressed,
            controlled: false,
            disabled: false,
            focus: None,
        }
    }
    pub fn controlled(label: impl Into<String>, pressed: bool) -> Self {
        Self { label: label.into(), pressed, controlled: true, disabled: false, focus: None }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }
    pub fn set_pressed(&mut self, pressed: bool, cx: &mut Context<Self>) {
        self.pressed = pressed;
        cx.notify();
    }
    fn request_toggle(&mut self, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let next = !self.pressed;
        if !self.controlled {
            self.pressed = next;
            cx.notify()
        }
        cx.emit(ValueChanged(next));
    }
}
impl Render for ToggleButton {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let focus =
            self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(true)).clone();
        let pressed = self.pressed;
        div()
            .id(("mkit-toggle-button", cx.entity().entity_id()))
            .key_context(KEY_CONTEXT)
            .when(!self.disabled, |d| d.track_focus(&focus))
            .role(gpui_pre::accesskit::Role::Button)
            .aria_label(self.label.clone())
            .aria_toggled(if pressed {
                gpui_pre::accesskit::Toggled::True
            } else {
                gpui_pre::accesskit::Toggled::False
            })
            .when(self.disabled, |e| e.aria_description("Unavailable"))
            .when(self.disabled, |e| {
                e.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .on_action(cx.listener(|this, _: &Toggle, _, cx| this.request_toggle(cx)))
            .on_click(cx.listener(|this, _, _, cx| this.request_toggle(cx)))
            .h(px(t.controls.medium))
            .px(px(t.spacing.medium))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(t.radii.medium))
            .border(px(t.borders.regular))
            .border_color(if pressed { t.colors.accent } else { t.colors.border })
            .bg(if pressed { t.colors.accent } else { t.colors.surface })
            .text_color(if self.disabled {
                t.colors.disabled
            } else if pressed {
                t.colors.accent_text
            } else {
                t.colors.text
            })
            .text_size(px(t.typography.body))
            .focus_visible(|s| s.border_color(t.colors.focus))
            .child(self.label.clone())
    }
}
impl Focusable for ToggleButton {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn constructors_distinguish_control_modes() {
        let u = ToggleButton::new("bold", false);
        let c = ToggleButton::controlled("italic", true);
        assert!(!u.controlled && !u.pressed);
        assert!(c.controlled && c.pressed);
    }
}
