---
spec_version: 1
component: context-menu
states:
  - id: closed
    description: Context Menu closed state.
    fixture: context-menu_closed
  - id: open
    description: Context Menu open state.
    fixture: context-menu_open
  - id: outside_dismissed
    description: An outside pointer press requests close and returns focus to the host target after close applies.
    fixture: context-menu_outside_dismissed
  - id: submenu
    description: Context Menu with the active item's child pane open.
    fixture: context-menu_submenu
keys:
  - key: Escape
    modifiers: []
    when: component is open
    action: Request dismissal and return focus to the host focus target after close applies.
    initial_state: open
    expect:
      event: open_changed
  - key: ArrowRight
    modifiers: []
    when: an enabled active item has children
    action: Open its nested pane and move active focus to its first enabled item.
    initial_state: open
    expect:
      focus_target: submenu-first-enabled
  - key: ArrowLeft
    modifiers: []
    when: a nested pane is open
    action: Return to the parent pane and restore its parent item as active.
    initial_state: submenu
    expect:
      focus_target: submenu-parent
  - key: Home
    modifiers: []
    when: component is open
    action: Move active focus to the first enabled item in the current pane.
    initial_state: open
    expect:
      focus_target: first-enabled
  - key: End
    modifiers: []
    when: component is open
    action: Move active focus to the last enabled item in the current pane.
    initial_state: open
    expect:
      focus_target: last-enabled
accessibility:
  role: menu
  properties:
    - name: state
      value: closed
      when: closed
    - name: item_names
      value: visible item labels
      when: open
    - name: disabled_item
      value: AccessKit disabled state
      when: open
---

# Context Menu

## Purpose

Pointer context menu for commands including nested and checkable items.

## Anatomy

The host supplies the invocation gesture, point anchor, and optional focus return target. The
component renders its panes in a GPUI deferred window-anchored surface and one adjacent pane for
each active nested level. Rows contain an optional check mark, label, optional shortcut label, and
submenu indicator.

## States

`closed` hides every pane; `open` shows the root pane; `submenu` shows the root and child panes; `outside_dismissed` is the closed state after a real outside pointer-down and focus restoration.

### Screenshot fixture contract (E7.5)

The screenshot matrix is the Cartesian product of `closed`, `open`, `outside_dismissed`, and `submenu` with the `light`, `dark`, and `high-contrast` shadcn themes at 1x and 2x scales. The gallery fixture is a 480×320 logical-pixel window with a `ContextMenu` heading and a focusable host target named `Invocation target`. It anchors the menu at window point (136, 88) and uses the following deterministic item tree:

- Source-order items: `arrange` / Arrange (submenu); child items `bring-forward` / Bring forward (shortcut `⌘]`) and `send-backward` / Send backward (shortcut `⌘[`); `rename` / Rename (shortcut `F2`); `pin` / Pin to top (checked checkable item); `remove` / Remove (disabled).
- The fixture creates and retains one `Entity<ContextMenu>`, uses the uncontrolled `new(items)` API, calls `set_open(true)` to model the host context-invoked, and supplies the host focus handle through `return_focus_to`.
- `closed`: leave the menu closed; assert `is_open() == false`.
- `open`: focus the host target, open through the host's `set_open(true)` call, and assert `is_open() == true`; the anchored menu must be visible in the capture.
- `submenu`: focus the menu entity after opening, dispatch the registered `right` action, and assert it remains open with the menu focus handle still focused. The screenshot must show both root and child panes.
- `outside_dismissed`: focus the host target, open the menu, focus the menu entity, then dispatch a real left-button mouse-down at window point (456, 292), outside the anchored surface. After the close has rendered, assert it is closed and focus has returned to `Invocation target` before capture.

The menu overlay is intentionally kept well inside the window in the `open` and `submenu` fixtures so the visible pane contents are not clipped. The dismissed capture shows the host target with the menu surface absent.

## Props and events

Entity<ContextMenu>; uncontrolled `new(items)` and controlled `controlled(items, open)` both expose `set_open`. `OpenChanged(bool)` requests open state changes; in controlled mode the owner must call `set_open` to apply them. `ItemSelected(id)` is emitted for enabled command items, and dismisses the menu. `CheckedChanged(id, checked)` is emitted for enabled checkable items; toggling a checkable item keeps the menu open. Items support shortcuts, disabled, checked and children.

## Keyboard map

| Key | Behaviour |
|---|---|
| ContextMenu / Shift+F10 | Host opens at the focused item's anchor. |
| Up / Down | Move among enabled items in the current pane, wrapping at either end. |
| Right / Left | Open an active submenu / return to its parent item. |
| Enter / Space | Open an active submenu or activate its leaf. |
| Escape | Request dismissal of the root menu. |
| Home / End | Focus the first / last enabled item in the current pane. |

## Pointer behaviour

Pointer movement activates enabled rows. Clicking a submenu row opens its child pane; clicking a
leaf activates it. Checkable items toggle and keep the menu open. The host owns the opening gesture and supplies the point through `anchor_at` or `set_anchor`. The
menu routes outside pointer-down dismissal itself. `return_focus_to` supplies the invoking element;
without one, the menu captures the previously focused handle when it opens. Escape and outside
dismissal emit `OpenChanged(false)` and reset the active path. In controlled mode, close and focus
restoration wait until the owner applies `set_open(false)`. Reopening starts at the first enabled
root item.
Shortcut labels are display only and do not install key bindings.

## Focus and dismissal boundary

While open, keyboard actions are handled in the menu key context. The menu owns active-row
navigation but does not move OS focus to individual rows. The host supplies the invocation point and may specify the focus return target. Closing returns
focus after the closed state is applied. Outside pointer handling applies to the rendered menu
surface; nested overlay routing remains the host's responsibility.

## Accessibility role and properties

menu; host trigger retains its role; entries use menuitem/menuitemcheckbox, expose their label as the accessible name, and set the AccessKit disabled state on unavailable entries. Checkable entries expose their checked state.

## WAI-ARIA pattern reference

Follows the WAI-ARIA Authoring Practices Menu / Menubar interaction patterns for nested menus,
arrow navigation, activation, and dismissal.

## Platform notes

The host owns pointer coordinates and the platform invocation gesture (ContextMenu or Shift+F10).
Displayed shortcuts follow host platform conventions; the component does not bind them.

## Theme tokens used

Read colors, typography, spacing, radii, border widths, and control sizing from the GPUI Global `Theme` tokens. Enabled, non-active menu rows use `Theme.colors.text`; disabled rows use `Theme.colors.disabled`; shortcut labels and submenu indicators use `Theme.colors.text_muted`. In shadcn themes, the pane uses `surface`, and the pointer-hovered or keyboard-active row keeps `text` over the web preview's subtle `text`/`background` mix (4% text in light, 12% in dark), with `radii.small` corners and the pane's `spacing.xsmall` inset. Other themes retain their accent/contrast active colors and `elevated_surface` pane. Shadows and motion are not used by this menu surface. Do not hard-code colors.

## Open questions

GPUI 0.3.5 provides window-point anchoring with viewport fitting but does not expose arbitrary
trigger bounds for edge alignment. Native platform accessibility and whether nested overlay routing
should dismiss only the topmost menu remain unverified.
