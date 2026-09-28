# Benchmark 04: File browser

## Goal

Build a keyboard-driven file browser rooted at the fixture directory. It lists a directory, moves
into and out of subdirectories, and opens files.

## Required features

- The browser starts in the fixture directory, which is its root: it never shows anything above it.
- Each listing shows directories first, then files; each group is sorted by name ignoring case.
  Files show their size in bytes. Entries whose names start with `.` are hidden until the user
  shows hidden entries.
- A header shows the current path relative to the root (empty at the root).
- The selected entry is highlighted and kept visible when the list is taller than the window.
- Opening a file records it as the opened file and shows its path in a status line. Opening a
  directory lists it and selects its first entry.
- Going to the parent selects the directory you just left.
- Refresh re-reads the current directory from disk and keeps the current path.
- Keyboard focus starts on the list. An unreadable directory shows an error message instead of
  crashing.

## Keyboard

| Key | Command |
|---|---|
| `up` / `down` | Move the selection (stops at the ends) |
| `enter` | Open the selected directory or file |
| `backspace` | Go to the parent directory (does nothing at the root) |
| `secondary-shift-.` | Show or hide entries starting with `.` |
| `secondary-r` | Refresh |

## Targets

| Name | Element |
|---|---|
| `entry-<name>` | Row for the entry with that name, e.g. `entry-notes.txt`; clicking selects it |

## Snapshot

```json
{
  "path": "",
  "entries": [{"name": "docs", "kind": "dir", "size": null},
              {"name": "notes.txt", "kind": "file", "size": 30}],
  "selected": 0,
  "opened": null,
  "show_hidden": false
}
```

`path` and `opened` are relative to the root and use `/`. `selected` is `null` for an empty
directory. `kind` is `"dir"` or `"file"`; `size` is `null` for directories.

## Acceptance criteria

- **AC0** The crate builds, including `src/main.rs`.
- **AC1** The first listing is `docs`, `src`, `Zeta` (directories), then `Cargo.toml`, `notes.txt`,
  `README.md` (files) with their sizes on disk, `selected` 0, path `""`.
- **AC2** `enter` opens `docs` (entries `guide`, `readme.md`), `enter` opens `docs/guide`, and
  `backspace` returns to `docs` with `guide` selected. From the root, `down enter` then
  `backspace` returns to the root with `src` (index 1) selected.
- **AC3** `backspace` at the root changes nothing. `up` at the first entry and `down` past the last
  entry keep the selection in range.
- **AC4** `.hidden` is not listed at first; `secondary-shift-.` lists it first among the files and
  sets `show_hidden`; pressing it again hides it.
- **AC5** Clicking `entry-notes.txt` selects it; `enter` sets `opened` to `notes.txt` without
  changing `path`. Opening `src/main.rs` sets `opened` to `src/main.rs`.
- **AC6** After a new file `new.txt` is created in the root, `secondary-r` lists it.
- **AC7** (screenshot) The first frame is not blank, and entering `docs` changes the frame.
- **AC8** (accessibility) When GPUI exposes an accessibility tree, it names the root entries.

<!-- maintainer-only -->

## Book coverage

Answerable from Parts I–IV with synchronous `std::fs` reads: `interaction/keyboard-list.md` is the
core pattern (selection, actions, key context), `elements/conditional-lists.md` covers keyed rows,
`elements/size-and-scroll.md` scrolling, and `interaction/mouse.md` row clicks. Part VII
(`async/file-browser.md`) shows the background-loading version; it is optional here because the
fixture is small, and it becomes relevant if a later revision adds a slow-directory criterion.

## Harness coverage and gaps

Scroll-into-view of the selection and the unreadable-directory message are specified but not
asserted: the first needs a scroll-offset readout, and the second is hard to create portably
(permissions differ on Windows). AC8 is skipped while headless accessibility is inactive.
