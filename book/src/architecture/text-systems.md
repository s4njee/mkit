# Trace text across platforms

## What you'll build

Run the single-line field and compare its focused, selected, and marked states. Then trace the shared text API to each platform's text implementation. The current visual checks are macOS captures.

![Focused single-line text field showing a caret](../images/text-field-focused.png)

## Concept

GPUI's shared [`TextSystem`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs) asks a [`PlatformTextSystem`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs) for fonts, line layout, and glyph rasterization. The pinned macOS path uses [CoreText](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_macos/src/text_system.rs#L17-L97). Windows uses [DirectWrite](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_windows/src/direct_write.rs#L38-L226). Linux aliases the [CosmicTextSystem](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_linux/src/linux/text_system.rs) implemented in the [wgpu crate](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_wgpu/src/cosmic_text_system.rs).

These implementations can choose different fonts, shaping, fallback, and glyph pixels. A macOS screenshot does not establish identical text output on Windows or Linux.

## Minimal compiling example

```rust
{{#include ../../../examples/text_input/src/lib.rs:text_field_element}}
```

```rust
{{#include ../../../examples/text_input/src/lib.rs:text_field_ime}}
```

Run `cargo test -p mkit-example-text-input --locked`. Synthetic handler tests cover UTF-16 ranges and candidate origin math with the test window. Four macOS screenshot baselines compare idle, focused, selected, and marked field states; a macOS headless check also compares shaped narrow and wide glyph advances. These checks do not test native OS candidate windows.

## How it works

The example keeps text and selection in its handler, then uses GPUI elements to lay out and paint the visible field. The platform text system handles font metrics and glyphs below the shared API. For candidate bounds, the handler converts a UTF-16 range to byte indices, shapes the line with `Window::text_system().shape_line`, and reads the shaped x positions. The macOS headless check exercises that platform shaping; the synthetic test window uses `NoopTextSystem` and cannot distinguish ordinary glyph advances.

Text rendering and text input are related but separate routes. The example's synthetic `EntityInputHandler` calls do not prove that an OS IME shows a candidate popup at the computed rectangle. See the [text input chapter](../text-input/ime-composition.md) for that limit.

## Exercises

Render a string with combining marks and emoji on each target OS. Record font fallback, glyph bounds, and screenshot differences. Test a native IME candidate popup manually before claiming its placement works.

## API reference links

- [`TextSystem`, `PlatformTextSystem`, `StyledText`, and `EntityInputHandler`](../appendices/api-inventory.md) — shared text and input contracts.
