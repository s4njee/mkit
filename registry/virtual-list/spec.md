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
theme_tokens: [surface, text, border, accent, accent_text, focus, spacing.small, spacing.medium]
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

`surface`, `accent`, `accent_text`, `text`, `border`, `borders.hairline`, and `spacing.medium` come from GPUI `Theme`; row height is an API value.

## WAI-ARIA pattern reference

[WAI-ARIA APG Listbox Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/listbox/) guides the selection and keyboard contract.

## Platform notes

GPUI 0.3.5 `uniform_list` owns scrolling and lazy row construction. The screenshot fixture shows an empty state, a populated unselected viewport, and a selected first row while the active row has moved to the second item with ArrowDown. The selected fixture dispatches Space and ArrowDown to the focused list so active and selected state remain visibly independent. Each state is captured with the light, dark, and high-contrast themes at 1× and 2× scale. The generated conformance manifest declares fixture cases, not executed harness evidence.

## Open questions

Verify active-item announcements and multi-selection semantics on the active platform. Variable-height rows are not supported.
