# Benchmark 05: Markdown previewer

## Goal

Build a read-only previewer for the Markdown file `fixture/doc.md`. It renders headings,
paragraphs, lists, inline styles, and code blocks, and can switch to the raw source.

## Required features

Parse this Markdown subset yourself (no parser crate is available):

- `#` to `######` headings.
- Paragraphs: consecutive non-blank lines, joined with one space. Blank lines separate blocks.
- List items: lines starting with `- `. Each item is its own block.
- Fenced code blocks between lines starting with three backticks; the text after the opening
  fence is the language. Code keeps its line breaks and is not styled inline.
- Inline styles in paragraphs and list items: `**bold**`, `*italic*`, and `` `code` ``. Markers are
  removed from the displayed text.

The preview shows headings larger by level, bold and italic text, inline code and code blocks in a
monospace font on a tinted background, and list bullets. Long lines wrap to the window width, and
the document scrolls vertically. Source mode shows the file text unchanged in a monospace font.
Keyboard focus starts on the document.

## Keyboard

| Key | Command |
|---|---|
| `secondary-e` | Switch between preview and source |
| `secondary-r` | Reload the file from disk |
| `ctrl-down` / `ctrl-up` | Make the next or previous heading current and scroll it to the top of the view |
| `home` | Scroll to the top; no heading is current |

## Targets

None required.

## Snapshot

```json
{
  "mode": "preview",
  "blocks": [
    {"kind": "heading", "level": 1, "text": "Field notes"},
    {"kind": "paragraph", "text": "GPUI renders retained state ...",
     "spans": [{"text": "GPUI renders ", "bold": false, "italic": false, "code": false}]},
    {"kind": "list_item", "text": "first item", "spans": []},
    {"kind": "code_block", "lang": "rust", "text": "fn main() {\n..."}
  ],
  "current_heading": null,
  "scroll_top": 0.0
}
```

`text` is the displayed text without markers; for paragraphs and list items it equals the spans'
texts joined. Adjacent spans with the same style may be merged or not. `current_heading` indexes
the heading blocks only (0 is the first heading) and is `null` when none is current. `scroll_top`
is the preview's vertical scroll offset in logical pixels.

## Acceptance criteria

- **AC0** The crate builds, including `src/main.rs`.
- **AC1** The blocks of `doc.md` are, in order: heading 1 `Field notes`; a paragraph starting
  `GPUI renders retained state`; heading 2 `Lists`; three list items; heading 2 `Code`; a `rust`
  code block of three lines; heading 2 `Long section`; 60 paragraphs; heading 2 `Last heading`;
  paragraph `Final paragraph.`
- **AC2** The first paragraph has a bold span `retained state`, an italic span `immediate-style`,
  and a code span `cx.notify()`; its text contains no `*` or backtick. The second list item has a
  bold span `bold`.
- **AC3** `secondary-e` sets `mode` to `source`, and again to `preview`.
- **AC4** After `doc.md` is replaced with `# Changed` and a paragraph `New body.`, `secondary-r`
  shows exactly those two blocks.
- **AC5** `ctrl-down` five times makes heading 4 (`Last heading`) current with `scroll_top` above
  0; `ctrl-up` makes heading 3 current; `home` returns `scroll_top` to 0 and `current_heading` to
  `null`.
- **AC6** (screenshot) The first frame is not blank, and source mode renders differently from the
  preview.

<!-- maintainer-only -->

## Book coverage

Answerable from Parts I–IV: `elements/text.md` (font sizes, weights, styles, wrapping, rich text
runs), `elements/size-and-scroll.md` (a scrollable region and its offset), and
`elements/conditional-lists.md` (a block per parsed item). The cookbook's rich-text and scroll-region
recipes are optional shortcuts. Scrolling a specific heading into view needs a scroll handle and
child bounds; if agents fail AC5 often, that points at a gap in `size-and-scroll.md`.

## Harness coverage and gaps

Rendering fidelity (heading sizes, monospace code, wrapping) is judged only by AC6's frame change
and by human review of the saved screenshots. No accessibility criterion is defined yet; the
document structure (headings, lists) is a good candidate once the harness can read the tree.
