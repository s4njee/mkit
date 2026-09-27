# Reactivity: notifications and events

## What you'll build

Run a small app with a source count and two ways to react to it. Click **Change source** once. The observed count, notification count, and typed event count should each become one. Run `cargo run -p mkit-example-reactivity-views` from the workspace root.

![Initial reactivity view with observed count, notifications, and typed events all at zero](../images/reactivity-views.png)

## Concept

An [entity](../appendices/api-inventory.md) keeps data between renders. Here, `Source` owns `count`, and `ReactivityDemo` keeps a handle to that source. Calling [`Context::notify`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L228-L232) says an entity changed. [`Context::observe`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L60-L82) receives that change notice and a handle to the changed entity. The observer reads the new count. The notice itself has no `CountChanged` payload.

A *typed event* carries a value chosen by the source. `Source` implements [`EventEmitter<CountChanged>`](../appendices/api-inventory.md), which allows [`Context::emit`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L782-L800) to send `CountChanged { count }`. [`Context::subscribe`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L100-L123) receives that payload. Emitting and notifying are separate calls; this example makes both after one change.

Both methods return a [`Subscription`](../appendices/api-inventory.md). The view stores the handles in `_observation` and `_event_subscription`. [Dropping a subscription cancels its callback](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/subscription.rs#L150-L194), so retaining the handles matters.

## Minimal compiling example

The event type and emitter contract come from the compiling example crate:

```rust
{{#include ../../../examples/reactivity_views/src/lib.rs:reactivity_views_events}}
```

The root view owns the source and both subscriptions. It renders through the components shown in the next included region:

```rust
{{#include ../../../examples/reactivity_views/src/lib.rs:reactivity_views_view}}
```

```rust
{{#include ../../../examples/reactivity_views/src/lib.rs:reactivity_views_component}}
```

The binary opens the view:

```rust
{{#include ../../../examples/reactivity_views/src/main.rs:reactivity_views_main}}
```

Run `cargo run -p mkit-example-reactivity-views`. Run `cargo test -p mkit-example-reactivity-views --lib --locked` to check one click, including the rendered `Observed count: 1` selector and the stored `(1, 1, 1, 1)` counters. On macOS, `cargo test -p mkit-example-reactivity-views --test screenshot --locked` compares the initial 1280 × 800 image at scale 2.0 with the PNG above. The pinned GPUI release has no headless screenshot renderer on Windows or Linux; the screenshot test reports that limit there. This image is an initial-state baseline, not an interaction screenshot.

## How it works

1. The first render calls `initialize`. It creates one `Source` entity and returns early on later renders, so a redraw does not reset the source.
2. `cx.observe(&source, ...)` registers a change callback. When the source notifies, the callback reads `source.read(cx).count`, stores it as `observed_count`, raises `notification_count`, and notifies the root view so its labels can update.
3. `cx.subscribe(&source, ...)` registers a callback for the `CountChanged` event. It raises `event_count`, copies `event.count` into `last_event_count`, and notifies the root view. The two returned `Subscription` handles stay in the root view.
4. The click handler updates `Source`. It raises `source.count`, emits `CountChanged` with the new count, and calls `cx.notify()`. The event callback receives the payload; the observer receives a change notice and reads the source. Neither callback is a replacement for the other.
5. The test draws the view, clicks the element found by `debug_selector("increment")`, draws again, and finds `observed-count-1`. It also reads the four stored values and asserts `(1, 1, 1, 1)`. The example does not register keyboard actions or an accessibility role for the click target, so these checks establish mouse interaction and rendering only.

## Common mistakes

**Expecting a state update to notify observers automatically.** The source can change while its observer's displayed value stays old. Call `cx.notify()` after a change that observers must see. An [official Zed GPUI discussion](https://github.com/zed-industries/zed/discussions/45246) proposes automatic notification; it is a proposal, not a feature in pinned `gpui-pre` 0.3.5. The pinned [`observe` contract](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L68-L73) explicitly ties the callback to notification.

**Dropping a returned subscription immediately.** The callback stops running, so the corresponding counter will not advance. Keep the handle in a retained owner such as `ReactivityDemo`; the pinned [`Subscription::drop` implementation](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/subscription.rs#L188-L194) explains why. This is source-grounded guidance; we did not find a matching official issue or discussion to cite for this separate mistake.

## Exercises

1. Temporarily remove `cx.emit(...)` but leave `cx.notify()`. Click once and compare **Observed count** with **Typed events**. Restore the call before running the existing test.
2. Temporarily remove the source's `cx.notify()` but leave `cx.emit(...)`. Click once and compare the two counters. Restore the call before running `cargo test -p mkit-example-reactivity-views --lib --locked`.

## API reference links

- [`gpui::Entity`, `gpui::Context`, and `gpui::Subscription`](../appendices/api-inventory.md) — retained source, callbacks, and their lifetimes.
- [`gpui::EventEmitter` and `gpui::Render`](../appendices/api-inventory.md) — the typed event contract and root view.
