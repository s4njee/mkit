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

## Composition in Dialog and Sheet

`examples/e7_compositions` hosts the Stepper in a Dialog and in a Sheet without adding a
registry-to-registry dependency. `cargo test -p mkit-example-e7-compositions --test
stepper_in_modal` passes five GPUI tests. They check focus after every Tab and Shift+Tab press across
the container surface, step content, Back, and Next. They also cover Escape from stepper buttons and
step content, focus return to the opener, the step kept after dismissal and reopen, Back/Next/Alt
shortcuts inside both containers, and a host-initiated close and reset after Finish. The
`screenshots` target matched 12/12 new captures of the middle step (Dialog and Sheet × light, dark,
and high-contrast × 1× and 2×). The dark 2× captures are the book images.

This work fixed three Stepper bugs within the existing contract. Back and Next were not Tab stops
because their focus handles never set `tab_stop`, so Tab skipped them. `Focusable::focus_handle`
panicked before the first render. Alt+Left and Alt+Right worked only on the button they would
activate anyway. `cargo test -p mkit-registry-stepper` adds a regression test for all three.

Container gaps are recorded, not redesigned. Dialog Shift+Tab from its surface stays on the surface
instead of wrapping to the last control. Sheet has no rendered-order fallback and cannot update focus
stops after construction, so the conditional Back button is not a Sheet Tab stop. Back remains
reachable with Alt+Left and the pointer, and Tab from Back re-enters the stop list. Stepper buttons
also have no visible focus indicator yet. Adding one would change the accepted Stepper baselines.

Native screen-reader announcement timing inside real Dialog/Sheet focus traps requires platform
validation. The current API does not support async validation or direct jumps among
completed steps; those decisions remain open for maintainer review.

The component API and its synchronous validator are a draft for maintainer review. See
`registry/stepper/spec.md` and `registry/stepper/tests/conformance.json` for the complete contract.
