# Save the current window size

## What you'll build

A local file containing the width and height read from a live GPUI window.

## Concept

Window geometry can be read from `Window::window_bounds`. Persist only validated finite, positive sizes before reconstructing options.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_persist_window_size}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `current_window_bounds_can_be_saved_to_a_local_config_file` calls the save helper on a live test window and reads it back. Native reopen placement is not tested.

## How it works

`save_window_size` reads current bounds and writes two numbers. `load_window_size` parses them and rejects invalid or extra values.

## Exercises

Save, close, and reopen a native window on each OS, then compare actual bounds within a documented tolerance.

## API reference links

- [`gpui::Window`](../../appendices/api-inventory.md)
- [`gpui::WindowBounds`](../../appendices/api-inventory.md)

