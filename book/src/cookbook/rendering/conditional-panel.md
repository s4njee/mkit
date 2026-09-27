# Render a panel conditionally

## What you'll build

A panel whose child text appears only while a boolean is true.
![Cookbook overview on macOS](../../images/cookbook.png)

## Concept

A conditional element describes the visible state at render time. The owner must retain and update the boolean.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_conditional_render}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. The overview screenshot shows the visible branch. It does not establish the hidden branch.

## How it works

`conditional_panel` adds its child only when `visible` is true. The cookbook overview mounts a visible instance.

## Exercises

Toggle the boolean through an action and assert both visible and absent states.

## API reference links

- [`gpui::IntoElement`](../../appendices/api-inventory.md)
