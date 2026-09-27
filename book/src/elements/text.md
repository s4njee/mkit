# Style and fit text

## What you'll build

Add a settings heading, explanatory text, a label that fits a narrow row, and a line with differently styled runs. Run `cargo run -p mkit-example-settings-screen` from the workspace root.

![Settings screen with a bold Preferences heading, wrapped description, and truncated account email](../images/settings-screen.png)

![Scrolled settings screen showing bold Workspace sync at the start of a rich text sentence](../images/settings-screen-scrolled.png)

## Concept

[`Styled`](../appendices/api-inventory.md) text builders set color, font family, weight, size, and line height on an element. These values cascade to children unless a child overrides them. `whitespace_normal()` permits wrapping; `whitespace_nowrap()` prevents it. The pinned [`truncate()`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L139-L141) combines hidden overflow, no wrapping, and an ellipsis; its effect becomes visible when the text has a narrower available width than it needs.

For differently styled parts of one string, [`StyledText::with_runs`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L526-L539) accepts [`TextRun`](../appendices/api-inventory.md) values. A run contains a font, color, optional decorations, and a length in **UTF-8 bytes**. The run lengths must partition the whole string at valid byte boundaries; the pinned method panics when they do not.

## Minimal compiling example

The heading, bounded account label, and rich line come from the compiling settings-screen crate:

```rust
{{#include ../../../examples/settings_screen/src/lib.rs:settings_screen_text}}
```

```rust
{{#include ../../../examples/settings_screen/src/lib.rs:settings_screen_text_truncate}}
```

```rust
{{#include ../../../examples/settings_screen/src/lib.rs:settings_screen_text_runs}}
```

Run `cargo run -p mkit-example-settings-screen`. On macOS, `cargo test -p mkit-example-settings-screen --test screenshot --locked` compares top and bottom 1920 × 1280 images at scale 2.0. The first shows the heading, wrapped description, and truncated email; the second shows the styled sync line. The tests do not assert text shaping or truncation independently. Windows and Linux report unsupported headless capture.

## How it works

`intro` sets a large bold heading and bounds explanatory copy to two lines with `line_clamp(2)`. `account_card` gives the email a narrow available width and uses `truncate()` so it stays within its row. `sync_section` constructs one text string and two runs from `window.text_style()`, then makes the first run bold and uses the theme's primary color. The lengths come from Rust string `.len()`, which counts UTF-8 bytes. The source's [`TextStyle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L438-L475) and [`Styled` methods](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L514-L543) control the inherited and local values.

The compiled `with_runs` call establishes that the two run lengths cover the message. The scrolled screenshot verifies their initial appearance in this theme and scale. A focused unit assertion would give stronger evidence for a future change to the run boundaries.

## Exercises

1. Replace one rich-text segment with a word containing a multibyte character. Derive each run length from its segment, run the example and tests, and confirm the line still renders.
2. Widen the truncated label's box and observe whether its ellipsis disappears. Restore the original width before comparing the committed image.

## API reference links

- [`Styled`, `TextStyle`, `StyledText`, and `TextRun`](../appendices/api-inventory.md) — text builders, inherited style, and rich runs.
- [`Font`, `FontWeight`, `WhiteSpace`, and `TextOverflow`](../appendices/api-inventory.md) — font and fitting values.
