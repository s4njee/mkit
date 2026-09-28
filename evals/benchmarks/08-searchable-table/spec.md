# Benchmark 08: Searchable table

## Goal

Build a table of 500 rows from `fixture/rows.csv` with a search field, sortable columns, keyboard
selection, and a details panel. Only the rows on screen may be built each frame.

## Required features

- Load `rows.csv`: a header line `id,name,city,amount`, then one row per line with no quoting.
  `id` and `amount` are integers.
- A search field above the table, focused at start. The table shows rows whose name or city
  contains the query, ignoring case. The query updates on every keystroke; `backspace` deletes.
- Column headers. Clicking a header sorts ascending by that column; clicking the same header again
  toggles descending. The default is `id` ascending. Names and cities compare ignoring case;
  numbers compare numerically. Ties are ordered by `id` ascending in both directions.
- The table is virtualized: each frame builds only the rows in or near the visible area, never
  more than 100, whatever the number of matches. The selected row is scrolled into view.
- A details panel shows every field of the selected row.
- A status line shows `N matches`.

## Keyboard

| Key | Where | Command |
|---|---|---|
| `down` | Search | Focus the table and select the first row |
| `up` / `down` | Table | Move the selection one row |
| `pageup` / `pagedown` | Table | Move the selection 20 rows |
| `home` / `end` | Table | Select the first or last row |
| `escape` | Table | Focus the search field, keeping the query |
| `escape` | Search | Clear the query |
| `secondary-f` | Anywhere | Focus the search field |

## Targets

| Name | Element |
|---|---|
| `search` | The search field |
| `header-id`, `header-name`, `header-city`, `header-amount` | Column headers |
| `row-<id>` | A rendered row, by its `id` value; clicking selects it and focuses the table |

## Snapshot

```json
{
  "query": "",
  "sort": {"column": "id", "direction": "asc"},
  "match_count": 500,
  "top_ids": [1, 2, 3],
  "rendered_ids": [1, 2, 3],
  "selected_id": null,
  "details": null,
  "focus": "search"
}
```

`top_ids` holds the ids of the first 20 matching rows in display order. `rendered_ids` holds the ids
of the rows built in the most recent frame. `details` is `{"id", "name", "city", "amount"}` for the
selected row, or `null`. `focus` is `"search"` or `"table"`.

## Acceptance criteria

- **AC0** The crate builds, including `src/main.rs`.
- **AC1** The first snapshot has 500 matches, `top_ids` 1 to 20, `id` ascending, no selection,
  focus on the search field, and between 1 and 100 rendered rows.
- **AC2** Typing `osl` matches exactly the rows whose name or city contains `osl` ignoring case,
  in `id` order. `backspace` three times restores 500 matches.
- **AC3** Clicking `header-amount` sorts by amount ascending; clicking it again sorts descending;
  clicking `header-name` sorts by name ascending. `top_ids` follows each order.
- **AC4** `down` focuses the table and selects id 1; `down down` selects id 3, and `details` matches
  the CSV row. Clicking `row-5` selects id 5.
- **AC5** In the table, `escape` focuses the search field and keeps the query; `escape` again clears
  it. `secondary-f` from the table focuses the search field.
- **AC6** In the table, `end` selects id 500, which is among `rendered_ids`, with at most 100 rows
  rendered; `home` selects id 1; `pagedown` selects id 21.
- **AC7** (screenshot) The first frame is not blank, and typing `osl` changes the frame.

<!-- maintainer-only -->

## Book coverage

Needs Part V for the search field (`text-input/single-line-field.md`; a key-down draft as in
benchmark 02 also passes) and Part VI virtualization (`custom-rendering/virtualization.md`,
`uniform_list`). Part IV covers selection, focus, and header clicks. The mkit DataTable component
is optional; a candidate may use it if the book's component chapters are in context.

## Harness coverage and gaps

`rendered_ids` is self-reported by the candidate; the grader cannot yet count built elements
independently. Debounced search is not required.
