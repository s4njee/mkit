# KeyHint

KeyHint presents modifier chords in platform-aware form. Apps supply the main key label so special keys can be localized.

KeyHint also provides the shared shortcut formatter used by the dropdown and context menus, CommandPalette, and ShortcutEditor. `KeyChord::parse` reads one GPUI keystroke such as `cmd-shift-s`, a `+` label such as `Ctrl+Shift+P`, or a macOS glyph label such as `⌘⇧S`. `KeyChord::label` and `shortcut_label` then write the platform form: `⇧⌘S` on macOS and `Shift+Super+S` elsewhere, with modifiers in platform order. Text that cannot be parsed, such as a two-step sequence or a chord using `fn`, is shown unchanged. Those components render the chord with `KeyHint::inline()`, which draws the label as one text run in the row's own color and size, not as keycaps.

![KeyHint platform chord, dark theme, 2×](../../images/e7/key-hint.png)

The draft registry contract is in `registry/key-hint/spec.md`. The screenshot matrix spans declared states, three themes, and two scales. Platform glyph support, spoken labels, the parse grammar, and API review remain pending. The inline form is covered by the menu, command palette, and shortcut editor screenshot matrices.
