# Hello, GPUI

## What you'll build

Open a GPUI window with a counter and a small keyboard readout. The example teaches how an application creates a root view and how that view redraws after input. From the repository root, run `cargo run -p mkit-example-hello`.

![Hello window with a dark blue background, a zero-click counter, and empty keyboard readouts](../images/hello.png)

## Concept

An [**Application**](../appendices/api-inventory.md) owns GPUI's startup and event loop. Its launch callback receives an [**App**](../appendices/api-inventory.md), the mutable application context used to create a [**Window**](../appendices/api-inventory.md). The window has one root view: an [**Entity**](../appendices/api-inventory.md) that owns state and implements [**Render**](../appendices/api-inventory.md). GPUI asks that view for an element tree to draw. This is the [startup and view model in the pinned `gpui-pre` 0.3.5 guide](https://docs.rs/crate/gpui-pre/0.3.5).

## Minimal compiling example

The root view lives in the compiling `examples/hello` crate:

```rust
{{#include ../../../examples/hello/src/lib.rs:hello_view}}
```

The binary starts the application and opens the window:

```rust
{{#include ../../../examples/hello/src/main.rs:hello_main}}
```

Run `cargo run -p mkit-example-hello`. On macOS, `cargo test -p mkit-example-hello --test screenshot` captures this view with `mkit-harness` and compares it with the image above. The committed capture uses GPUI Kit initialization, a 640 × 400 logical window, and scale 2.0. Windows and Linux compile the example but skip image comparison because this GPUI pin has no headless renderer there.

## How it works

[`gpui_platform::application()`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_platform/src/gpui_platform.rs#L13) chooses a platform implementation and returns an `Application`. Its `run` callback starts after launch and receives `&mut App`. The example initializes GPUI Kit before creating a window, then passes default [**WindowOptions**](../appendices/api-inventory.md) to `open_window`. The closure uses the [**AppContext**](../appendices/api-inventory.md) `new` method to create `InspectorFixture::hello()` as the root entity. `activate(true)` asks the platform to bring the app forward. These steps follow the [pinned `Application::run`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L234-L245) and [`App::open_window`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L1283-L1290) source.

`InspectorFixture` stores the click count, typed characters, and last key. Its `render` method builds the visible text and button. On the first render it creates a [**FocusHandle**](../appendices/api-inventory.md), focuses the view, and attaches the handle to its element. A mouse click increments the count. A key-down listener records the key and appends one-character keys to the typed line. Each listener calls [**Context**](../appendices/api-inventory.md) `notify()`, which tells GPUI the entity changed so its view can render again. The [pinned `Render` trait](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L163-L166) and [`Context::notify`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L230-L232) define those contracts.

## Common mistakes

**Typing changes nothing.** The key listener is on the focused view; check that the window is active and that the view has its focus handle. This example calls `window.focus` and `track_focus` explicitly. A [Zed discussion about initial focus in dialogs](https://github.com/zed-industries/zed/discussions/57205) reports that focusing a dialog container does not automatically focus its child input. That report is about a later dialog scenario, not a claim that this hello view has the same bug. The practical check here is to click the hello window, type a letter, and confirm that both readouts change.

## Exercises

1. Change the initial `clicks` value in `InspectorFixture::new` to `5`. Run the binary and confirm the label starts at `Clicks: 5`. Restore `0` before running the screenshot test against the committed baseline.
2. Change the click handler to add `2` instead of `1`. Run the binary, click once, and check that the label reads `Clicks: 2`. Restore the original handler before comparing the baseline.

## API reference links

- Startup: [Application](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L144), [App](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L686), [Window](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L1144), [WindowOptions](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2044)
- State and rendering: [AppContext](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L172), [Entity](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L414), [Render](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L163), [Context](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L21)
- Input and focus: [FocusHandle](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L527), [KeyDownEvent](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L25)
