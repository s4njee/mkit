---
spec_version: 1
component: toolbar
states:
  - id: horizontal
    description: Horizontal toolbar with a roving tab stop.
    fixture: toolbar_horizontal
  - id: vertical
    description: Vertical toolbar with vertical-axis navigation.
    fixture: toolbar_vertical
  - id: overflow
    description: Constrained toolbar exposes overflow actions in a menu.
    fixture: toolbar_overflow
  - id: disabled
    description: Toolbar contains disabled actions which are skipped by roving focus.
    fixture: toolbar_disabled
screenshots:
  themes: [shadcn-light, shadcn-dark, high-contrast]
  scales: [1, 2]
  matrix: every state × theme × scale
keys:
  - key: ArrowRight
    modifiers: []
    when: an item in a horizontal toolbar is focused
    action: Move focus to the next enabled actionable item, wrapping at the end.
    initial_state: horizontal
    expect:
      focus_target: next_enabled_item
  - key: ArrowLeft
    modifiers: []
    when: an item in a horizontal toolbar is focused
    action: Move focus to the previous enabled actionable item, wrapping at the beginning.
    initial_state: horizontal
    expect:
      focus_target: previous_enabled_item
  - key: ArrowDown
    modifiers: []
    when: an item in a vertical toolbar is focused
    action: Move focus to the next enabled actionable item, wrapping at the end.
    initial_state: vertical
    expect:
      focus_target: next_enabled_item
  - key: ArrowUp
    modifiers: []
    when: an item in a vertical toolbar is focused
    action: Move focus to the previous enabled actionable item, wrapping at the beginning.
    initial_state: vertical
    expect:
      focus_target: previous_enabled_item
  - key: Home
    modifiers: []
    when: an item in the toolbar is focused
    action: Move focus to the first enabled actionable item.
    initial_state: horizontal
    expect:
      focus_target: first_enabled_item
  - key: End
    modifiers: []
    when: an item in the toolbar is focused
    action: Move focus to the last enabled actionable item.
    initial_state: horizontal
    expect:
      focus_target: last_enabled_item
  - key: ArrowDown
    modifiers: []
    when: the overflow menu is open
    action: Move the active menu item to the next overflow action, wrapping at the end.
    initial_state: overflow
    expect:
      focus_target: next_overflow_item
  - key: ArrowUp
    modifiers: []
    when: the overflow menu is open
    action: Move the active menu item to the previous overflow action, wrapping at the beginning.
    initial_state: overflow
    expect:
      focus_target: previous_overflow_item
  - key: Escape
    modifiers: []
    when: the overflow menu is open
    action: Close the menu and retain the overflow trigger as the active toolbar item.
    initial_state: overflow
    expect:
      event: overflow_closed
accessibility:
  role: toolbar
  properties:
    - name: label
      value: Document actions
    - name: orientation
      value: horizontal
      except_when: [vertical]
    - name: orientation
      value: vertical
      when: vertical
---

# Toolbar

## Purpose

Group frequently used actions into a compact, keyboard-efficient region. The toolbar is a stateful
`Entity<Toolbar>` because its active roving-focus item and available overflow items depend on
interaction and available width.

## Anatomy

The toolbar accepts ordered items: push buttons, toggle buttons, labelled toggle groups, and
non-interactive separators. Each actionable item has a stable ID, accessible label, disabled
state, and overflow policy (`MayOverflow` or `NeverOverflow`). Toggle buttons optionally carry an
initial pressed value. A toggle group has a single-selection value and owns its own item semantics
and internal selection behavior. Separators are removed when they would be leading, trailing, or
adjacent after items move to overflow.

Overflow follows the menu interaction model: a labelled overflow button is appended when at least
one item is moved. Activating it opens a menu containing the moved actions in original order; menu
activation dispatches the same action/toggle/group request as its toolbar counterpart. GPUI does
not currently provide the package with a stable child-measurement callback, so the first draft
accepts an explicit `visible_capacity` (number of actionable controls kept inline) rather than
guessing pixel fit. `NeverOverflow` items do not count against the capacity and remain inline. If
all candidates are `NeverOverflow`, the toolbar may exceed its available width; it must not silently
move them. Automatic pixel-fit integration remains an open question.

