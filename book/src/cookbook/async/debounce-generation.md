# Reject an old generation

## What you'll build

A generation counter that identifies the latest input request.

## Concept

Every new input increments the generation. A result is current only if its captured number still matches.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_debounce_generation}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `debounce_discards_old_generation_and_toasts_are_fifo` asserts an old number is rejected and the latest accepted.

## How it works

`begin` raises the counter; `is_current` compares a captured value with the latest one. This is a pure guard, not a timer or UI by itself.

## Exercises

Start three generations and assert only the last is current.

## API reference links

This generation guard is plain Rust and calls no GPUI public API directly. The [timer recipe](debounce-timer.md) places the guard in an entity task.
