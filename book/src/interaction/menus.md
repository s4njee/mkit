# Application and context menus

## What you'll build

An application menu and a row context menu with an Activate action. Run `cargo run -p mkit-example-interaction --locked` from the workspace root.

![Interaction list whose rows offer a context menu; the idle screenshot does not show a popup](../images/interaction.png)

## Concept

An *application menu* belongs to the desktop app. Pinned GPUI supplies `Menu`, `MenuItem`, and `App::set_menus` for it. On macOS, this pin builds an `NSMenu` and installs it as the app's main menu. Its [Windows implementation](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_windows/src/platform.rs#L811-L819) and [Linux implementation](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_linux/src/linux/platform.rs#L683-L691) store the menu definitions; those methods do not create a visible menu bar. A *context menu* belongs to content under the pointer; this workspace's pinned GPUI Kit supplies `ContextMenuExt::context_menu`. `Window::show_window_menu` opens the native title-bar window menu and is not the content-menu API. See [core menus](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L4-L114), [app registration](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L2465-L2476), [macOS installation](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_macos/src/platform.rs#L1077-L1089), and [GPUI Kit context menu](https://github.com/longbridge/gpui-kit/blob/3c387ae0a3e9b14ee39fe98be2b51a882800aa16/crates/component/src/menu/context_menu.rs#L13-L42).

## Minimal compiling example

The application menu setup comes from the compiling `examples/interaction/src/lib.rs`. The row context menu is attached inside `interaction_list` in the same file.

```rust
{{#include ../../../examples/interaction/src/lib.rs:interaction_menus}}
```

Run `cargo test -p mkit-example-interaction --lib --locked` to check application-menu registration and keyboard selection in the GPUI Kit row popup. The idle screenshot above is checked by `cargo test -p mkit-example-interaction --test screenshot --locked` on macOS at 1280 × 1040 and scale 2.0. It does not show the open popup.

## How it works

`set_interaction_menus` registers Activate and Add Row application menu items. Each row attaches a GPUI Kit popup item containing `ActivateSelection`, the same action handled by the focused list. A headless test opens the row popup with a right click, checks that Down is consumed there, then presses Enter and verifies activation. Registration is confirmed by the compiling fixture, but no test selects a native application-menu item, checks a displayed accelerator, returns focus from the popup, or inspects menu accessibility.

## Native verification

On an unlocked macOS desktop, run the example, keep its window active, and open the **List** menu in the system menu bar. Choose **Activate selected row** and confirm that the `Activated` count in the window increases by one. Choose **Add row** and confirm that a new row appears. Note whether the visible menu labels and shortcut equivalents match the registered actions, and whether keyboard navigation and VoiceOver announce the menu items. Then right-click a row, use Down and Enter to activate the popup item, and check whether focus returns to the list after dismissal. Record the OS version, result, and any failure separately. These native checks have not been run for this chapter. On Windows or Linux, the pinned `set_menus` implementations alone are not evidence of a visible native application menu; the GPUI Kit row popup is a separate path.

## Common mistakes

No directly relevant official Zed issue or discussion has been verified for confusing title-bar menus with content menus. The [pinned `show_window_menu` source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L2808-L2812) limits it to the native title-bar menu. Use GPUI Kit's content context-menu extension for a row, and keep native application-menu selection in the manual check.

## Exercises

Use the native verification steps above on macOS. The headless test already checks the row popup's Enter path.

## API reference links

- [`Menu`, `MenuItem`, `App::set_menus`, `Window::show_window_menu`](../appendices/api-inventory.md) · [pinned core source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L4-L114)
- [`ContextMenuExt::context_menu` in GPUI Kit](https://github.com/longbridge/gpui-kit/blob/3c387ae0a3e9b14ee39fe98be2b51a882800aa16/crates/component/src/menu/context_menu.rs#L13-L42)
