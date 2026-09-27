# Book style guide

Use [CHAPTER_TEMPLATE.md](CHAPTER_TEMPLATE.md) for every teaching chapter. The page should move from the result to the idea, a working example, an explanation, mistakes, exercises, and API links. Keep setup details close to the step that needs them.

## Write for a new GPUI reader

- Use plain language and short sentences. Prefer a concrete action or observation to an abstract summary.
- Define a term when it first appears. For example, explain that an *entity* owns state before introducing `gpui::Entity<T>`. Do not use unexplained jargon or assume the reader knows Zed internals.
- Use the exact public names and signatures in the pinned `gpui-pre` version (`=0.3.5` in the workspace `Cargo.toml`). Check the [public API inventory](src/appendices/api-inventory.md) and its source links. Do not copy an API from Zed's moving `main` branch into an example without checking the pin.
- Describe what the example actually does. Separate observed behavior from a proposed fix or an upstream report. State platform limits when they affect the instructions.
- Keep paragraphs focused. Introduce one behavior, show the relevant code or output, then explain it.

## Code, images, and links

- Put Rust code in a compiling crate under `examples/`. Use `// ANCHOR: name` and `// ANCHOR_END: name` in the Rust source, then include that region through mdBook `{{#include}}` in the chapter's `rust` fence. The [book checker](../scripts/check_book.py) rejects handwritten Rust blocks and invalid anchors.
- Build the example crate and run its relevant tests. Give exact commands and expected results in the chapter. Do not present a fragment as a runnable app if the included region needs other setup.
- If an example renders, use a PNG under `book/src/images/` produced by an `examples/` harness screenshot test. Write alt text that identifies the visible state. Inspect screenshot diffs before updating a baseline. Report the tested theme, scale, and platform; explain any untested matrix entries.
- Link a public GPUI name to its entry in the [inventory](src/appendices/api-inventory.md) at first useful mention. The inventory links to the pinned Zed commit. When linking source directly, use a commit-specific `https://github.com/zed-industries/zed/blob/<commit>/...` URL and verify that the cited line contains the claim. Use upstream issue or discussion URLs for reported problems, and check that each URL resolves.
- Use descriptive link text. Prefer a local chapter link for background already explained in the book. Do not link to a moving source branch as evidence for a pinned API.

## Ground “Common mistakes” in evidence

Each entry needs a symptom, a cause supported by the cited report or pinned source, and a correction the reader can apply. A report may describe a newer Zed revision or a narrow platform case. Say so instead of presenting it as a universal `gpui-pre` 0.3.5 bug. These official reports are useful starting points:

- **Typing goes to the view behind a prompt.** [Zed issue #60840](https://github.com/zed-industries/zed/issues/60840) reports two Windows fallback-prompt cases where the prompt did not take focus and keyboard navigation affected the picker behind it. For a chapter on dialogs or prompts, tell readers to test initial focus and key dispatch after opening the overlay. Do not claim every GPUI prompt has this problem.
- **A new dialog does not focus its first input.** [Zed discussion #57205](https://github.com/zed-industries/zed/discussions/57205) describes a dialog whose container holds focus, so typing does not reach the input. It also explains why calling `Window::focus_next` during opening can miss tab stops registered in the new frame. For a chapter on focus, show an explicit focus target that exists in the pinned API and test opening and closing quickly. The discussion's proposed `request_initial_focus` names are illustrative, not public APIs.

Choose a report that matches the chapter's behavior. Verify its details in the report before writing, and verify any GPUI API name against the pinned inventory. If no relevant upstream report exists, leave the entry out and record the gap for review; do not invent one.

## Before review

Run the book checker, the example crate's build and tests, and `mdbook build book`. Check links and rendered output. Record commands, screenshot evidence, and any platform limits in the PR. Ask a maintainer to review API and naming claims, keyboard and accessibility behavior, visual baseline changes, and book accuracy.
