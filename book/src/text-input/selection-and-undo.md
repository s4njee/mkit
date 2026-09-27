# Selection, cursor movement, and undo

## What you'll build

A single-line field that moves its caret, replaces a selection, and undoes and redoes text edits. Run `cargo run -p mkit-example-text-input --locked`; on macOS the harness also checks a selected-state image.

![Text field with River selected](../images/text-field-selected.png)

## Concept

`UTF16Selection` stores the platform's selection range and whether its head is reversed. Rust strings use UTF-8 byte indices, while the input-handler methods speak in UTF-16 code units. The field converts offsets at that boundary and clamps them to valid UTF-8 character boundaries. It does **not** implement grapheme-aware movement: a user-perceived character can contain several code points. The pinned [selection type](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1778-L1788) and [handler range methods](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/input.rs#L17-L113) define the platform side. GPUI's handler does not include an undo stack; the view owns simple text snapshots.

## Minimal compiling example

```rust
{{#include ../../../examples/text_input/src/lib.rs:text_field_editing}}
```

Run `cargo test -p mkit-example-text-input --locked`. The library tests cover synthetic input, UTF-16 boundaries, selection replacement, and undo; `tests/screenshot.rs` compares the selected state on macOS.

## How it works

Left and Right key events collapse a selection or move the caret by a UTF-16 unit, then clamp to a UTF-8 character boundary. Platform+A selects all. Platform+Z and Platform+Shift+Z call the view's undo and redo methods. Text replacement records the prior string, updates text and caret, then notifies GPUI. Undo restores the prior text and places the caret at its end; it does not restore the old selection. There is no Shift+Arrow selection extension or grapheme-aware movement in this example.

## Common mistakes

**The caret lands one position before the expected end.** [Zed issue #42794](https://github.com/zed-industries/zed/issues/42794) reports that symptom after selection in a later Windows Zed editor. The report does not diagnose this fixture's cause. Test selection direction, end-of-text movement, emoji, and combined characters; inspect offset conversions rather than assuming a byte or UTF-16 index is a grapheme boundary.

## Exercises

Select text containing an emoji and replace it. Assert the result and UTF-16 caret position. Undo, then redo, and check the text and the documented end-of-text caret policy after each step. Try a combining mark and observe where this minimal movement policy differs from grapheme movement.

## API reference links

- [`UTF16Selection`, `EntityInputHandler::selected_text_range`, and `EntityInputHandler::replace_text_in_range`](../appendices/api-inventory.md) · [pinned selection contract](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/input.rs#L17-L113)
