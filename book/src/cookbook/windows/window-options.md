# Request centered window bounds

## What you'll build

A `WindowOptions` value with a centered 960 × 640 requested size.

## Concept

`WindowOptions` describes a window request. The OS window manager may adjust actual placement or dimensions.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_window_options}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `window_options_and_second_window_are_applied` checks the options and an opened second view in the test platform. Native titlebar and placement remain manual.

## How it works

`cookbook_window_options` computes centered bounds from the current app display and returns options for `open_window`.

## Exercises

Request a smaller size and compare the requested options with actual native bounds on two OSes.

## API reference links

- [`gpui::WindowOptions`](../../appendices/api-inventory.md)
- [`gpui::WindowBounds`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)

