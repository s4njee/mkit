# Coming from iced or egui

## What you'll build

Run the retained GPUI counter and inspect what persists between clicks. Click **Increment** and check that the label follows the stored count. Run `cargo run -p mkit-example-counter` from the workspace root.

![Initial GPUI counter window with an Increment button](../images/counter.png)

## Concept

[iced 0.14](https://docs.rs/iced/latest/iced/) explains an application through state, messages, `update`, and `view`; its `Task` can carry asynchronous work. [egui](https://docs.rs/egui/latest/egui/#understanding-immediate-mode) describes an immediate-mode UI that runs UI-building code each frame and leaves persistent application values with the caller. These are useful starting points, but GPUI uses different public contracts.

GPUI stores a view's data in an [`Entity<T>`](../appendices/api-inventory.md). A type implementing [`Render`](../appendices/api-inventory.md) describes elements from that retained data when GPUI needs it. The click handler changes the entity and calls `cx.notify()`. The [pinned view element](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/view.rs#L380-L415) can reuse an unchanged subtree. This is an immediate-style render method over retained state, not egui's every-frame UI callback. The example also has no iced `Message` enum or centralized `update` function.

## Minimal compiling example

```rust
{{#include ../../../examples/counter/src/lib.rs:counter_view}}
```

```rust
{{#include ../../../examples/counter/src/main.rs:counter_main}}
```

Run `cargo test -p mkit-example-counter --locked`. Its library test clicks the button and asserts the retained count is one. On macOS, its screenshot test compares the initial 1280 × 800, scale-2 image above. The pinned headless pixel renderer is unavailable on Windows and Linux.

## How it works

`Counter` keeps `count` as a field. `render` reads it and returns a new element description. The button callback updates the entity and notifies GPUI. The next needed draw uses the new count. The [GPUI frame chapter](../getting-started/how-gpui-thinks.md) explains why describing elements again does not mean the entity data is recreated.

If you come from iced, keep typed GPUI actions and events in mind, but do not look for one required message dispatcher. If you come from egui, keep the persistent data idea, but do not rely on a continuously executed UI function to observe changes; call `notify` for changes that need a redraw.

## Common mistakes

**Assuming a changed field will appear without notification.** A click can update retained state while a cached view remains old. Call `cx.notify()` after the update. The [official Zed discussion about automatic notification](https://github.com/zed-industries/zed/discussions/45246) asks for that automatic behavior; the pinned [`Context::notify`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L228-L232) remains the explicit route.

## Exercises

Change the initial count and check the first rendered label. Restore it before running the screenshot comparison. Then add one GPUI action that changes the count and verify it with a keyboard test; the current button example tests only a mouse click.

## API reference links

- [`Entity`, `Render`, and `Context`](../appendices/api-inventory.md) — retained view data, element description, and notification.
