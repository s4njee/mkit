# E7.15 display primitives draft

This slice introduces stateless, theme-token-driven `Badge`, `Avatar`, `KeyHint`, and `Link` components.
Each component has an individual registry spec and generated conformance manifest in `registry/`.

- Badge presents a count or text-labeled status; number formatting remains app-owned.
- Avatar accepts an application image source or caller-provided initials. It does not derive names or fetch images.
- KeyHint renders caller-described chords using target-platform modifier names. It also owns the shared shortcut formatter (`KeyChord::parse`, `KeyChord::label`, `shortcut_label`) and an inline presentation (`KeyHint::inline()`). DropdownMenu, ContextMenu, CommandPalette, and ShortcutEditor render their shortcut labels through it, so platform key names and modifier order are consistent. Each consumer keeps its string API and muted inline text look. The `mkit-registry-key-hint` dependency is a draft registry exception pending maintainer approval. The menu baselines are unchanged. CommandPalette (`⌘⇧S` now reads `⇧⌘S`) and ShortcutEditor (`cmd-o` now reads `⌘O`) have baseline candidates that need maintainer review. MenuBar adoption is a follow-up.
- Link is callback-based. Destination resolution/history stay with the app; keyboard Enter dispatches its component action.

Each public API and spec requires maintainer review. The screenshot matrices cover declared states in light, dark, and high-contrast themes at 1× and 2×. Platform accessibility snapshots remain pending.
