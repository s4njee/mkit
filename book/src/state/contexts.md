# Which GPUI context do you have?

## What you'll build

Follow one app-owned refresh state from startup into an async task and back to an entity update. The `examples/contexts` crate is nonvisual: its tests verify the update, and its `main` is a compile fixture that does not open a window. For a visible [Window](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L1144), run the separate hello example with `cargo run -p mkit-example-hello`.

![The separate hello window with a counter and keyboard readouts](../images/hello.png)

## Concept

GPUI passes a context to code that needs application services. The type tells you what work is available:

| Type | Where this chapter gets it | Use it for |
| --- | --- | --- |
| [`App`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L686) | The `Application::run` callback and `setup` | App-wide state, creating entities, spawning work, opening windows |
| [`Context<T>`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L21) | An entity's update or render method | The current entity's `notify`, plus app operations through its `App` access |
| [`Window`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L1144) | A window's root-view builder or `Render::render` | Window-local drawing, focus, and input operations |
| [`AsyncApp`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/async_context.rs#L22) | The `App::spawn` callback | Holding an app handle across `await`, then re-entering app state with `update` |

[`AppContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L172) is the public trait shared by several context types for operations such as creating and updating entities. It does not mean every callback has a window. The [pinned `gpui-pre` 0.3.5 guide](https://docs.rs/crate/gpui-pre/0.3.5) introduces the app, root view, and element tree.

## Minimal compiling example

These anchored regions come from the compiling `examples/contexts` crate. `RefreshState::finish` uses its entity context; `start_refresh` crosses into `AsyncApp`; `setup` starts from `&mut App`:

```rust
{{#include ../../../examples/contexts/src/lib.rs:contexts_model}}
```

```rust
{{#include ../../../examples/contexts/src/lib.rs:contexts_start_refresh}}
```

```rust
{{#include ../../../examples/contexts/src/lib.rs:contexts_app_setup}}
```

```rust
{{#include ../../../examples/contexts/src/main.rs:contexts_main}}
```

The hello binary shows where a real window enters startup:

```rust
{{#include ../../../examples/hello/src/main.rs:hello_main}}
```

Run `cargo test -p mkit-example-contexts --locked`. The two unit tests drive the async task deterministically and check both a completed update and a released weak entity. `cargo run -p mkit-example-contexts` only checks that its nonvisual startup compiles and launches; it does not keep a window open or prove that the task completed. Run `cargo run -p mkit-example-hello` to see a window. On macOS, `cargo test -p mkit-example-hello --test screenshot --locked` compares that *separate* hello view with the image above at 1280 × 800 and scale 2.0 after GPUI Kit initialization. Windows and Linux compile the screenshot test but report unsupported headless capture for this pin. The contexts example itself has no screenshot.

## How it works

`gpui_platform::application().run` calls the startup closure with `&mut App`. The nonvisual `contexts` main passes that value to `setup`. There, [`AppContext::new`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L172-L181) creates a `RefreshState` entity. `setup` stores a cloned strong handle in a `Global` owner, starts a refresh, and detaches its task. The global owner keeps the model alive in this fixture; the returned handle gives tests another way to inspect it.

`start_refresh` receives an app context and converts its entity handle to a `WeakEntity`. [`App::spawn`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L2009-L2020) gives the task `&mut AsyncApp`, which can be held across async suspension. `apply_refresh` uses [`AsyncApp::update`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/async_context.rs#L165-L171) to enter `&mut App` briefly. It upgrades the weak handle only if the model is still alive, then calls `state.update` and `RefreshState::finish`. That method receives `&mut Context<RefreshState>` and calls `notify()` after setting `completed`.

The first test calls `setup`, runs the test executor until parked, and reads `completed == true`. The second keeps the task handle, drops its only strong model handle, checks that the task becomes ready after the executor runs, and confirms that the weak handle still cannot upgrade. It verifies the task ran; it does not claim a window or screenshot was rendered.

The hello binary passes [`WindowOptions`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2044) to [`App::open_window`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L1283-L1290). Its root-view builder receives a `Window` and `App`, although it does not need the window argument there. The resulting view receives `&mut Window` in its `Render::render` method, as shown in the [hello chapter](../examples/hello.md). That is where window-local work belongs.

## Common mistakes

**Moving a borrowed app context into background work.** The compiler may reject a borrowed `cx` that must outlive its callback. A [Zed discussion about asynchronously initializing a global](https://github.com/zed-industries/zed/discussions/41041) reports this situation and a reply recommends `cx.spawn` for app work. That discussion concerns a dynamic global value, not this refresh fixture. In this pin, use `App::spawn` to receive `AsyncApp`, then use `AsyncApp::update` for a short app-state update, as the compiling example and tests do. Keep the returned [`Task`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13) or detach it intentionally; [`Context::spawn` documents that lifetime rule](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L236-L246).

## Exercises

1. In `RefreshState::finish`, add another Boolean field and set it before `cx.notify()`. Extend `app_spawn_updates_entity_through_async_app` to read both fields; run `cargo test -p mkit-example-contexts --locked` and expect both to be true.
2. In the released-entity test, keep one extra cloned strong handle until after `run_until_parked`. Confirm `weak.upgrade()` remains possible while that clone exists, then drop it and confirm upgrade fails. This tests ownership without needing a window.
3. In the hello example, locate the `Window` argument in `Render::render`. Compare it with the `&mut App` passed to the startup closure. Run the hello binary to confirm the view still opens; leave the example unchanged for its screenshot comparison.

## API reference links

- [App](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L686), [AppContext](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L172), [Context](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L21), [Window](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L1144), [AsyncApp](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/async_context.rs#L22), [Entity](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L414), [WeakEntity](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L740), [Task](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13)
