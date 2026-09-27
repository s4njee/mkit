# Declare typed actions

## What you'll build

A set of typed actions for navigation, activation, palette change, and modal controls.

## Concept

GPUI actions are Rust types used by key bindings and action handlers. Declaring them does not bind any keys by itself.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_actions}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `keyboard_actions_update_state_through_the_rendered_view` sends Down and Enter and checks selection and activation. This test covers the combined setup, not the declaration alone.

## How it works

The `actions!` macro creates action types. The cookbook view attaches handlers for navigation and activation; `install_cookbook_keys` binds keys when the app starts.

## Exercises

Add a new action, bind it in the app, and assert its handler changes retained state.

## API reference links

- [`gpui::Action`](../../appendices/api-inventory.md)
- [`gpui::KeyBinding`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)

