---
spec_version: 1
component: data-table
states:
  - id: idle
    description: Rows and sortable column headers are shown; no cell is active.
    fixture: idle_fixture
  - id: sorted
    description: Rows are ordered by a selected column and direction.
    fixture: sorted_fixture
  - id: selected
    description: A row is selected.
    fixture: selected_fixture
  - id: active-cell
    description: A data cell is active for keyboard navigation.
    fixture: active_cell_fixture
  - id: active-header
    description: A sortable column header is active for keyboard activation.
    fixture: active_header_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: table is focused
    action: Move active cell down one data row, retaining its column and clamping at the last row.
    initial_state: active-cell
    expect:
      event: ActiveCellChanged
  - key: ArrowUp
    modifiers: []
    when: table is focused
    action: Move active cell up one data row, retaining its column and clamping at the first row.
    initial_state: active-cell
    expect:
      event: ActiveCellChanged
  - key: ArrowRight
    modifiers: []
    when: table is focused
    action: Move right one column in the current row, clamped at the last column.
    initial_state: active-cell
    expect:
      event: ActiveCellChanged
  - key: ArrowLeft
    modifiers: []
    when: table is focused
    action: Move left one column in the current row, clamped at the first column.
    initial_state: active-cell
    expect:
      event: ActiveCellChanged
  - key: Home
    modifiers: []
    when: table is focused
    action: Move to the first column in the active row.
    initial_state: active-cell
    expect:
      event: ActiveCellChanged
  - key: End
    modifiers: []
    when: table is focused
    action: Move to the last column in the active row.
    initial_state: active-cell
    expect:
      event: ActiveCellChanged
  - key: Enter
    modifiers: []
    when: table is focused
    action: Sort the active sortable header or toggle selection of the active data row.
    initial_state: active-cell
    expect:
      event: SelectionChanged
  - key: Enter
    modifiers: []
    when: table is focused on an active sortable header
    action: Sort that column in ascending order on first activation.
    initial_state: active-header
    expect:
      event: SortChanged
  - key: Space
    modifiers: []
    when: table is focused on an active data row
    action: Toggle the active data row selection regardless of activation mode.
    initial_state: active-cell
    expect:
      event: SelectionChanged
accessibility:
  role: grid
  properties:
    - name: accessible-name
      value: component label
    - name: aria-rowcount
      value: row count plus header
    - name: aria-columncount
      value: column count
    - name: rows-and-cells
      value: data rows have row role, a name formed from their cell values, and selection state; each cell has gridcell role and a name combining its column label and cell value
controlled: Owner supplies selected row IDs and sort descriptor; interaction emits SelectionChanged and SortChanged, applied by set_selection/set_sort. Uncontrolled mode applies requests before emitting. Active row and column are navigation state; ActiveChanged reports the data row ID and ActiveCellChanged reports row and column IDs (header row has no row ID). Root aria-activedescendant references the active header or cell.
events: [ActiveChanged, ActiveCellChanged, SelectionChanged, SortChanged, ColumnResized, RowActivated]
theme_tokens: [surface, elevated_surface, text, text_muted, border, accent, accent_text, focus, spacing.small, spacing.medium]
open_questions: []
---

# Data table

## Purpose

Display a small sortable grid with keyboard cell navigation and selectable rows. Consumer apps may render rich cell contents while the table retains grid semantics and interaction ownership.

## Anatomy

A focusable full-width grid root stacks the header and data rows vertically. The header contains sortable column headers; each data row contains grid cells. `Column` carries id, label, width, and sortability; `DataRow` carries id and cell strings. Those strings remain the sort keys and accessible cell values even when an optional cell renderer supplies a richer visual element.

## States

`idle` has no active data cell, `sorted` holds a sort descriptor, `selected` contains selected row IDs, `active-cell` identifies the active data cell, and `active-header` identifies a sortable header for keyboard activation. Active navigation is separate from selection. Empty rows or columns leave navigation inert.

### Screenshot fixture contract (E7.4)

The screenshot matrix is the Cartesian product of `idle`, `sorted`, `selected`, `active-cell`, and `active-header` with the `light`, `dark`, and `high-contrast` shadcn themes at 1x and 2x scales. Each fixture uses a 620×250 logical-pixel window with 16 logical pixels of Global theme padding and one persistent `Entity<DataTable>` labelled `Recent files`. Its deterministic columns, in order, are `name` / Name / width 180, `type` / Type / width 130, and `modified` / Modified / width 130. Its deterministic rows, in source order, are `row-c` / Meeting notes.md / Markdown / Today; `row-a` / Brand guide.pdf / PDF / Yesterday; `row-b` / Budget.xlsx / Spreadsheet / Sep 18; and `row-d` / Project plan.md / Markdown / Sep 17. All columns are sortable.

