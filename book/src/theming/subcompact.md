# Subcompact: a high-density design system

Subcompact is a second design system built on the mkit token set. It is for keyboard-driven, terminal-style tools such as log viewers, consoles, ops dashboards, and inspectors, where the goal is to fit as much on screen as possible. Controls are 18 to 26 logical pixels tall, body text is 11 px monospace, corners are square, and regions are separated by hairlines instead of shadows.

Subcompact does not fork any component. Registry components already read their sizes and colours from the installed [`Theme`](design-language.md), so installing a subcompact theme makes every component denser. The design system lives in `crates/mkit-core/src/subcompact.rs`.

## Principles

- **Density first.** Every size scale moves one or more steps down from the [default design language](design-language.md). A pane holds roughly twice as many rows.
- **Lines, not depth.** Shadows are transparent. Panels, menus, and dialogs are separated by a 1 px `border` and an `elevated_surface` fill. In every subcompact palette the border is at least 3:1 against every surface, so these lines are real boundaries, not decoration.
- **Square, except where shape means something.** `radii.small`, `medium`, and `large` are 0. `radii.pill` stays fully round, because radio buttons, switch tracks, and slider thumbs use the circle to say what they are.
- **Monospace, tight leading.** Text is set in the platform's monospace face with a 1.3 line height, so columns align and one line fits inside the smallest control.
- **Fast.** Motion is 0, 50, 90, and 150 ms. State changes land almost immediately.
- **Contrast is not traded for density.** All three palettes pass every audited WCAG pair, including the border and focus pairs that the default themes still fail.

## Density

| Token group | Default | Subcompact |
| --- | --- | --- |
| `controls` (xsmall / small / medium / large) | 28 / 32 / 36 / 40 | 18 / 20 / 22 / 26 |
| `spacing` (none … xxlarge) | 0 / 4 / 8 / 12 / 16 / 24 / 32 | 0 / 2 / 4 / 8 / 12 / 16 / 24 |
| `typography` (caption / body / body_emphasis / heading_small / heading / heading_large) | 12 / 14 / 14 / 16 / 20 / 28 | 10 / 11 / 11 / 12 / 14 / 18 |
| `radii` (none / small / medium / large / pill) | 0 / 4 / 8 / 12 / 999 | 0 / 0 / 0 / 0 / 999 |
| `borders` (hairline / regular / strong) | 1 / 1 / 2 | 1 / 1 / 2 |
| `motion` (instant / fast / normal / slow, ms) | 0 / 100 / 180 / 280 | 0 / 50 / 90 / 150 |
| `shadows` | soft drop shadows | none (transparent) |

The spacing scale is the default scale shifted down one step, with a new 2 px bottom step. It is not halved, because components also use spacing tokens as glyph sizes: an icon, a checkbox box, a radio circle, and a slider thumb are all `spacing.large`, and a check mark is `spacing.medium`. With the shifted scale those glyphs are 12 px and 8 px. Halving the scale would shrink them to 8 px and 6 px.

`with_subcompact_density(theme)` applies the `typography`, `spacing`, `radii`, and `controls` scales to any theme and keeps its colours, borders, shadows, motion, and name. `with_subcompact_density(SHADCN_DARK)` is a dense shadcn theme. `with_subcompact_density(HIGH_CONTRAST)` keeps the high-contrast borders and the high-contrast component looks, which components select by theme name.

## Palettes

| Constant | Name | Character |
| --- | --- | --- |
| `subcompact::PHOSPHOR` | `subcompact-phosphor` | Green phosphor on near-black, with a bright green accent and an amber focus cue. The default. |
| `subcompact::AMBER` | `subcompact-amber` | Amber monochrome on near-black, with a blue focus cue. |
| `subcompact::PAPER` | `subcompact-paper` | Dark ink on warm paper, with a deep green accent. The light variant. |

`subcompact::PALETTES` lists them dark-first. The focus colour is never a tint of the accent. In a dense layout the focus ring is often the only thing that tells the user where the keyboard is, so each palette gives focus its own hue and keeps it at least 3:1 against the accent fill and every surface.

Install a palette with `set_theme(cx, subcompact::PHOSPHOR)` or with one of the helpers `subcompact::set_phosphor_theme`, `set_amber_theme`, and `set_paper_theme`.

## Text style

