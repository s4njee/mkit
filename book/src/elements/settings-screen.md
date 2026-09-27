# Put the settings screen together

## What you'll build

Run a complete settings window with a sidebar, scrollable preferences, a landscape preview, a sync status card, notification toggle, and keyed workspace rows. From the workspace root, run `cargo run -p mkit-example-settings-screen`.

![Initial settings screen showing the sidebar, Workspace defaults cards, Desktop alerts, and Display section](../images/settings-screen.png)

![Settings screen scrolled to the bottom, showing Display, sync status, and three pinned workspaces](../images/settings-screen-scrolled.png)

## Concept

A GPUI screen is a tree of elements returned by a retained [`Render`](../appendices/api-inventory.md) view. The settings view holds an [`Entity<SettingsState>`](../appendices/api-inventory.md) for preferences once initialized and an in-memory image. Each render reads that state, builds a fresh tree, and uses GPUI Kit's active theme colors. [`gpui_kit::init`](https://docs.rs/crate/gpui-component/0.6.4/source/src/theme/mod.rs#L35-L42) installs the default theme before the view reads [`ActiveTheme`](https://docs.rs/crate/gpui-component/0.6.4/source/src/theme/mod.rs#L44-L53).

The focused chapters explain [layout](div-and-layout.md), [sizing and scroll](size-and-scroll.md), [text](text.md), [media](images-and-svg.md), and [conditional lists](conditional-lists.md). This page follows their execution together.

## Minimal compiling example

The runnable entry point comes from the example crate:

```rust
{{#include ../../../examples/settings_screen/src/main.rs:settings_screen_main}}
```

The root view assembles its parts:

```rust
{{#include ../../../examples/settings_screen/src/lib.rs:settings_screen_layout}}
```

Run `cargo run -p mkit-example-settings-screen`. Run `cargo test -p mkit-example-settings-screen --lib --locked` for the notification transition and scroll input. On macOS, `cargo test -p mkit-example-settings-screen --test screenshot --locked` compares top and bottom 1920 × 1280 captures at scale 2.0 after GPUI Kit initialization. Windows and Linux compile the screenshot test but report unsupported headless capture. The images do not show the toggled state; the library test checks it.

## How it works

`main` creates the platform application, calls `gpui_kit::init(cx)`, opens a window with `SettingsScreen::default()`, and activates the application. The view default decodes the embedded landscape once. On its first render, it creates the preferences entity, then observes settings changes, reads the two current switches, and gets colors from the active GPUI Kit theme. This deferred entity setup also lets the headless harness construct the same view without an `App` argument.

The outer `div` divides the window into a fixed-width sidebar and a flexible main area. The main area holds a header and bounded scrollable content. Content functions produce the settings cards, notification and display sections, sync card, and pinned-workspace list. Clicking the desktop-alert control changes the entity and notifies the view; the next render shows the other conditional note. The library test verifies that transition. The fixture currently uses pointer clicks; it does not establish keyboard operation, focus behavior, or an accessibility contract for these controls. Those belong to the later interaction chapters.

## Exercises

1. Start the app and click **Desktop alerts**. The label and explanatory note should change. Run the library test to check the same stored state and rendered note.
2. Change the two-column workspace cards to one column, then restore them before the screenshot comparison. Use the [layout chapter](div-and-layout.md) to trace that builder call.

## API reference links

- [`Application`, `App`, `WindowOptions`, `Render`, `Context`, `Entity`, and `Div`](../appendices/api-inventory.md) — application startup, retained view, state, and tree.
- [GPUI Kit 0.6.4 theme initialization and `ActiveTheme`](https://docs.rs/crate/gpui-component/0.6.4/source/src/theme/mod.rs#L35-L53) — theme global used by the example.
