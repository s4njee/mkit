# Connect a field to platform text input

## What you'll build

A focused single-line field that receives text through GPUI's input handler. Run `cargo run -p mkit-example-text-input --locked`. The harness captures its idle and focused states at 1280 × 440 pixels, scale 2 on macOS.

![Idle text field with a placeholder](../images/text-field-idle.png)

![Text field with River and a visible caret](../images/text-field-focused.png)

## Concept

`EntityInputHandler` is the contract a stateful GPUI view implements to expose its text, selection, replacement, and geometry to the platform. `ElementInputHandler` adapts that entity to `InputHandler`, the platform-facing trait. During paint, the element calls `Window::handle_input` with its focus handle and the adapter. The [pinned interface](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/input.rs#L13-L145) and [paint-time registration](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L5102-L5124) define this route.

## Minimal compiling example

```rust
{{#include ../../../examples/text_input/src/lib.rs:text_field_handler}}
```

```rust
{{#include ../../../examples/text_input/src/lib.rs:text_field_element}}
```

These regions compile together in the example crate. Run `cargo test -p mkit-example-text-input --locked` for the handler and screenshot checks. The image baseline test is `tests/screenshot.rs`; pixel capture is available on macOS in this pin.

## How it works

The view owns text, selection, marked range, and history. `EntityInputHandler` converts platform UTF-16 ranges at the boundary. The `Render` implementation gives the field a focus handle and paints a `canvas` child. Its paint callback calls `Window::handle_input` with the painted bounds and `ElementInputHandler`. The test focuses the field, sends synthetic input, and checks retained text. The focused screenshot checks the visible caret, while native OS input methods need separate manual checks.

## Common mistakes

**Assuming the handler is a finished text component.** [Zed issue #42774](https://github.com/zed-industries/zed/issues/42774) requests a complete GPUI input control with key, pointer, caret, and IME behavior. This later feature request is not a bug report against this pin. Implement and test the field's own editing rules rather than expecting the low-level handler to supply them.

## Exercises

Focus the field and type two characters. Run `cargo test -p mkit-example-text-input --lib --locked`, then inspect the retained text and UTF-16 selection. Try typing after moving focus elsewhere and record whether the field accepts it.

## API reference links

- [`EntityInputHandler`, `ElementInputHandler`, `InputHandler`, and `Window::handle_input`](../appendices/api-inventory.md) · [pinned implementation](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/input.rs#L13-L145)