- `idle`: no input; assert there is no sort descriptor and no selected row. The table's initial keyboard coordinate is the Name header.
- Keyboard-focused states show the Global focus token around the grid.
- `sorted`: focus the table and dispatch Enter; assert sort is Name ascending and no row is selected.
- `selected`: focus the table, dispatch Down then Enter; assert `row-c` is the only selected row.
- `active-cell`: focus the table, dispatch Down then Right; assert the last `ActiveCellChanged` event identifies `row-c` and column `type`, with the table retaining keyboard focus. Selection remains empty, and the active cell has a focus-token border so it remains distinct from row selection.
- `active-header`: focus the table, dispatch End; assert the last `ActiveCellChanged` event identifies the header row (`row: None`) and column `modified`, with the table retaining keyboard focus. The active header uses the accent and accent-text tokens; no sort or selection is applied.

The fixture captures the actual table rendered in the headless window. Keyboard states are produced through registered GPUI actions, not by assigning private navigation fields.

## Props and events

`DataTable::new` owns sort and selection; `controlled` accepts selected IDs and a sort descriptor and emits requests without applying them. `set_selection` and `set_sort` apply owner state. Column widths remain table-owned in both modes. `multi_select` toggles multiple selection. `sort_by` changes ascending/descending direction and sorts strings in uncontrolled mode. `resize_column` enforces a 24 px minimum and emits `ColumnResized`; default column width is 160 px. Navigation emits `ActiveChanged` when the active data row changes and `ActiveCellChanged` when the active coordinate changes.

`with_cell_renderer` accepts a view-local callback receiving the row, column, selected flag, and current Global theme, and returning one GPUI element for the cell interior. It does not own selection, focus, keyboard handling, or accessibility: the table wraps the returned element in its gridcell node. `DataRow.cells` must contain useful plain-text equivalents of the visual content. If the renderer is absent, the table renders that text as before. The callback may capture application-owned data, but the owner must call `set_rows` when data changes so the table redraws. `set_rows` preserves the active row by ID where possible, clamps its column, drops selection IDs absent from the replacement rows, and reapplies an uncontrolled sort. It emits no user-change events for this programmatic update.

`with_header_renderer` analogously receives the column, current sort direction for that column, and Global theme. It supplies only the column-header interior; the table retains sorting pointer and keyboard behavior and the accessible column name. Apps must keep any visual sort indicator in sync with the supplied direction.

`row_height`, `header_height`, `column_gap`, `horizontal_padding`, and `cell_padding` are optional instance layout overrides for dense application grids; their defaults preserve the current small table. `row_divider(true)` applies a hairline theme border between rows. `selection_tint(true)` uses a translucent accent background with ordinary text for selected and active rows. `resizable(false)` hides pointer resize grips. These values are explicit layout policy supplied by the consuming app, not component colors. The table still reads every color, border, focus, and default spacing value from the Global theme.

`RowActivated(row_id)` is emitted on a double-click of a data row. With the default `activate_on_enter(false)`, Enter retains the existing sort-or-select behavior. `activate_on_enter(true)` makes Enter on an active data row emit `RowActivated` while Space still toggles selection. Neither activation mode changes the owner-controlled selection automatically.

## Keyboard map

`MkitDataTable` binds arrows to cell navigation. Up/Down retain the column and move through the header and data rows; Left/Right move columns within the current row. Home/End move to the first/last column in the active row. Enter sorts an active sortable header or toggles selection of the active data row by default; the activation mode emits `RowActivated` for a data row instead. Space always toggles data-row selection. Navigation clamps at boundaries. Actions are rebindable.

## Pointer behaviour

Clicking a sortable column header calls `sort_by`. Clicking a data row focuses the table, makes its first cell active, and toggles row selection. A thin handle on the trailing edge of each header drags to resize that column when resizing is enabled, applying the same 24 px minimum as the API. Releasing the pointer ends the drag. The cell renderer must leave row click handling to the table unless it intentionally contains a separately focusable control.

The table label, row IDs, header labels (`header-label-<column ID>`), and resize handles (`column-resize-<column ID>`) are stable GPUI debug selectors for pointer harness scripts.

## Accessibility role and properties

The root has grid role, accessible label, row count including header, and column count. Headers use columnheader role and expose sort state; data rows use row role, a name composed from their cell values, and `aria-selected`; cells use gridcell role and an accessible name composed from the corresponding column label and cell value. Root active-descendant identifies the active header or cell. Keyboard focus has a visible focus-token ring. The active header uses accent and accent-text emphasis; the active cell has a focus-token border independent of row selection.

## Theme tokens used

`surface`, `elevated_surface`, `text`, `accent`, `accent_text`, `border`, `focus`, `borders.hairline`, `controls.medium`, and `spacing.small/xsmall` are read from the GPUI `Theme`. Default width 160 px and minimum width 24 px are grid sizing policy values.

## WAI-ARIA pattern reference

[WAI-ARIA APG Grid Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/grid/). Focus stays on the root and active-descendant tracks the active cell. Tab enters/exits the grid; interactive widgets inside cells are not supported.

## Platform notes

AccessKit supplies grid semantics. The generated manifest declares fixture cases, not executed harness evidence. Active-descendant and sort semantics still need an active platform accessibility snapshot. A custom cell renderer never substitutes its visual strings for `DataRow.cells` in the accessible name; the caller must provide the text equivalent explicitly.

## Open questions

Keyboard resizing and platform accessibility snapshot coverage remain future work. Maintainer review is required for the cell-renderer callback shape and the layout override names before this becomes a stable public API.
