# Arrange two grid cells

## What you'll build

Two elements in a two-column grid.
![Cookbook overview on macOS](../../images/cookbook.png)

## Concept

A grid parent assigns children to columns and applies a gap. This helper does not implement responsive breakpoints.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_grid_layout}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. The macOS overview baseline shows the initial grid. Narrow-window layout is not covered.

## How it works

`two_column_grid` uses `grid_cols(2)` and `gap_3()`. The cookbook overview mounts grid cards.

## Exercises

Shrink the window, capture a second baseline, and decide when one column would be clearer.

## API reference links

- [`gpui::IntoElement`](../../appendices/api-inventory.md)
