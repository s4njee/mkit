# E7.22 StatusBar evidence and review notes

`StatusBar` is a stateless `RenderOnce` builder. Hosts own status/progress values and button actions;
there are no component events or controlled/uncontrolled modes. Status text uses polite live-region
semantics and text updates do not request focus.

## Narrow layout contract

`available_width` is an explicit host-provided logical width because the current RenderOnce API does
not expose a reliable child measurement callback. Items may supply `estimated_width`; text otherwise
uses a typography-based estimate and arbitrary elements/progress use token-based defaults. The bar
collapses `Low`, then `Normal`, then `High` priority items and never collapses `Never` items. Equal
priority items retain their leading/trailing order. If `Never` items alone exceed the supplied width,
the bar clips; the host must allocate more width or mark some content collapsible. Collapsed items
are removed from layout and the accessibility tree, with no overflow menu in this draft.

## Evidence

- `cargo test -p mkit-registry-status-bar`: 3 passed. Tests cover collapse priority, no-width behavior,
  supplied-button Enter/Space activation, and focus retention while status text changes.
- `cargo test -p mkit-gallery --test e7_status_bar_matrix -- --nocapture`: 30 screenshot cases
  compare five states × light/dark/high-contrast × 1×/2×. Candidate images were inspected before
  accepting the baselines; compact/narrow captures show priority collapse while retaining essential
  status text and the high-priority Export action.
- `python3 scripts/check_component_specs.py`: passed with the StatusBar spec and generated manifest.
- The StatusBar book image was directly verified byte-for-byte against the dark 2× full-state
  baseline, and the page's relative links pass. The earlier `book/src/images/e7/search-field.png`
  provenance mismatch that blocked the full book gate is resolved: on 2026-09-27
  `python3 scripts/check_book.py --skip-cargo` passed (168 pages, 15 example crates).
- Native accessibility snapshots and spoken polite-announcement timing remain pending. Public API,
  `available_width` estimation, and omission-versus-overflow behavior need maintainer review.
