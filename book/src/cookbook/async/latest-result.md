# Ignore an out-of-date result

## What you'll build

A guard that accepts only the result for the current request generation.

## Concept

Concurrent tasks may finish out of order. Compare each result's generation with the current request before applying it.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_latest_result}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `stale_async_result_is_ignored` checks rejection of generation 3 and acceptance of generation 4.

## How it works

`apply_latest` returns `Some(result)` only when the generations match. It is a pure helper, not a complete network request flow.

## Exercises

Start two controlled tasks that resolve in reverse order, then apply the guard before updating a view.

## API reference links

This result guard is plain Rust and calls no GPUI public API directly. Pair it with a [`gpui::Task`](../../appendices/api-inventory.md) that produces versioned results.
