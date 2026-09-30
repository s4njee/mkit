---
spec_version: 1
component: tree
states:
  - id: collapsed
    description: Root nodes are visible and descendants are hidden.
    fixture: collapsed_fixture
  - id: expanded
    description: Expanded descendants are visible.
    fixture: expanded_fixture
  - id: loading
    description: An expanded node awaits asynchronously supplied children.
    fixture: loading_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: tree is focused
    action: Move to the next visible node.
    initial_state: collapsed
    expect:
      event: ActiveChanged
  - key: ArrowRight
    modifiers: []
    when: active node has children
    action: Request expansion of the active branch.
    initial_state: collapsed
    expect:
      event: ExpansionChanged
accessibility:
  role: tree
  properties:
    - name: accessible-name
      value: component label
    - name: items
      value: each visible treeitem has its node label as accessible name and its one-based level
    - name: expanded
      value: expandable treeitems expose expanded state
controlled: Owner supplies expanded node IDs; requests emit ExpansionChanged and apply only after set_expanded. Uncontrolled mode updates before emitting.
events: [ActiveChanged, ExpansionChanged, LoadChildrenRequested]
theme_tokens: [background, surface, text, text_muted, border, accent, accent_text, focus, spacing.xsmall, spacing.small, spacing.large, controls.small, radii.small, radii.large, borders.hairline, typography.body]
open_questions: []
---

# Tree

## Purpose

Navigate a hierarchy with owner-supplied children, including lazy branches.

## Anatomy

A focusable tree root renders visible `TreeNode` rows. Each row has an ID, label, nesting level, and optional children. A lazy branch uses `children: None`; expansion adds a temporary loading row until `set_children` supplies children.

## States

`collapsed` shows root rows, `expanded` shows descendants, and `loading` shows a loading row beneath an expanded lazy branch. The first root is initially active when present; an empty tree has no active row. The active ID and expanded IDs are separate state.

## Props and events

`Tree::new` applies expansion locally; `controlled` receives expanded IDs and emits `ExpansionChanged` requests while the owner applies `set_expanded`. Both modes emit `ActiveChanged` when the active row changes. Expanding a lazy node emits one `LoadChildrenRequested` while its children remain absent; `set_children` replaces its children and clears loading. Controlled mode does not display the loading row until the owner applies the expansion.

## Keyboard map

`MkitTree` binds Down/Up to adjacent visible rows, Right to expand a collapsed branch or move into the first child of an expanded branch, Left to collapse an expanded branch or move to its parent, and Home/End to first/last visible row. These actions are rebindable. Navigation does not wrap. Repeated Right while a controlled expansion request awaits the owner does not issue a duplicate lazy-load request.

## Pointer behaviour

Clicking a visible row outside its disclosure target makes that node active; the tree root remains the keyboard focus target. Expandable rows provide a separate disclosure hit target, spanning the row's leading `spacing.small` padding and its 16px (`spacing.large`) chevron, 24px in all, before the label. Clicking it requests expansion or collapse without activating the row. Controlled expansion changes remain owner-applied: clicking disclosure emits `ExpansionChanged`, but expanded state and lazy loading presentation change only after `set_expanded`. A lazy expansion request emits `LoadChildrenRequested` once while that node remains expanded with children absent; repeated clicks while the request is pending do not duplicate the child-load event. Keyboard Right/Left retain their existing active-row expansion and navigation behavior.

## Accessibility role and properties

The root has tree role and accessible label. Visible rows have treeitem role, the node label as accessible name, and level; expandable rows expose expanded state. The active row requests active-descendant semantics and uses an accent color. Loading is rendered as a treeitem named `Loading…` pending an active-platform announcement design.

## Theme tokens used

`background`, `surface`, `text`, `text_muted`, `border`, `accent`, `accent_text`, `focus`,
`borders.hairline`, `spacing.xsmall/small/large`, `controls.small`, `radii.small/large`, and
`typography.body` are read from the GPUI `Theme`.

The look follows the docs-site web preview (`site/src/demos/e7.ts`, the `tree` demo built from
`.ui-menu__item` rows, styled in `site/src/ui/ui.css` with the shadcn token mapping in
`site/src/ui/tokens.ts`), which matches the restyled Sidebar items. Colours are resolved from the installed `Theme` in three variants, the way Button, Select, Tabs,
and Sidebar do it: `high-contrast` is selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Accent" below is shadcn's `accent`/`muted`: `text` mixed 4% (light) or 12% (dark)
into `background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Tree fill | `surface` | `surface` | `surface` |
| Tree border | `border` | `text` at 10% | `border` |
| Row text | `text` | `text` | `text` |
| Chevrons and loader | `text_muted` | `text_muted` | `text` (`accent_text` on the active row) |
| Active row | accent fill, `text` | accent fill, `text` | `accent` fill, `accent_text` |
| Pointer hover | accent fill | accent fill | row outline in `border` |
| Active row with keyboard focus | `borders.hairline` outline in `focus` inside the row | same | same |
| Loading row text | `text_muted` | `text_muted` | `text_muted` |

- **Geometry.** The tree is a card: radius `radii.large`, a `borders.hairline` border, and
  `spacing.xsmall` (4px) padding. Rows are `controls.small` (32px) tall, the web's 14px text with
  6px vertical padding, with radius `radii.small` and `typography.body` text. Each level indents by
  `spacing.large` (16px, the web's `depth * 16px`). A row starts with a disclosure slot:
  `spacing.small` (8px, the web's row padding) plus a `spacing.large` (16px) icon square, then an
  8px (`spacing.small`) gap before the label. The whole slot is the disclosure pointer target, so
  it is 24px wide. Labels truncate with an ellipsis.
- **Icons.** Expanded branches show Lucide `chevron-down`, collapsed branches `chevron-right`, and
  the placeholder row of a branch whose children are loading shows Lucide `loader` (eight rays),
  all drawn as 2-unit vector strokes on a 24-unit grid in a 16px square, like Select's chevron.
  Leaves keep an empty slot so labels align. The web preview's folder and file icons are content,
  not part of this component's data model.
- **Active row.** The active node (the tree's active descendant) always shows the accent fill, as
  the web preview's `is-active` row does. It gains an inside focus outline while the tree has
  keyboard focus (`:focus-visible`); rows reserve a transparent hairline border so the outline does
  not shift content. A ring outside the row would overlap neighbouring rows, so the outline stays
  inside. The tree now tracks its own focus handle internally (it was created implicitly by
  `tab_index`) so render can tell when it has keyboard focus; tab order and click-to-focus are
  unchanged and no public API is added.
- In high contrast the active row keeps the solid `accent` fill, and hover draws the reserved row
  border in `border` instead of a subtle fill.

## WAI-ARIA pattern reference

[WAI-ARIA APG Tree View Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/) guides arrow and Home/End behavior. Selection is not part of this navigation-only tree API.

## Platform notes

AccessKit supplies tree roles. The screenshot fixture captures a collapsed branch, an expanded branch reached with Right, and a lazy branch showing its loading row after Right. Each state is captured with the light, dark, and high-contrast themes at 1× and 2× scale. The generated conformance manifest declares fixture cases, not executed harness evidence.

## Open questions

Review the pointer disclosure target size and visual treatment, active-row announcements, and how loading should be announced.
