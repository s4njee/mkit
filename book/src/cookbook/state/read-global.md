# Read an app global

## What you'll build

A read of the shared label from the application context.

## Concept

`App::global::<T>()` returns the installed value for a `Global` type. Callers must install it before reading.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_read_global}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `app_global_is_available_to_later_recipe_code` checks the returned string after installation.

## How it works

`global_label` obtains `SharedLabel` and clones its string. Pair this anchor with `recipe_shared_global` for the installation path.

## Exercises

Read the label from two separate entities and check that both see the installed value.

## API reference links

- [`gpui::Global`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)

