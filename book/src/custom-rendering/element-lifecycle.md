# Write a custom Element

## What you'll build

A small custom element that records its layout, prepaint, and paint phases and draws a shape. Run `cargo test -p mkit-example-custom-rendering --lib --locked` to check its phase order and bounds. This element has no dedicated screenshot baseline.

## Concept

An `Element` participates in GPUI's layout and paint tree. `request_layout` asks for space and returns state; `prepaint` receives final bounds and can register a hitbox; `paint` draws using that state. The [pinned trait](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L51-L116) defines the phase order and associated state types. A custom element needs an ID and accessibility role if it should appear in the accessibility tree.

## Minimal compiling example

```rust
{{#include ../../../examples/custom_rendering/src/lib.rs:custom_element}}
```

The source compiles in `examples/custom_rendering`. Run `cargo test -p mkit-example-custom-rendering --lib --locked`; `custom_element_runs_layout_prepaint_then_paint` checks this element.

## How it works

`PhaseElement` requests layout, retains final bounds in prepaint, and paints a quad. The test records the three phases in order and checks that prepaint received positive bounds. The element does not register a hitbox or accessibility role; those behaviors are not claimed.

## Exercises

Change the requested size and assert the prepaint bounds change. Then add a hitbox and a test for it. An accessibility role would need its own tree and assistive-action checks.

## API reference links

- [`Element`, `IntoElement`, and `GlobalElementId`](../appendices/api-inventory.md) · [pinned trait](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L51-L116)
