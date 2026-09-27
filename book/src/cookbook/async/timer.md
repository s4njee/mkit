# Wait with an executor timer

## What you'll build

A delay task that resolves after a requested duration.

## Concept

GPUI executors expose a timer future. Holding its `Task` lets the caller await or cancel the delay.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_timer}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `timer_completion_and_drop_cancellation_are_deterministic` advances a controlled clock and checks readiness after 40 milliseconds. It does not promise wall-clock precision.

## How it works

`delay` passes the duration to `background_executor().timer`. The focused timer test calls this wrapper.

## Exercises

Use a controlled test clock to advance just before and after the duration and assert completion.

## API reference links

- [`gpui::Task`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)
