# Views and components

## What you'll build

Run one retained view that uses two small, reusable display components. The view shows counts, while `StatusPill` supplies the dark label and `DemoSurface` supplies the themed background. Run `cargo run -p mkit-example-reactivity-views` from the workspace root.

![Light reactivity view with three count lines, a dark RenderOnce label, and Change source text](../images/reactivity-views.png)

## Concept

Use [`Render`](../appendices/api-inventory.md) for a *view* whose entity retains state. Its method borrows `&mut self` and receives [`Context<Self>`](../appendices/api-inventory.md). `ReactivityDemo` is such a view: it keeps a source handle, counts, and subscriptions.

Use [`RenderOnce`](../appendices/api-inventory.md) for a component built from supplied data. Its method consumes `self` and receives [`App`](../appendices/api-inventory.md). `StatusPill` owns the text it displays. `DemoSurface` owns one [`AnyElement`](../appendices/api-inventory.md) child. Neither component owns the changing count. The [pinned trait definitions](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L160-L183) show the different method signatures.

[`IntoElement`](../appendices/api-inventory.md) is the conversion contract that lets a value become an element in the tree. `#[derive(IntoElement)]` gives each `RenderOnce` component that conversion; the pinned [element trait](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L145-L158) and [derive documentation](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L173-L183) describe this pairing. The example also uses `into_any_element()` for the mixed children stored inside `DemoSurface`.

## Minimal compiling example

Both components are included from the compiling crate:

```rust
{{#include ../../../examples/reactivity_views/src/lib.rs:reactivity_views_component}}
```

The root view constructs them. Its typed event declaration is also part of the same crate:

```rust
{{#include ../../../examples/reactivity_views/src/lib.rs:reactivity_views_events}}
```

```rust
{{#include ../../../examples/reactivity_views/src/lib.rs:reactivity_views_view}}
```

```rust
{{#include ../../../examples/reactivity_views/src/main.rs:reactivity_views_main}}
```

Run `cargo run -p mkit-example-reactivity-views`. Run `cargo test -p mkit-example-reactivity-views --lib --locked` for the single click and rendered-count check. On macOS, `cargo test -p mkit-example-reactivity-views --test screenshot --locked` checks this initial image at 1280 × 800 and scale 2.0. Windows and Linux report unsupported headless capture with the pinned GPUI release. The tests do not cover other themes or scales.

## How it works

1. `extern crate gpui_pre as gpui` supplies the conventional `gpui` path used by this pinned `IntoElement` derive. The source still imports public types through `gpui_pre`; the alias supports the derive expansion.
2. `StatusPill::new` takes text. Its `RenderOnce::render` consumes the component, reads the active GPUI Kit theme, and builds a padded, rounded `div` with `primary` and `primary_foreground` colors.
3. `DemoSurface::render` reads the same theme's `background` and `foreground`, then places its `AnyElement` child inside a full-size `div`. `gpui_kit::init(cx)` in `main` initializes the Kit layer before the view is opened, so the theme lookup has an active theme.
4. `ReactivityDemo::render` reads its retained fields and builds labels. It constructs `StatusPill::new("RenderOnce component")` and wraps the whole tree in `DemoSurface`. The `RenderOnce` values are recipes produced for this render; the `ReactivityDemo` entity holds the state that must persist.
5. The root `div` uses an `on_click` handler for **Change source**. The lib test checks one mouse click and the resulting rendered observed count. This example does not establish keyboard, accessibility, or alternate-theme behavior.

## Common mistakes

**Creating a new stateful entity inside every render and expecting it to keep its old value.** The count appears to reset because the prior entity is no longer retained. Store its handle in a retained owner, as `ReactivityDemo.source` does, and create it once. An [older official Zed GPUI discussion](https://github.com/zed-industries/zed/discussions/7120) reports a counter that never increments when recreated each frame; the answer explains that a live reference path is needed. That discussion predates `gpui-pre` 0.3.5, so use the [pinned `Entity` definition](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L414-L430) and this compiling example for current API names.

## Exercises

1. Change the text passed to `StatusPill::new`. Run the app and check that only the pill label changes. Restore it before comparing the existing screenshot baseline.
2. Change `DemoSurface` to use another active-theme background token. Run the app and inspect the surface. Restore the original token before the screenshot test; this exercise has no committed alternate-theme baseline.

## API reference links

- [`gpui::Render`, `gpui::RenderOnce`, and `gpui::IntoElement`](../appendices/api-inventory.md) — stateful view, consuming component, and element conversion.
- [`gpui::Context`, `gpui::App`, `gpui::AnyElement`, and `gpui::Entity`](../appendices/api-inventory.md) — contexts, child storage, and retained state.
