# Shared configuration with Global

## What you'll build

Configure the title and increment step of the shared counter before its window opens. Run `cargo run -p mkit-example-state-entities` from the workspace root. This page uses the same application and image as [Entities and weak handles](entities.md).

![Shared counter using default global settings, with a Shared counter title and count of zero](../images/state-entities.png)

## Concept

A [**Global**](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/global.rs#L22) is a marker trait for app-wide state stored by type. [`App::set_global`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L2105) installs a value; [`App::global`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L2067) borrows it. [`App::try_global`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L2075) returns `None` when that type has not been installed. This example treats `CounterSettings` as configuration set once at startup. GPUI also has ways to observe and update a global, but this example does not implement dynamic observation.

## Minimal compiling example

The settings type implements `Global` and supplies defaults:

```rust
{{#include ../../../examples/state_entities/src/lib.rs:global_settings}}
```

The model and view read those settings:

```rust
{{#include ../../../examples/state_entities/src/lib.rs:entity_model}}
```

```rust
{{#include ../../../examples/state_entities/src/lib.rs:state_entities_view}}
```

Startup installs the value before opening the window:

```rust
{{#include ../../../examples/state_entities/src/main.rs:state_entities_main}}
```

Run `cargo run -p mkit-example-state-entities`. The library test `entity_updates_and_global_configuration_are_shared_with_the_view` installs a title and step of three, clicks both buttons, and checks counts of three and six. Run it with `cargo test -p mkit-example-state-entities --lib --locked`. On macOS, `cargo test -p mkit-example-state-entities --test screenshot --locked` compares the default initial view with the image above. This is GPUI Kit's initialized default appearance at 1280 × 800 and scale 2.0; alternate themes and scales are not covered by this baseline. Windows and Linux compile the test but report unsupported headless capture.

## How it works

`CounterSettings` holds a title and an `increment_by` value. Its empty `Global` implementation opts that type into GPUI's global storage. `main` calls `gpui_kit::init`, then `cx.set_global(CounterSettings::default())` before opening the window. The render path reads `cx.global::<CounterSettings>().title` for the heading. Each button reads `increment_by` when clicked, then updates the model and calls `notify()` so the displayed count changes. The settings themselves stay unchanged.

The screenshot test uses `CounterWindow::preview()` because `mkit-harness::screenshot` creates the app internally. The preview creates its model on first render. The test's initialization callback installs `CounterSettings::default()` before the capture window is built, matching the runnable app's configuration. The `try_global` method is part of this pinned API, but the current example does not call it; use it when absence is expected rather than making `global()` panic.

## Common mistakes

**Reading before installing.** If `CounterWindow` renders before `CounterSettings` is set, `cx.global::<CounterSettings>()` panics because that type is absent. Install the value before opening the window as `main` and the screenshot callback do, or use `try_global` for optional configuration. The [pinned `App::global` implementation](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L2067-L2079) establishes this behavior. No official Zed report was found for this exact initialization mistake; that report-level evidence remains a review gap. A [Zed discussion about asynchronously initializing a global](https://github.com/zed-industries/zed/discussions/41041) concerns a later dynamic pattern and does not describe a failure of this immutable configuration example.

## Exercises

1. Change `CounterSettings::default().title` to `Team counter`. Run the app and confirm the heading changes. Restore the default before the screenshot comparison.
2. Set `increment_by` to two in `CounterSettings::default()`. Click each button once and confirm the count reaches four. Run the library tests to see their separate custom step of three still yields six. Restore the default before comparing the committed image.

## API reference links

- [Global](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/global.rs#L22), [App](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L686), [Context](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L21), [Entity](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L414), [Render](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L163)