mkit leaves the font to the application, so font family and line height are not theme tokens. Subcompact needs both. GPUI's default line height is about 1.618 times the font size, so 11 px body text would need almost 18 px, and it would overflow the fixed-height boxes in badges, key hints, and inspector fields. Call `subcompact::apply_text_style` once on the window's root element. It sets these values, and every child inherits them:

| Setting | Value |
| --- | --- |
| Font family | `subcompact::FONT_FAMILY`: Menlo on macOS, Consolas on Windows, DejaVu Sans Mono elsewhere, all of which ship with the operating system |
| Line height | `subcompact::LINE_HEIGHT`, 1.3 times the font size (14.3 px at the 11 px body size) |
| Text size and colour | `typography.body` and `colors.text` |

The operations console example applies it to its root view:

```rust
{{#include ../../../examples/subcompact_console/src/lib.rs:subcompact_root}}
```

Switching palettes at runtime is an ordinary theme switch:

```rust
{{#include ../../../examples/subcompact_console/src/lib.rs:subcompact_palette_switch}}
```

Run the console with `cargo run -p mkit-example-subcompact-console`, and pass `amber` or `paper` to start in another palette. It combines Breadcrumbs, Badge, SegmentedControl, Button, Tree, Tabs, DataTable, FormLayout, TextField, Switch, Checkbox, Slider, Progress, KeyHint, and StatusBar in one 960 × 600 window.

## Layout guidance

- Use `controls.medium` (22) for rows and inputs, and `controls.small` (20) for toolbar buttons.
- Gap panes with `spacing.xsmall` (2) and pad inside them with `spacing.small` (4).
- Title a pane with an uppercase `caption` in `text_muted` above a hairline, as a terminal multiplexer does.
- Show status as text in a semantic colour (`[WARN]` in `warning`) rather than as a filled chip. Filled `accent` is for the one primary action in view.
- Keep fixed widths in multiples of the 8 px step, so columns line up across panes.

## Common mistakes

- **Skipping the root text style.** Symptom: badges, key hints, and inspector fields clip their text, and columns of numbers do not line up. Cause: GPUI's 1.618 default line height and a proportional font. Correction: call `subcompact::apply_text_style` on the root.
- **Adding shadows back for depth.** Symptom: a popover blurs into the panel beneath it. Correction: keep shadows off and give the overlay `elevated_surface` and a `border`. The subcompact border already meets 3:1 against every surface.
- **Reusing the accent as a focus ring.** In a phosphor layout everything is green. Correction: use `colors.focus`, which each palette gives its own hue.

## Verification status and known gaps

These checks run today on every platform:

- `cargo test -p mkit-core` checks the token relationships (each spacing step equals the default's next step down, one line of body or heading text fits the smallest matching control, shadows are transparent, and corners are square). It also runs the contrast audit, which checks all 23 pairs in each palette, and round-trips every palette through the theme-file format.
- `cargo test -p mkit-example-subcompact-console` renders the console headlessly in each palette and switches palettes at runtime. This is a GPUI test-platform render, not a pixel comparison.

Screenshot baselines are not recorded yet. The harness captures pixels only on macOS, and no subcompact capture has been reviewed. Until it has, the console is unverified visually.

An audit of the registry under subcompact tokens found these issues, which need component changes rather than token changes:

- `FOCUS_RING_WIDTH` is a 3 px constant copied into about 45 components, not a token. Around 12 px glyphs it is relatively heavy. In Tabs and SegmentedControl it is wider than the 2 px list padding, so the ring extends past the list background.
- The DataTable column-resize grip is `spacing.xsmall` wide, which is a 2 px pointer target under subcompact.
- The SplitPane grip dots and the timeline keyframe diamond become very small (about 1 px dots and an 8 px diamond whose focus ring is twice its size).
- Some layouts use fixed pixel sizes that do not scale with tokens: node-editor geometry, the 180 px drag previews in LayerPanel and ReorderableList, and the VirtualList default row height of 32 px. Set `row_height` explicitly in dense lists.
- TextArea positions its wrap width, hit-testing, and IME bounds with a fixed 12 px inset, while it draws text at `spacing.medium` and `spacing.small`. Under subcompact, wrapping and hit-testing are off by a few pixels.
- Menu minimum widths (`spacing.xxlarge * 4`) shrink from 128 to 96 px. They are still wide enough for short labels.

Open questions for maintainers: whether the focus ring width and a font family should become theme tokens; whether subcompact should get its own high-contrast palette, or rely on `with_subcompact_density(HIGH_CONTRAST)`; and whether icons should get their own size token instead of reusing `spacing.large`.
