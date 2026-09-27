# E8.5 Colour tools draft

The first implementation of E8.5 is in `registry/colour-tools`. It provides a hue/saturation disc, HSL lightness input, editable sRGB/HSL/OKLCH spinbuttons, shadows/midtones/highlights grading wheels, and an explicit eyedropper-unavailable status. The model uses sRGB D65 conversions. When an OKLCH value is outside sRGB, conversion independently clips the encoded red, green, and blue channels to [0, 1]; the model stores that clipped sRGB colour, and displayed OKLCH is derived from it. It does not chroma-map while preserving requested hue/lightness.

The component exposes an uncontrolled constructor and a controlled constructor. `ColourChanged` and `GradingChanged` are typed proposal events; owners can apply controlled values with `set_colour` and `set_grading`. Clicking a numeric field or focusing it and pressing Enter starts number entry. Digits, decimal point, minus, and Backspace edit the value; Enter finishes and Escape proposes restoration to the starting color. The grading wheels accept pointer drag and keyboard arrows after receiving focus.

## Checks run

- `cargo fmt -- registry/colour-tools/src/lib.rs` — passed.
- `cargo test -p mkit-registry-colour-tools --lib --offline --locked -j 2` — passed, 9 tests: colour conversion, wheel orientation, GPUI numeric and grading interactions, controlled owner echo/no-op paths, and out-of-sRGB clipping.
- Focused controlled tests now exercise typed numeric colour input and grading keyboard input, verify that proposals leave the displayed value unchanged until owner echo, and verify that equal reapplication emits no new event.
- The out-of-sRGB gamut test uses OKLCH L=0.6, C=0.5, H=40° and confirms current per-channel clipping produces sRGB red (255, 0, 0), with the read-back OKLCH reflecting that clipped result.
- `cargo check -p mkit-registry-colour-tools --all-targets --offline --locked -j 2` — passed.
- `cargo clippy -p mkit-registry-colour-tools --all-targets --offline --locked -j 2 -- -D warnings` — passed.
- `python3 scripts/generate_conformance.py registry/colour-tools/spec.md --output registry/colour-tools/tests/conformance.json --check` — passed with six keyboard, three accessibility, and 18 screenshot cases.
- Four light/dark 1×/2× colour-tool candidates were captured; the 1× images were inspected, and the hue orientation and grading-wheel fill were corrected before the images were accepted as initial-surface baselines.
- `cargo test -p mkit-gallery --test e8_colour_tools_matrix -- --nocapture` — all 18 selected,
  eyedropper-unavailable, and grading state captures matched at light, dark, high contrast, 1×,
  and 2×. Representative candidates were visually inspected before baselines were accepted.

## Checks still needed

- Compare the shared E8 preview images again after the remaining pro examples settle; the
  dedicated ColourTools state/theme/scale matrix passes.
- Capture native accessibility snapshots on an active platform. Headless GPUI accessibility is not evidence of the announced role and value behavior.
- Verify keyboard traversal through all nine numeric fields and grading controls in the E8 host app; the focused test covers numeric editing and a grading wheel's pointer and keyboard behavior.
- Eyedropper sampling has no backend. The UI reports it as unavailable; OS permission, cancel, and sampling flows remain open.
- Maintainer review is needed for type names and public API, component keyboard and accessibility contracts, color-space and gamut policy, and visual design.
