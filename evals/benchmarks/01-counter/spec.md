# Benchmark 01: Counter

## Goal

Build a window that shows a count and lets the user change it with buttons and with the keyboard.

## Required features

- A label reading `Count: N`, starting at 0.
- Three buttons: **Increment** (+1), **Decrement** (-1, never below 0), and **Reset** (back to 0).
- The same three commands as GPUI actions, bound to keys in a key context named `Counter`, so an
  app could rebind them.
- Keyboard focus starts on the counter view.

## Keyboard

| Key | Command |
|---|---|
| `up` | Increment |
| `down` | Decrement (stays at 0) |
| `escape` | Reset |

## Targets

| Name | Element |
|---|---|
| `increment` | Increment button |
| `decrement` | Decrement button |
| `reset` | Reset button |

## Snapshot

```json
{"count": 0}
```

## Acceptance criteria

- **AC0** The crate builds, including `src/main.rs`.
- **AC1** The first snapshot is `{"count": 0}`.
- **AC2** Clicking `increment` three times, then `decrement` once, gives count 2. Clicking `reset`
  gives 0.
- **AC3** Pressing `up up up down` gives 2, and `escape` then gives 0.
- **AC4** Pressing `down` at 0, or clicking `decrement` at 0, leaves the count at 0.
- **AC5** (screenshot) The first frame is not blank, and the frame after three `up` presses differs
  from the first frame.
- **AC6** (accessibility) When GPUI exposes an accessibility tree, it contains a button named
  `Increment`.

<!-- maintainer-only -->

## Book coverage

Answerable from Parts I–IV: `examples/counter.md` (retained count, GPUI Kit button, `notify`),
`state/entities.md` and `state/reactivity.md` (update and notify), `interaction/actions.md`
(`actions!`, `KeyBinding`, `key_context`), `interaction/focus.md` (initial focus), and
`examples/hello.md` (focus on first render, `Default`-style root construction).

## Harness coverage and gaps

AC1–AC4 run in `acceptance.rs` with GPUI's `VisualTestContext`. AC5 runs in `screenshots.rs` on
macOS only; other hosts record it as skipped. AC6 is recorded as skipped while headless GPUI
windows report accessibility as inactive (see `mkit_harness::AccessibilitySnapshot`). Rebinding
through a different key is not tested because the grader cannot name the candidate's action types.
