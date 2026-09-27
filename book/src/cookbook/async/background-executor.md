# Run work on the background executor

## What you'll build

A vector calculation scheduled away from GPUI's foreground executor.

## Concept

The background executor runs work outside the foreground scheduler. This example computes squares; it does not perform a blocking file read.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_background_executor}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `foreground_and_background_executor_recipes_complete` checks all 100 values; no performance or thread-affinity measurement is claimed.

## How it works

`background_work` spawns an async computation and returns a `Task<Vec<u64>>`. The cookbook test awaits it and checks the complete vector.

## Exercises

Await the task in a test and check its first and last values.

## API reference links

- [`gpui::Task`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)
