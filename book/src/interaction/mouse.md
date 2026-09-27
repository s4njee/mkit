# Pointer input and hit testing

## What you'll build

A list row that responds to hover, press, click, and drag. Run `cargo run -p mkit-example-interaction --locked` from the workspace root.

![Interaction list with the first row selected and pointer status below it](../images/interaction.png)

## Concept

A *hitbox* is the region GPUI uses to route pointer input to an element. A `div` can attach pointer listeners such as `on_click`, `on_hover`, and `on_drag`. Its `hover` and `active` style methods change appearance; `active` is a style state, not a separate event. GPUI's frame hit testing is internal, so observe the routed events instead of calling it directly. See the [pinned event handlers](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L126-L390) and [hitbox implementation](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L822-L882).

## Minimal compiling example

This region comes from the compiling `examples/interaction/src/lib.rs`.

```rust
{{#include ../../../examples/interaction/src/lib.rs:interaction_pointer}}
```

Run `cargo test -p mkit-example-interaction --lib --locked` to check pointer and drag behavior. The macOS baseline above is checked by `cargo test -p mkit-example-interaction --test screenshot --locked` at 1280 × 1040 and scale 2.0; the pinned headless renderer does not capture other platforms.

## How it works

Each row gets an ID and a debug selector so the test can find its bounds. `on_hover` updates the hovered row; left-button down and up update pressed state; `on_click` changes selection. Each handler calls `notify` so the state is rendered again. `hover` and `active` use theme colors. The pointer test checks entry, pressed state, and the click on the second row. It does not test releasing outside the hitbox. The [pinned `div` methods](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L570-L704) define the callbacks.

## Common mistakes

**Hover appears on the wrong cell during a live resize.** [Zed issue #57354](https://github.com/zed-industries/zed/issues/57354) reports stale pointer coordinates during a macOS live resize in a later GPUI version. The report does not establish a bug in `gpui-pre` 0.3.5. Test the resize path on the target platform; do not use an idle-window hover test as evidence for it.

## Exercises

Move the pointer across a row boundary and watch “Hover row” change. Press and release on the second row; “Pressed” should return to false and “Clicks” should rise by one. As a new test, release outside the row and assert the resulting selection. The current test does not cover that path.

## API reference links

- [`div` pointer methods and style states](../appendices/api-inventory.md) · [pinned source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L570-L704)
