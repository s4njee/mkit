# Register an application menu

## What you'll build

One menu item that dispatches a typed action.

## Concept

GPUI's `Menu` and `MenuItem` describe application menu entries. Registration is distinct from user selection through the native menu.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_application_menu}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `app_menu_recipe_registers_a_native_menu` verifies registration in the test context. Native menu selection remains manual.

## How it works

`install_application_menu` installs a Cookbook menu with an Activate item. The action type is declared by `recipe_actions`.

## Exercises

Add a second item and assert both labels and action types in the test menu model.

## API reference links

- [`gpui::Menu`](../../appendices/api-inventory.md)
- [`gpui::MenuItem`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)

