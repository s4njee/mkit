# Editable combobox pilot

A combobox lets someone type a value and choose from suggestions. Use this pilot when the choices have stable IDs and a short, local list is enough. Its text field keeps focus while the suggestion list is open. This is a draft registry component; the spec at `registry/combobox/spec.md` records the intended full contract.

## Create the state

The example crate constructs the component state. It does not render a window, so there is no screenshot for this page. A rendered example and its harness captures remain pilot work. Its choices use stable IDs:

```rust
{{#include ../../../examples/pilot_components/src/lib.rs:combobox_options}}
```

For an **uncontrolled** combobox, `new` takes an accessible label, options, and an optional initial option ID. The component owns the committed selection:

```rust
{{#include ../../../examples/pilot_components/src/lib.rs:combobox_uncontrolled}}
```

For a **controlled** combobox, the owner supplies the selected ID. Listen for `ValueChanged` and pass the accepted ID back through `set_value` on the component entity. Until then, the supplied value remains authoritative:

```rust
{{#include ../../../examples/pilot_components/src/lib.rs:combobox_controlled}}
```

The example runs with `cargo test -p mkit-example-pilot-components --lib --locked`. The option IDs are values; labels are what people see. Supply a meaningful label such as “Country” even when the current text looks self-explanatory.

## What interaction reports

Typing emits `InputChanged(query)` and filters option labels using the current case-insensitive substring match. Opening or closing emits `OpenChanged(bool)`. Choosing an option emits `ValueChanged(id)` and `OptionSelected(id)`. A custom text commit, when `allow_custom_value(true)` is enabled, emits `ValueChanged(text)` without `OptionSelected`. Escape emits `Cancelled(session_start_id)`; Tab dismissal emits `Dismissed`. A controlled owner should use these typed GPUI entity events to decide which value to keep.

ArrowDown and ArrowUp open and move through enabled matches. Enter commits the active option. Escape restores the value from the start of the editing session. Tab and Shift+Tab close the popup, preserve uncommitted typed text, and move focus forward or backward. Alt+ArrowDown opens the popup without changing the text. The component declares named actions in the `Combobox` key context; register its default bindings or replace them with app bindings.

## Accessibility and current limits

The current view exposes an editable combobox role and label, expanded state, a listbox role, and option roles. Disabled options remain visible but cannot be selected. The input keeps focus during option navigation. The complete popup relationship, active-descendant behavior, result announcements, and screen-reader checks are still pending. The pinned headless GPUI test window does not activate AccessKit, so the generated accessibility cases are **not verified** by a live accessibility snapshot.

The view reads colors, spacing, borders, and typography from the GPUI theme `Global`, including Ayu when that theme is active. Popup positioning and outside-click dismissal are still under development. The full state, theme, and scale screenshot matrix has not passed. Review the registry spec before relying on the draft API in a shipped app.
