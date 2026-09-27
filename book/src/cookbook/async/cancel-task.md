# Cancel a retained task handle

## What you'll build

A helper that drops a previous `Task` before replacing it.

## Concept

A retained `Task` keeps its future active. Dropping the handle cancels work when no other handle owns it in this pinned API.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_cancel_task}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `timer_completion_and_drop_cancellation_are_deterministic` advances the fake clock past the canceled task's deadline and finds its flag unchanged. This does not prove cancellation of external I/O already in progress.

## How it works

`cancel_previous` takes the `Option<Task<()>>` and drops it. The focused test verifies that a delayed side effect does not run after this helper drops the task.

## Exercises

Add a controlled-clock test where the old task would mutate state, then prove it does not after the handle is dropped.

## API reference links

- [`gpui::Task`](../../appendices/api-inventory.md)
