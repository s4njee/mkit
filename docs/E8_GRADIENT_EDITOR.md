# E8.6 Gradient editor

`mkit-registry-gradient-editor` edits a normalized, one-dimensional sRGB gradient. Its public
`Gradient` and `GradientStop` values can be stored or translated by the consuming application; the
editor does not own persistence or rendering outside its live preview.

The component keeps endpoint stops at positions 0 and 1, limits the model to 16 stops, and prevents
stops from crossing. Add inserts a stop into the widest gap and samples the existing gradient there.
The preview is composed from sampled color segments. Interior stops can be selected, moved by
dragging or keyboard, and removed. Selected-stop controls edit its normalized position and RGB byte
channels.

Use `GradientEditor::new` for an uncontrolled editor or `GradientEditor::controlled` when the owner
must approve edits. In controlled mode, subscribe to `GradientChanged`, apply accepted values with
`set_gradient`, and observe selection through `StopSelected`. Registry dependencies remain limited
to GPUI and `mkit-core`; the component has a local sRGB color value to preserve that boundary.

The keyboard and accessibility contracts, constraints, states, and open questions are recorded in
[`registry/gradient-editor/spec.md`](../registry/gradient-editor/spec.md). A dedicated 30-case
macOS matrix now covers five declared states across light, dark, and high contrast at 1×/2×.
The first initial-surface candidates exposed unreadable stop labels and
cramped controls; the stop list and grouped steppers were revised, then the 1× and dark 2× images
were inspected before accepting baselines. The five-stop high-contrast matrix candidate exposed
a cropped lower border; its fixture height was increased and the candidate was re-inspected before
the full 30-case comparison passed. Six focused package tests pass, including real GPUI checks for
controlled Tab navigation, owner echo and equal-value silence, plus disabled keyboard and pointer
selection suppression. Public API, naming, keyboard, accessibility behavior, and visual baselines
need maintainer review. Native accessibility
snapshots remain open.
