# Choose a navigation key at startup

## What you'll build

A validated alternate key binding for moving down a list.

## Concept

A key string is parsed before GPUI stores its binding. An app can pass a different string at startup, while the action type stays the same.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_rebindable_keys}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `caller_rebinds_navigation_to_a_valid_keystroke` binds Ctrl-J, sends it through the rendered view, and checks selection moves. The helper parses key strings, but this test does not assert an invalid-key error or provide a settings UI.

## How it works

`bind_navigation` validates `down_key`, then binds it to `MoveDown` in the `Cookbook` key context. Other keys keep their typed actions.

## Exercises

Change the alternate key and assert both its dispatch and the displayed help text in a small view.

## API reference links

- [`gpui::KeyBinding`](../../appendices/api-inventory.md)
- [`gpui::Keystroke`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)
