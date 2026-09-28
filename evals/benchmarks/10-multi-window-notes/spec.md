# Benchmark 10: Multi-window notes app

## Goal

Build a notes app with a main window listing notes and a separate window per open note. Edits in a
note window update the list immediately, and notes are saved to a JSON file.

## Required features

- Notes load from `fixture/notes.json`: `{"notes": [{"id": 1, "body": "..."}]}`. A note's title
  is the first line of its body, or `Untitled` when that line is empty.
- The main window (the root view) lists note titles in `id` order, highlights the selected note,
  and has keyboard focus at start.
- Opening a note shows it in its own window. A note never has two windows: opening it again
  activates the existing window. The note window's title is the note's title and follows edits.
- A note window edits the body as text: typed characters and `space` insert at the cursor, which
  starts at the end of the body; `enter` inserts a line break; `backspace` deletes before the
  cursor. Focus starts in the editor when the window opens.
- Each note is a single shared entity read by both the list and its window, so edits appear in the
  list without a save.
- Saving writes every note to `notes.json` in the same format, ordered by `id`, and clears the
  unsaved-changes flag. Closing a note window does not delete the note.

## Keyboard

| Key | Where | Command |
|---|---|---|
| `up` / `down` | Main window | Move the selection |
| `enter` | Main window | Open the selected note's window, or activate it |
| `secondary-n` | Main window | Create an empty note (next `id`), select it, and open its window |
| `secondary-s` | Any window | Save all notes |
| `secondary-w` | Note window | Close that window |

## Targets

| Name | Element |
|---|---|
| `note-<n>` | Row `n` in the main list; clicking selects it |

## Snapshot

The root (main window) reports:

```json
{
  "notes": [{"id": 1, "title": "Groceries", "body": "Groceries\nmilk, eggs"}],
  "selected": 0,
  "open_note_ids": [],
  "dirty": false
}
```

`open_note_ids` lists, in ascending order, the notes that currently have a window. `dirty` is true
when there are changes since the last load or save.

## Acceptance criteria

- **AC0** The crate builds, including `src/main.rs`.
- **AC1** The first snapshot lists notes 1 (`Groceries`) and 2 (`Ideas`) with their bodies,
  selection 0, no note windows, and `dirty` false. Only the main window is open.
- **AC2** `secondary-n` adds note 3 (`Untitled`, empty body), selects it, and opens a second window
  for it. Typing `plan` there makes note 3's body and title `plan` in the main snapshot, and the
  note window's title `plan`.
- **AC3** `down enter` opens note 2's window; typing `!` there appends `!` to note 2's body in the
  main snapshot and sets `dirty`.
- **AC4** After `secondary-n` and typing `saved`, `secondary-s` in the note window writes
  `notes.json` containing note 3 with body `saved`, and clears `dirty`.
- **AC5** Pressing `enter` twice in the main window leaves exactly one note window, for note 1.
- **AC6** `secondary-w` in the note window closes it: one window remains and `open_note_ids` is
  empty; the note is still listed.
- **AC7** (screenshot) The main window's first frame is not blank.

<!-- maintainer-only -->

## Book coverage

Needs Part VIII windows (`windows-platform-shipping/windows-and-appearance.md`: opening windows,
titles, activation), Part V text input for the editor, Part VII persistence
(`async/settings-files-sqlite.md`), and Part II shared entities and observation
(`state/entities.md`, `state/reactivity.md`).

## Harness coverage and gaps

The tests drive note windows through `VisualTestContext::from_window`. The screenshot harness
renders one headless window, so note windows are not captured. Window activation order is not
asserted.
