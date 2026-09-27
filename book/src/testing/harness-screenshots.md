# Capture a harness screenshot

## What you'll build

A pixel comparison for the counter's initial state. On macOS, run `cargo test -p mkit-example-counter --test screenshot --locked` and compare the generated frame with the checked-in baseline.

![Counter showing Count: 0 and an Increment button at 1280 by 800 pixels, scale 2](../images/counter.png)

## Concept

`mkit-harness::screenshot` creates a `HeadlessAppContext`, opens a window, renders it, and returns an RGBA image. `PixelTolerance` compares images of the same dimensions with bounded channel and pixel differences. The current `gpui-pre` platform provides this offscreen renderer only on macOS; [the harness returns `UnsupportedPlatform` elsewhere](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_platform/src/gpui_platform.rs).

## Minimal compiling example

```rust
{{#include ../../../examples/counter/tests/screenshot.rs:counter_screenshot_test}}
```

This function compiles within the example's screenshot test file. Run `cargo test -p mkit-example-counter --test screenshot --locked`. `UPDATE_SNAPSHOTS=1 cargo test -p mkit-example-counter --test screenshot --locked` updates the baseline intentionally; inspect the diff before keeping it.

## How it works

The test renders `Counter::default()` at 640 by 400 logical pixels and scale 2 after `gpui_kit::init`. It checks the output size and loads `book/src/images/counter.png`. `PixelTolerance` allows a channel delta of 2 over at most 8 pixels. On mismatch, it writes a diff image and fails. [The pinned headless context](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/headless_app_context.rs#L38-L87) supplies the GPUI context; the workspace's `crates/mkit-harness/src/screenshot.rs` owns capture and comparison. This baseline covers one default theme, size, scale, and macOS renderer.

## Common mistakes

Accepting a new baseline before inspecting the diff can hide a visual regression. Run the ordinary comparison first and review the diff file. A [Zed report about locked-screen capture](https://github.com/zed-industries/zed/issues/63217) concerns an ordinary macOS window in a later GPUI revision; it is not evidence that this headless 0.3.5 test has the same failure.

## Exercises

Change the counter's label, run the comparison, and inspect the diff. Restore the label and confirm the test passes. Add a second theme baseline only after initializing that theme and naming its screenshot test.

## API reference links

- [`HeadlessAppContext`](../appendices/api-inventory.md) · [pinned source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/headless_app_context.rs#L38-L87)
