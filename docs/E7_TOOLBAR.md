# E7.13 Toolbar evidence and review notes

The component spec and generated conformance manifest define the APG toolbar contract, typed
activation/value events, roving focus, overflow menu behavior, orientation, and accessibility role.
The registry implementation is a reviewable draft. Public API, toggle-group focus semantics,
overflow behavior, toolbar orientation semantics, and native accessibility mapping need maintainer
review before stabilization.

## Implementation boundary

GPUI 0.3.5 does not currently give this component a stable child-measurement callback for deciding
pixel fit. `visible_capacity` therefore specifies how many overflow-eligible controls remain inline.
Never-overflow items always stay visible; toggle groups move atomically if the complete group does
not fit; disabled items moved to overflow remain listed but are skipped by keyboard navigation.
Automatic pixel-fit overflow is pending a GPUI measurement integration point.

## Evidence

- `cargo test -p mkit-registry-toolbar`: 5 passed, covering horizontal navigation, disabled
  skipping, pointer invocation, controlled state, toggle-group choice navigation, capacity-zero
  focus, atomic group overflow, and disabled overflow retention.
- `python3 scripts/check_component_specs.py`: passed for all 52 component specs/manifests.
- `cargo test -p mkit-gallery --test e7_toolbar_matrix -- --nocapture`: all 24 cases matched:
  four states × light/dark/high-contrast × 1×/2×. Inspected the dark 2× montage; the overflow menu
  is positioned below the More actions trigger and does not expand the toolbar row.
- The book preview is an exact byte copy of the inspected `overflow/dark-2x` registry baseline;
  `scripts/check_book.py` verifies this provenance.
- Native accessibility snapshots are pending. The spec records them as `unsupported_pending`; no
  platform AX result is claimed here.
