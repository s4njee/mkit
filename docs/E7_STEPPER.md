# E7.23 Stepper evidence and review

Stepper presents ordered progress and Back/Next/Finish controls. It does not own a form, page
content, persistence, or dialog/sheet lifecycle. Hosts place the active step content alongside it
and own modal focus containment and dismissal.

## Public API draft

`Stepper::new(label, Vec<Step>)` starts uncontrolled at the first step. `default_step(index)` seeds
uncontrolled state; `current_step(index)` selects controlled state; `set_current_step` lets a host
apply a new controlled value. `validate_with` installs a synchronous callback by stable step id.
Successful forward navigation emits `StepChangeRequested { from, to, kind }`; uncontrolled mode
updates before emitting, controlled mode waits for the parent. Validation failure emits
`StepValidationFailed` and leaves the current step active. Back skips forward validation. Enter and
Space activate whichever named navigation button owns focus. Alt+Left and Alt+Right are registered
Stepper-context shortcuts and can be rebound by the host.

## Evidence and limitations

`cargo test -p mkit-registry-stepper --offline -j 2` passes two focused GPUI tests for uncontrolled
and controlled activation plus synchronous validation failure. The gallery fixture captured and
matched 24/24 screenshots: first, middle, final, and validation-error states × light, dark, and
high-contrast themes × 1× and 2×. `cargo clippy -p mkit-registry-stepper --offline -j 2 -- -D
warnings` passes. Candidate images were inspected in a 1× theme/state montage; the
Back control contrast was corrected before the accepted capture. The book preview uses the exact
bytes of the accepted middle/dark/2× baseline.

Native screen-reader announcement timing and placement inside real Dialog/Sheet focus traps require
platform validation. The current API does not support async validation or direct jumps among
completed steps; those decisions remain open for maintainer review.

The component API and its synchronous validator are a draft for maintainer review. See
`registry/stepper/spec.md` and `registry/stepper/tests/conformance.json` for the complete contract.
