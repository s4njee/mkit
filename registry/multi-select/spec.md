---
spec_version: 1
component: multi-select
states:
  - id: closed
    description: Trigger shows selected labels or placeholder.
    fixture: closed_fixture
  - id: open
    description: Multi-selection listbox is anchored to the measured trigger, constrained to the window with a margin, and uses active row; long lists use a clipped virtual scroll viewport.
    fixture: open_fixture
  - id: disabled
    description: Control cannot receive focus or change.
    fixture: disabled_fixture
  - id: all_options_disabled
    description: Control is enabled but every option is unavailable, so navigation leaves it closed.
    fixture: all_options_disabled_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: Multi-select is focused
    action: Open and move active row.
    initial_state: closed
    expect: { state: open, event: open, focus_target: multi_select }
  - key: Space
    modifiers: []
    when: An option is active
    action: Toggle its selected membership and keep popup open.
    initial_state: open
    expect: { state: open, event: change, focus_target: multi_select }
  - key: Escape
    modifiers: []
    when: Popup is open
    action: Close and preserve current selected values.
    initial_state: open
    expect: { state: closed, focus_target: multi_select }
  - key: Home
    modifiers: []
    when: Popup is open
    action: Move active row to first enabled item and scroll it into view.
    initial_state: open
    expect: { state: open, focus_target: multi_select }
  - key: End
    modifiers: []
    when: Popup is open
    action: Move active row to last enabled item and scroll it into view.
    initial_state: open
    expect: { state: open, focus_target: multi_select }
  - key: Tab
    modifiers: []
    when: Multi-select is focused and popup is open
    action: Follow normal forward focus traversal and close the popup when focus leaves the trigger.
    initial_state: open
    expect: { state: closed, focus_target: next_tab_stop }
  - key: Tab
    modifiers: [Shift]
    when: Multi-select is focused and popup is open
    action: Follow normal backward focus traversal and close the popup when focus leaves the trigger.
    initial_state: open
    expect: { state: closed, focus_target: previous_tab_stop }
  - key: ArrowDown
    modifiers: []
    when: Multi-select is disabled
    action: Keep the popup closed, preserve values, and emit no event.
    initial_state: disabled
    expect: { state: disabled, event: none, focus_target: none }
  - key: ArrowDown
    modifiers: []
    when: Every option is disabled
    action: Keep the popup closed and emit no event.
    initial_state: all_options_disabled
    expect: { state: all_options_disabled, event: none, focus_target: multi_select }
accessibility:
  role: combobox
  properties:
    - { name: aria-haspopup, value: listbox }
    - { name: aria-value, value: selected option labels joined with commas or empty string }
    - { name: aria-multiselectable, value: true }
    - { name: aria-expanded, value: false, when: closed }
    - { name: aria-expanded, value: true, when: open }
    - { name: aria-disabled, value: true, when: disabled }
    - { name: active descendant, value: active option while open }
---

# Multi-select

## Purpose

Multi-select lets users choose option values from a bounded collection. Choose zero or more values.

## Anatomy

A labeled trigger displays the selected option labels. Opening it reveals an in-window multi-select listbox; active navigation is distinct from committed selection.

## States

Closed, open, disabled, and all-options-disabled states are defined in the manifest. The open state includes an active enabled option; disabled rows remain visible and cannot be selected. If every option is disabled, navigation and trigger activation leave the popup closed. A disabled control is omitted from Tab order and ignores keyboard actions, including navigation and dismissal.

## Props and events

Provide a persistent `label`, stable option IDs, labels, and optional disabled flags. Uncontrolled `new` accepts default values. Controlled construction emits a value request and applies owner updates through `set_values`. The `values`, `is_open`, and `active_option_id` accessors expose the current state for host coordination. `OpenChanged` fires once on user-driven visibility changes.

## Keyboard map

Register the component key context so host applications can rebind actions. Arrow keys move among enabled options; Home/End go to boundaries; Enter/Space opens or toggles membership while open; Escape closes without changing selection. Tab and Shift+Tab use GPUI's normal forward and backward focus traversal and close the popup when leaving the trigger. Selected values remain unchanged. Disabled ignores all of these actions.

## Pointer behaviour

Clicking the trigger toggles the popup. Clicking an enabled row toggles its membership and keeps the popup open; the row consumes its click so the trigger does not toggle. Disabled rows cannot change values and also keep the popup open.

## Accessibility role and properties

