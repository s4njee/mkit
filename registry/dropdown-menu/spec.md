---
spec_version: 1
component: dropdown-menu
states:
  - id: closed
    description: Dropdown Menu closed state.
    fixture: dropdown-menu_closed
  - id: open
    description: Dropdown Menu open state.
    fixture: dropdown-menu_open
  - id: outside_dismissed
    description: An outside pointer press requests close and returns focus to the host target after close applies.
    fixture: dropdown-menu_outside_dismissed
  - id: submenu
    description: Dropdown Menu with the active item's child pane open.
    fixture: dropdown-menu_submenu
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

# Dropdown Menu

## Purpose

Button-triggered command menu supporting submenus, checkable items, and displayed shortcuts.

## Anatomy

Trigger is supplied by the host. The component renders its panes in a GPUI deferred surface
anchored at a host-provided window point. While a submenu is active, one adjacent pane is rendered
for every level in the active path. Each row contains a leading check-mark slot (checkable items only), the label, an optional
right-aligned shortcut label, and a trailing chevron for submenu parents.

## States

`closed` hides every pane; `open` shows the root pane; `submenu` shows the root and child panes; `outside_dismissed` is the closed state after a real outside pointer-down and focus restoration.

### Screenshot fixture contract (E7.5)

The screenshot matrix is the Cartesian product of `closed`, `open`, `outside_dismissed`, and `submenu` with the `light`, `dark`, and `high-contrast` shadcn themes at 1x and 2x scales. The gallery fixture is a 480×320 logical-pixel window with a `DropdownMenu` heading and a focusable host target named `Invocation target`. It anchors the menu at window point (136, 88) and uses the following deterministic item tree:

- Source-order items: `more-actions` / More actions (submenu); child items `duplicate-view` / Duplicate view (shortcut `⌘D`) and `move-to-folder` / Move to folder… (shortcut `⌘M`); `new-file` / New file (shortcut `⌘N`); `favorite` / Favorite (unchecked checkable item); `archive` / Archive (disabled).
- The fixture creates and retains one `Entity<DropdownMenu>`, uses the uncontrolled `new(items)` API, calls `set_open(true)` to model the host button-triggered, and supplies the host focus handle through `return_focus_to`.
- `closed`: leave the menu closed; assert `is_open() == false`.
- `open`: focus the host target, open through the host's `set_open(true)` call, and assert `is_open() == true`; the anchored menu must be visible in the capture.
- `submenu`: focus the menu entity after opening, dispatch the registered `right` action, and assert it remains open with the menu focus handle still focused. The screenshot must show both root and child panes.
- `outside_dismissed`: focus the host target, open the menu, focus the menu entity, then dispatch a real left-button mouse-down at window point (456, 292), outside the anchored surface. After the close has rendered, assert it is closed and focus has returned to `Invocation target` before capture.

The menu overlay is intentionally kept well inside the window in the `open` and `submenu` fixtures so the visible pane contents are not clipped. The dismissed capture shows the host target with the menu surface absent.

## Props and events

Entity<DropdownMenu>; uncontrolled `new(items)` and controlled `controlled(items, open)` both expose `set_open`. `OpenChanged(bool)` requests open state changes; in controlled mode the owner must call `set_open` to apply them. `ItemSelected(id)` is emitted for enabled command items, and dismisses the menu. `CheckedChanged(id, checked)` is emitted for enabled checkable items; toggling a checkable item keeps the menu open. Items have id, label, disabled, optional shortcut, checked and children.

