# Queue and dismiss toast messages

## What you'll build

A first-in, first-out queue of transient messages.

## Concept

A queue keeps notification order. It does not decide how or when a toast is drawn.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_toast_queue}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `debounce_discards_old_generation_and_toasts_are_fifo` asserts two messages come out in insertion order.

## How it works

`ToastQueue::push` appends text and `dismiss_current` removes the oldest message. The anchor does not render a toast or start a timeout.

## Exercises

Add a view that renders the front message and a timer that dismisses it; capture that state before calling it a visual toast.

## API reference links

The queue itself uses only Rust's `VecDeque`. Store it in a [`gpui::Entity`](../../appendices/api-inventory.md) if a view must show it.
