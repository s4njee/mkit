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

The docs-site web preview (`site/src/demos/e7_expansion.ts`, `.e7-stepper*` in
`site/src/demos/e7_expansion.css`) lists the steps with muted labels and state captions above Back
and Next buttons. GPUI keeps that structure and adds shadcn-style step indicators: a numbered circle
per step, filled with the primary colour when current or complete, a vector check once complete,
and connector lines between steps. The navigation buttons reproduce Button's outline (Back) and
default (Next/Finish) variants; the component depends only on mkit-core, so it does not use the
Button crate. Colours are resolved from the installed `Theme` in three variants. `high-contrast`
is selected by theme name (the convention other registry components use); every other theme is
treated as dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise
light. Derived colours use a crate-local `color-mix` helper built on
`mkit_core::contrast::composite`; no mkit-core API or tokens are added. "Outline" is `border` in
light and shadcn's dark `input`, `text` at 15% over `background` composited opaque, in dark (the 10%
dark `border` is too faint for an empty indicator ring). The Back button keeps Button's 10% outline.

| Part | Light / dark | High contrast |
|---|---|---|
| Complete indicator | `accent` fill and border, `accent_text` Lucide check | `accent`, `accent_text` |
| Current indicator | `accent` fill and border, `accent_text` number | `accent`, `accent_text` |
| Current indicator after failed validation | `danger` fill, the theme's near-white number (Button's destructive text) | `danger`, `accent_text` |
| Upcoming indicator | `background` fill, outline border, `text_muted` number | `background`, `border`, `text` |
| Connector after a complete step | `accent` | `accent` |
| Other connectors | outline | `border` |
| Step label | `text` (current at medium weight); upcoming `text_muted` | `text`; upcoming `text_muted` |
| State caption | `text_muted` | `text_muted` |
| Back button | Button outline: `background`, `text`, outline border, `shadows.small`; hover muted fill | `background`, `text`, `border`; hover border `accent` |
| Next / Finish button | Button default: `accent`, `accent_text`, `shadows.small`; hover `accent` 90% over `background` | `accent`, `accent_text`, `accent`; hover border `text` |
| Button focus (`focus_visible`) | border `focus` plus a 3px ring of `focus` at 50% | border `focus` plus a 3px ring of opaque `focus` |
| Validation message | `danger` | `danger` |

"Muted" is shadcn's `accent`: `text` mixed 4% (light) or 12% (dark) into `background`.

- **Indicators.** `spacing.xlarge` (24px, shadcn `size-6`) circles (`radii.pill`) with a
  `borders.regular` border and a `typography.caption` number at medium weight. Complete steps draw a
  `spacing.large` Lucide `check` (20,6 → 9,17 → 4,12) as a vector path instead of the number,
  stroked at 2/24 of its size (`borders.regular` in high contrast); it is decorative. After failed
  validation the current indicator turns `danger` and its caption reads "Needs attention", so the
  error is not conveyed by colour alone.
- **Connectors.** A `borders.strong` wide, `spacing.medium` tall rounded rule centred under each
  indicator, with `spacing.xsmall` above and below.
- **Labels.** Each row places the indicator, the step label (`typography.body`) and its state
  caption ("Complete", "Current", "Upcoming"; `typography.caption`) in a line separated by
  `spacing.medium` and `spacing.small`. The number moves into the indicator, so labels no longer
  repeat it. The preview colours complete labels with `success`; GPUI uses the check and the primary
  fill instead and keeps labels neutral, as shadcn steppers do.
- **Buttons.** Button's default size: `controls.medium` (36px) tall, `spacing.large` horizontal
  padding, radius `radii.medium`, a `borders.regular` border and `typography.body` at medium weight
  (500; there is no font-weight token yet). Back sits at the start and Next/Finish at the end of the
  row. GPUI paints drop shadows as filled shapes, so both buttons have opaque fills under the ring;
  the 3px ring is the shadcn/ui ring width, a fixed component value.
- **Focus state.** The `validation-error` screenshot presses Enter on the focused Next button, so
  it also records the keyboard focus ring.

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
