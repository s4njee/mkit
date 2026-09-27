# mkit

mkit is a planned GPUI book and component kit for desktop and pro apps. The [project plan](plan.md) describes the roadmap. The current foundation establishes the workspace, dependency pin, CI, and the first book pages; the component and example APIs are still under development.

## Build

See the book's [Install chapter](book/src/install.md) for platform dependencies. From the repository root:

```sh
cargo build --workspace
cargo test --workspace
```

The book source is in [`book/`](book/). With mdBook installed, run `mdbook build book`.

The [Aria2 Manager consumer sample](apps/aria2-sample/README.md) is a standalone GPUI application built against the public `mkit` crate with mock data. It exercises several widgets and records the remaining release gaps.

## Inspector

On macOS, start the MCP inspector as a stdio server from the repository root:

```sh
cargo run -p mkit-inspector --locked
```

Configure an MCP client to launch that command and communicate over stdin/stdout. Launch `hello` or `gallery` first; this build supports one fixture window per process. The tools are `launch` (`example: "hello"` or `"gallery"`), `screenshot` (PNG), `press` (`keys`, using space-separated GPUI keystrokes), `type` (`text`), `click` (`target: "@increment"` in the hello fixture, or numeric `x` and `y`), `entity_state`, and `a11y_tree`. The last tool currently returns an accessibility-inactive error in the stock headless window. The pinned GPUI version provides headless screenshots only through Metal on macOS; inspector launch therefore cannot render on Windows or Linux. See the [headless spike](docs/spikes/headless.md) for test results and limits.

## Dependency baseline

mkit pins the `gpui-pre` package at `=0.3.5`, matching the GPUI version used by GPUI Kit 0.6.4. See the [Versions appendix](book/src/appendices/versions.md) for the Cargo dependency form and upgrade policy.

## License

mkit is currently licensed under [Apache License 2.0](LICENSE). The plan's choice between Apache-2.0 alone and a future MIT/Apache-2.0 dual license remains open. [NOTICE](NOTICE) is the place for required third-party notices as source and assets are added.
