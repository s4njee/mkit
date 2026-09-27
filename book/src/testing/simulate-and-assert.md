# Simulate input and assert state

## What you'll build

A test that sends a default navigation key, a caller-chosen alternate shortcut, and Enter to the keyboard-driven list. Run `cargo test -p mkit-example-interaction --lib --locked`.

## Concept

`VisualTestContext::simulate_keystrokes` sends GPUI's key syntax through the test dispatch path. An assertion on the retained view state checks the result of the input, rather than merely checking that a handler was called. The [pinned input methods](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_context.rs#L839-L905) also cover text input and mouse events.

## Minimal compiling example

```rust
{{#include ../../../examples/interaction/src/lib.rs:interaction_keyboard_test}}
```

The included function compiles inside the example's test module with its imports and setup helper. Run `cargo test -p mkit-example-interaction --lib --locked`.

## How it works

`setup` installs the `ctrl-k` alternate binding. The test creates a list, draws it, and sends `down ctrl-k enter`. Down and Ctrl-K advance selection, so `selected` becomes 2. Enter activates the selected row, so `activated` becomes 1. The assertion also checks that the displayed alternate key setting is `ctrl-k`. The [list chapter](../interaction/keyboard-list.md) explains the view and action contract.

## Common mistakes

Changing only the displayed shortcut while leaving the installed binding unchanged makes the test's key sequence select a different row. Pass the same alternate key to binding setup and the view, then assert both state and configured value. [Official Zed discussion #17477](https://github.com/zed-industries/zed/discussions/17477) describes GPUI's test macro as a way to simulate interactions; it is not a report of this particular mismatch.

## Exercises

Change the alternate key and simulated sequence together, then rerun the library test. Remove Enter and check that selection moves but activation stays at 0. For pointer input, adapt the [counter test](test-contexts.md) to a second target.

## API reference links

- [`TestAppContext`, `VisualTestContext::simulate_keystrokes`, `Entity::read_with`](../appendices/api-inventory.md) · [pinned test source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_context.rs#L783-L848)
