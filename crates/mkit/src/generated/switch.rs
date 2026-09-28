//! Compact shadcn-inspired switch.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render, Rgba, Window,
    actions, div, point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};

/// Resolved track and thumb colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    track: Rgba,
    track_border: Rgba,
    thumb: Rgba,
    ring: Rgba,
}
fn look(t: &Theme, on: bool) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            track: if on { c.accent } else { c.background },
            track_border: if on { c.accent } else { c.border },
            thumb: if on { c.accent_text } else { c.text },
            ring: c.focus,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // Track fills stay opaque: GPUI paints the drop shadow as a filled shape inside the element.
    let off_track = if dark { composite(c.text.opacity(0.15), c.background) } else { c.border };
    Look {
        track: if on { c.accent } else { off_track },
        track_border: c.background.opacity(0.),
        thumb: if dark && !on { c.text } else { c.background },
        ring: c.focus.opacity(0.5),
    }
}
/// Disabled controls render at 50% opacity as one layer, like the web preview's `opacity: .5`.
/// GPUI applies element opacity to each painted part separately, so overlapping parts would show
/// through each other; instead each colour is composited opaque over `background` first.
fn dim(color: Rgba, background: Rgba) -> Rgba {
    if color.a == 0. {
        color
    } else {
        composite(composite(color, background).opacity(0.5), background)
    }
}
fn box_shadow(shadow: ShadowToken) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: shadow.color.into(),
        offset: point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}
/// shadcn/ui focus ring width, drawn outside the track.
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let f = self
            .focus
            .get_or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(!self.disabled))
            .clone();
        let t = *cx.global::<Theme>();
        let label = self.label.clone();
        let on = self.checked;
        let disabled = self.disabled;
        let look = look(&t, on);
        let focus_visible = !disabled && f.is_focused(window) && window.last_input_was_keyboard();
        let paint = |color: Rgba| if disabled { dim(color, t.colors.background) } else { color };
        let mut shadow = t.shadows.small;
        if disabled {
            shadow.color = shadow.color.opacity(0.5);
        }
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
            .pr(px(t.spacing.xsmall))
            .child(
                div()
                    .w(px(t.spacing.xxlarge))
                    .h(px(t.spacing.large + 2. * t.borders.hairline))
                    .flex_none()
                    .rounded(px(t.radii.pill))
                    .border(px(t.borders.hairline))
                    .border_color(if focus_visible {
                        t.colors.focus
                    } else {
                        paint(look.track_border)
                    })
                    .bg(paint(look.track))
                    .shadow(if focus_visible {
                        vec![focus_ring(look.ring)]
                    } else {
                        vec![box_shadow(shadow)]
                    })
                    .flex()
                    .items_center()
                    .when(on, |d| d.justify_end())
                    .child(
                        div()
                            .size(px(t.spacing.large))
                            .rounded(px(t.radii.pill))
                            .bg(paint(look.thumb))
                            .shadow(vec![box_shadow(shadow)]),
                    ),
            )
            .child(
                div()
                    .text_size(px(t.typography.body))
                    .font_weight(gpui_pre::FontWeight::MEDIUM)
                    .text_color(paint(t.colors.text))
                    .child(label),
            )
    }
}
