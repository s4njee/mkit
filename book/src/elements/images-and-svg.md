# Add an image and SVG icon

## What you'll build

Add a small landscape preview to the settings sidebar and an SVG sync mark beside status text. Run `cargo run -p mkit-example-settings-screen` from the workspace root.

![Settings screen with a landscape preview in the account card and sync marks in the brand and header](../images/settings-screen.png)

## Concept

[`img()`](../appendices/api-inventory.md) accepts a value convertible to [`ImageSource`](../appendices/api-inventory.md). This example embeds PNG bytes in the crate, decodes them, converts their channels to the BGRA format expected by [`RenderImage`](../appendices/api-inventory.md), and passes an `Arc<RenderImage>` to `img`. That path is deterministic for an example and does not depend on a runtime asset loader.

[`svg()`](../appendices/api-inventory.md) creates an [`Svg`](../appendices/api-inventory.md) element. Its `.data(&[u8])` method passes SVG bytes directly; the source [draws raw data only when a text color is available](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/svg.rs#L137-L161). The example supplies `text_color` explicitly.

## Minimal compiling example

The raster conversion and both media elements are included from the compiling settings-screen crate:

```rust
{{#include ../../../examples/settings_screen/src/lib.rs:settings_screen_raster}}
```

```rust
{{#include ../../../examples/settings_screen/src/lib.rs:settings_screen_media}}
```

Run `cargo run -p mkit-example-settings-screen`. On macOS, `cargo test -p mkit-example-settings-screen --test screenshot --locked` compares the initial 1920 × 1280 image at scale 2.0. Windows and Linux compile the test but report unsupported headless capture.

## How it works

Startup decodes the checked-in PNG once per settings view. `landscape_photo` gives that cached image a bounded height, full available width, rounded corners, and `ObjectFit::Cover`. `sync_glyph` embeds the SVG file's bytes at compile time, then gives the glyph an explicit size and color. The same glyph appears in the brand mark, page header, and sync card.

A string passed to `img` without a URI scheme is treated as an embedded asset key by the [pinned `ImageSource` conversions](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L53-L91); it is not an arbitrary filesystem filename. A real path uses a `Path` or `PathBuf` conversion, while an embedded key requires an application asset source. This example uses in-memory media for its baseline.

## Exercises

1. Change the sync glyph's size at one call site. Run the app and check only that placement. Restore the original size before the screenshot comparison.
2. Change the photo's `ObjectFit` value and inspect how the same image fills its 74-pixel-high box. Restore `Cover` before the baseline test.

## API reference links

- [`ImageSource`, `RenderImage`, `Img`, `Svg`, and `ObjectFit`](../appendices/api-inventory.md) — image content, elements, and sizing behavior.
- [`Styled` and `Hsla`](../appendices/api-inventory.md) — SVG size and text color.
