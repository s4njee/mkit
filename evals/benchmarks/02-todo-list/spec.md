# Benchmark 02: Todo list with keyboard shortcuts

## Goal

Build a todo list that can be driven entirely from the keyboard: type new items into a draft line,
move through the list, mark items done, delete them, and reorder them.

## Required features

- The list starts with three items, in order: `buy milk`, `write chapter`, `review pr`. None is done.
- A draft line above the list. When it has focus, printable keys append to the draft, `space`
  appends a space, `backspace` removes the last character, and `enter` adds the trimmed draft as a
  new item at the end of the list, selects it, and clears the draft. A blank draft adds nothing.
- The list shows every item with a done marker, highlights the selected item, and shows a status
  line reading `N of M done`.
- Two focus stops: the draft (focused first) and the list. `tab` and `shift-tab` move focus
  between them. Focused elements show a visible focus style.
- List commands are GPUI actions bound in a key context, so they can be rebound.

## Keyboard

With the list focused:

| Key | Command |
|---|---|
| `up` / `down` | Move the selection (stops at the ends) |
| `space` | Toggle done on the selected item |
| `backspace` | Delete the selected item; the selection stays at the same index, or moves to the new last item |
| `alt-up` / `alt-down` | Move the selected item up or down one place, keeping it selected |

## Targets

| Name | Element |
|---|---|
| `draft` | The draft line |
| `todo-<n>` | Item row `n` |
| `toggle-<n>` | Done marker of item `n`; clicking toggles done without changing the selection |

Clicking a row selects it and focuses the list.

## Snapshot

```json
{
  "items": [{"text": "buy milk", "done": false}],
  "selected": 0,
  "draft": "",
  "focus": "draft"
}
```

`selected` is `null` when the list is empty. `focus` is `"draft"` or `"list"`.

## Acceptance criteria

- **AC0** The crate builds, including `src/main.rs`.
- **AC1** The first snapshot has the three seeded items, none done, `selected` 0, an empty draft,
  and focus on the draft.
- **AC2** Typing `stretch` then `enter` adds `stretch` as item 3, selects it, and clears the draft.
  Typing `a b` gives a draft of `a b`; `backspace` gives `a `.
- **AC3** `enter` with an empty draft, or a draft of only spaces, adds nothing.
- **AC4** `tab` focuses the list; `down space` marks `write chapter` done; `up up` keeps the
  selection at 0; `shift-tab` returns focus to the draft.
- **AC5** In the list, `down backspace` deletes `write chapter` and selects `review pr` (index 1);
  `alt-up` moves `review pr` to index 0 and keeps it selected; `alt-down` moves it back.
- **AC6** Clicking `todo-2` selects item 2 and focuses the list; clicking `toggle-0` marks
  `buy milk` done and leaves the selection at 2.
- **AC7** (screenshot) The first frame is not blank, and toggling an item done changes the frame.
- **AC8** (accessibility) When GPUI exposes an accessibility tree, it names every item.

<!-- maintainer-only -->

## Book coverage

Answerable from Parts I–IV. The draft uses the key-down listener from `examples/hello.md`, not the
Part V input handler. `interaction/focus.md` covers the two tab stops and focus-visible styling,
`interaction/actions.md` and `interaction/keyboard-list.md` cover the bound list commands, and
`elements/conditional-lists.md` covers keyed rows and the conditional done marker. A candidate that
uses the Part V text field (`text-input/single-line-field.md`) also passes, since the tests type
through GPUI keystroke dispatch.

## Harness coverage and gaps

All behavior is checked through snapshots after keyboard and pointer scripts. Focus-visible
styling is not asserted; AC7 checks only that done state is visible. AC8 is skipped while headless
accessibility is inactive.