Shortcut labels are display-only strings (`MenuItem::shortcut` keeps its `String` API). The menu passes each label to KeyHint's shared `KeyChord::parse` and renders a parsed chord with `KeyHint::inline()`, so platform modifier names and order match CommandPalette and ShortcutEditor: GPUI keystroke text such as `cmd-shift-s` and glyph text such as `⌘⇧S` both display as `⇧⌘S` on macOS and `Shift+Super+S` elsewhere (`secondary-s` displays as `⌘S` on macOS and `Ctrl+S` elsewhere). Labels that KeyHint cannot parse, such as spaced or multi-keystroke text, are shown verbatim. The inline presentation keeps the existing muted-text shortcut column; keycap boxes were not adopted because they would enlarge fixed-height menu rows and change the established menu look. Menu item accessible names remain the item label only.

The implementation depends on `mkit-registry-key-hint` for this shared formatting and presentation. This is a documented draft exception to the mkit-core/GPUI-only default pending maintainer approval.

## Keyboard map

| Key | Behaviour |
|---|---|
| Up / Down | Move among enabled items in the current pane, wrapping at either end. |
| Right | Open the active enabled item's submenu and focus its first enabled item. |
| Left | Close the current submenu and restore its parent item as active. |
| Enter / Space | Open an active submenu or activate its leaf. |
| Escape | Request dismissal of the root menu. |
| Home / End | Focus the first / last enabled item in the current pane. |

## Pointer behaviour

Pointer movement activates enabled rows. Clicking a submenu row opens its child pane; clicking a
leaf activates it. Checkable items toggle and keep the menu open. The host owns the trigger and
supplies the point through `anchor_at` or `set_anchor`; the menu routes outside pointer-down
dismissal itself. `return_focus_to` supplies the invoking element, or the menu captures the
previously focused handle when it opens. Escape and outside dismissal emit `OpenChanged(false)`
and reset the active path. Reopening starts at the first enabled root item. Displayed shortcuts
do not invoke commands and do not install key bindings.

## Focus and dismissal boundary

While open, keyboard actions are handled in the menu key context. The menu owns active-row
navigation but does not move OS focus to individual rows. The host supplies an invocation point and may specify the focus return target. Focus remains on the
invoking element until the host/menu moves it into the menu. Closing returns focus after the closed
state is applied. Outside pointer handling applies to the rendered menu surface; the host must
coordinate trigger clicks so an opening click is not also treated as a close request.

## Accessibility role and properties

menu; trigger button has aria-haspopup=menu and aria-expanded; entries use menuitem or menuitemcheckbox and expose their label as the accessible name. Disabled entries set the AccessKit disabled state. Checkable entries expose their toggled state, submenu parents expose expanded state, and the active row is reported through GPUI’s active-descendant accessibility mapping.

## WAI-ARIA pattern reference

Follows the WAI-ARIA Authoring Practices Menu Button and Menu / Menubar keyboard interaction
patterns for arrow navigation, submenu entry/exit, activation, and dismissal.

## Platform notes