Expose a named combobox trigger with expanded state, selected option labels joined in option order as its accessible value (empty when none are selected), and a listbox popup. Options expose labels, selection, and disabled state. Multi-select listboxes expose multiselectable semantics. Disabled option rows set the AccessKit disabled property.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7.ts`, styled by `.ui-input`,
`.ui-badge--secondary`, `.ui-popover`, `.ui-menu*`, and `.ui-check` in `site/src/ui/ui.css` with the
shadcn token mapping in `site/src/ui/tokens.ts`). It shares Select's trigger and popup treatment and
is resolved from the installed `Theme` in three variants, the same way Button, Checkbox, and Tabs do
it. `high-contrast` is selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" below is shadcn's `accent`/`secondary`: `text` mixed 4% (light) or 12%
(dark) into `background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Trigger fill | `background` | `background` | `background` |
| Trigger border ("input") | `border` | `text` at 15% | `border` |
| Trigger shadow | `shadows.small` | `shadows.small` | none |
| Value chip | muted fill, `text` | muted fill, `text` | `background` fill, `borders.hairline` in `border`, `text` |
| Placeholder (no values; shows the label) | `text_muted` | `text_muted` | `text_muted` |
| Chevrons | `text_muted` | `text_muted` | `text` |
| Popup fill ("popover") | `surface` | `surface` | `background` |
| Popup border | `border` | `text` at 10% | `border` |
| Popup shadow | `shadows.medium` | `shadows.medium` | none |
| Option checkbox, unselected | `background` fill, "input" border | same | `background` fill, `border` border |
| Option checkbox, selected | `accent` fill and border, `accent_text` check | same | same |
| Active option | muted fill, `text` | muted fill, `text` | `accent` fill, `accent_text`; its checkbox inverts to an `accent_text` border (and fill when selected) with an `accent` check |
| Disabled option | text and checkbox colours 50% over the popup fill | same | `disabled` text and checkbox (a `background` check when selected) |
| Keyboard focus | trigger border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Disabled control | every trigger colour mixed 50% over `background`, shadow alpha halved | same | `background` fill, `disabled` text, chips, border, and chevrons |

- **Focus** follows `:focus-visible`: the ring shows while the control owns keyboard focus and the
  last input was from the keyboard (`Window::last_input_was_keyboard`), including while the popup is
  open, because focus stays on the trigger. The ring replaces the resting shadow. GPUI paints drop
  shadows as filled shapes that are not clipped to the element's outside, so the trigger and popup
  fills are always opaque (the web trigger's transparent fill composites to `background`).
- **Disabled** matches the web preview's `opacity: .5` as one layer, the way Checkbox does it: each
  colour is composited over `background` and mixed 50% with it. GPUI element opacity is not used
  because it dims each painted part separately. High contrast keeps solid colours instead.
- **Geometry**: the trigger is at least `controls.medium` (36px) tall and wraps when the chips do
  not fit; it has `spacing.xsmall`/`spacing.small` (4/8px) padding, a `spacing.xsmall` gap, radius
  `radii.medium`, and a `borders.regular` border, as in the preview's `padding: 4px 8px; gap: 4px`.
  The placeholder adds a `spacing.xsmall` leading margin so its text lines up with Select's 12px
  inset. Chips follow `.ui-badge`: `spacing.small` (8px) horizontal padding, radius `radii.medium`,
  and `typography.caption` (12px) text at medium weight (500). Their height is `spacing.xlarge`
  (24px); the preview's 22px has no token, and 24px is the nearest one that still fits inside the
  36px trigger with its padding and border (including the 2px high-contrast border). Chips carry no
  remove button: the preview's `x` icon would suggest a pointer target the component does not
  implement, so it is left out until chip removal is specified. The trailing icon is Lucide
  `chevrons-up-down` (7,15 → 12,20 → 17,15 and 7,9 → 12,4 → 17,9 on a 24-unit grid, 2-unit stroke)
  drawn as a vector path in a `spacing.large` (16px) square; there are no icon assets.
- **Popup**: radius `radii.medium`, a `borders.regular` border, and `spacing.xsmall` (4px, `p-1`)
  padding. Option rows are `controls.small` (32px) tall with `spacing.small` (8px) horizontal
  padding, radius `radii.small`, and a `spacing.small` gap. Each row leads with a decorative
  checkbox that matches Checkbox: a `spacing.large` (16px) box with radius `radii.small`, a
  `borders.hairline` border, and Lucide `check` drawn with a 3-unit stroke in a `spacing.medium`
  (12px) square. The checkbox adds no accessibility node; the option's selected state carries the
  meaning. Pointer hover gives enabled rows the muted fill (high contrast leaves rows unchanged on
  hover). The 3px focus ring is the shadcn/ui ring
  width, a fixed component value. The popup keeps the trigger's width and an 8px
  (`spacing.small`) gap below the trigger.

The viewport displays up to eight rows and uses GPUI `uniform_list` virtualization for the full option collection. Keyboard navigation scrolls the active row into view. The popup is placed in the window overlay at the measured trigger's left edge with a small spacing gap. Its full height (the row viewport plus the popup padding and border) is compared with available space below and above; it flips above when the popup plus gap does not fit below and above offers more room. Window snapping preserves a medium spacing margin, and the viewport remains clipped when neither side fits.

For virtualized lists, scrolling to a later viewport must preserve source-option identity: clicking a mounted row after wheel scrolling toggles that row's membership and keeps the popup open. This interaction is covered by a large-list GPUI test.

## WAI-ARIA pattern reference

Follow the WAI-ARIA APG combobox and listbox patterns, adapted to GPUI's accessibility tree.

## Platform notes

Popup placement uses measured trigger geometry and available window space, flipping above when the bottom placement cannot fit and the upper side offers more room. GPUI window snapping preserves the medium margin; platform screen-reader verification remains pending. Outside pointer dismissal uses GPUI's popup hit testing.

## Open questions

Maintainer review is required for public naming and keyboard/accessibility contracts, including Tab dismissal and whether controlled selection should show a pending request before owner application.
