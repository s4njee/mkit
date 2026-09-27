# Commit only the latest query

## What you'll build

A retained query that commits only after input stays quiet for 250 milliseconds.

## Concept

Debouncing delays work until a quiet interval. The view cancels the old timer and also checks a generation number before applying a result.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_debounce_task}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `debounce_timer_commits_only_the_latest_input` uses a controlled clock: an early query is replaced, and only `latest` is committed.

## How it works

`DebouncedQuery::input` increments the generation, drops the prior task, spawns a timer, then weakly updates the entity only if that generation remains current. It calls `notify()` after committing.

## Exercises

Change the delay to 100 milliseconds and update the fake-clock test at both sides of the boundary.

## API reference links

- [`gpui::Task`](../../appendices/api-inventory.md)
- [`gpui::Entity`](../../appendices/api-inventory.md)
- [`gpui::Context`](../../appendices/api-inventory.md)

