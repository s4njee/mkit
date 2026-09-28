# GPUI Book evaluation: benchmark {number}, {title}

You are building a small desktop app with GPUI, the Rust UI framework behind the Zed editor.
Your only reference is **The GPUI Book** in `{book}` inside this workspace. You also have the
`mkit-inspector` MCP server, which can launch the book's example apps, send them input, and return
screenshots, accessibility trees, and state.

Rules:

- Work only inside this directory. Do not read files outside it, do not search the web, and do not
  read GPUI or GPUI Kit source code from the cargo registry. The point of this evaluation is to
  find out whether the book alone is enough, so when the book does not answer a question, say so
  in your final message and make your best attempt.
- Cargo runs offline. Use only the dependencies already in `app/Cargo.toml`.
- Read `TASK.md` for what to build and `CONTRACT.md` for the items the grader needs from your
  crate. Put your code in `app/`. Test data for the app is in `fixture/`.
- Check your work with `cargo build` and `cargo test` in this directory. Your own tests are welcome.

When you are done, stop and reply with a short summary: what you built, what you verified, and
any place where the book was missing, unclear, or wrong.
