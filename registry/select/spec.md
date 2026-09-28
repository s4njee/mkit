---
spec_version: 1
component: select
states:
  - id: closed
    description: Select button shows the selected label or placeholder.
    fixture: closed_fixture
  - id: open
    description: Listbox is anchored to the measured trigger, constrained to the window with a margin, and uses an active enabled option; long lists use a clipped virtual scroll viewport.
    fixture: open_fixture
  - id: disabled
    description: Select is omitted from sequential Tab focus and cannot change.
    fixture: disabled_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: Select is focused
    action: Open and move active option to next enabled item.
    initial_state: closed
    expect: { state: open, event: open }
  - key: Enter
    modifiers: []
    when: Popup is open
    action: Commit active option and close.
    initial_state: open
    expect: { state: closed, event: change }
  - key: Escape
    modifiers: []
    when: Popup is open
    action: Close without changing committed value.
    initial_state: open
    expect: { state: closed }
  - key: Home
    modifiers: []
    when: Popup is open
    action: Move active option to first enabled item and scroll it into view.
    initial_state: open
    expect: { state: open }
  - key: End
    modifiers: []
    when: Popup is open
    action: Move active option to last enabled item and scroll it into view.
    initial_state: open
    expect: { state: open }
accessibility:
  role: combobox
  properties:
    - { name: aria-haspopup, value: listbox }
    - { name: aria-value, value: selected option label or empty string }
    - { name: aria-expanded, value: false, when: closed }
    - { name: aria-expanded, value: true, when: open }
    - { name: active descendant, value: active option while open, when: open }
    - { name: aria-disabled, value: true, when: disabled }
---

# Select

## Purpose

Select lets users choose option values from a bounded collection. Choose one value.

## Anatomy

A labeled trigger displays the selected option labels. Opening it reveals an in-window combobox and listbox; active navigation is distinct from committed selection.

## States

Closed, open, and disabled states are defined in the manifest. The open state includes an active enabled option; disabled rows remain visible and cannot be selected.

## Props and events

Provide a persistent `label`, stable option IDs, labels, and optional disabled flags. Uncontrolled `new` accepts a default value. `value` reads the committed value and `is_open` reads popup visibility. Controlled construction emits `ValueChanged` as a value request and applies owner updates through `set_value`; controlled display state does not change until the owner applies the request. `OpenChanged` fires once when user input changes visibility. Disabled Selects ignore keyboard and pointer input.

## Keyboard map

Register the component key context so host applications can rebind actions. Arrow keys move among enabled options; Home/End go to boundaries; Enter/Space opens or commits (Space toggles membership for multi-select); Escape closes without a single-value change; Tab closes and traverses normally.

An enabled Select trigger participates in normal sequential Tab order. A disabled Select trigger is skipped by Tab traversal. Tab closes an open popup and advances to the next focus stop; Shift+Tab closes it and moves to the previous focus stop. Traversal preserves the committed value.

## Pointer behaviour

Clicking the trigger toggles the popup. Clicking an enabled row commits a single selection, emits `ValueChanged`, and closes the popup. Disabled rows cannot change values. Clicking outside the inline popup closes it through GPUI outside-hit testing.

## Accessibility role and properties

Expose a named combobox trigger with expanded state, the committed option label as its accessible value (empty when no option is selected), and a listbox popup. Options expose labels, selected state, and disabled state. GPUI supplies an active-descendant focus mapping while the trigger is focused; platform-specific screen reader announcements still require native accessibility verification. Multi-select listboxes expose multiselectable semantics.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7.ts`, styled by `.ui-input`,
`.ui-popover`, and `.ui-menu*` in `site/src/ui/ui.css` with the shadcn token mapping in
`site/src/ui/tokens.ts`), which in turn follows the shadcn/ui select. It is resolved from the
installed `Theme` in three variants, the same way Button, Checkbox, and Tabs do it. `high-contrast`
is selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" below is shadcn's `accent`/`secondary`: `text` mixed 4% (light) or 12%
(dark) into `background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Trigger fill | `background` | `background` | `background` |
| Trigger border ("input") | `border` | `text` at 15% | `border` |
| Trigger shadow | `shadows.small` | `shadows.small` | none |
| Value text | `text` | `text` | `text` |
| Placeholder (no value; shows the label) | `text_muted` | `text_muted` | `text_muted` |
| Chevron and option check | `text_muted` | `text_muted` | `text` |
| Popup fill ("popover") | `surface` | `surface` | `background` |
| Popup border | `border` | `text` at 10% | `border` |
| Popup shadow | `shadows.medium` | `shadows.medium` | none |
| Active option | muted fill, `text` | muted fill, `text` | `accent` fill, `accent_text` (check too) |
| Disabled option | `text` 50% over the popup fill | same | `disabled` |
| Keyboard focus | trigger border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Disabled control | every trigger colour mixed 50% over `background`, shadow alpha halved | same | `background` fill, `disabled` text, border, and chevron |

