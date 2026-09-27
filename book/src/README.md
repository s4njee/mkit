# The GPUI Book

GPUI is a Rust library for making desktop apps. You describe the content of a window in Rust, and GPUI lays it out, draws it, and sends input back to your code. This book teaches those ideas through small, working examples in the mkit workspace.

You can read from start to finish if GPUI is new to you. If you already have an app in mind, use the table below to jump to a topic. Each teaching chapter explains the idea, shows code from a compiling example, points out common mistakes, and gives you something to try. The Cookbook offers shorter answers to specific tasks.

The examples target the workspace's pinned `gpui-pre` version. Read [Versions](appendices/versions.md) before adding GPUI to your own app. The book is still growing: some native OS checks, release steps, and public API coverage are unfinished. The [API inventory](appendices/api-inventory.md) and [concept index](appendices/concept-index.md) show what is covered now.

## Table of contents

| Part | What you will learn |
| --- | --- |
| [I. Getting started](install.md) | Set up the tools, open a window, and make a counter. |
| [II. State and views](state/entities.md) | Store state in entities, update it, and build views from it. |
| [III. Elements, styling and layout](elements/div-and-layout.md) | Arrange controls, style text, add images, and build a settings screen. |
| [IV. Interaction](interaction/mouse.md) | Handle pointer and keyboard input, focus, drag and drop, the clipboard, and menus. |
| [V. Text input and IME](text-input/handler-contract.md) | Build a text field, track selections, and handle composing text. |
| [VI. Custom rendering](custom-rendering/element-lifecycle.md) | Draw your own elements, use overlays and lists, and build a canvas. |
| [VII. Async, data and persistence](async/executors-and-tasks.md) | Keep slow work off the UI thread and bring results back safely. |
| [VIII. Windows, platform and shipping](windows-platform-shipping/windows-and-appearance.md) | Work with windows, accessibility, diagnostics, and packaging. |
| [IX. Testing GPUI apps](testing/test-contexts.md) | Test state and input, then compare screenshots on supported platforms. |
| [X. Components](components/e7/) | Browse E7 everyday controls and the new E8 pro-app drafts, with behavior and captures. |
| [XI. Coming from another UI framework](coming-from/react.md) | Connect GPUI ideas to React, Tauri, iced, and egui. |
| [XII. Architecture internals](architecture/frame-pipeline.md) | Follow a frame, text rendering, and an entity's lifetime. |
| [XIII. Cookbook](cookbook/state/entity-counter.md) | Copy short, tested recipes for common tasks. |
| [Appendices](appendices/versions.md) | Find version notes, a glossary, the API inventory, and the concept index. |

You can also use the chapter list at the side of the page or search at the top. Start with [Install](install.md) if you are setting up a new workspace.

## Build and check the book

Book Rust samples come from compiling example crates. Screenshots of rendered examples come from the test harness. On macOS, screenshot tests compare captures with checked-in images. The pinned GPUI renderer does not support headless pixel comparison on Linux or Windows; [Testing GPUI apps](testing/harness-screenshots.md) explains that limit.

From the workspace root, install mdBook and Node.js, then run:

```sh
python3 scripts/check_book.py
mdbook build book
python3 scripts/check_book.py --skip-cargo --built-html
```

The checker compiles examples and checks code includes, local links, and image sources. For spelling, run `uvx --from codespell==2.4.3 codespell --quiet-level 2 book/src`. The local link check does not fetch external sites. The version banner reads the exact `gpui-pre` pin from `Cargo.toml` during the build.
