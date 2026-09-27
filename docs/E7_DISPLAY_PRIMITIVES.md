# E7.15 display primitives draft

This slice introduces stateless, theme-token-driven `Badge`, `Avatar`, `KeyHint`, and `Link` components.
Each component has an individual registry spec and generated conformance manifest in `registry/`.

- Badge presents a count or text-labeled status; number formatting remains app-owned.
- Avatar accepts an application image source or caller-provided initials. It does not derive names or fetch images.
- KeyHint renders caller-described chords using target-platform modifier names; adoption by menus, CommandPalette, and ShortcutEditor is a follow-up integration.
- Link is callback-based. Destination resolution/history stay with the app; keyboard Enter dispatches its component action.

Each public API and spec requires maintainer review. The screenshot matrices cover declared states in light, dark, and high-contrast themes at 1× and 2×. Platform accessibility snapshots remain pending.
