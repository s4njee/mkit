# Trace a GPUI frame

## What you'll build

Run the pan-and-zoom canvas, then trace its custom element from layout through the platform draw call. The macOS harness captures its initial 1280 × 960 frame at scale 2.

![Initial pan-and-zoom canvas with a grid and scene marks](../images/custom-rendering-initial.png)

## Concept

An [element](../appendices/api-inventory.md) describes content for a draw. Its [`request_layout`, `prepaint`, and `paint`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L51-L116) phases first ask for size, then prepare bounds and hit data, then add drawing primitives. A `Scene` holds primitives for the platform renderer. [`Window::draw`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L3143-L3235) prepares the next frame and swaps it into the rendered frame. [`Window::present`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L3320-L3336) passes that frame's scene to the platform window.

The pinned backend paths are [Metal on macOS](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_apple/src/metal_renderer.rs), [DirectX on Windows](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_windows/src/directx_renderer.rs), and [wgpu on Linux X11](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_linux/src/linux/x11/window.rs) or [Wayland](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_linux/src/linux/wayland/window.rs). The underlying Linux graphics API depends on wgpu and the host; it is not one fixed renderer API.

## Minimal compiling example

```rust
{{#include ../../../examples/custom_rendering/src/lib.rs:custom_element}}
```

Run `cargo test -p mkit-example-custom-rendering --locked`. The library test checks layout → prepaint → paint order and bounds. On macOS, the screenshot test compares the visible canvas. Those tests do not benchmark every platform backend.

## How it works

The custom element records the phase order. In the full window path, `draw_roots` requests root layout, prepaints roots and deferred content, then paints them into the next frame's scene. After `draw` finishes, the window swaps frames. `present` passes the rendered scene to `platform_window.draw`. See [the pinned root path](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L3387-L3475) and [frame scene storage](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L974-L1134).

This is a logical pipeline. GPUI may reuse unchanged view work, and the screenshot only exercises the macOS renderer in this workspace.

## Exercises

Change the custom element's layout size and assert its new bounds before updating the macOS baseline. Inspect a platform window's `draw` implementation on a second OS and record which backend is selected there.

## API reference links

- [`Element`, `Window`, `Scene`, and `Render`](../appendices/api-inventory.md) — public names that connect the example to the frame path.
