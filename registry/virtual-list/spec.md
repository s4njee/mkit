---
spec_version: 1
component: virtual-list
states:
  - id: empty
    description: No items are available.
    fixture: empty_fixture
  - id: idle
    description: A bounded scroll viewport renders only rows intersecting its current scroll position from a larger collection.
    fixture: idle_fixture
  - id: selected
    description: One or more rows are selected.
    fixture: selected_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: list is focused
    action: Move active row to the next item.
    initial_state: idle
    expect:
      event: ActiveChanged
  - key: Space
    modifiers: []
    when: list is focused
    action: Toggle the active row's selection.
    initial_state: idle
    expect:
      event: SelectionChanged
accessibility:
  role: listbox
  properties:
    - name: accessible-name
      value: component label
    - name: options
      value: each rendered option has listbox-option role, its item label as accessible name, selection state, set position, and set size
    - name: active-descendant
      value: active rendered option is exposed as the root's active descendant where supported
controlled: Owner supplies selected IDs; requests emit SelectionChanged and apply only after set_selection. Uncontrolled mode updates selection before emitting.
events: [ActiveChanged, SelectionChanged]
theme_tokens: [background, surface, text, border, accent, accent_text, focus, spacing.xsmall, spacing.small, radii.small, radii.large, borders.hairline, typography.body]
open_questions: []
---

# Virtual list

## Purpose

Render a bounded, scrollable viewport over a large, fixed-height, selectable list. GPUI `uniform_list` determines the visible row range from the live scroll offset and lazily builds only those rows.

## Anatomy

A focusable list root contains a fixed-height `uniform_list` viewport. The GPUI scroll handle owns its scroll position; GPUI computes the visible range and lazily builds rows. Each item has a stable ID and label.

## States

`empty` has no items, `idle` displays an unselected window with the first row active, and `selected` has one or more selected IDs. Active and selected rows are independent. An empty list has no active row; when items first become available, the first row becomes active without emitting a navigation event.

## Props and events

`VirtualList::new` owns selection; `controlled` accepts selected IDs and emits selection requests until `set_selection` applies owner state. `single_selection` switches from the default multiple selection. `row_height` defaults to 32 px and clamps to at least 1 px. `viewport_height` defaults to 320 px and clamps to at least 1 px. GPUI `uniform_list` owns live scroll position and derives visible rows; `set_items` replaces items, and `visible_range` remains a pure clamped range helper. `set_visible_window` is retained as a deprecated compatibility method; it requests the start row at the top and ignores count while GPUI derives the rendered range. Navigation emits `ActiveChanged` and scrolls the active row into view; Space toggles the active ID and emits `SelectionChanged`. Clicking a row activates it and uses the same selection toggle request; in controlled mode the owner must apply that request through `set_selection`.

## Keyboard map

`MkitVirtualList` binds Down/Up to adjacent items, Home/End to first/last item, and Space to toggle selection of the active item. The actions are rebindable and navigation does not wrap. Navigation scrolls the active row into view, including when it lies outside the currently rendered range.

## Pointer behaviour

The list viewport responds to GPUI wheel scrolling. Clicking a rendered row activates it and toggles its selection. In single-selection mode, selecting a row clears the previous selection; clicking the selected row clears it.

## Accessibility role and properties

The focusable root has listbox role and accessible label. Rendered rows have listbox-option role, the item's label as their accessible name, selected state, set position, and set size; the active row requests active-descendant semantics. Off-window rows are absent from the accessibility tree. The pinned GPUI bridge does not expose a multi-selectable setter here, so the default multi-selection mode needs active-platform semantics review.

## Theme tokens used

`background`, `surface`, `text`, `border`, `accent`, `accent_text`, `focus`, `borders.hairline`,
`spacing.xsmall/small`, `radii.small/large`, and `typography.body` come from the GPUI `Theme`; row
height is an API value.

The look follows the docs-site web preview (`site/src/demos/e7.ts`, the `virtual-list` demo: a card
holding `.ui-menu__item` option rows, styled in `site/src/ui/ui.css` with the shadcn token mapping in
`site/src/ui/tokens.ts`). Colours are resolved from the installed `Theme` in three variants, the way Button, Select, Tabs,
and Sidebar do it: `high-contrast` is selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Accent" below is shadcn's `accent`/`muted`: `text` mixed 4% (light) or 12% (dark)
into `background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| List fill | `surface` | `surface` | `surface` |
| List border | `border` | `text` at 10% | `border` |
| Row text | `text` | `text` | `text` |
| Selected row | accent fill, `text` | accent fill, `text` | `accent` fill, `accent_text` |
| Pointer hover (unselected) | accent fill | accent fill | row outline in `border` |
| Active row with keyboard focus | `borders.hairline` outline in `focus` inside the row | same | same |
| Empty list with keyboard focus | list border `focus` plus a 3px ring of `focus` at 50% | same | ring of opaque `focus` |

- **Geometry.** The list is a card: radius `radii.large`, a `borders.hairline` border, and
  `spacing.xsmall` (4px, the web's `padding: 4px`) padding inside the scroll viewport. Rows fill the
  width, keep the API row height (default 32px, the web's 32px rows), and have radius `radii.small`,
  `spacing.small` (8px) horizontal padding, and `typography.body` (14px) text.
- **Active row.** The active row is the listbox's active descendant. It is outlined only while the
  list has keyboard focus (`:focus-visible`: focused and the last input was from the keyboard), so
  a pointer-selected row shows just its selection fill, as the web preview does. Each row reserves a
  transparent hairline border so the outline does not shift content. A focus ring outside a row
  would overlap its neighbours inside the scroll viewport, so rows use the inside outline instead;
  the list itself shows the border-plus-ring focus treatment only when it has no rows to outline.
- In high contrast the selected row keeps the solid `accent` fill with `accent_text`, and hover draws
  the reserved row border in `border` instead of a subtle fill.

## WAI-ARIA pattern reference

[WAI-ARIA APG Listbox Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/listbox/) guides the selection and keyboard contract.

## Platform notes

GPUI 0.3.5 `uniform_list` owns scrolling and lazy row construction. The screenshot fixture shows an empty state, a populated unselected viewport, and a selected first row while the active row has moved to the second item with ArrowDown. The selected fixture dispatches Space and ArrowDown to the focused list so active and selected state remain visibly independent. Each state is captured with the light, dark, and high-contrast themes at 1× and 2× scale. The generated conformance manifest declares fixture cases, not executed harness evidence.

## Open questions

Verify active-item announcements and multi-selection semantics on the active platform. Variable-height rows are not supported.
