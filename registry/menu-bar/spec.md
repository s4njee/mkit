---
spec_version: 1
component: menu-bar
states:
  - id: idle
    description: No top-level menu is expanded and the bar is not keyboard-focused.
    fixture: menu_bar_idle
  - id: focused
    description: The bar has keyboard focus with one top-level menu active and no popup open.
    fixture: menu_bar_focused
  - id: open
    description: The active top-level menu popup is open and an enabled command may be active.
    fixture: menu_bar_open
  - id: disabled
    description: All menu headers and items are unavailable.
    fixture: menu_bar_disabled
keys:
  - key: Alt
    modifiers: []
    when: the window is active and the menu bar is enabled
    action: Focus the first enabled top-level menu without opening it.
    initial_state: idle
    expect:
      focus_target: first-enabled-menu
  - key: ArrowRight
    modifiers: []
    when: the menu bar is focused or a menu is open
    action: Move to the next enabled top-level menu; if a popup is open, open its replacement.
    initial_state: open
    expect:
      focus_target: next-enabled-menu
  - key: ArrowDown
    modifiers: []
    when: a top-level menu is focused and closed
    action: Open its popup and activate the first enabled item.
    initial_state: focused
    expect:
      state: open
  - key: ArrowUp
    modifiers: []
    when: a top-level menu is focused and closed
    action: Open its popup and activate the last enabled item.
    initial_state: focused
    expect:
      state: open
  - key: F
    modifiers: [Alt]
    when: an enabled top-level menu has the matching mnemonic
    action: Focus and open the matching menu.
    initial_state: idle
    expect:
      focus_target: mnemonic-menu
  - key: Escape
    modifiers: []
    when: a popup is open
    action: Close the popup and return focus to its top-level menu header.
    initial_state: open
    expect:
      focus_target: active-menu-header
  - key: Enter
    modifiers: []
    when: a top-level menu header is focused
    action: Open its popup.
    initial_state: focused
    expect:
      state: open
accessibility:
  role: menubar
  properties:
    - name: top_level_menus
      value: named menuitem elements with expanded state
      when: idle
    - name: popup_items
      value: menuitem, menuitemcheckbox, or menuitemradio according to model
      when: open
    - name: disabled
      value: unavailable headers and commands expose AccessKit disabled state
      when: disabled
---

# MenuBar

## Purpose

In-window application command bar with APG menubar keyboard navigation. It can be rendered on
platforms where the operating system does not show an app menu in the window.

## Anatomy

A horizontal menubar contains named top-level menu buttons. An expanded menu renders a popup of
command rows, separators, and nested submenus. Command rows may show checked state, disabled state,
and a shortcut label sourced from the menu model.

## States

`idle` has no keyboard focus and no popup. `focused` has one active top-level header. `open` has an
expanded popup and an active enabled row. `disabled` makes all headers and commands unavailable.
Submenu and checkable entries are represented by the shared model and render their corresponding
expanded/checked states.

### Screenshot fixture contract

The matrix covers `idle`, `focused`, `open`, and `disabled` across shadcn light, dark, and
high-contrast themes at 1× and 2×. The open fixture includes File (New, disabled Save, separator,
Recent submenu) and Edit (Undo checked, Copy with displayed shortcut). Focused and open fixtures
use the File mnemonic. Images demonstrate the in-window renderer; they do not claim native-menu
parity.

## Props and events

`MenuBar` is an `Entity` with controlled and uncontrolled open-menu state. The owner supplies a
`MenuBarModel` of named menus, explicit optional mnemonics, and GPUI actions. `OpenMenuChanged` is
emitted when the requested top-level menu changes; in controlled mode the owner applies it with
`set_open_menu`. `CommandInvoked` reports the stable command ID after dispatching its cloned GPUI
action. `MenuBarModel::to_gpui_menus` creates the native `gpui_pre::Menu` tree from the same data;
applications can pass it to `App::set_menus` so command labels/actions are shared. Check state and
disabled state are model properties and are not changed by the renderer.