## States

- `horizontal` and `vertical`: orientation controls layout and the arrow axis used for roving focus.
- `overflow`: one or more `MayOverflow` actions have moved to the overflow menu.
- `disabled`: disabled actions remain discoverable but are skipped during keyboard navigation and
  do not activate.
- Focus is represented by the active enabled action. Separators never receive focus.

## Props and events

The builder creates an `Entity<Toolbar>` with label, orientation, items and optional controlled
values. Uncontrolled mode initializes toggle values from item defaults and updates them on
activation. Controlled mode retains values supplied by the app and emits typed requests; the app
applies updates through setters. This applies independently to toggle button values and each
toggle group's selected ID. Toggle groups are atomic for overflow and their choices remain
individually labelled toggle buttons in the row. While a group is active, the toolbar arrow axis
cycles its enabled choices; Enter/Space commits the marked choice before arrow navigation advances
to a neighboring toolbar item.

Typed events are `ActionInvoked { id }`, `ToggleRequested { id, pressed }`,
`GroupValueRequested { group_id, value }`, `OverflowOpened`, and `OverflowClosed`. Focus movement
itself emits no value event. Pointer and keyboard activation share the same event contract. Disabled
items emit nothing.
Resizing recalculates overflow without emitting action events.

## Keyboard map

The toolbar is one Tab stop. Tab enters at the currently active enabled item, or the first enabled
item when no prior active item exists. Shift+Tab leaves from the active item. On the horizontal
toolbar, Left and Right move focus among enabled actions, wrapping at the ends. On the vertical
toolbar, Up and Down do so. Within an active toggle group, those arrows mark the previous/next
enabled choice; when the group reaches an edge, the next arrow moves to the adjacent toolbar item.
Enter/Space commits the marked group value. Home and End move to the first and last enabled action.
The orthogonal arrows do not change focus. When overflow is open, Up/Down move between enabled menu
rows (disabled actions remain listed), Home/End select the first/last row, Enter/Space invokes it,
and Escape closes the menu while retaining the trigger
as the focus anchor. The overflow trigger participates as the final actionable item when rendered.
Bindings are in the `MkitToolbar` key context and exposed as defaults for app rebinding.

## Pointer behaviour

Primary activation of an enabled button emits `ActionInvoked`. Toggle activation requests the
opposite pressed state. Toggle group activation requests the chosen value. Clicking an enabled item
also makes it the active roving-focus item. Disabled items ignore pointer input. Overflow activation
opens the menu and emits `OverflowOpened`.

## Accessibility role and properties

