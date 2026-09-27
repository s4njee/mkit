# Multi-select

Choose zero or more IDs from a bounded option list. This E7 component is a draft with an `implementation_in_progress` registry entry. Its public contract needs maintainer review.

![Multi-select dark theme baseline](../../images/e7/multi-select.png)

*Dark theme, 2× baseline capture: `open` state.*

## API and state

`MultiSelect::new(label, options, default_values)` owns selected IDs; `controlled(...)` emits `ValuesChanged(Vec<String>)` requests until the owner calls `set_values`. `OpenChanged(bool)` reports visibility. `values()`, `is_open()`, and `active_option_id()` expose state. Clicking a row toggles it without closing.

## Keyboard behavior

The component's named actions can be rebound by the host where applicable.

| Key or action | Current behavior |
| --- | --- |
| Arrow keys, Home, End | Move the active row among enabled options. |
| Enter, Space | Open or toggle active membership. |
| Escape | Close without changing values. |
| Tab, Shift+Tab | Close and traverse focus while preserving selected values. |

## Accessibility

Requests a named combobox trigger with expanded state and selected labels joined in option order as its value. Popup requests multiselectable listbox and option selection/disabled state. A scoped macOS check observed trigger values; full platform semantics remain unverified.

## Theme tokens

Theme surface/elevated surface/text/muted/border/accent/focus/disabled, spacing, radii, borders, controls. Popup shows up to eight virtualized rows and flips above near the window bottom.

## Usage scenarios

- Assign multiple teams to a project.
- Choose several report columns from a long list.
- Keep controlled selected tags in an owner-owned filter state.

## Verification and limits

9/9 keyboard cases, 8/8 registry tests, and 24/24 screenshot comparisons passed. Active-platform accessibility remains open. These are focused draft checks, not release approval. The full E5 conformance matrix and maintainer review are outstanding. The current contract and evidence are in `registry/multi-select/spec.md` and `docs/E7_AUDIT.md` in the repository.
