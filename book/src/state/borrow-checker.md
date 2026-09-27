# Borrow-checker survival guide

## What you'll build

Work with a shared counter without holding a read borrow across an update, then follow two small examples that keep callbacks and async work within GPUI's context rules. The visible counter comes from `examples/state_entities`; run it from the workspace root with `cargo run -p mkit-example-state-entities`.

![Shared counter at zero with strong and weak entity buttons](../images/state-entities.png)

## Concept

GPUI owns each model behind an [`Entity<T>`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L414). Code reads it through an [`App`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L686) or updates it in a closure that receives `&mut T` and a [`Context<T>`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L21). Keep each access short: copy the small value needed for display, finish that read, and then update through a handle. See [Entities and weak handles](entities.md) for the ownership model.

Rust prevents an outstanding immutable borrow of the app from overlapping a mutable update. GPUI adds one runtime rule: while `Entity::update` has leased a particular entity, reading or updating that *same entity* again through its handle panics. The [pinned lease implementation](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L134-L164) and [panic path](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L206-L212) are distinct from a Rust compiler error.

## Minimal compiling example

The shared counter example copies `count` out of `read`, captures strong or weak handles for button callbacks, and updates the model through one closure. These named regions together form its runnable crate:

```rust
{{#include ../../../examples/state_entities/src/lib.rs:global_settings}}
```

```rust
{{#include ../../../examples/state_entities/src/lib.rs:entity_model}}
```

```rust
{{#include ../../../examples/state_entities/src/lib.rs:state_entities_view}}
```

```rust
{{#include ../../../examples/state_entities/src/main.rs:state_entities_main}}
```

Run `cargo run -p mkit-example-state-entities` and `cargo test -p mkit-example-state-entities --locked`. The latter runs the shared-state and released-weak-handle tests and, on macOS, compares the initial view with the image above. That harness capture uses GPUI Kit initialization, a 640 × 400 window, and scale 2.0. Windows and Linux compile the test but report unsupported headless capture.

## How it works

**Read, then update.** `let count = counter.read(cx).count` copies a `u32`; the borrow ends before rendering code retains any callback. The click handler later calls `counter.update(cx, |counter, cx| { ... })` and uses the supplied `counter` directly. [`Entity::read_with`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L470-L474) is another way to return an owned value from a short read closure. The test `entity_updates_and_global_configuration_are_shared_with_the_view` uses it to inspect counts after clicks.

**Do not re-enter the same model.** Inside `counter.update`, the model is already available as `&mut CounterModel`. Read or change its fields there; do not call `counter.read(cx)` or `counter.update(cx, ...)` on that handle from inside the closure. The pinned [`App::update_entity`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L2856-L2873) temporarily removes the entity from its map during the update, so a second access to that entity reaches the lease panic.

**Capture a handle for callbacks.** The two buttons capture a cloned `Entity` or a `WeakEntity`, rather than a borrowed `&mut self`. The weak callback calls `upgrade()` and does nothing if the model has been released. For a callback that should update its *own view*, [`Context::listener`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L248-L261) provides a weak view handle internally; the [hello example](../examples/hello.md) uses it for input and clicks.

The smaller compiling counter example uses `cx.entity()` to capture a strong handle to its own view for the click callback:

```rust
{{#include ../../../examples/counter/src/lib.rs:counter_view}}
```

Run `cargo test -p mkit-example-counter --locked` to check that its button increments retained state. Its macOS screenshot compares the initial view only; the click result is asserted by the unit test.

**Keep subscriptions alive.** The verified `examples/reactivity_views` view stores the `Subscription` values returned by `observe` and `subscribe` in fields. Dropping either handle cancels that callback in this GPUI pin. This anchored region is a supporting excerpt from that compiling crate; its event type and component are defined in the same source file:

```rust
{{#include ../../../examples/reactivity_views/src/lib.rs:reactivity_views_view}}
```

Run `cargo test -p mkit-example-reactivity-views --locked` to check notification and typed event delivery. Its macOS screenshot test covers the initial rendered view at scale 2.0; the subscription assertion is in the unit test, not proven by the screenshot.

**Cross an async boundary with an owned context.** The nonvisual `examples/contexts` crate uses `App::spawn` to receive an [`AsyncApp`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/async_context.rs#L22), then `AsyncApp::update` to return to `&mut App` before updating a weak model handle. It does not hold a borrowed `&mut App` across async work:

```rust
{{#include ../../../examples/contexts/src/lib.rs:contexts_start_refresh}}
```

Run `cargo test -p mkit-example-contexts --locked`; its first test verifies a completed update. The second checks that a weak handle cannot upgrade after the model is released. This example has no window or screenshot. In this pin, `AsyncApp::as_mut` [panics and directs callers to `update`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/async_context.rs#L69-L74).

## Common mistakes

- **Borrow held across an update.** A read reference from `Entity::read` still needs `&App`; trying to call `update` through `&mut App` while using that reference produces a borrow conflict. Return an owned value with `read_with`, or copy a small field as the counter does, before updating. This is a source-backed Rust borrowing pattern; no matching official Zed report was found for this exact synchronous case.
- **Nested update of the same entity.** Code may compile yet panic at runtime because the model is leased for the outer update. Use the `&mut T` already supplied by that closure. The [pinned panic path](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L206-L212) is the evidence; no matching official report was found.
- **Observer stops firing.** A `Subscription` dropped immediately after `observe` or `subscribe` cancels the callback. Store it on the observer, as the reactivity example does; the [pinned `Subscription` contract](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/subscription.rs#L147-L169) establishes that lifetime. No matching official report was found.
- **Borrowing `cx` into background work.** A [Zed discussion about asynchronously initializing a global](https://github.com/zed-industries/zed/discussions/41041) shows a borrow/lifetime problem from moving a context into background work; a reply points to `cx.spawn`. That discussion is a separate dynamic-global scenario, not a test of this example. Here, `App::spawn` supplies `AsyncApp`, and the verified `contexts` tests demonstrate the supported update path for this pin.

## Exercises

1. In the state-entities view, replace the short `read` of `count` with `read_with` returning the `u32`. Run `cargo test -p mkit-example-state-entities --locked`; the two buttons should still add the configured step.
2. In the reactivity view, temporarily remove one stored subscription assignment while leaving the `observe` or `subscribe` call. Run the unit test and inspect the missing notification or event count, then restore the field. The intended implementation retains both handles.
3. In the contexts example, change the placeholder work in `start_refresh` without carrying `&mut App` across an async suspension. Keep the final `AsyncApp::update` boundary and rerun the two context tests.

## API reference links

- [Entity](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L414), [WeakEntity](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L740), [App](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L686), [Context](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L21), [AsyncApp](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/async_context.rs#L22), [Subscription](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/subscription.rs#L150)