Expose the container as a labelled `Toolbar` with orientation. Action items retain button semantics;
toggle buttons expose pressed state; toggle groups expose a labelled group with their selected
button state. Separators are decorative to accessibility. Disabled actions expose disabled state.
Only the active enabled toolbar action and, when present, the overflow trigger participate in the
toolbar's single tab stop. When menu items are open they use the menu component's accessibility
contract.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts`, styled by
`.e7-toolbar` in `site/src/demos/e7_expansion.css` and `.ui-btn*` in `site/src/ui/ui.css`, with the
shadcn token mapping in `site/src/ui/tokens.ts`): a bordered shadcn menubar-style container holding
small outline buttons, with pressed toggles in the primary fill. The overflow menu reuses
DropdownMenu's look. Colours are resolved from the installed `Theme` in three variants.
`high-contrast` is selected by theme name (the convention other registry components use); every
other theme is treated as dark when `mkit_core::contrast::relative_luminance(colors.background) <
0.5`, otherwise light. Derived colours use a crate-local `color-mix` helper built on
`mkit_core::contrast::composite`; no mkit-core API or tokens are added. "Muted" is shadcn's
`accent`/`muted`: `text` mixed 4% (light) or 12% (dark) into `background`. "Outline" is `border` in
light and `text` at 10% over `background`, composited opaque, in dark.

| Part | Light / dark | High contrast |
|---|---|---|
| Container | `background` fill, outline border, `shadows.small` (shadcn `shadow-xs`) | `background`, `border` |
| Separator | outline colour | `border` |
| Button, unpressed toggle | `background`, `text`, outline border, `shadows.small`; hover muted fill | `background`, `text`, `border`; hover border `accent` |
| Pressed toggle or group choice | `accent` fill and border, `accent_text`, `shadows.small`; hover `accent` 90% over `background` | `accent`, `accent_text`, `accent`; hover border `text` |
| Overflow trigger while open | muted fill (shadcn `data-[state=open]:bg-accent`), `text`, outline border | `background`, `text`, `accent` border |
| Disabled control | every resting colour mixed 50% over `background`, no shadow or hover | `background`, `disabled`, `disabled` border (pressed: `disabled` fill, `accent_text`) |
| Focus (`focus_visible`) | border `focus` plus a 3px ring of `focus` at 50% | border `focus` plus a 3px ring of opaque `focus` |
| Overflow menu pane | DropdownMenu's pane: `surface`, border `border` (dark: `text` 10% over `surface`), `shadows.medium` | `background`, `border` |
| Overflow row | `text`; active row muted fill; disabled `text` 50% over the pane | `text`; active `accent`/`accent_text`; disabled `disabled` |
| Overflow check mark | `text_muted` Lucide `check` vector | `text_muted` (active: `accent_text`), 2px stroke |

- **Container.** Padding `spacing.xsmall` (shadcn `p-1`), radius `radii.medium`, a
  `borders.regular` border and an opaque `background` fill (GPUI fills the inside of drop shadows).
  The toolbar hugs its controls, as in the preview, instead of stretching to the parent width.
  Controls are separated by `spacing.xsmall`; the preview uses 6px and there is no 6px token, so
  shadcn menubar's `gap-1` (4px) is used. A vertical toolbar stacks its controls and stretches them
  to the widest one.
- **Controls.** The preview's `ui-btn--sm`: height `controls.small` (32px), horizontal padding
  `spacing.medium` (12px), radius `radii.medium`, a `borders.regular` border, `typography.body`
  (14px) labels at medium weight (500); there is no font-weight token yet. Toggle-group choices use
  the same control with the same gap.
- **Separators.** A `borders.hairline` rule `spacing.xlarge` (24px) tall in a horizontal toolbar,
  or full-width and hairline-thick in a vertical one; decorative, no accessibility node.
- **Disabled.** The preview uses `opacity: .5`. GPUI applies element opacity to each painted part
  separately, so each colour is mixed 50% over `background` instead and the shadow is dropped.
  High contrast keeps solid `disabled` colours.
- **Focus.** GPUI paints drop shadows as filled shapes that are not clipped to the element's
  outside, so every control keeps an opaque fill under the ring. The ring replaces the resting
  shadow, its corners use the control radius, and the 3px width is the shadcn/ui ring width, a
  fixed component value.
- **Overflow menu.** DropdownMenu's pane and rows: padding `spacing.xsmall`, radius `radii.medium`,
  minimum width 4 × `spacing.xxlarge` (128px), rows `controls.small` tall with `spacing.small`
  padding and gap and `radii.small` corners. Moved toggles reserve a leading `spacing.large` slot
  and draw a Lucide `check` (20,6 → 9,17 → 4,12) as a vector path when pressed. The menu opens
  `spacing.xsmall` below the toolbar (shadcn `sideOffset={4}`), aligned to its trailing edge when
  horizontal and its leading edge when vertical.

## WAI-ARIA pattern reference

Follow the [APG Toolbar Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/) for one tab stop,
orientation-specific arrow navigation and Home/End. Buttons and toggles follow the APG Button
pattern. The overflow trigger follows the existing menu button pattern.

## Platform notes

Overflow uses a local menu-role surface in the toolbar draft. It supports Up/Down, Home/End,
Enter/Space, and Escape while leaving focus anchored at the trigger. Disabled moved actions remain
listed but are skipped by keyboard navigation. Native accessibility snapshots remain pending.
Explicit action capacity avoids guessing at GPUI child measurement; groups move as one unit when
the remaining capacity cannot fit every choice.

## Open questions

- Confirm whether the toolbar should expose per-item shortcut hints as part of the first public API.
- Confirm the precise overflow measurement/integration contract with GPUI layout before stabilizing
  a resize observer API.
- Confirm the pinned AccessKit bridge's Toolbar role and orientation property map against a native
  accessibility snapshot.
