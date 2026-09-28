//! E7 composition scenes: one registry component hosted inside another.
//!
//! The scenes live in an example crate so that registry components never depend on each other.
//! The host owns the stepper entity, the active step's content, and the modal's open state.

use gpui_pre::{
    App, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Subscription, Window, div,
    prelude::*, px,
};
use mkit::{
    core::{focus::focus_visible_with_theme, theme::Theme},
    dialog::{self, Dialog},
    sheet::{self, Sheet},
    stepper::{self, Step, StepChangeKind, StepChangeRequested, Stepper},
};

/// Install the default, rebindable keys for every component these scenes compose.
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys(dialog::default_key_bindings());
    cx.bind_keys(sheet::default_key_bindings());
    cx.bind_keys(stepper::default_key_bindings());
}

/// Which modal container hosts the stepper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalKind {
    Dialog,
    Sheet,
}

/// Events the host observed, in order. Tests read this log to check the event contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneEvent {
    Step(StepChangeRequested),
    ModalOpenChanged(bool),
}

const STEP_COPY: [(&str, &str, &str); 3] = [
    ("Source", "Pick the folder to import from.", "Choose source folder"),
    ("Options", "Decide how duplicates are handled.", "Edit duplicate handling"),
    ("Review", "Confirm the import summary.", "Show import summary"),
];

fn import_steps() -> Vec<Step> {
    vec![
        Step::new("source", "Source"),
        Step::new("options", "Options"),
        Step::new("review", "Review"),
    ]
}

// ANCHOR: stepper_modal_step_content
/// Host-owned content for the active step. It re-renders whenever the stepper changes.
pub struct StepContent {
    stepper: Entity<Stepper>,
    control: FocusHandle,
    _observe_stepper: Subscription,
}

impl StepContent {
    pub fn new(stepper: Entity<Stepper>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&stepper, |_, _, cx| cx.notify());
        Self { stepper, control: cx.focus_handle().tab_stop(true), _observe_stepper: observe }
    }

    /// The focus handle of the step's own control, for containers that need explicit stops.
    pub fn control_focus(&self) -> FocusHandle {
        self.control.clone()
    }
}

impl Render for StepContent {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let index = self.stepper.read(cx).current_step_index().min(STEP_COPY.len() - 1);
        let (title, body, action) = STEP_COPY[index];
        div()
            .flex()
            .flex_col()
            .gap(px(theme.spacing.small))
            .pb(px(theme.spacing.medium))
            .mb(px(theme.spacing.medium))
            .border_b(px(theme.borders.regular))
            .border_color(theme.colors.border)
            .child(div().text_color(theme.colors.text).child(title))
            .child(
                div()
                    .text_size(px(theme.typography.caption))
                    .text_color(theme.colors.text_muted)
                    .child(body),
            )
            .child(focus_visible_with_theme(
                div()
                    .id("mkit-example-step-control")
                    .debug_selector(|| "mkit-example-step-control".to_owned())
                    .track_focus(&self.control)
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label(action)
                    .px(px(theme.spacing.medium))
                    .py(px(theme.spacing.small))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.surface)
                    .text_color(theme.colors.text)
                    .child(action),
                &theme,
            ))
    }
}
// ANCHOR_END: stepper_modal_step_content

enum Modal {
    Dialog(Entity<Dialog>),
    Sheet(Entity<Sheet>),
}

/// A window whose import flow runs a [`Stepper`] inside a [`Dialog`] or a [`Sheet`].
pub struct StepperModalScene {
    kind: ModalKind,
    opener: Option<FocusHandle>,
    stepper: Option<Entity<Stepper>>,
    content: Option<Entity<StepContent>>,
    modal: Option<Modal>,
    events: Vec<SceneEvent>,
    subscriptions: Vec<Subscription>,
}

impl StepperModalScene {
    /// The modal opens on the first render, as if the user had just pressed the opener.
    pub fn new(kind: ModalKind) -> Self {
        Self {
            kind,
            opener: None,
            stepper: None,
            content: None,
            modal: None,
            events: Vec::new(),
            subscriptions: Vec::new(),
        }
    }

    pub fn events(&self) -> &[SceneEvent] {
        &self.events
    }
    pub fn stepper(&self) -> Entity<Stepper> {
        self.stepper.clone().expect("scene rendered")
    }
    pub fn content(&self) -> Entity<StepContent> {
        self.content.clone().expect("scene rendered")
    }
    pub fn opener_focus(&self) -> FocusHandle {
        self.opener.clone().expect("scene rendered")
    }

    /// The modal container's own focus handle; focus inside the modal is contained by it.
    pub fn modal_focus(&self, cx: &App) -> FocusHandle {
        match self.modal.as_ref().expect("scene rendered") {
            Modal::Dialog(dialog) => dialog.read(cx).focus_handle(cx),
            Modal::Sheet(sheet) => sheet.read(cx).focus_handle(cx),
        }
    }

