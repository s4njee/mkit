# Laika seeds

Laika is the first production-style consumer of mkit components. Its slider, histogram, and segmented-control surfaces adopt the registry versions while Laika keeps its edit semantics, palette, and layout. This is an E8.14 draft: the histogram mount and the theme bridge are wired, and the slider and segmented-control migration is blocked on adapter work described below.

## Migration contract

- Histogram: Laika's adapter returns the mkit `Histogram`, preserving the 48-bin luminance input and the 74/78 px host heights in the Develop and Library views. Values and the accessibility summary are supplied by the host on each render; the component itself is stateless.
- Slider: controlled mode with the current effective value, mapping change requests into Laika's existing edit transactions (`Laika::set_value` and `commit_edit`). The host must keep the component entity stable across renders and update controlled values after presets, resets, undo, and multi-photo changes.
- Segmented control: controlled mode with the selected Laika view key, mapping `ValueChanged(Option<String>)` to the existing view actions, kept synchronized after keyboard shortcuts and external view changes.
- Theme: Laika maps its background, panel, text, border, accent, and disabled tokens to the mkit `Theme` Global at startup and whenever appearance or accent preferences change, so embedded controls follow Laika's palette instead of installing a competing theme.

## Keyboard, accessibility, and theme

Registry slider keyboard behavior and slider roles are owned by mkit, as are the segmented control's radio-group keyboard behavior, group label, and item selection. The histogram is an image with an accessible name and textual summary. Host shortcuts outside these controls remain Laika-owned. All three controls read the GPUI Global theme.

## Current limits

The slider and segmented-control usages are not yet replaced. Their interactions live in the large `Laika` parent view: slider gestures feed undoable parameter transactions, text fields, presets, reset behavior, and multi-selection, while segmented controls dispatch Laika-specific actions kept in sync with keyboard shortcuts. Recreating the stateful entity views inside `&self` render helpers would reset focus and selection on rerender; wiring them correctly needs persistent child entities and subscriptions in `Laika` plus routing their events through the existing transactions. The old slider also has per-parameter formatting and scrub gestures not currently represented by the mkit Slider API. No behavior was removed to force an incomplete swap.

Verified so far in an isolated Laika worktree with aligned `gpui-kit 0.6.4` and `gpui-pre 0.3.5` pins and a local mkit path dependency: `cargo check` and `cargo build` pass, and `cargo test` passes the 36 existing Laika unit tests. No interactive GPUI harness, keyboard script, accessibility snapshot, or screenshot matrix was run in Laika; the mkit histogram and theme bridge are compile-checked in the consuming application only. Open questions include a relocatable dependency for local adoption before mkit is published, a displayed-value formatter, segmented tooltips, and an adapter API for stable child entities. Maintainer review of the migration and visual behavior is outstanding.

The full contract, spike result, and remaining migration blockers are recorded in `docs/E8_LAIKA_ADOPTION.md` in the repository.
