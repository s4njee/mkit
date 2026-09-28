//! Stepper hosted inside Dialog and Sheet: focus containment, Escape, and Back/Next navigation.

use gpui_pre::{Entity, Focusable, TestAppContext, VisualTestContext};
use mkit::stepper::{StepChangeKind, StepChangeRequested};
use mkit_example_e7_compositions::{ModalKind, SceneEvent, StepperModalScene, bind_keys};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Opener,
    ModalRoot,
    StepControl,
    StepperNext,
    /// Focus inside the modal on a stop the scene does not own: the stepper's Back button.
    StepperBack,
    Outside,
}

fn open(
    cx: &mut TestAppContext,
    kind: ModalKind,
) -> (Entity<StepperModalScene>, &mut VisualTestContext) {
    cx.update(mkit::core::theme::set_light_theme);
    cx.update(bind_keys);
    let (scene, visual) = cx.add_window_view(|_, _| StepperModalScene::new(kind));
    // The first frame opens the modal and moves focus into it; the second records its tab stops.
    visual.update(|window, cx| window.draw(cx).clear(cx));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    (scene, visual)
}

fn focused(scene: &Entity<StepperModalScene>, visual: &mut VisualTestContext) -> Target {
    visual.update(|window, cx| {
        let scene = scene.read(cx);
        let modal = scene.modal_focus(cx);
        let control = scene.content().read(cx).control_focus();
        let next = scene.stepper().read(cx).focus_handle(cx);
        if scene.opener_focus().is_focused(window) {
            Target::Opener
        } else if modal.is_focused(window) {
            Target::ModalRoot
        } else if control.is_focused(window) {
            Target::StepControl
        } else if next.is_focused(window) {
            Target::StepperNext
        } else if modal.contains_focused(window, cx) {
            Target::StepperBack
        } else {
            Target::Outside
        }
    })
}

/// Press `key` once per expected target, checking focus after each press.
fn walk(
    scene: &Entity<StepperModalScene>,
    visual: &mut VisualTestContext,
    key: &str,
    expected: &[Target],
) {
    for (press, target) in expected.iter().enumerate() {
        visual.simulate_keystrokes(key);
        assert_eq!(focused(scene, visual), *target, "{key} press {}", press + 1);
    }
}

fn step(scene: &Entity<StepperModalScene>, visual: &mut VisualTestContext) -> usize {
    scene.read_with(visual, |scene, cx| scene.stepper().read(cx).current_step_index())
}

fn events(scene: &Entity<StepperModalScene>, visual: &mut VisualTestContext) -> Vec<SceneEvent> {
    scene.read_with(visual, |scene, _| scene.events().to_vec())
}

fn change(from: usize, to: usize, kind: StepChangeKind) -> SceneEvent {
    SceneEvent::Step(StepChangeRequested { from, to, kind })
}

#[gpui_pre::test]
fn dialog_contains_tab_through_step_content_and_stepper_navigation(cx: &mut TestAppContext) {
    use Target::*;
    let (scene, visual) = open(cx, ModalKind::Dialog);
    assert_eq!(focused(&scene, visual), ModalRoot, "Dialog focuses its surface on open");

    // First step: Back is absent, so Tab cycles surface, step control, and Next.
    walk(&scene, visual, "tab", &[StepControl, StepperNext, ModalRoot, StepControl]);
    // Shift+Tab reaches the surface; the Dialog contract wraps outside stops to the surface.
    walk(&scene, visual, "shift-tab", &[ModalRoot, ModalRoot]);

    walk(&scene, visual, "tab", &[StepControl, StepperNext]);
    visual.simulate_keystrokes("enter");
    assert_eq!(step(&scene, visual), 1);
    assert_eq!(focused(&scene, visual), StepperNext, "Next keeps focus after advancing");

    // Middle step: Back joins the rendered tab order between the step content and Next.
    walk(&scene, visual, "tab", &[ModalRoot, StepControl, StepperBack, StepperNext, ModalRoot]);
    walk(&scene, visual, "shift-tab", &[ModalRoot]);
    walk(&scene, visual, "tab", &[StepControl, StepperBack, StepperNext]);
    walk(&scene, visual, "shift-tab", &[StepperBack, StepControl, ModalRoot]);

    // Back works from inside the dialog and hands focus to Next when Back disappears.
    walk(&scene, visual, "tab", &[StepControl, StepperBack]);
    visual.simulate_keystrokes("enter");
    assert_eq!(step(&scene, visual), 0);
    assert_eq!(focused(&scene, visual), StepperNext);

    // Stepper shortcuts work from either navigation button while the dialog is open.
    visual.simulate_keystrokes("alt-right alt-right");
    assert_eq!(step(&scene, visual), 2);
    visual.simulate_keystrokes("alt-left");
    assert_eq!(step(&scene, visual), 1);
    assert_eq!(focused(&scene, visual), StepperBack);

    assert_eq!(
        events(&scene, visual),
        vec![
            change(0, 1, StepChangeKind::Next),
            change(1, 0, StepChangeKind::Back),
            change(0, 1, StepChangeKind::Next),
            change(1, 2, StepChangeKind::Next),
            change(2, 1, StepChangeKind::Back),
        ]
    );
    assert!(scene.read_with(visual, |scene, cx| scene.is_modal_open(cx)));
}