    pub fn is_modal_open(&self, cx: &App) -> bool {
        match self.modal.as_ref().expect("scene rendered") {
            Modal::Dialog(dialog) => dialog.read(cx).is_open(),
            Modal::Sheet(sheet) => sheet.read(cx).is_open(),
        }
    }

    /// Reopen or close the modal from the host, as the opener button does.
    pub fn set_modal_open(&mut self, open: bool, cx: &mut Context<Self>) {
        match self.modal.as_ref().expect("scene rendered") {
            Modal::Dialog(dialog) => dialog.update(cx, |dialog, cx| dialog.set_open(open, cx)),
            Modal::Sheet(sheet) => sheet.update(cx, |sheet, cx| sheet.set_open(open, cx)),
        }
        cx.notify();
    }

    // ANCHOR: stepper_modal_compose
    fn ensure_children(&mut self, cx: &mut Context<Self>) {
        if self.modal.is_some() {
            return;
        }
        let kind = self.kind;
        let opener = cx.focus_handle().tab_stop(true);
        // The host owns the stepper. Closing the modal never resets or drops it.
        let stepper = cx.new(|_| Stepper::new("Import photos", import_steps()));
        let content = cx.new(|cx| StepContent::new(stepper.clone(), cx));
        let body = {
            let (stepper, content) = (stepper.clone(), content.clone());
            move || div().flex().flex_col().child(content.clone()).child(stepper.clone())
        };
        let modal = match kind {
            // Dialog follows GPUI's rendered tab order inside its surface and wraps at the edge.
            ModalKind::Dialog => Modal::Dialog(
                cx.new(|_| Dialog::with_content("Import photos", body).return_focus_to(&opener)),
            ),
            // Sheet traps Tab among explicit stops fixed at construction. Back is rendered only
            // after the first step, so it cannot be one of them (see the Stepper spec).
            ModalKind::Sheet => {
                let stops =
                    vec![content.read(cx).control_focus(), stepper.read(cx).focus_handle(cx)];
                Modal::Sheet(cx.new(|_| {
                    Sheet::with_content("Import photos", body)
                        .focus_stops(stops)
                        .return_focus_to(&opener)
                }))
            }
        };
        self.subscriptions.push(cx.subscribe(
            &stepper,
            |scene, stepper, event: &StepChangeRequested, cx| {
                scene.events.push(SceneEvent::Step(event.clone()));
                if event.kind == StepChangeKind::Finish {
                    // Completing the flow is a host decision: close the modal and start over.
                    stepper.update(cx, |stepper, cx| stepper.set_current_step(0, cx));
                    scene.set_modal_open(false, cx);
                }
            },
        ));
        match &modal {
            Modal::Dialog(dialog) => self.subscriptions.push(cx.subscribe(
                dialog,
                |scene, _, event: &dialog::OpenChanged, cx| {
                    scene.events.push(SceneEvent::ModalOpenChanged(event.0));
                    cx.notify();
                },
            )),
            Modal::Sheet(sheet) => self.subscriptions.push(cx.subscribe(
                sheet,
                |scene, _, event: &sheet::OpenChanged, cx| {
                    scene.events.push(SceneEvent::ModalOpenChanged(event.0));
                    cx.notify();
                },
            )),
        }
        self.opener = Some(opener);
        self.stepper = Some(stepper);
        self.content = Some(content);
        self.modal = Some(modal);
    }
    // ANCHOR_END: stepper_modal_compose
}

impl Render for StepperModalScene {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_children(cx);
        let theme = *cx.global::<Theme>();
        let opener = self.opener_focus();
        let modal = match self.modal.as_ref().expect("children initialized") {
            Modal::Dialog(dialog) => dialog.clone().into_any_element(),
            Modal::Sheet(sheet) => sheet.clone().into_any_element(),
        };
        div()
            .size_full()
            .relative()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .items_start()
            .gap(px(theme.spacing.medium))
            .child(
                div()
                    .text_size(px(theme.typography.heading_small))
                    .text_color(theme.colors.text)
                    .child(match self.kind {
                        ModalKind::Dialog => "Stepper in a dialog",
                        ModalKind::Sheet => "Stepper in a sheet",
                    }),
            )
            .child(focus_visible_with_theme(
                div()
                    .id("mkit-example-modal-opener")
                    .debug_selector(|| "mkit-example-modal-opener".to_owned())
                    .track_focus(&opener)
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label("Import photos")
                    .on_click(cx.listener(|scene, _, _, cx| scene.set_modal_open(true, cx)))
                    .px(px(theme.spacing.medium))
                    .py(px(theme.spacing.small))
                    .rounded(px(theme.radii.medium))
                    .bg(theme.colors.accent)
                    .text_color(theme.colors.accent_text)
                    .child("Import photos…"),
                &theme,
            ))
            .child(modal)
    }
}
