//! Stateful progress header and navigation controls for multi-step flows.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render, Window,
    actions, div, prelude::*, px,
};
use mkit_core::{
    a11y::{AccessibilityExt, LiveRegionPriority},
    theme::Theme,
};
use std::{cell::OnceCell, rc::Rc};

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
            .child(div().flex().flex_col().gap(px(theme.spacing.small)).children(
                steps.iter().enumerate().map(|(index, step)| {
                    let (state, color) = if index == current {
                        ("Current", theme.colors.accent)
                    } else if index < current {
                        ("Complete", theme.colors.success)
                    } else {
                        ("Upcoming", theme.colors.text_muted)
                    };
                    div()
                        .id(format!("mkit-stepper-step-{index}"))
                        .flex()
                        .items_center()
                        .gap(px(theme.spacing.small))
                        .text_color(color)
                        .child(div().text_size(px(theme.typography.body)).child(format!(
                            "{}. {}",
                            index + 1,
                            step.label
                        )))
                        .child(
                            div()
                                .text_size(px(theme.typography.caption))
                                .text_color(theme.colors.text_muted)
                                .child(state),
                        )
                }),
            ))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(theme.spacing.medium))
                    .when(!is_first, |el| {
                        el.child(
                            div()
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
                                .px(px(theme.spacing.medium))
                                .py(px(theme.spacing.small))
                                .bg(theme.colors.surface)
                                .text_color(theme.colors.text)
                                .border_1()
                                .border_color(theme.colors.border)
                                .child("Back"),
                        )
                    })
                    .child(div().flex_1())
                    .child(
                        div()
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
                            .px(px(theme.spacing.medium))
                            .py(px(theme.spacing.small))
                            .bg(theme.colors.accent)
                            .text_color(theme.colors.accent_text)
                            .child(if is_last { "Finish" } else { "Next" }),
                    ),
            )
            .when_some(error, |el, message| {
                el.child(
                    div()
                        .id("mkit-stepper-error")
                        .role(gpui_pre::accesskit::Role::Status)
                        .aria_label(message.clone())
                        .a11y_live_region(LiveRegionPriority::Polite)
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
