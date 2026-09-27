# Install an app global

## What you'll build

One application-wide label stored with GPUI `Global`.

## Concept

A `Global` is shared through the application context rather than passed down every view. Installing it replaces the value for that type in the app.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_shared_global}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `app_global_is_available_to_later_recipe_code` installs a label and reads it back. This does not persist it to disk.

## How it works

`SharedLabel` implements `Global`. `install_shared_label` calls `App::set_global` with an owned string.

## Exercises

Install a different label, read it back, and assert the new string.

## API reference links

- [`gpui::Global`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)

