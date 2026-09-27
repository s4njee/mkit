# Clipboard text

## What you'll build

A Copy Selected command and a Paste command for text in the keyboard list. Run `cargo run -p mkit-example-interaction --locked` from the workspace root.

![Interaction list with a clipboard status line beneath the file drop area](../images/interaction.png)

## Concept

The *clipboard* holds data for transfer between commands or applications. GPUI exposes `App::write_to_clipboard` and `read_from_clipboard`. `ClipboardItem::new_string` makes a text item; `text` reads its text. An asynchronous read exists for platforms where access may require permission. See the [pinned App methods](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L1453-L1484) and [clipboard item](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2520-L2629).

## Minimal compiling example

These methods come from the compiling `examples/interaction/src/lib.rs`.

```rust
{{#include ../../../examples/interaction/src/lib.rs:interaction_clipboard}}
```

Run `cargo test -p mkit-example-interaction --lib --locked` for text and empty-clipboard cases. The screenshot above is checked by `cargo test -p mkit-example-interaction --test screenshot --locked` on macOS at 1280 × 1040 and scale 2.0.

## How it works

Copy Selected writes the selected row's label with `ClipboardItem::new_string`. Paste reads text and shows it in the status line; missing text becomes a message. The test uses GPUI's [test clipboard methods](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/visual_test_context.rs#L343-L351) to cover copy, paste, and empty cases. This proves the app's text path in the test, not cross-application paste or OS permission behavior.

## Common mistakes

**A test clipboard pass is taken as proof of system clipboard behavior.** [Zed issue #61322](https://github.com/zed-industries/zed/issues/61322) reports a later Wayland case affecting other applications' copy and paste while Zed is running. It does not establish a defect in this pinned GPUI version. Verify system clipboard exchange on each supported platform before claiming it.

## Exercises

Select the second row, copy, and paste. The status line should show “Second row.” Change the test clipboard to an empty item and check that the app handles no text without a panic.

## API reference links

- [`App::read_from_clipboard`, `App::read_from_clipboard_async`, `App::write_to_clipboard`, `ClipboardItem`](../appendices/api-inventory.md) · [pinned source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L1453-L1484)
