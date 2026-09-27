# Controlled and uncontrolled component state

## What you'll build

Build a view with two toggles. One owns its value; the other asks its parent to accept a new value. Run `cargo test -p mkit-example-state-entities --lib --locked` from the workspace root to check both paths.

![State patterns view with two false toggle values and buttons for each mode](../images/component-state-demo.png)

The screenshot test also checks [dark](../images/component-state-demo-dark.png),
[high contrast](../images/component-state-demo-high-contrast.png), and
[light at scale 2](../images/component-state-demo-scale-2.png) using the same
token-driven view.

## Concept

A GPUI [`Entity<T>`](../appendices/api-inventory.md) keeps a view or model alive between updates. A stateful component uses that entity to retain its current value. An *uncontrolled* component owns the value and changes it as soon as the user acts. A *controlled* component receives the value from its parent. It reports a proposed change and waits for the parent to supply the next value.

`mkit_core::state::ComponentState<T>` stores the value and mode. Its `request_change` method returns a `StateChange<T>` with the old value, requested value, and mode. In uncontrolled mode it also commits the requested value. In controlled mode it leaves the visible value alone until the owner calls `set_value`.

GPUI's [`EventEmitter<E>`](../appendices/api-inventory.md) lets the entity publish a component-specific event. In this example, `ToggleChangeRequested` carries a `StateChange<bool>` payload. The entity type, rather than the `ComponentState` field, implements `EventEmitter`.

## Minimal compiling example

The following code comes from the tested `mkit-example-state-entities` crate:

```rust
{{#include ../../../examples/state_entities/src/component_state.rs:component_state_pattern}}
```

Run `cargo test -p mkit-example-state-entities --lib --locked` to exercise both clicks and the controlled proposal. Run `cargo test -p mkit-example-state-entities --test component_state_screenshot --locked` to compare light, dark, and high-contrast states at 640 × 400 logical pixels and scale 2. The captures use macOS Metal headless rendering. The pinned headless pixel renderer does not capture on Windows or Linux. The demo has pointer controls only; it does not establish keyboard or accessibility behavior for a reusable toggle component.

## How it works

1. The parent view creates two `Entity<ToggleModel>` handles, one with `ComponentState::uncontrolled(false)` and one with `ComponentState::controlled(false)`. It renders the values it observes from the entities.
2. An interaction calls `request_change`. The returned `StateChange` records the requested value and whether the entity committed it locally.
3. The entity emits its own typed `ToggleChangeRequested` event through [`Context<Self>::emit`](../appendices/api-inventory.md). The owner subscribes to that event. For the controlled toggle, this example accepts the proposal and calls `set_value` on the child entity.
4. If the entity changed the value it renders, it calls [`Context<Self>::notify`](../appendices/api-inventory.md). Event emission and render notification are separate GPUI operations.

The event is emitted even when the requested value equals the current one. A real component should document whether it suppresses duplicate requests; this helper leaves that choice to the component.

## Exercises

1. Change the uncontrolled toggle's initial value to `true`. Run the library test after updating its expected starting state, and check that clicking it commits `false` locally.
2. In the controlled event subscriber, remove the owner's `set_value` call. Run the library test and check that the entity still emits a request while the rendered controlled value stays `false`.

## API reference links

- [`Entity<T>`, `Context<T>`, and `EventEmitter<E>`](../appendices/api-inventory.md) — retained state, entity updates, and typed events.
- [`App`](../appendices/api-inventory.md) — application context passed to GPUI event subscribers.
