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
theme_tokens: [surface, text, border, accent, accent_text, focus, spacing.medium]
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

Clicking a visible row outside its disclosure target makes that node active; the tree root remains the keyboard focus target. Expandable rows provide a separate disclosure hit target, sized to two `spacing.medium` units and aligned before the label. Clicking it requests expansion or collapse without activating the row. Controlled expansion changes remain owner-applied: clicking disclosure emits `ExpansionChanged`, but expanded state and lazy loading presentation change only after `set_expanded`. A lazy expansion request emits `LoadChildrenRequested` once while that node remains expanded with children absent; repeated clicks while the request is pending do not duplicate the child-load event. Keyboard Right/Left retain their existing active-row expansion and navigation behavior.

## Accessibility role and properties

The root has tree role and accessible label. Visible rows have treeitem role, the node label as accessible name, and level; expandable rows expose expanded state. The active row requests active-descendant semantics and uses an accent color. Loading is rendered as a treeitem named `Loading…` pending an active-platform announcement design.

## Theme tokens used

`surface`, `accent`, `accent_text`, `text`, `border`, `borders.hairline`, `spacing.small`, and `spacing.medium` are read from the GPUI `Theme`.

## WAI-ARIA pattern reference

[WAI-ARIA APG Tree View Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/) guides arrow and Home/End behavior. Selection is not part of this navigation-only tree API.

## Platform notes

AccessKit supplies tree roles. The screenshot fixture captures a collapsed branch, an expanded branch reached with Right, and a lazy branch showing its loading row after Right. Each state is captured with the light, dark, and high-contrast themes at 1× and 2× scale. The generated conformance manifest declares fixture cases, not executed harness evidence.

## Open questions

Review the pointer disclosure target size and visual treatment, active-row announcements, and how loading should be announced.
