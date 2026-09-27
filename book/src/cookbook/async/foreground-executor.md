# Run a short foreground task

## What you'll build

A small task scheduled on GPUI's foreground executor.

## Concept

The foreground executor runs UI-adjacent futures through GPUI's application scheduler. It is suitable for short work that does not block the UI thread.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_foreground_executor}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `foreground_and_background_executor_recipes_complete` awaits the result. It does not measure UI responsiveness.

## How it works

`foreground_work` spawns an async result and returns its `Task` handle. The cookbook test awaits this helper and checks its result.

## Exercises

Await the result in a focused test and assert its text. Move genuinely blocking work to the background executor.

## API reference links

- [`gpui::Task`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)
