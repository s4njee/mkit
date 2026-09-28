---
spec_version: 1
component: stepper
states:
  - id: first
    description: The first step is current and only Next is available.
    fixture: first_fixture
  - id: middle
    description: An intermediate step is current with Back and Next available.
    fixture: middle_fixture
  - id: final
    description: The last step is current and offers completion instead of Next.
    fixture: final_fixture
  - id: validation-error
    description: Synchronous validation blocks advancement and describes the error.
    fixture: validation_error_fixture
keys:
  - key: Enter
    modifiers: []
    when: The enabled Next, Back, or Finish button is focused.
    action: Activate the focused step navigation action.
    initial_state: middle
    expect: { event: step_change_requested, focus_target: same_button }
  - key: Space
    modifiers: []
    when: The enabled Next, Back, or Finish button is focused.
    action: Activate the focused step navigation action.
    initial_state: middle
    expect: { event: step_change_requested, focus_target: same_button }
accessibility:
  role: group
  properties:
    - name: name
      value: caller-provided flow label
    - name: current
      value: current step label
    - name: position
      value: current step index and total count
    - name: validation_error
      value: polite error message
      when: validation-error
---

# Stepper

## Purpose

Guide users through a sequence of related steps, showing progress and providing Back, Next, and Finish actions. Stepper owns no form data; apps supply steps, validation, and page content.

## Anatomy

A labeled progress header shows the ordered step labels, completed/current/upcoming state, and current position. The host renders the active step's content. Navigation contains Back when a prior step exists, Next for non-final steps, and Finish on the final step. Validation feedback appears beside navigation and is announced politely.

## States

`first` disables or omits Back. `middle` exposes Back and Next. `final` exposes Back and Finish. `validation-error` keeps the current step active and exposes a concise validation message. The host places the stepper and active content inside a window, dialog, or sheet.

## Props and events

`Stepper::new(label, steps)` accepts an ordered list of stable step ids and labels. `.default_step(index)` selects uncontrolled mode and internal navigation; `.current_step(index)` selects controlled mode. `StepChangeRequested { from, to, kind }` is emitted for user navigation. In uncontrolled mode a valid request updates the current index before emitting; in controlled mode the current index remains parent-owned until the parent supplies a new index. A synchronous `.validate_with` callback receives the current step id before Next/Finish. Success advances/emits a request; failure emits `StepValidationFailed { step_id, message }`, keeps the current index, and renders the message. Back never runs forward validation. The component does not own step content, persistence, or async validation.

## Keyboard map

Tab order follows the active step's host content, then enabled Back and Next/Finish controls according to host layout. Both enabled navigation buttons are Tab stops in rendered order (Back before Next/Finish) at the default tab index, so they join the surrounding window or container order rather than jumping ahead of or behind it. Navigation buttons use ordinary button behavior: Enter and Space activate them. The step progress header is informational, not a tablist; arrow keys do not change steps. The component registers a `Stepper` key context with rebindable Next, Back, and Finish actions; Alt+Left (Back) and Alt+Right (Next, or Finish on the last step) work while either navigation button has focus. The Stepper binds no Escape, Tab, or Shift+Tab action, so those keys reach the host or container. Disabled actions are absent from tab order.

## Pointer behaviour

Back/Next/Finish buttons invoke the same typed navigation path as keyboard actions. The progress header is not clickable, avoiding unvalidated direct jumps. A failed validator leaves focus on the same action and renders the error. Dialog/sheet dismissal remains owned by the container and host.

## Accessibility role and properties

The component exposes a named Group. The header provides ordered step labels and completed/current state text; current step also exposes current/position semantics when supported. Navigation controls are named buttons and expose disabled state. Validation failure uses a polite Status region, preserves focus, and associates the message with the active step. Native assistive-technology output and exact step-position property support need platform validation.

## Theme tokens used

Use `Theme.colors.text`, `text_muted`, `surface`, `border`, `accent`, `success`, `danger`, and `disabled`; spacing, radii, borders, typography, and control-size tokens. Completed/current/error states use text or shape cues in addition to color.

## WAI-ARIA pattern reference

There is no dedicated APG stepper pattern. Use labeled group, ordered-list progress information, ordinary buttons, and a polite status message. Do not expose the header as a tablist because steps are not direct navigation tabs.

## Platform notes

Keyboard activation and AccessKit roles/properties are provided by GPUI. The validation callback is synchronous in this draft; async or server-backed validation must be coordinated by the host before updating the controlled current step. The same component may be placed inside existing Dialog and Sheet containers; focus trapping and dismissal remain those containers' responsibilities. `Focusable::focus_handle` returns the Next/Finish handle and is valid before the first render, so hosts can pass it to a container's explicit focus stops at construction.

## Composition in Dialog and Sheet

The host owns the composition: it creates the `Entity<Stepper>` and the active step's content, passes both to `Dialog::with_content` or `Sheet::with_content`, and decides what closing and finishing mean. Registry crates do not depend on each other; the composition lives in `examples/e7_compositions`.

- Focus containment belongs to the container. Inside Dialog (no explicit stops), Tab and Shift+Tab follow rendered order across the surface, step content, Back, and Next/Finish, and wrap to the surface at either edge. Sheet traps Tab among explicit stops fixed at construction; hosts pass the step content handles and `stepper.focus_handle(cx)`. Back appears only after the first step, so it cannot be a fixed Sheet stop; it stays reachable with Alt+Left and pointer, and Tab or Shift+Tab from Back re-enters the stop list.
- Escape is not bound by Stepper, so it reaches the container's `Dismiss` action from any stepper button or step content. Dismissal emits the container's `OpenChanged(false)` and no `StepChangeRequested`.
- Stepper state is host-owned. Dismissal does not reset, validate, or drop the stepper; reopening shows the same step unless the host calls `set_current_step` (or supplies a new controlled index). Finish emits `StepChangeRequested { kind: Finish }`; closing the container after Finish is a host decision.
- Back/Next/Finish, Enter, Space, and Alt+Left/Right behave the same inside either container, and focus moves to the surviving navigation button when Back disappears.

Evidence: `cargo test -p mkit-example-e7-compositions --test stepper_in_modal` checks focus after every Tab/Shift+Tab press, Escape from stepper buttons and step content, focus restoration, preserved step after reopen, and host-initiated close after Finish, in both containers. Container gaps recorded for maintainer review: Dialog Shift+Tab from its surface stays on the surface instead of wrapping to the last control, and Sheet has neither a rendered-order fallback nor a way to update focus stops after construction.

## Open questions

- Maintainers should review whether synchronous validation belongs in the component API or should remain exclusively host-owned.
- Confirm the public Step/status representation and whether direct jumps among completed steps are needed.
- Verify current-step/position semantics and polite error timing with native screen readers.
- Decide whether Sheet should gain Dialog's rendered-order Tab fallback or updatable focus stops so a conditionally rendered Back can be a Sheet Tab stop, and whether Dialog Shift+Tab from its surface should wrap to the last control.