## Keyboard map

| Key | Behaviour |
|---|---|
| Alt | Focus first enabled header. |
| Mnemonic | Focus and open the matching enabled top-level menu. |
| Left / Right | Move among top-level headers; an open popup switches with the header. |
| Down / Enter / Space | Open the focused header. |
| Up / Down | Move among enabled popup items. |
| Right | Open an active submenu; otherwise move to the next top-level menu. |
| Left | Close a submenu or move to the prior top-level menu. |
| Enter / Space | Activate a command or expand an active submenu. |
| Home / End | Move to first / last enabled popup item. |
| Escape | Close popup and return focus to its header. |
| Tab | Close popup and continue normal window traversal. |

Actions are bound in the `MenuBar` key context where GPUI supports action dispatch. Mnemonics are
explicit single characters, case-insensitive, and are only handled while the window is active.

## Pointer behaviour

Clicking a top-level header opens its menu; clicking another header while a menu is open switches
the popup. Pointer movement over another header switches the popup only while one is already open.
Clicking an enabled leaf dispatches its action and closes the popup. Disabled items do nothing.
Submenu rows open their child popup on pointer hover and click.
An outside pointer-down closes the popup; Tab closes it while preserving normal window traversal.

## Accessibility role and properties

The container uses the `menubar` role. Headers use `menuitem` with accessible names, expanded
state, and disabled state. Popup containers use `menu`; entries use `menuitem`, `menuitemcheckbox`,
or `menuitemradio`, with checked and disabled state as specified by the model. The active row is
exposed through the active-descendant mapping. Keyboard focus remains on the menubar/popup owner,
not individual row elements.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts`, styled by
`.e7-menubar*` in `site/src/demos/e7_expansion.css` and `.ui-popover`/`.ui-menu*` in
`site/src/ui/ui.css`, with the shadcn token mapping in `site/src/ui/tokens.ts`), which is shadcn/ui's
Menubar with each header's mnemonic stacked above its label. The popup reuses DropdownMenu's look.
Colours are resolved from the installed `Theme` in three variants. `high-contrast` is selected by
theme name (the convention other registry components use); every other theme is treated as dark
when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived
colours use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no
mkit-core API or tokens are added. "Accent" is shadcn's `accent`: `text` mixed 4% (light) or 12%
(dark) into `background`, the mix Button, DropdownMenu, Tabs and Sidebar use.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Bar fill, border | `background`, `border` | `background`, `text` at 10% over `background` composited | `background`, `border` |
| Bar shadow | `shadows.small` (shadcn `shadow-xs`) | same | `shadows.small` (transparent in this theme) |
| Header label, mnemonic | `text` (medium weight), `text_muted` | same | `text`, `text_muted` |
| Open, focused or hovered header | accent fill, `text` | same | `accent` fill, `accent_text` label and mnemonic; hover underlines |
| Keyboard-focused header on a closed bar | accent fill, border `focus`, 3px ring of `focus` at 50% | same | `accent` fill, border `focus`, 3px ring of opaque `focus` |
| Disabled bar or header | label, mnemonic and bar border mixed 50% over `background`; no shadow | same | solid `disabled` |
| Popup pane | `surface`, `border`, `shadows.medium` | `surface`, `text` at 10% over `surface` composited, `shadows.medium` | `background`, `border` |
| Popup row | `text`; shortcut, check and chevron `text_muted` | same | `text`, `text_muted` |
| Active popup row | accent fill | accent fill | `accent` fill, `accent_text` for every part |
| Disabled popup row | `text` and `text_muted` at 50% over the pane | same | solid `disabled` |
| Separator | the pane border colour | same | `border` |

- **Bar.** Full width, padding `spacing.xsmall`, radius `radii.medium`, a `borders.regular` border
  and an opaque `background` fill (GPUI fills the inside of drop shadows). Headers are separated by
  `spacing.xsmall`; the preview uses 3px padding and a 2px gap, and there are no such tokens, so
  shadcn menubar's `p-1` and `gap-1` (4px) are used. The preview draws no bar shadow; shadcn's
  Menubar draws `shadow-xs`, which is kept.
- **Headers.** When any menu has a mnemonic, headers stack the mnemonic (`typography.caption`,
  12px, for the preview's 11px) above the label (`typography.body` at medium weight), are
  `controls.large + spacing.small` tall (48px, the preview's height) and at least `controls.large`
  wide (the preview's 44px has no token). Without mnemonics, headers are single-line and
  `controls.xsmall` (28px) tall, shadcn's trigger inside an `h-9` bar. Horizontal padding is
  `spacing.small` (`px-2`), the radius `radii.small`, and a transparent `borders.regular` border is
  reserved for the focus ring. Font weight uses GPUI's medium (500) and normal weights; there is no
  font-weight token yet.
- **Focus.** Keyboard focus stays on the bar and the active header is exposed as the active
  descendant. The preview shows it with the accent fill (`:focus-visible`); GPUI adds the shared
  focus treatment, border `focus` and a 3px ring, on that header while the bar has keyboard focus
  and no popup is open, because a 4% fill alone is too faint for a focus indicator. Pointer focus
  shows the fill only. The header fill under the ring is opaque, since GPUI paints drop shadows as
  filled shapes. The 3px ring is the shadcn/ui ring width, a fixed component value.
- **Disabled.** The preview dims unavailable rows with opacity. GPUI applies element opacity to
  each painted part separately, so colours are mixed 50% over the opaque fill beneath instead.
  High contrast keeps solid `disabled` colours so unavailable items stay legible.
- **Popup.** DropdownMenu's pane: padding `spacing.xsmall`, radius `radii.medium`, a
  `borders.regular` border, `shadows.medium`, and a minimum width of 5 × `controls.large` (200px,
  the preview's width), placed `spacing.small` below the bar. Rows are `controls.small` tall with
  `spacing.small` padding and gap, `radii.small` corners and `typography.body` labels. Shortcuts are
  right-aligned `typography.caption` text shown as supplied (the component depends only on
  mkit-core, so it does not format them through KeyHint). Checkable rows reserve a leading
  `spacing.large` slot and draw a Lucide `check` (20,6 → 9,17 → 4,12) when checked; submenu rows end
  with a Lucide `chevron-right` (9,6 → 15,12 → 9,18). Both are vector paths stroked at 2/24 of the
  icon size (`borders.regular` in high contrast) and are decorative. Separators are a
  `borders.hairline` rule spanning the pane padding (shadcn `-mx-1 my-1`).

## WAI-ARIA pattern reference

Follows the WAI-ARIA Authoring Practices Guide Menubar pattern for focus entry, roving top-level
navigation, popup item navigation, submenu entry/exit, activation, and dismissal.

## Platform notes

GPUI exposes the application menu tree as `Menu`/`MenuItem`, `App::set_menus`, and `App::get_menus`.
This component's `MenuBarModel` converts to that same GPUI tree for native integration. GPUI's
macOS backend installs a native application menu; Linux and Windows backends retain the menu model
but do not render it as an in-window bar. Therefore the in-window component remains the visible
menu on Windows/Linux, while macOS applications may choose native app menus or render the in-window
bar explicitly. Alt/mnemonic behavior, native accessibility integration, and cross-platform action
dispatch must be checked on each target; screenshot and GPUI tests cover only the in-window path.

## Open questions

Confirm whether exposing conversion to the GPUI native menu tree is sufficient for the “one model”
requirement, since GPUI's native tree has no explicit mnemonic field. Validate modifier/Alt handling
and native menu coexistence on Windows, Linux, and macOS.
