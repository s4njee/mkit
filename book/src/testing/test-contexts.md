# Choose a test context

## What you'll build

A counter test that opens a GPUI window, clicks its button, and checks the retained count. Run `cargo test -p mkit-example-counter --lib --locked`.

## Concept

`TestAppContext` owns a test app. `add_window_view` returns a view handle and a window-specific `VisualTestContext`. The latter draws the window and simulates input. `HeadlessAppContext` is a separate low-level context used by our [screenshot harness](harness-screenshots.md), which can capture pixels when a renderer is available. The [pinned test context](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_context.rs#L21-L57) and [headless context](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/headless_app_context.rs#L38-L87) keep these jobs distinct.

## Minimal compiling example

```rust
{{#include ../../../examples/counter/src/lib.rs:counter_interaction_test}}
```

The test needs its enclosing test module's imports and the [`Counter` view](../examples/counter.md); the include shows the smallest useful function from that compiling module. Run `cargo test -p mkit-example-counter --lib --locked`.

## How it works

The test initializes GPUI Kit, creates a window with `Counter`, and draws a frame. `debug_bounds("increment")` finds the rendered target's bounds. `simulate_click` dispatches a pointer click at its center. Finally `read_with` reads the retained `count` from the entity and checks that it is 1. This confirms behavior after a real GPUI test input dispatch; it does not assert any pixel colors.

## Common mistakes

Reading the view before dispatching and drawing can check the wrong moment. Draw the window, simulate the input, then read the entity. [Official Zed discussion #17477](https://github.com/zed-industries/zed/discussions/17477) points to GPUI's test macro and simulated interaction for this kind of test. The discussion does not report a `gpui-pre` 0.3.5 bug.

## Exercises

Click the button twice and expect 2. Then change the target name in the test and observe the clear `debug_bounds` failure. Restore it and rerun the package's library tests.

## API reference links

- [`TestAppContext`, `VisualTestContext`, `Entity::read_with`, `VisualTestContext::simulate_click`](../appendices/api-inventory.md) · [pinned test source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_context.rs#L783-L905)
- [`HeadlessAppContext`](../appendices/api-inventory.md) · [pinned headless source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/headless_app_context.rs#L38-L87)
