# Style a span of text

## What you'll build

A sentence with one bold span.
![Cookbook overview on macOS](../../images/cookbook.png)

## Concept

`StyledText` receives text and style runs. Its run lengths use byte offsets for this fixed ASCII sentence.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_rich_text}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. The cookbook overview mounts the sentence and compares its initial macOS screenshot. This fixed ASCII example does not test complex scripts.

## How it works

`rich_text` finds the word `emphasis`, builds three runs from the window's text style, and changes one run's weight.

## Exercises

Replace the text with non-ASCII characters and recalculate run boundaries from valid UTF-8 byte offsets.

## API reference links

- [`gpui::StyledText`](../../appendices/api-inventory.md)
- [`gpui::Window`](../../appendices/api-inventory.md)

