# Build a layout with `div`

## What you'll build

Build the shell of a settings screen: a sidebar beside a header and two-column group of settings. Run `cargo run -p mkit-example-settings-screen` from the workspace root.

![Settings screen with a left sidebar, header, and two-column Workspace defaults cards](../images/settings-screen.png)

## Concept

[`div()`](../appendices/api-inventory.md) creates a container element. A container can accept child elements through [`ParentElement`](../appendices/api-inventory.md) and layout instructions through [`Styled`](../appendices/api-inventory.md). Fluent calls such as `flex()`, `flex_col()`, and `grid()` update the container's style; they do not create new child state.

GPUI's [`Display`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1131-L1140) values map to [Taffy's `Display`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1279-L1288) for layout. Flex positions children along an axis; grid assigns them to tracks. The pinned [`Styled::grid_cols`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L752-L760) sets a repeated column template. This is GPUI's API, so do not assume every browser CSS grid property exists here.

## Minimal compiling example

The layout is an anchored part of the compiling settings-screen crate:

```rust
{{#include ../../../examples/settings_screen/src/lib.rs:settings_screen_layout}}
```

Run `cargo run -p mkit-example-settings-screen`. On macOS, `cargo test -p mkit-example-settings-screen --test screenshot --locked` compares the initial screen above at 1920 × 1280 and scale 2.0 after GPUI Kit initialization. On Windows and Linux, the test compiles and reports that this GPUI pin has no headless screenshot capture.

## How it works

Read the included code from the outer `div` inward. The shell supplies the available area and arranges a header and content region. The content's flex and grid builders choose each child's placement. [`ParentElement::child` and `children`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L188-L208) attach elements to those containers.

The builder call order matters when setting the same style field twice: [`flex()` and `grid()`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L45-L53) both set `display`, so the later call supplies the final display mode. The sidebar is a fixed 224-pixel flex child and the main area grows to fill the rest. `workspace_cards` uses `grid().grid_cols(2)` for the Language and Appearance cards. The screenshot shows both cards in one row.

## Exercises

1. Change the settings grid from two columns to one in the example. Run the app and check that the fields stack vertically. Restore two columns before comparing its committed screenshot.
2. Swap the shell's flex direction. Run the app and observe where the header moves, then restore the original direction.

## API reference links

- [`Div`, `Styled`, `ParentElement`, and `Display`](../appendices/api-inventory.md) — container, fluent style, children, and layout mode.
- [`GridTemplate` and `FlexDirection`](../appendices/api-inventory.md) — pinned layout values used by the builders.
