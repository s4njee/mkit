# Benchmark 06: Split-pane editor

## Goal

Build a two-pane text editor: a file list on the left, a line editor on the right, and a divider
the user can drag or move with the keyboard. Edits are kept per file until saved.

## Required features

- The left pane lists the files in the fixture directory, sorted by name. Each shows a marker
  while it has unsaved changes.
- The right pane edits the open file as lines of text with a visible cursor. A file's lines are its
  text split on `\n`, ignoring one trailing newline. Saving writes the lines joined with `\n` plus a
  trailing newline, and clears the unsaved marker.
- Each file keeps its own edited buffer while you switch between files, until it is saved.
- A vertical divider between the panes. The left pane starts 200 logical pixels wide and stays
  between 120 and 400. Dragging the divider changes the width by the drag distance.
- Focus starts on the file list. Nothing is open at first. Focused panes show a visible focus
  style.
- A plain key-down listener is enough for editing; IME and selection are not required.

## Keyboard

| Key | Where | Command |
|---|---|---|
| `up` / `down` | File list | Move the selection |
| `enter` | File list | Open the selected file and focus the editor (cursor at line 0, column 0 on first open) |
| printable keys, `space` | Editor | Insert at the cursor |
| `enter` | Editor | Split the line at the cursor |
| `backspace` | Editor | Delete before the cursor; at column 0, join with the previous line |
| `left` / `right` | Editor | Move one character, crossing line ends |
| `up` / `down` | Editor | Move one line, keeping the column when possible, otherwise the line end |
| `home` / `end` | Editor | Start or end of the line |
| `secondary-s` | Editor | Save the open file |
| `escape` | Editor | Focus the file list |
| `alt-left` / `alt-right` | Anywhere | Make the left pane 20 pixels narrower or wider |

## Targets

| Name | Element |
|---|---|
| `file-<name>` | File row, e.g. `file-b.txt`; clicking opens it and focuses the editor |
| `divider` | The divider |

## Snapshot

```json
{
  "files": [{"name": "a.txt", "dirty": false}, {"name": "b.txt", "dirty": false}],
  "selected": 0,
  "open": null,
  "lines": [],
  "cursor": {"line": 0, "column": 0},
  "sidebar_width": 200.0,
  "focus": "list"
}
```

`lines` and `cursor` describe the open file (`lines` is empty when none is open). Columns count
characters. `focus` is `"list"` or `"editor"`.

## Acceptance criteria

- **AC0** The crate builds, including `src/main.rs`.
- **AC1** The first snapshot lists `a.txt` and `b.txt` with nothing open and focus on the list.
  `enter` opens `a.txt` with lines `alpha`, `beta`, cursor (0, 0), and focus on the editor.
- **AC2** Typing `x` gives `xalpha`, cursor column 1, and marks `a.txt` dirty. `end` then `!` gives
  `xalpha!`.
- **AC3** In a freshly opened `a.txt`: `end enter` gives `alpha`, `""`, `beta` with the cursor at
  (1, 0); typing `mid` fills the empty line; `home backspace` joins to `alphamid`, `beta` with the
  cursor at (0, 5); `down` moves to (1, 4); `left left up` moves to (0, 2). `left` at (0, 0) stays.
- **AC4** Typing `z` then `secondary-s` writes `zalpha\nbeta\n` to `a.txt` and clears its dirty
  flag.
- **AC5** After typing `q` in `a.txt`, `escape down enter` opens `b.txt` (lines `gamma`) while
  `a.txt` stays dirty; reopening `a.txt` shows `qalpha`.
- **AC6** Dragging `divider` 80 pixels right gives width 280; dragging far right clamps at 400.
  `alt-left` repeated clamps at 120, and `alt-right` then gives 140.
- **AC7** (screenshot) The first frame is not blank; opening `a.txt` and typing `hello` changes the
  frame.

<!-- maintainer-only -->

## Book coverage

Answerable from Parts I–IV plus the Part XIV cookbook: the key-down listener from
`examples/hello.md` is enough for line editing, `interaction/focus.md` covers the two focus areas,
`interaction/actions.md` the bindings, `interaction/mouse.md` the divider drag, and
`elements/div-and-layout.md` the two-pane layout (with `cookbook/rendering/split-pane.md` as an
optional recipe). Part V (`text-input/*`) is the better foundation for a real editor but is not
required by these criteria. Maintainer decision: plan §8 lists this as an M1 app while Part V is
P1; the spec deliberately avoids IME and selection so M1 does not depend on Part V.

## Harness coverage and gaps

Cursor rendering and focus styling are judged only by AC7's frame change and human review.
Pointer placement of the cursor is not specified.