- **Focus** follows `:focus-visible`: the ring shows while the Select owns keyboard focus and the
  last input was from the keyboard (`Window::last_input_was_keyboard`), so it also marks the trigger
  while the popup is open, because focus stays on the trigger. The ring replaces the resting shadow.
  GPUI paints drop shadows as filled shapes that are not clipped to the element's outside, so the
  trigger and popup fills are always opaque (the web trigger's transparent fill composites to
  `background`). Ring corners use the trigger radius rather than CSS's radius-plus-spread.
- **Disabled** matches the web preview's `opacity: .5` as one layer, the way Checkbox does it: each
  colour is composited over `background` and mixed 50% with it. GPUI element opacity is not used
  because it dims each painted part separately. High contrast keeps solid colours instead.
- **Geometry**: trigger height `controls.medium` (36, shadcn `h-9`), horizontal padding
  `spacing.medium` (12, `px-3`), radius `radii.medium`, a `borders.regular` border, value text
  `typography.body` (14px), and a `spacing.small` gap before the chevron. The chevron is Lucide
  `chevron-down` (6,9 → 12,15 → 18,9 on a 24-unit grid, 2-unit stroke) drawn as a vector path in a
  `spacing.large` (16px) square, like the Breadcrumbs separator; there are no icon assets. The popup
  has radius `radii.medium`, a `borders.regular` border, and `spacing.xsmall` (4px, `p-1`) padding.
  Option rows are `controls.small` (32px) tall, matching shadcn's `py-1.5` around a 20px line, with
  `spacing.small` (8px, `px-2`) horizontal padding and radius `radii.small`. A selected option shows
  Lucide `check` (20,6 → 9,17 → 4,12) in a 16px square at the row's end. Pointer hover gives
  enabled rows the muted fill (not in high contrast). The 3px focus ring is the shadcn/ui ring width, a fixed component
  value. The popup keeps the trigger's width and an 8px (`spacing.small`) gap below the trigger;
  shadcn uses a 4px side offset, but the placement contract below is unchanged.

The viewport displays up to eight rows and uses GPUI `uniform_list` virtualization for the full
option collection. Keyboard navigation scrolls the active row into view. The popup is placed in the
window overlay at the measured trigger's left edge with a small spacing gap. Its full height (the
row viewport plus the popup padding and border) is compared with available space below and above;
it flips above when the popup plus gap does not fit below and above offers more room. Window
snapping preserves a medium spacing margin, and the viewport remains clipped when neither side fits.

For virtualized lists, scrolling to a later viewport must preserve source-option identity: clicking a mounted row after wheel scrolling commits the row at that visible index, closes the popup, and emits only its value request. This interaction is covered by a large-list GPUI test.

## WAI-ARIA pattern reference

Follow the WAI-ARIA APG combobox and listbox patterns, adapted to GPUI's accessibility tree.

## Platform notes

Popup placement uses measured trigger geometry and available window space, flipping above when the bottom placement cannot fit and the upper side offers more room. GPUI window snapping preserves the medium margin; platform screen-reader verification remains pending. The current component exposes GPUI's active-descendant focus mapping, but platform screen-reader behavior and the synthesized combobox/listbox relationship have not yet been verified on a supported native platform. Outside pointer dismissal uses GPUI's outside-hit testing.

## Open questions

Maintainer review is required for public naming and keyboard/accessibility contracts.
