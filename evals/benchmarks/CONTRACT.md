# Benchmark app contract

Every benchmark app is graded by automated acceptance tests that load your crate as a library.
The tests can only find your app through the items below, so follow them exactly. Everything
else about the design (types, modules, layout, colours) is up to you.

## Crate

- The crate lives in `app/` and is named `bench-app` (library name `bench_app`). Keep the
  provided `app/Cargo.toml` self-contained: do not use `workspace = true` inheritance, because
  the grader copies `app/` into a different workspace.
- The workspace builds offline. Use only the dependencies already listed in `app/Cargo.toml`
  (`gpui-pre`, `gpui-pre-platform`, `gpui-kit`, `serde_json`). New crates cannot be downloaded.
- `src/main.rs` must open a real window showing the same root view. It receives the fixture
  directory as its first command-line argument and defaults to `fixture`.

## Required library items

```text
pub fn init(cx: &mut gpui_pre::App)
pub struct Root            // or `pub use your::View as Root;`, implementing gpui_pre::Render
pub fn root(fixture: &std::path::Path) -> Root
impl Root { pub fn snapshot(&self, cx: &gpui_pre::App) -> serde_json::Value }
```

- `init` runs once before any window opens. Register key bindings (with a key context) and
  install globals or themes here, for example `gpui_kit::init(cx)` if you use GPUI Kit.
- `root` builds the root view **without** an `App` or `Window`. Create focus handles, child
  entities, and subscriptions lazily on the first render if you need them. The fixture argument is
  a directory prepared by the grader; benchmarks that need no data ignore it.
- On its first render the root view must move keyboard focus to wherever the task says keyboard
  input starts, so that key presses work without a click.
- `snapshot` returns a JSON object whose fields are listed in the task's **Snapshot** section. It
  must describe real state that the rendered UI reflects, not a separate record kept for tests.

## Targets

Elements that tests click or drag carry a stable name through GPUI's debug selector:

```text
div().debug_selector(|| "increment".into())
```

The task lists every required name in its **Targets** section. Names with `<n>` are indexed from 0
in the order the items are shown, and names with `<name>` use the item's displayed name.

## How grading runs

- Keyboard and pointer tests open your root with GPUI's test platform, send the keystrokes named in
  the task in GPUI syntax (`down`, `enter`, `secondary-s`, `alt-up`), click targets at their
  centres, and read `snapshot`. `secondary` means Cmd on macOS and Ctrl elsewhere.
- Typed text arrives as one keystroke per character. Spaces arrive as the `space` key.
- Screenshot checks render the root at 800 x 600 logical pixels, scale 1, on macOS.
- Accessibility checks read GPUI's accessibility tree when the platform exposes one.
- The grader also runs `cargo build --bins` on your crate.

You can run `cargo build` and `cargo test` in the workspace at any time. Writing your own tests
with `#[gpui_pre::test]` is encouraged; the book's testing part shows how.
