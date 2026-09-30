---
spec_version: 1
component: key-hint
states:
  - id: single_key
    description: A single key is shown with its platform display name.
    fixture: single_fixture
  - id: modifier_chord
    description: Modifier keys and a main key are shown in platform order.
    fixture: chord_fixture
  - id: named_key
    description: A special key uses a localized caller-provided display label.
    fixture: named_fixture
keys: []
accessibility:
  role: generic
  properties:
    - name: label
      value: spoken shortcut chord
      when: single_key
---

# KeyHint

## Purpose

Render a keyboard shortcut consistently in menus, command surfaces, and shortcut settings.

## Anatomy

A sequence of compact keycaps shows zero or more modifiers followed by a main key. Modifier names and order are selected for the current operating system; the key label is app-supplied so special key names can be localized. The inline presentation shows the same platform label as a single text run for dense rows such as menu items.

## States

- `Single key`: a main key without modifiers.
- `Modifier chord`: platform-aware modifier keycaps and a main key.
- `Named key`: caller-provided visible label for a special key or localized key name.

## Props and events

Stateless `RenderOnce` builder; no events. A `KeyChord` carries modifier flags and a main-key display string. An optional accessible label can provide localized speech; otherwise the component generates a platform-aware spoken chord name.

Shared formatting API (used by DropdownMenu, ContextMenu, CommandPalette, and ShortcutEditor):

- `KeyChord::parse(text)` accepts one GPUI keystroke string (`cmd-shift-s`, `ctrl-alt-delete`, `secondary-k`), a `+`-separated label (`Ctrl+Shift+P`), or a macOS glyph label (`⌘⇧S`). Modifier names are case-insensitive; `cmd`, `command`, `super`, `win`, `meta`, and `platform` map to the command flag, `alt`, `option`, and `opt` to alt, `ctrl` and `control` to control, and `secondary` to command on macOS and control elsewhere. It returns `None` for empty text, text with inner whitespace (for example multi-keystroke sequences such as `cmd-k cmd-s`), and unrecognized modifiers such as `fn`.
- The parsed main key is a display name: single characters are upper-cased, arrow names become `↑ ↓ ← →`, `escape` becomes `Esc`, `pageup`/`pagedown` become `Page Up`/`Page Down`, lower-case function keys become `F1`…, and other lower-case names are capitalized. Mixed-case key text is kept as written.
- `KeyChord::label()` returns the compact platform label: modifier glyphs followed by the key with no separator on macOS (`⇧⌘S`); `Ctrl`, `Alt`, `Shift`, `Super`, and the key joined with `+` elsewhere (`Ctrl+Shift+S`). `KeyChord::spoken_label()` returns the spoken name joined with ` plus `.
- `shortcut_label(text)` formats text through `parse` and `label`, and returns the trimmed text unchanged when it cannot be parsed, so callers never lose an app-supplied label.
- `KeyHint::inline()` selects the inline presentation: the platform label as one text run without keycap borders, background, text color, or text size, so it inherits the surrounding row typography and color. Menus, CommandPalette, and ShortcutEditor use this presentation to keep their established muted-text shortcut column. The default presentation remains keycaps.

## Keyboard map

No key context; this is a display primitive and does not handle key presses.

## Pointer behaviour

None.

## Accessibility role and properties

Generic, nonfocusable content with an accessible name describing the full chord in reading order. Decorative keycap borders do not create nested accessibility nodes.

## Theme tokens used

The standalone keycap presentation follows the docs-site web preview (`site/src/demos/e7_expansion.ts`,
styled by `.ui-kbd` in `site/src/ui/ui.css` with the shadcn token mapping in
`site/src/ui/tokens.ts`), which in turn follows the shadcn/ui `Kbd`. It is resolved from the
installed `Theme` in three variants, the same way Button, Select, and Tabs do it. `high-contrast` is
selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" below is shadcn's `muted`: `text` mixed 4% (light) or 12% (dark) into
`background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Keycap fill | muted | muted | `background` |
| Keycap border | none | none | `borders.hairline` in `border` |
| Keycap text | `text_muted` | `text_muted` | `text` |

- **Geometry.** Each keycap is 20px tall and at least 20px wide (`spacing.large +
  spacing.xsmall`, shadcn's `h-5 min-w-5`), with `spacing.xsmall` (4px) horizontal padding (the
  preview's 5px has no token), radius `radii.small` (shadcn `rounded-sm`), and centred
  `typography.caption` (12px) text at medium weight (500; there is no font-weight token yet); the
  preview's 11px has no token. Keycaps are separated by `spacing.xsmall` (shadcn `KbdGroup`'s
  `gap-1`).
- **Inline presentation.** `KeyHint::inline` reads no theme tokens: it is one text run that
  inherits colour and size from its parent (menus, CommandPalette, ShortcutEditor), and its
  rendering is unchanged by the keycap styling.
- KeyHint is not focusable, so its screenshot matrix has no focus state.

## WAI-ARIA pattern reference

No dedicated APG pattern applies. Shortcut text follows the ARIA keyboard shortcut guidance; this visual hint does not bind an action or set `aria-keyshortcuts` on another control.

## Platform notes

macOS displays Command, Option, Control, and Shift glyph names. Other platforms display Ctrl, Alt, Super, and Shift text labels. Modifier order follows the platform convention: Control, Option, Shift, Command on macOS, and Ctrl, Alt, Shift, Super elsewhere, so a label written as `⌘⇧S` is displayed as `⇧⌘S`. Main-key text is caller-supplied to allow locale-aware labels; `KeyChord::parse` supplies English display names for GPUI key names, and apps that need localized key names construct `KeyChord` directly. Platform rendering is compile-target dependent: screenshot baselines are macOS captures.

## Open questions

- Maintainer review is needed for platform modifier ordering, glyphs, and spoken labels.
- Maintainer review is needed for the public `KeyChord::parse`, `label`, `spoken_label`, `shortcut_label`, and `KeyHint::inline` API, the accepted parse grammar, and the English default key names (macOS menus conventionally use glyphs such as `↩`, `⌫`, and `⎋`; this draft uses text names for those keys).
- The inline presentation is exercised by the DropdownMenu, ContextMenu, CommandPalette, and ShortcutEditor screenshot matrices rather than by a separate KeyHint state fixture.
- GPUI 0.3.5 reports only role-bearing elements to AccessKit, so KeyHint's generated label is not present in the accessibility tree. Consumers therefore carry the formatted shortcut label in their own row accessible names. Whether KeyHint should take an explicit role, or consumers should set `aria_keyshortcuts`, needs review.
- `fn` chords and multi-keystroke sequences are shown verbatim until KeyChord gains a function modifier and sequence support.
- MenuBar (E7.21) still shows its shortcut text directly; adopting `shortcut_label` there is a follow-up.
