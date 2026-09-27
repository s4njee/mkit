# Paint paths, quads, shadows, and images

## What you'll build

A compact scene with a path, a quad, a shadow, and a fixed PNG image. Run `cargo run -p mkit-example-custom-rendering --locked`. The initial 1280 × 960 macOS harness image shows the canvas scene; no separate baseline isolates every primitive.

![Initial custom-rendering canvas with a grid and marker](../images/custom-rendering-initial.png)

## Concept

`canvas()` is a lightweight element with prepaint and paint callbacks. Its paint callback can use `Window::paint_quad`, `paint_path`, and `paint_image`; shadows are a separate paint operation using `BoxShadow`. `PathBuilder` builds path geometry. These are GPUI scene primitives, not arbitrary shader or external GPU texture interop. See [canvas](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/canvas.rs#L10-L91), [path builder](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L25-L115), and [shadow painting](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L4254-L4334).

## Minimal compiling example

```rust
{{#include ../../../examples/custom_rendering/src/lib.rs:canvas_primitives}}
```

This helper compiles in `examples/custom_rendering`. The running view uses `canvas()` and loads `assets/mark.png` through the pinned image path. Run `cargo test -p mkit-example-custom-rendering --test screenshots --locked` to compare the initial, panned, and zoomed canvas baselines on macOS.

## How it works

`paint_scene` calls `paint_primitives` after drawing the grid and marker. The helper paints a separate shadow, quad, and stroked path in order. The running view also reads `assets/mark.png` for `paint_image`. The three screenshot baselines include these primitives together; they do not isolate one primitive's pixels from the rest of the scene.

## Exercises

Wire `paint_primitives` into the running view, then add a dedicated screenshot before changing path or shadow geometry. Keep the asset and viewport fixed and inspect the resulting pixel diff.

## API reference links

- [`canvas`, `PathBuilder`, `PaintQuad`, `BoxShadow`, and Window paint methods](../appendices/api-inventory.md) · [pinned paint source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L4386-L4457)
