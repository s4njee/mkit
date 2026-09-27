# Dropdown menu

> **Draft E7 component.** Its public API, accessibility contract, and visual baselines still need maintainer review. It is not marked `source_ready` for `cargo mkit add`.

Use a dropdown menu to put several commands behind a button without crowding the main screen. The app supplies the button and the point where the menu opens.

## When to use it

- Put Rename, Duplicate, and Archive behind a file toolbar button.
- Group export formats in a submenu while keeping common commands at the top level.
- Show a checked “Show hidden files” command alongside ordinary commands.

## Preview

![Dropdown menu in its open state](../../images/e7/dropdown-menu.png)

This is a checked-in dark-theme, 2× screenshot baseline of one state. The component spec lists the other captured states, themes, and scales.

## API and state

`Entity<DropdownMenu>`; uncontrolled `new(items)` and controlled `controlled(items, open)` both expose `set_open`. `OpenChanged(bool)` requests open state changes; in controlled mode the owner must call `set_open` to apply them. `ItemSelected(id)` is emitted for enabled command items, and dismisses the menu. `CheckedChanged(id, checked)` is emitted for enabled checkable items; toggling a checkable item keeps the menu open. Items have id, label, disabled, optional shortcut, checked and children.

## Keyboard and pointer behavior

| Key | Behaviour |
|---|---|
| Up / Down | Move among enabled items in the current pane, wrapping at either end. |
| Right | Open the active enabled item's submenu and focus its first enabled item. |
| Left | Close the current submenu and restore its parent item as active. |
| Enter / Space | Open an active submenu or activate its leaf. |
| Escape | Request dismissal of the root menu. |
| Home / End | Focus the first / last enabled item in the current pane. |

Pointer movement activates enabled rows. Clicking a submenu row opens its child pane; clicking a
leaf activates it. Checkable items toggle and keep the menu open. The host owns the trigger and
supplies the point through `anchor_at` or `set_anchor`; the menu routes outside pointer-down
dismissal itself. `return_focus_to` supplies the invoking element, or the menu captures the
previously focused handle when it opens. Escape and outside dismissal emit `OpenChanged(false)`
and reset the active path. Reopening starts at the first enabled root item. Displayed shortcuts
do not invoke commands and do not install key bindings.

## Accessibility

menu; trigger button has aria-haspopup=menu and aria-expanded; entries use menuitem or menuitemcheckbox and expose their label as the accessible name. Disabled entries set the AccessKit disabled state. Checkable entries expose their toggled state, submenu parents expose expanded state, and the active row is reported through GPUI’s active-descendant accessibility mapping.

## Theme

Read colors, typography, spacing, radii, border widths, and control sizing from the GPUI Global `Theme` tokens. Enabled, non-active menu rows use `Theme.colors.text`; disabled rows use `Theme.colors.disabled`; shortcut labels and submenu indicators use `Theme.colors.text_muted`. In shadcn themes, the pane uses `surface`, and the pointer-hovered or keyboard-active row keeps `text` over the web preview's subtle `text`/`background` mix (4% text in light, 12% in dark), with `radii.small` corners and the pane's `spacing.xsmall` inset. Other themes retain their accent/contrast active colors and `elevated_surface` pane. Shadows and motion are not used by this menu surface. Do not hard-code colors.

## Current limits

Menu commands and displayed shortcuts are separate: showing a shortcut does not register it.

For the exact state and event contract, see the checked-in `registry/dropdown-menu/spec.md`. The [E7 overview](../everyday-components.md) tracks current test evidence; these pages are documentation drafts, not a claim that the full E5 conformance matrix has passed.
