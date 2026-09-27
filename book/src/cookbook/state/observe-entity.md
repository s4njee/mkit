# Observe an entity change

## What you'll build

A view that watches a source entity and shows the last observed count.

![Observed count before the click on macOS](../../images/cookbook-counter.png)

## Concept

`cx.observe` registers a callback for `notify()` on the source. Its returned `Subscription` must stay in the view; dropping it cancels the callback.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_observe_entity}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. The screenshot shows the initial zero value. The click test checks the observer value after a rendered interaction.

## How it works

`CounterDemo` creates the source once, stores the observation handle, and reads `source.read(cx).value` after notification. It also subscribes to a typed event, which is a separate channel.

## Exercises

Temporarily remove source `notify()` and check that the observed value stops advancing; restore it before rerunning tests.

## API reference links

- [`gpui::Entity`](../../appendices/api-inventory.md)
- [`gpui::Subscription`](../../appendices/api-inventory.md)
- [`gpui::Context`](../../appendices/api-inventory.md)
