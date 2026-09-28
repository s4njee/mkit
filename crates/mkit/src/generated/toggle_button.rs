//! Stateful, controlled-or-uncontrolled toggle button.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement, Render, Rgba, Window,
    actions, div, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::Theme,
};

/// Resolved colours for one pressed/disabled state; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    bg: Rgba,
    fg: Rgba,
    border: Rgba,
    hover_bg: Option<Rgba>,
    hover_fg: Option<Rgba>,
    hover_border: Option<Rgba>,
    /// Opaque fill used while the focus ring is drawn.
    focus_bg: Rgba,
    ring: Rgba,
    /// Whether the look is dimmed to 50% opacity (disabled, shadcn-style themes).
    dim: bool,
}
fn look(t: &Theme, pressed: bool, disabled: bool) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        let (bg, fg, border) = match (pressed, disabled) {
            (true, false) => (c.accent, c.accent_text, c.accent),
            (false, false) => (c.background, c.text, c.border),
            (true, true) => (c.disabled, c.accent_text, c.disabled),
            (false, true) => (c.background, c.disabled, c.disabled),
        };
        return Look {
            bg,
            fg,
            border,
            hover_bg: None,
            hover_fg: None,
            hover_border: (!pressed).then_some(c.accent),
            focus_bg: bg,
            ring: c.focus,
            dim: false,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn "muted"/"accent": text mixed into the background.
    let muted = composite(c.text.opacity(if dark { 0.12 } else { 0.04 }), c.background);
    let transparent = c.background.opacity(0.);
    Look {
        bg: if pressed { muted } else { transparent },
        fg: c.text,
        border: transparent,
        hover_bg: (!pressed).then_some(muted),
        hover_fg: (!pressed).then_some(c.text_muted),
        hover_border: None,
        // GPUI fills the inside of drop shadows, so the fill under the focus ring is opaque.
        focus_bg: if pressed { muted } else { c.background },
        ring: c.focus.opacity(0.5),
        dim: disabled,
    }
}
/// shadcn/ui focus ring width, drawn outside the toggle.
const FOCUS_RING_WIDTH: f32 = 3.0;
fn focus_ring(color: Rgba) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: color.into(),
        offset: gpui_pre::point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(FOCUS_RING_WIDTH),
        inset: false,
    }
}
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
        let look = look(&t, pressed, self.disabled);
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
            .min_w(px(t.controls.medium))
            .px(px(t.spacing.small))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(t.spacing.small))
            .rounded(px(t.radii.medium))
            .border(px(t.borders.regular))
            .border_color(look.border)
            .bg(look.bg)
            .text_color(look.fg)
            .text_size(px(t.typography.body))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .when(look.dim, |e| e.opacity(0.5))
            .when(!self.disabled, |e| {
                e.hover(move |s| {
                    let s = match look.hover_bg {
                        Some(color) => s.bg(color),
                        None => s,
                    };
                    let s = match look.hover_fg {
                        Some(color) => s.text_color(color),
                        None => s,
                    };
                    match look.hover_border {
                        Some(color) => s.border_color(color),
                        None => s,
                    }
                })
            })
            .focus_visible(move |s| {
                s.border_color(t.colors.focus).bg(look.focus_bg).shadow(vec![focus_ring(look.ring)])
            })
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
