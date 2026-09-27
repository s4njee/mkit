# E8.12 Command palette

The command palette lets an application search and invoke actions registered by its host. It uses the shadcn Command surface as a visual reference: one compact search field, a quiet border, a clear active row, and muted keybinding hints.

## Draft API

Create a `CommandPalette` entity with a collection of `CommandAction` values and an initial open state. Each action needs a stable ID and label. Optional group names, search keywords, and keybinding text enrich search and display. Keybinding text is presentation only; the palette does not install shortcuts or execute action callbacks. Listen for `ActionActivated` and dispatch the matching ID in the host.

For externally managed visibility, construct the controlled palette and apply caller state through `set_open`. Escape emits `OpenChanged(false)`; prop application itself emits no interaction events. In uncontrolled mode, the palette owns Escape dismissal. Replacing the action list with `set_actions` preserves the active action when its ID remains in the matching results.

## Interaction

Search uses case-insensitive subsequence matching over label, group, and keywords, preserving caller order. Disabled actions do not appear. Arrow Up and Arrow Down wrap through results, Enter activates the active result, and Escape closes without activation. Native GPUI text input handles insertion, deletion, selection, and IME composition. Opening resets the query and focuses the search field.

The host owns the action registry, action execution, and when the palette is mounted/opened. While open, the search field is the only focus stop; Tab and Shift+Tab are named, rebindable actions that keep focus there. Opening records the previously focused control, and closing restores it where the host window supports that behavior. The host must keep the palette mounted while open. Popup result virtualization, async providers, result ranking, and result-count announcements are not included.

## Accessibility and theme

The palette surface uses the dialog role and accessible name “Command palette”. Its search field is named “Search commands”; results use listbox and option roles, including the action label, group, and displayed keybinding in their accessible names. The active result exposes the active-descendant focus marker. Colors and dimensions come from the Global `Theme`; the result viewport displays at most eight rows before scrolling is added in a later pass.

## Review and verification status

This is a draft public API. Maintainer review is required for event and constructor naming, keyboard behavior, dialog and result accessibility semantics, and visual baselines.

Verified with `cargo test -p mkit-registry-command-palette` (3 tests): fuzzy matching across labels and metadata, empty query matching, disabled-action filtering, real GPUI open/search/focus behavior, ArrowDown selection, Tab and Shift+Tab focus containment, Enter activation and dismissal, Escape dismissal without activation, and focus restoration to the opener. `cargo test -p mkit-harness` passes, including real text insertion through the focused GPUI input handler.

The manifest-driven E8 matrix captures five states (closed, open empty, filtered, no matches, disabled) in light, dark, and high-contrast themes at 1× and 2×. Filtered and no-match screenshots use real typed text (`save` and `zz-no-match`) and assert the query and active result. All 30 macOS Metal screenshots were captured, candidates visually inspected before baseline updates, and the matrix passes in comparison mode. Screenshot capture is skipped on platforms without the GPUI macOS Metal headless renderer. Native platform accessibility snapshots and manual keyboard review remain pending; no accessibility adapter result is claimed. Maintainer review is required for the public API, keyboard/focus and accessibility contracts, and visual baselines.
