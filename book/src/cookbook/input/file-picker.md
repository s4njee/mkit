# Receive selected file paths

## What you'll build

A file picker request whose selected paths return to Rust.

## Concept

`PathPromptOptions` requests files or directories and single or multiple selection. The result is asynchronous.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_file_dialog}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `file_picker_returns_the_platform_selected_paths` simulates the platform response and checks the request options and returned path. Native picker UI remains manual.

## How it works

`choose_files` asks the app for multiple files and returns a task resolving to optional paths. Cancellation and platform errors become `None` in this small recipe.

## Exercises

Request one directory instead, and assert the prompt options and returned path in a test.

## API reference links

- [`gpui::PathPromptOptions`](../../appendices/api-inventory.md)
- [`gpui::Task`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)

