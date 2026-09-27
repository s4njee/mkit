# E7.21 — In-window menu bar

`MenuBar` renders a horizontal application menu inside the window and uses an explicit
`MenuBarModel` for both the in-window surface and GPUI native-menu conversion. `to_gpui_menus()`
produces `gpui_pre::Menu` values suitable for `App::set_menus`, so command actions and menu labels
come from one source of truth.

Top-level menus support optional explicit mnemonics. A `MenuBar` entity emits
`OpenMenuChanged(Option<String>)` for controlled and uncontrolled popup state and
`CommandInvoked(String)` after it dispatches an enabled leaf action. Call `set_open_menu` to apply
controlled requests. The public model/API and its keyboard/accessibility contract are drafts
pending maintainer review.

## Platform boundary

The GPUI macOS backend installs native application menus through `App::set_menus`. GPUI's Windows
and Linux backends retain the menu model but do not render a native in-window menu bar, so the
component provides that visible surface. Rendering it on macOS is optional; applications may use
the native bar instead. A native conversion preserves labels, nested submenus, actions, disabled
and checked values; explicit mnemonics and displayed shortcut text are in-window-only because the
GPUI native model has no mnemonic property and derives shortcut display from the app keymap.

## Verification

Focused GPUI tests pass for keyboard action navigation, disabled/separator skipping, Escape
dismissal, Alt mnemonic selection, controlled open requests, and native-model conversion. The
gallery update and compare runs pass across all 24 idle, focused, open, and disabled combinations
in three themes at 1× and 2×. Inspected focused dark 2×, open dark/light 2×, idle dark 2×, and
disabled high-contrast 2×. Native accessibility, hardware Alt behavior, and menu coexistence have
not been validated on Windows, Linux, and macOS hosts.
