# Write text to the clipboard

## What you'll build

A string sent to GPUI's clipboard.

## Concept

`ClipboardItem::new_string` wraps plain text for the platform clipboard. Reading it back is a separate call.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_clipboard}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `clipboard_recipe_round_trips_text` writes and reads text in the GPUI test context. Native cross-app paste remains a manual platform check.

## How it works

`copy_text` writes a string through `App::write_to_clipboard`. The helper does not handle rich text or file paths.

## Exercises

Copy non-ASCII text and assert the same string on read-back.

## API reference links

- [`gpui::ClipboardItem`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)