The host owns the trigger button and supplies the popover anchor. Displayed shortcuts use KeyHint's
compile-target platform modifier names and order (glyphs on macOS). Keyboard actions target the active menu pane;
individual rows do not receive OS focus.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7.ts`, styled by `.ui-popover`,
`.ui-menu` and `.ui-menu__*` in `site/src/ui/ui.css` with the shadcn token mapping in
`site/src/ui/tokens.ts`) and is resolved from the installed `Theme` in three variants.
`high-contrast` is selected by theme name (the convention other registry components use); every
other theme is treated as dark when `mkit_core::contrast::relative_luminance(colors.background) <
0.5`, otherwise light. Derived colours use a crate-local `color-mix` helper built on
`mkit_core::contrast::composite`; no mkit-core API or tokens are added. "Accent" below is shadcn's
`accent`: `text` mixed 4% (light) or 12% (dark) into `background`, the same mix Button, Tabs and
Sidebar use.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Pane fill (shadcn `popover`) | `surface` | `surface` | `background` |
| Pane border | `border` | `text` at 10% over `surface`, composited opaque | `border` |
| Pane shadow | `shadows.medium` (shadcn `shadow-md`) | `shadows.medium` | `shadows.medium` (transparent in this theme) |
| Row text | `text` | `text` | `text` |
| Shortcut, check mark, submenu chevron | `text_muted` | `text_muted` | `text_muted` |
| Active row fill | accent | accent | `accent` |
| Active row text, shortcut, check, chevron | `text`, `text_muted` | `text`, `text_muted` | `accent_text` for every part |
| Disabled row | `text` and `text_muted` at 50% over the pane fill | same | solid `disabled` |

- **Active row.** The pointer-hovered or keyboard-active row (shadcn `focus:bg-accent`, the web
  preview's `.is-active`) uses the accent fill with `radii.small` corners. A submenu parent keeps
  that fill while its child pane is open, like shadcn's `data-[state=open]:bg-accent`. High
  contrast keeps a solid `accent` fill with `accent_text` so the active row is unmistakable.
- **Disabled.** The web preview renders unavailable rows at `opacity: .5`. GPUI applies element
  opacity to each painted part separately, so each colour is instead mixed 50% over the opaque pane
  fill. High contrast keeps solid `disabled` text so unavailable rows stay legible without
  transparency.
- **Opaque fills.** GPUI paints drop shadows as filled shapes that are not clipped to the
  element's outside, so the pane fill and its border are always opaque; the dark border is the
  web's translucent `text` at 10% composited over `surface`, which is the colour the browser shows.
- **Geometry.** The pane has radius `radii.medium`, a `borders.regular` border, padding
  `spacing.xsmall` (shadcn `p-1`) and a minimum width of 4 × `spacing.xxlarge` (128px, shadcn's
  `min-w-[8rem]`; there is no width token, so it is expressed through the largest spacing token).
  Rows are `controls.small` (32px, the web's 6px vertical padding around a 20px line) tall with
  horizontal padding `spacing.small` (shadcn `px-2`), a `spacing.small` gap between parts, and
  `typography.body` (14px) labels. Adjacent submenu panes are separated by `spacing.xsmall`.
- **Check mark.** Checkable rows reserve a leading `spacing.large` (16px, the web's
  `.ui-menu__indicator`) square slot and draw a Lucide `check` (20,6 → 9,17 → 4,12 on a 24-unit
  grid) as a vector path when checked; unchecked rows leave the slot empty. Rows without a check
  state do not reserve the slot, as in the preview.
- **Submenu chevron.** Submenu parents end with a Lucide `chevron-right` (9,6 → 15,12 → 9,18) drawn
  as a vector path in a trailing `spacing.large` square pushed to the row end. Both icons stroke at
  Lucide's 2/24 of the icon size (about 1.3px) in light and dark and at `borders.regular` (2px) in
  high contrast. Icons are decorative and add no accessibility node.
- **Shortcut.** Right-aligned (`margin-left: auto`) in `typography.caption` (12px) and
  `text_muted`. The web preview adds `letter-spacing: .08em`; GPUI text has no letter-spacing
  control, so the shortcut is drawn with normal spacing.
- **Not rendered.** The web preview also shows a group label, separators, leading item icons and a
  destructive row. `MenuItem` has no label, separator, icon or destructive variant, so these parts
  are not drawn; adding them needs a public API change (see open questions).

Do not hard-code colors.

## Open questions

GPUI 0.3.5 provides window-point anchoring with viewport fitting but does not expose arbitrary
trigger bounds for edge alignment. Native platform accessibility and whether nested overlay routing
should dismiss only the topmost menu remain unverified.

The `mkit-registry-key-hint` dependency, the `⌘⇧S` to `⇧⌘S` modifier reordering, and whether menu items should expose the chord through `aria_keyshortcuts` need maintainer review.

`MenuItem` has no group label, separator, leading icon or destructive variant, so the web preview's `.ui-menu__label`, `.ui-menu__sep`, item icons and destructive (`danger`) rows are not rendered. Adding them is a public API change that needs maintainer review.
