# Actions and key bindings

## What you'll build

List commands such as Move Up and Activate, with a caller-selected alternate Move Down binding. Run `cargo run -p mkit-example-interaction --locked` from the workspace root. To change the alternate key at startup, run `MKIT_MOVE_DOWN_KEY=ctrl-k cargo run -p mkit-example-interaction --locked`.

![Interaction list showing its arrow, Alt-J, Enter, and clipboard shortcut help](../images/interaction.png)

## Concept

An *action* names a command independently of its key. `actions!` defines actions. `KeyBinding::new` pairs a key sequence with an action in a named context, and `App::bind_keys` registers the bindings in the app's *keymap*, its set of shortcut rules. A `key_context` describes where a binding applies; `on_action` handles it. The [pinned dispatch guide](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/key_dispatch.rs#L1-L52) says focused inner contexts take precedence over outer contexts.

## Minimal compiling example

This action setup comes from the compiling `examples/interaction/src/lib.rs`.

```rust
{{#include ../../../examples/interaction/src/lib.rs:interaction_actions}}
```

Run `cargo test -p mkit-example-interaction --lib --locked`. The screenshot above is checked by `cargo test -p mkit-example-interaction --test screenshot --locked` on macOS at 1280 × 1040 and scale 2.0.

## How it works

At startup, `bind_interaction_keys` validates one caller-selected alternate keystroke and registers it with the default bindings. `main` reads `MKIT_MOVE_DOWN_KEY`, defaulting to Alt-J, and the view shows the chosen key in its help text. The focused list declares `InteractionList` as its key context and handles Move Up, Move Down, Activate, Copy, Paste, and Add Row. The test supplies `ctrl-k`, presses Down and Ctrl-K, then checks that Enter activates row three. This verifies a configurable binding at startup; it is not a live keymap editor. In nested views, the pinned dispatch guide says an inner focused context wins a binding conflict. For keyboard listeners, [capture runs from root toward focus, then bubble runs from focus toward root](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L91-L105). This fixture does not test a nested-context conflict or both dispatch phases.

## Common mistakes

**A replacement shortcut invokes a different command.** [Zed issue #61941](https://github.com/zed-industries/zed/issues/61941) reports a later Zed keymap case where rebinding Ctrl-W behavior to Ctrl-Q in the same contexts did not reproduce it. This is not evidence that this fixture or GPUI pin has that bug. Use the action and context path, inspect competing bindings, and test the alternate key in the focused view. [`KeyBinding::new`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/binding.rs#L33-L50) also requires a valid key string.

## Exercises

Change the caller-supplied alternate in the test to another valid keystroke; the same Move Down action should run. Add a nested key context with a competing key and assert which action receives it before making a claim about this app's dispatch order.

## API reference links

- [`actions!`, `KeyBinding`, `App::bind_keys`, `key_context`, `on_action`, `DispatchPhase`](../appendices/api-inventory.md) · [pinned binding source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/binding.rs#L33-L50)
