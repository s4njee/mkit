# Keep settings, files, and SQLite separate

## What you'll build

Use an in-memory display setting, list a read-only sample directory, and save a preference in a separate SQLite database adapter. Run the browser with `cargo run -p mkit-example-async-file-browser --locked` from the workspace root. The SQLite adapter is a tested library example; the browser does not call it.

![Read-only file browser showing sorted sample entries](../images/async-file-browser-loaded.png)

## Concept

GPUI's [`Global`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/global.rs#L22-L35) marks app-scoped, type-keyed memory state. [`App::set_global`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L2072-L2109) installs it; this example reads it when starting a directory load. A global does not save to disk. Files come from Rust's [`std::fs::read_dir`](https://doc.rust-lang.org/std/fs/fn.read_dir.html). Database persistence belongs to a separate adapter built with pinned [`rusqlite` 0.40.2](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html).

## Minimal compiling example

The setting and file adapter come from the compiling browser crate.

```rust
{{#include ../../../examples/async_file_browser/src/lib.rs:file_browser_settings}}
```

```rust
{{#include ../../../examples/async_file_browser/src/lib.rs:file_browser_fs}}
```

The SQLite adapter is a separate compiling crate.

```rust
{{#include ../../../examples/async_persistence/src/lib.rs:sqlite_store}}
```

Run `cargo test -p mkit-example-async-file-browser --lib --locked` for directory sorting, hidden entries, empty directories, and missing paths. Run `cargo test -p mkit-example-async-persistence --locked` for transaction, parameter, close-and-reopen, and replacement behavior. The browser's macOS screenshot is checked by `cargo test -p mkit-example-async-file-browser --test screenshot --locked` at 1800 × 1200 and scale 2.0. That image shows a controlled loaded state, not a database view.

## How it works

`BrowserSettings` is a `Global` with a `show_hidden` flag. The browser copies that flag when it starts a request. `read_directory` reads one chosen directory, skips symlinks, optionally hides dotfiles, and sorts names because the OS gives no stable iteration order. `PreferenceStore` opens a caller-selected database path, creates its own table, saves rows in a transaction with bound parameters, and reads them back. Its test uses a temporary database, closes it, reopens it, and verifies the rows survived. The adapter is synchronous and blocking; a GPUI app should invoke it from background work. This example does not yet connect SQLite writes to the browser UI or provide a secure filesystem sandbox against concurrent path replacement.

## Common mistakes

**Carrying a GPUI context into file or database work.** [Official Zed discussion #41041](https://github.com/zed-industries/zed/discussions/41041) reports a borrow error while a beginner tried to initialize a `Global` in background work. The report is about global initialization, not this adapter. Copy owned inputs into background work and return a result through a foreground GPUI update. Do not treat `Global` as durable storage; the SQLite test demonstrates persistence only for `PreferenceStore`.

## Exercises

Add two files whose creation order differs from lexical order and verify the sorted listing test. Add a preference value containing a quote and run the SQLite test; it should round-trip as data after close and reopen. Keep both exercises on temporary paths.

## API reference links

- [`Global`, `App::set_global`, and `Context::read_global`](../appendices/api-inventory.md) · [pinned GPUI source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L2072-L2109)
- [Rust `read_dir`](https://doc.rust-lang.org/std/fs/fn.read_dir.html) · [rusqlite 0.40.2 `Connection`](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html)
