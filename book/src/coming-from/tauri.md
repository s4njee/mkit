# Coming from Tauri

## What you'll build

Run a GPUI counter whose view and click handler are Rust code in the same example binary. Click **Increment** and watch the label change. Run `cargo run -p mkit-example-counter` from the workspace root.

![Counter window with an Increment button and initial count](../images/counter.png)

## Concept

[Tauri 2](https://v2.tauri.app/concept/architecture/) combines a webview frontend with a Rust core. Its documented [IPC](https://v2.tauri.app/concept/inter-process-communication/) uses commands and events to send messages between them. The example here uses GPUI's native Rust element tree. A [`Render`](../appendices/api-inventory.md) view builds that tree from retained Rust state, and its Rust click handler updates the entity directly. No browser DOM, JavaScript frontend, or Tauri command bridge appears in this example.

This is an architecture comparison, not a process-count guarantee. A Tauri webview can have separate processes according to the platform, and a GPUI application can launch workers or use OS services. The specific counter example puts its UI and state logic in one Rust application process.

## Minimal compiling example

```rust
{{#include ../../../examples/counter/src/lib.rs:counter_view}}
```

```rust
{{#include ../../../examples/counter/src/main.rs:counter_main}}
```

Run `cargo test -p mkit-example-counter --locked`. The library test clicks **Increment** and asserts retained count 1. On macOS, the screenshot test compares the initial 1280 × 800, scale-2 image above. This pinned headless renderer does not capture on Windows or Linux.

## How it works

`Counter` owns `count`. Its `render` method reads that value and builds a label and GPUI Kit button. The button callback gets the `Counter` entity, calls `update`, increments `count`, and calls `cx.notify()`. GPUI then redraws the changed view. The test reaches the button through its debug selector and checks the entity state. The [counter chapter](../examples/counter.md) walks through those lines.

Tauri's frontend can call Rust commands through its IPC model; this GPUI example calls Rust code from a Rust handler. GPUI itself does not provide a Tauri webview or Tauri IPC contract in the pinned public API. If an app needs web content, that requires a separately chosen integration and its own evidence.

## Exercises

Add a second button action to the example crate and assert the resulting count with a GPUI test. Check the visual state in a harness screenshot before changing its baseline. Do not add a JavaScript layer to explain this counter's existing update path.

## API reference links

- [`Render`, `Entity`, `Context`, and `Window`](../appendices/api-inventory.md) — the Rust view, retained state, update context, and native window.
