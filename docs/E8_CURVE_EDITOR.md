# E8.4 curve editor implementation notes

## Implemented

- Added a stateful normalized curve editor with named independent channels, sorted bounded points,
  endpoint anchors, a fixed point cap, channel selection, and linear or monotone cubic interpolation.
- Added GPUI drawing for the subtle graph grid, diagonal reference, sampled curve, control points,
  selected point ring, channel selectors, and interpolation controls. Sizing and visual colors use
  Global Theme tokens; graph height derives from six large control-height tokens.
- Added typed `CurveChanged`, `ChannelChanged`, and `InterpolationChanged` events, with controlled
  point-edit requests and owner `set_channel_points` support.
- Added pointer add, drag, and right-click delete; keyboard regular/fine nudges, Delete, channel
  cycling, and interpolation mode actions; endpoint anchors stay protected.
- Added math tests for normalization and bounded interpolation, plus a GPUI interaction test for
  keyboard nudge/delete, pointer add/move beyond graph bounds, right-click delete, and channel
  selection.
- Added a real-GPUI controlled-mode test for keyboard nudge and Delete. It checks normalized typed
  proposals, unchanged displayed points until owner acceptance, and no user-change event on an
  equal owner echo.
- Added a real-GPUI keyboard test for channel and interpolation actions. Typed event payloads
  track state changes, while repeated mode selection emits no extra event.

## Known gaps and review

- The GPUI test confirms pointer movement beyond graph bounds updates the point and clamps it within
  its neighbors. It does not verify native platform pointer capture behavior outside the test window.
- Point marker labels and roles, group descriptions, and selected visual state are implemented;
  native screen-reader output, marker focus navigation, and generated accessibility snapshots remain
  unverified.
- The E8 example has four initial-surface baselines: light/dark at 1×/2×. A separate 30-case
  screenshot matrix covers linear, smooth, selected-point, multiple-channel, and disabled states
  across light/dark/high-contrast at 1×/2×. Representative captures were inspected, including the
  selected marker and disabled treatment. Public API naming,
  event payloads, controlled semantics, the 64-point cap, and normalized monotone smoothing need
  maintainer review.

## Checks

- `python3 scripts/generate_conformance.py registry/curve-editor/spec.md --output registry/curve-editor/tests/conformance.json` — passed (4 keyboard, 5 accessibility, 30 screenshot cases).
- `python3 scripts/check_component_specs.py` — passed (38 registry specs and manifests).
- `rustfmt --edition 2024 registry/curve-editor/src/lib.rs` — passed.
- `cargo test -p mkit-registry-curve-editor --offline --locked` — passed (5 tests, including three
  GPUI interaction tests).
- `cargo clippy -p mkit-registry-curve-editor --all-targets --locked -- -D warnings` — passed.
- `cargo test -p mkit-gallery --test e8_curve_editor_matrix -- --nocapture` — 30 cases passed.
