# Compute a count from source state

## What you'll build

A completed-row count derived from rows and completed IDs.

## Concept

Derived state is calculated from owned data instead of stored as a second value that can become stale.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_derived_state}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked` to compile the helper. No focused count assertion exists yet.

## How it works

`completed_count` scans rows and counts IDs in the completed set. The anchor compiles as a pure helper; the current cookbook tests do not call it or show it in a view.

## Exercises

Add a test with reordered rows and duplicate completed IDs, then display the count in a retained view.

## API reference links

This pure Rust helper calls no GPUI public API directly. A view can use its result while [rendering retained state](../../state/entities.md).
