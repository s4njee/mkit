# Lay out a simple split pane

## What you'll build

Two side-by-side regions separated by a fixed divider.
![Cookbook overview on macOS](../../images/cookbook.png)

## Concept

Flex layout lets each pane take remaining width. This helper does not implement draggable resizing.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_split_pane}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. The macOS `cookbook.png` baseline shows the two panes. No divider drag or resize test exists.

## How it works

`split_pane` places left and right content in flex children with a one-pixel divider. The cookbook overview mounts the helper for its editor/preview area.

## Exercises

Change the initial widths, capture a screenshot, then add a drag handler before calling it resizable.

## API reference links

- [`gpui::IntoElement`](../../appendices/api-inventory.md)