#[gpui_pre::test]
fn escape_from_stepper_dismisses_dialog_and_host_keeps_step(cx: &mut TestAppContext) {
    use Target::*;
    let (scene, visual) = open(cx, ModalKind::Dialog);
    walk(&scene, visual, "tab", &[StepControl, StepperNext]);
    visual.simulate_keystrokes("enter");
    assert_eq!(step(&scene, visual), 1);

    // Escape on a Stepper button reaches the Dialog: the Stepper binds no Escape action.
    visual.simulate_keystrokes("escape");
    assert!(!scene.read_with(visual, |scene, cx| scene.is_modal_open(cx)));
    assert_eq!(focused(&scene, visual), Opener, "Dialog restores focus to the opener");
    assert_eq!(step(&scene, visual), 1, "dismissal leaves the host-owned step unchanged");
    assert_eq!(
        events(&scene, visual),
        vec![change(0, 1, StepChangeKind::Next), SceneEvent::ModalOpenChanged(false)]
    );

    // Reopening resumes where the user left off; Escape from step content also dismisses.
    scene.update(visual, |scene, cx| scene.set_modal_open(true, cx));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    assert_eq!(focused(&scene, visual), ModalRoot);
    assert_eq!(step(&scene, visual), 1);
    walk(&scene, visual, "tab", &[StepControl]);
    visual.simulate_keystrokes("escape");
    assert!(!scene.read_with(visual, |scene, cx| scene.is_modal_open(cx)));
    assert_eq!(focused(&scene, visual), Opener);
}

#[gpui_pre::test]
fn finish_inside_dialog_lets_host_close_and_reset(cx: &mut TestAppContext) {
    use Target::*;
    let (scene, visual) = open(cx, ModalKind::Dialog);
    walk(&scene, visual, "tab", &[StepControl, StepperNext]);
    visual.simulate_keystrokes("enter enter");
    assert_eq!(step(&scene, visual), 2);
    visual.simulate_keystrokes("enter");
    visual.update(|window, cx| window.draw(cx).clear(cx));
    assert!(!scene.read_with(visual, |scene, cx| scene.is_modal_open(cx)));
    assert_eq!(step(&scene, visual), 0, "this host restarts the flow after Finish");
    assert_eq!(focused(&scene, visual), Opener);
    assert_eq!(
        events(&scene, visual).last(),
        Some(&change(2, 2, StepChangeKind::Finish)),
        "a host-initiated close emits no OpenChanged dismissal request"
    );
}

#[gpui_pre::test]
fn sheet_contains_tab_across_explicit_stops_and_stepper_back(cx: &mut TestAppContext) {
    use Target::*;
    let (scene, visual) = open(cx, ModalKind::Sheet);
    assert_eq!(focused(&scene, visual), StepControl, "Sheet focuses its first explicit stop");

    walk(&scene, visual, "tab", &[StepperNext, StepControl, StepperNext]);
    walk(&scene, visual, "shift-tab", &[StepControl, StepperNext]);

    visual.simulate_keystrokes("enter enter");
    assert_eq!(step(&scene, visual), 2);
    assert_eq!(focused(&scene, visual), StepperNext);

    // Alt+Left moves Back and focuses the Back button, which is not one of the Sheet's stops.
    // Tab and Shift+Tab from Back re-enter the stop list instead of leaving the sheet.
    visual.simulate_keystrokes("alt-left");
    assert_eq!(step(&scene, visual), 1);
    assert_eq!(focused(&scene, visual), StepperBack);
    walk(&scene, visual, "tab", &[StepControl]);
    walk(&scene, visual, "shift-tab", &[StepperNext]);
    visual.simulate_keystrokes("enter");
    assert_eq!(step(&scene, visual), 2);
    assert_eq!(focused(&scene, visual), StepperNext);
    visual.simulate_keystrokes("alt-left");
    assert_eq!(step(&scene, visual), 1);
    assert_eq!(focused(&scene, visual), StepperBack);
    walk(&scene, visual, "shift-tab", &[StepperNext]);

    // Enter on Back works inside the sheet too.
    visual.simulate_keystrokes("enter alt-left");
    assert_eq!(step(&scene, visual), 1);
    assert_eq!(focused(&scene, visual), StepperBack);
    visual.simulate_keystrokes("enter");
    assert_eq!(step(&scene, visual), 0);
    assert_eq!(focused(&scene, visual), StepperNext);
    assert!(scene.read_with(visual, |scene, cx| scene.is_modal_open(cx)));
}

#[gpui_pre::test]
fn escape_from_stepper_dismisses_sheet_and_host_keeps_step(cx: &mut TestAppContext) {
    use Target::*;
    let (scene, visual) = open(cx, ModalKind::Sheet);
    walk(&scene, visual, "tab", &[StepperNext]);
    visual.simulate_keystrokes("enter");
    assert_eq!(step(&scene, visual), 1);
    visual.simulate_keystrokes("escape");
    assert!(!scene.read_with(visual, |scene, cx| scene.is_modal_open(cx)));
    assert_eq!(focused(&scene, visual), Opener, "Sheet restores focus to the opener");
    assert_eq!(step(&scene, visual), 1, "dismissal leaves the host-owned step unchanged");
    assert_eq!(
        events(&scene, visual),
        vec![change(0, 1, StepChangeKind::Next), SceneEvent::ModalOpenChanged(false)]
    );

    scene.update(visual, |scene, cx| scene.set_modal_open(true, cx));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    assert_eq!(focused(&scene, visual), StepControl);
    assert_eq!(step(&scene, visual), 1);
}
