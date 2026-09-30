//! Stateful progress header and navigation controls for multi-step flows.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Context, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement, KeyBinding,
    PathBuilder, Render, Rgba, Window, actions, canvas, div, point, prelude::*, px,
};
use mkit_core::{
    a11y::{AccessibilityExt, LiveRegionPriority},
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
use std::{cell::OnceCell, rc::Rc};

/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight, ..foreground }, Rgba { a: 1.0, ..base })
}
/// Resolved step indicator colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    /// Complete and current indicator fill, and the completed connector.
    primary: Rgba,
    on_primary: Rgba,
    /// Current indicator fill after failed validation.
    error: Rgba,
    on_error: Rgba,
    /// Upcoming indicator fill, border and number.
    upcoming_bg: Rgba,
    upcoming_border: Rgba,
    upcoming_fg: Rgba,
    /// Connector between steps that are not yet complete.
    rail: Rgba,
    label: Rgba,
    upcoming_label: Rgba,
    caption: Rgba,
    icon_stroke: IconStroke,
}
#[derive(Clone, Copy)]
enum IconStroke {
    /// Lucide's 2-unit stroke on its 24-unit grid, scaled with the icon.
    Relative,
    Pixels(f32),
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            primary: c.accent,
            on_primary: c.accent_text,
            error: c.danger,
            on_error: c.accent_text,
            upcoming_bg: c.background,
            upcoming_border: c.border,
            upcoming_fg: c.text,
            rail: c.border,
            label: c.text,
            upcoming_label: c.text_muted,
            caption: c.text_muted,
            icon_stroke: IconStroke::Pixels(t.borders.regular),
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn's dark `input` (15% text), composited over the background: the 10% `border` is too
    // faint for an empty indicator ring.
    let outline = if dark { mix(c.text, c.background, 0.15) } else { c.border };
    Look {
        primary: c.accent,
        on_primary: c.accent_text,
        error: c.danger,
        // Button's destructive text: the theme's near-white.
        on_error: if dark { c.text } else { c.background },
        upcoming_bg: c.background,
        upcoming_border: outline,
        upcoming_fg: c.text_muted,
        rail: outline,
        label: c.text,
        upcoming_label: c.text_muted,
        caption: c.text_muted,
        icon_stroke: IconStroke::Relative,
    }
}
/// Resolved colours for a navigation button, matching Button's outline (Back) and default
/// (Next/Finish) variants.
#[derive(Clone, Copy)]
struct ButtonLook {
    bg: Rgba,
    fg: Rgba,
    border: Rgba,
    shadow: bool,
    hover_bg: Option<Rgba>,
    hover_border: Option<Rgba>,
    ring: Rgba,
}
fn button_look(t: &Theme, primary: bool) -> ButtonLook {
    let c = t.colors;
    if t.name == "high-contrast" {
        let (bg, fg, border, hover_border) = if primary {
            (c.accent, c.accent_text, c.accent, c.text)
        } else {
            (c.background, c.text, c.border, c.accent)
        };
        return ButtonLook {
            bg,
            fg,
            border,
            shadow: false,
            hover_bg: None,
            hover_border: Some(hover_border),
            ring: c.focus,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    let (bg, fg, border, hover_bg) = if primary {
        (c.accent, c.accent_text, c.accent, mix(c.accent, c.background, 0.9))
    } else {
        (c.background, c.text, if dark { mix(c.text, c.background, 0.1) } else { c.border }, muted)
    };
    ButtonLook {
        bg,
        fg,
        border,
        shadow: true,
        hover_bg: Some(hover_bg),
        hover_border: None,
        ring: c.focus.opacity(0.5),
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
/// Decorative Lucide `check` (20,6 → 9,17 → 4,12) drawn as a vector path on a 24-unit grid.
fn check_icon(size: f32, stroke: IconStroke, color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let width = match stroke {
                IconStroke::Relative => unit * 2.0,
                IconStroke::Pixels(width) => px(width),
            };
            let mut path = PathBuilder::stroke(width);
            for (i, (x, y)) in [(20.0, 6.0), (9.0, 17.0), (4.0, 12.0)].into_iter().enumerate() {
                let p = bounds.origin + point(unit * x, unit * y);
                if i == 0 { path.move_to(p) } else { path.line_to(p) }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
}
/// A navigation button styled like the registry Button at its default size.
fn nav_button(theme: Theme, primary: bool) -> gpui_pre::Div {
    let look = button_look(&theme, primary);
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(theme.controls.medium))
        .px(px(theme.spacing.large))
        .rounded(px(theme.radii.medium))
        .border(px(theme.borders.regular))
        .border_color(look.border)
        .bg(look.bg)
        .when(look.shadow, |el| el.shadow(vec![box_shadow(theme.shadows.small)]))
        .text_color(look.fg)
        .text_size(px(theme.typography.body))
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap()
}
fn nav_button_states<E: InteractiveElement + StatefulInteractiveElement + Styled>(
    element: E,
    theme: Theme,
    primary: bool,
) -> E {
    let look = button_look(&theme, primary);
    element
        .hover(move |s| {
            let s = match look.hover_bg {
                Some(color) => s.bg(color),
                None => s,
            };
            match look.hover_border {
                Some(color) => s.border_color(color),
                None => s,
            }
        })
        .focus_visible(move |s| {
            s.border_color(theme.colors.focus).shadow(vec![focus_ring(look.ring)])
        })
}

pub const KEY_CONTEXT: &str = "Stepper";
actions!(stepper, [ActivateStep, PreviousStep, NextStep, FinishStep]);

pub fn default_key_bindings() -> [KeyBinding; 4] {
    [
        KeyBinding::new("enter", ActivateStep, Some(KEY_CONTEXT)),
        KeyBinding::new("space", ActivateStep, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-left", PreviousStep, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-right", NextStep, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub id: String,
    pub label: String,
}
impl Step {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepChangeKind {
    Back,
    Next,
    Finish,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepChangeRequested {
    pub from: usize,
    pub to: usize,
    pub kind: StepChangeKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepValidationFailed {
    pub step_id: String,
    pub message: String,
}
impl EventEmitter<StepChangeRequested> for Stepper {}
impl EventEmitter<StepValidationFailed> for Stepper {}

type Validator = Rc<dyn Fn(&str) -> Result<(), String>>;

/// Step progress and navigation. Content and dialog/sheet ownership remain with the host.
pub struct Stepper {
    label: String,
    steps: Vec<Step>,
    current: usize,
    controlled: bool,
    validator: Option<Validator>,
    error: Option<String>,
    back_focus: OnceCell<FocusHandle>,
    next_focus: OnceCell<FocusHandle>,
}

/// Enabled navigation buttons are Tab stops at the default index, so they follow rendered order.
fn navigation_focus(cell: &OnceCell<FocusHandle>, cx: &App) -> FocusHandle {
    cell.get_or_init(|| cx.focus_handle().tab_stop(true)).clone()
}

impl Stepper {
    pub fn new(label: impl Into<String>, steps: Vec<Step>) -> Self {
        Self {
            label: label.into(),
            steps,
            current: 0,
            controlled: false,
            validator: None,
            error: None,
            back_focus: OnceCell::new(),
            next_focus: OnceCell::new(),
        }
    }
    /// Seed uncontrolled current step.
    pub fn default_step(mut self, index: usize) -> Self {
        self.current = index.min(self.steps.len().saturating_sub(1));
        self
    }
    /// Set parent-owned current step. User navigation emits proposals without changing it.
    pub fn current_step(mut self, index: usize) -> Self {
        self.current = index.min(self.steps.len().saturating_sub(1));
        self.controlled = true;
        self
    }
    pub fn validate_with(
        mut self,
        validator: impl Fn(&str) -> Result<(), String> + 'static,
    ) -> Self {
        self.validator = Some(Rc::new(validator));
        self
    }
    pub fn set_current_step(&mut self, index: usize, cx: &mut Context<Self>) {
        self.current = index.min(self.steps.len().saturating_sub(1));
        cx.notify();
    }
    pub fn current_step_index(&self) -> usize {
        self.current
    }

    fn request(&mut self, kind: StepChangeKind, window: &mut Window, cx: &mut Context<Self>) {
        let Some(step) = self.steps.get(self.current) else {
            return;
        };
        let last = self.steps.len().saturating_sub(1);
        if (matches!(kind, StepChangeKind::Back) && self.current == 0)
            || (matches!(kind, StepChangeKind::Finish) && self.current < last)
        {
            return;
        }
        if matches!(kind, StepChangeKind::Next | StepChangeKind::Finish)
            && let Some(validate) = &self.validator
            && let Err(message) = validate(&step.id)
        {
            self.error = Some(message.clone());
            cx.emit(StepValidationFailed { step_id: step.id.clone(), message });
            cx.notify();
            return;
        }
        self.error = None;
        let to = match kind {
            StepChangeKind::Back => self.current.saturating_sub(1),
            StepChangeKind::Next => (self.current + 1).min(last),
            StepChangeKind::Finish => self.current,
        };
        let from = self.current;
        if !self.controlled {
            self.current = to;
        }
        cx.emit(StepChangeRequested { from, to, kind });
        cx.notify();
        let target = if matches!(kind, StepChangeKind::Back) && to > 0 {
            self.back_focus.get()
        } else {
            self.next_focus.get()
        };
        if let Some(target) = target {
            target.focus(window, cx);
        }
    }
}

impl Focusable for Stepper {
    /// The Next/Finish button's handle. It is valid before the first render.
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        navigation_focus(&self.next_focus, cx)
    }
}

impl Render for Stepper {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let back_focus = navigation_focus(&self.back_focus, cx);
        let next_focus = navigation_focus(&self.next_focus, cx);
        let theme = *cx.global::<Theme>();
        let current = self.current.min(self.steps.len().saturating_sub(1));
        let steps = self.steps.clone();
        let count = steps.len();
        let label = self.label.clone();
        let current_name = steps.get(current).map(|step| step.label.as_str()).unwrap_or("No steps");
        let group_name = format!("{label}, step {} of {count}: {current_name}", current + 1);
        let error = self.error.clone();
        let is_first = current == 0;
        let is_last = current + 1 >= count;
        let has_error = error.is_some();
        let look = look(&theme);
        // Indicator circle: shadcn's `size-6` step marker.
        let indicator_size = theme.spacing.xlarge;
        div()
            .id("mkit-stepper")
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(group_name)
            // Shortcut actions live on the root so they work from either navigation button.
            .on_action(cx.listener(|this, _: &PreviousStep, window, cx| {
                this.request(StepChangeKind::Back, window, cx)
            }))
            .on_action(cx.listener(move |this, _: &NextStep, window, cx| {
                this.request(
                    if is_last { StepChangeKind::Finish } else { StepChangeKind::Next },
                    window,
                    cx,
                )
            }))
            .on_action(cx.listener(|this, _: &FinishStep, window, cx| {
                this.request(StepChangeKind::Finish, window, cx)
            }))
            .child(div().flex().flex_col().children(steps.iter().enumerate().map(
                |(index, step)| {
                    let complete = index < current;
                    let is_current = index == current;
                    let failed = is_current && has_error;
                    let (state, label_color) = if is_current {
                        ("Current", look.label)
                    } else if complete {
                        ("Complete", look.label)
                    } else {
                        ("Upcoming", look.upcoming_label)
                    };
                    let (bg, border, fg) = if failed {
                        (look.error, look.error, look.on_error)
                    } else if complete || is_current {
                        (look.primary, look.primary, look.on_primary)
                    } else {
                        (look.upcoming_bg, look.upcoming_border, look.upcoming_fg)
                    };
                    let indicator = div()
                        .size(px(indicator_size))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(theme.radii.pill))
                        .border(px(theme.borders.regular))
                        .border_color(border)
                        .bg(bg)
                        .text_color(fg)
                        .text_size(px(theme.typography.caption))
                        .font_weight(FontWeight::MEDIUM)
                        .map(|el| {
                            if complete {
                                el.child(check_icon(theme.spacing.large, look.icon_stroke, fg))
                            } else {
                                el.child(format!("{}", index + 1))
                            }
                        });
                    let row = div()
                        .id(format!("mkit-stepper-step-{index}"))
                        .flex()
                        .items_center()
                        .gap(px(theme.spacing.medium))
                        .child(indicator)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(theme.spacing.small))
                                .child(
                                    div()
                                        .text_size(px(theme.typography.body))
                                        .text_color(label_color)
                                        .when(is_current, |el| el.font_weight(FontWeight::MEDIUM))
                                        .child(step.label.clone()),
                                )
                                .child(
                                    div()
                                        .text_size(px(theme.typography.caption))
                                        .text_color(look.caption)
                                        .child(if failed { "Needs attention" } else { state }),
                                ),
                        );
                    // Connector to the next step: `accent` once this step is complete.
                    let connector = (index + 1 < count).then(|| {
                        div()
                            .w(px(indicator_size))
                            .flex()
                            .justify_center()
                            .py(px(theme.spacing.xsmall))
                            .child(
                                div()
                                    .w(px(theme.borders.strong))
                                    .h(px(theme.spacing.medium))
                                    .rounded(px(theme.radii.pill))
                                    .bg(if complete { look.primary } else { look.rail }),
                            )
                    });
                    div().flex().flex_col().child(row).children(connector)
                },
            )))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(theme.spacing.medium))
                    .when(!is_first, |el| {
                        el.child(nav_button_states(
                            nav_button(theme, false)
                                .id("mkit-stepper-back")
                                .key_context(KEY_CONTEXT)
                                .track_focus(&back_focus)
                                .tab_index(0)
                                .role(gpui_pre::accesskit::Role::Button)
                                .aria_label("Back")
                                .on_action(cx.listener(|this, _: &ActivateStep, window, cx| {
                                    this.request(StepChangeKind::Back, window, cx)
                                }))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.request(StepChangeKind::Back, window, cx)
                                }))
                                .child("Back"),
                            theme,
                            false,
                        ))
                    })
                    .child(div().flex_1())
                    .child(nav_button_states(
                        nav_button(theme, true)
                            .id("mkit-stepper-next")
                            .key_context(KEY_CONTEXT)
                            .track_focus(&next_focus)
                            .tab_index(0)
                            .role(gpui_pre::accesskit::Role::Button)
                            .aria_label(if is_last { "Finish" } else { "Next" })
                            .on_action(cx.listener(move |this, _: &ActivateStep, window, cx| {
                                this.request(
                                    if is_last {
                                        StepChangeKind::Finish
                                    } else {
                                        StepChangeKind::Next
                                    },
                                    window,
                                    cx,
                                )
                            }))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.request(
                                    if is_last {
                                        StepChangeKind::Finish
                                    } else {
                                        StepChangeKind::Next
                                    },
                                    window,
                                    cx,
                                )
                            }))
                            .child(if is_last { "Finish" } else { "Next" }),
                        theme,
                        true,
                    )),
            )
            .when_some(error, |el, message| {
                el.child(
                    div()
                        .id("mkit-stepper-error")
                        .role(gpui_pre::accesskit::Role::Status)
                        .aria_label(message.clone())
                        .a11y_live_region(LiveRegionPriority::Polite)
                        .text_size(px(theme.typography.body))
                        .text_color(theme.colors.danger)
                        .child(message),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steps() -> Vec<Step> {
        vec![Step::new("account", "Account"), Step::new("review", "Review")]
    }

    #[gpui_pre::test]
    async fn enter_advances_uncontrolled_and_controlled_modes_stay_parent_owned(
        cx: &mut gpui_pre::TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (view, visual) = cx.add_window_view(|_, _| Stepper::new("Setup", steps()));
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            view.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("enter");
        assert_eq!(view.read_with(visual, |stepper, _| stepper.current_step_index()), 1);

        let (controlled, visual) =
            cx.add_window_view(|_, _| Stepper::new("Setup", steps()).current_step(0));
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            controlled.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("enter");
        assert_eq!(controlled.read_with(visual, |stepper, _| stepper.current_step_index()), 0);
    }

    #[gpui_pre::test]
    async fn navigation_buttons_are_tab_stops_and_shortcuts_work_from_either_button(
        cx: &mut gpui_pre::TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let three =
            || vec![Step::new("a", "Account"), Step::new("b", "Options"), Step::new("c", "Review")];
        // Hosts may read the handle before the first render, e.g. for container focus stops.
        let unrendered = cx.new(|_| Stepper::new("Setup", three()));
        let _ = unrendered.read_with(cx, |stepper, cx| stepper.focus_handle(cx));

        let (view, visual) =
            cx.add_window_view(|_, _| Stepper::new("Setup", three()).default_step(1));
        let next = view.read_with(visual, |stepper, cx| stepper.focus_handle(cx));
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            next.focus(window, cx);
        });
        visual.update(|window, cx| window.focus_prev(cx));
        let back = visual.update(|window, cx| window.focused(cx)).expect("Back is a tab stop");
        assert_ne!(back, next);
        visual.update(|window, cx| window.focus_next(cx));
        assert!(visual.update(|window, _| next.is_focused(window)));

        // Alt+Left from Next goes back; Alt+Right from Back goes forward.
        visual.simulate_keystrokes("alt-left");
        assert_eq!(view.read_with(visual, |stepper, _| stepper.current_step_index()), 0);
        visual.simulate_keystrokes("alt-right");
        visual.update(|window, cx| back.focus(window, cx));
        visual.simulate_keystrokes("alt-right");
        assert_eq!(view.read_with(visual, |stepper, _| stepper.current_step_index()), 2);
    }

    #[gpui_pre::test]
    async fn failed_validation_keeps_current_step_and_exposes_polite_error(
        cx: &mut gpui_pre::TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (view, visual) = cx.add_window_view(|_, _| {
            Stepper::new("Setup", steps()).validate_with(|_| Err("Complete required fields".into()))
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            view.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("enter");
        assert_eq!(view.read_with(visual, |stepper, _| stepper.current_step_index()), 0);
        assert_eq!(
            view.read_with(visual, |stepper, _| stepper.error.clone()),
            Some("Complete required fields".into())
        );
    }
}
