# Emit a typed counter event

## What you'll build

A `CounterChanged` payload that tells a subscriber the new value.

![Counter event display before the click on macOS](../../images/cookbook-counter.png)

## Concept

`EventEmitter<CounterChanged>` declares which event type the source can emit. An event carries data; a `notify()` change notice does not carry this payload.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_typed_event}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. The click test asserts event value one. This anchor only declares the type; include `recipe_entity_counter` and `recipe_observe_entity` to see emission and receipt.

## How it works

`Counter::increment` emits the payload, while `CounterDemo` stores the event subscription. The subscriber copies `event.0` into its retained view state and notifies the view.

## Exercises

Remove `cx.emit` temporarily and check that the observer still advances while the event value stays zero.

## API reference links

- [`gpui::EventEmitter`](../../appendices/api-inventory.md)
- [`gpui::Context`](../../appendices/api-inventory.md)
- [`gpui::Subscription`](../../appendices/api-inventory.md)
