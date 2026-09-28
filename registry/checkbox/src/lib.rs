//! Compact, theme-driven checkbox with controlled and uncontrolled state.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, PathBuilder, Render,
    Rgba, Window, actions, canvas, div, point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};

/// Resolved box colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    input: Rgba,
    ring: Rgba,
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look { input: c.border, ring: c.focus };
    }
    let dark = relative_luminance(c.background) < 0.5;
    Look { input: if dark { c.text.opacity(0.15) } else { c.border }, ring: c.focus.opacity(0.5) }
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
/// shadcn/ui focus ring width, drawn outside the box.
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
/// Check or dash drawn as a vector stroke so it stays crisp at every scale. Geometry follows
/// Lucide `check` (20,6 → 9,17 → 4,12) and `minus` (5,12 → 19,12) on a 24-unit grid with a
/// 3-unit stroke, inside a square `size` box.
fn mark(size: f32, mixed: bool, color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let origin = bounds.origin;
            let mut path = PathBuilder::stroke(unit * 3.0);
            if mixed {
                path.move_to(origin + point(unit * 5.0, unit * 12.0));
                path.line_to(origin + point(unit * 19.0, unit * 12.0));
            } else {
                path.move_to(origin + point(unit * 20.0, unit * 6.0));
                path.line_to(origin + point(unit * 9.0, unit * 17.0));
                path.line_to(origin + point(unit * 4.0, unit * 12.0));
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
}
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self
            .focus
            .get_or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(!self.disabled))
            .clone();
        let theme = *cx.global::<Theme>();
        let label = self.label.clone();
        let checked = self.checked;
        let mixed = self.indeterminate;
        let disabled = self.disabled;
        let look = look(&theme);
        let paint =
            |color: Rgba| if disabled { dim(color, theme.colors.background) } else { color };
        let mut shadow = theme.shadows.small;
        if disabled {
            shadow.color = shadow.color.opacity(0.5);
        }
        let focus_visible =
            !disabled && focus.is_focused(window) && window.last_input_was_keyboard();
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
            .pr(px(theme.spacing.xsmall))
            .child(
                div()
                    .size(px(theme.spacing.large))
                    .flex_none()
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.hairline))
                    .border_color(paint(if focus_visible {
                        theme.colors.focus
                    } else if checked || mixed {
                        theme.colors.accent
                    } else {
                        look.input
                    }))
                    .bg(paint(if checked || mixed {
                        theme.colors.accent
                    } else {
                        theme.colors.background
                    }))
                    .shadow(if focus_visible {
                        vec![focus_ring(look.ring)]
                    } else {
                        vec![box_shadow(shadow)]
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(checked || mixed, |d| {
                        d.child(mark(theme.spacing.medium, mixed, paint(theme.colors.accent_text)))
                    }),
            )
            .child(
                div()
                    .text_size(px(theme.typography.body))
                    .font_weight(gpui_pre::FontWeight::MEDIUM)
                    .text_color(paint(theme.colors.text))
                    .child(label),
            )
    }
}
