---
spec_version: 1
component: layer-panel
states:
  - id: mixed-tree
    description: Expanded groups and layers show thumbnails, visibility and lock state.
    fixture: mixed_tree_fixture
  - id: selected-layer
    description: One visible row is selected and receives the active treatment.
    fixture: selected_layer_fixture
  - id: collapsed-group
    description: A group is collapsed and its descendants are hidden.
    fixture: collapsed_group_fixture
  - id: reordered
    description: A layer has moved to another sibling position through keyboard or pointer input.
    fixture: reordered_fixture
  - id: dragging
    description: A row is being dragged and a compatible same-parent row is highlighted as the insertion target; cross-parent rows cannot accept it.
    fixture: dragging_fixture
  - id: focused
    description: The panel has keyboard focus and ArrowDown moved the active row to the first child layer, which shows the active fill and the inside focus outline.
    fixture: focused_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: layer panel is focused
    action: Move active row to the next visible row.
    initial_state: mixed-tree
    expect:
      event: ActiveChanged
  - key: ArrowUp
    modifiers: []
    when: layer panel is focused
    action: Move active row to the previous visible row.
    initial_state: mixed-tree
    expect:
      event: ActiveChanged
  - key: ArrowRight
    modifiers: []
    when: active row is a collapsed group
    action: Expand the active group.
    initial_state: collapsed-group
    expect:
      event: GroupExpandedChanged
  - key: ArrowLeft
    modifiers: []
    when: active row is an expanded group
    action: Collapse the active group.
    initial_state: mixed-tree
    expect:
      event: GroupExpandedChanged
  - key: ArrowUp
    modifiers: [Alt]
    when: active row has a preceding sibling
    action: Move the active row before its preceding sibling.
    initial_state: mixed-tree
    expect:
      event: ReorderRequested
  - key: ArrowDown
    modifiers: [Alt]
    when: active row has a following sibling
    action: Move the active row after its following sibling.
    initial_state: mixed-tree
    expect:
      event: ReorderRequested
  - key: Space
    modifiers: []
    when: a layer row is active
    action: Toggle layer visibility.
    initial_state: mixed-tree
    expect:
      event: VisibilityChanged
  - key: l
    modifiers: []
    when: a layer row is active
    action: Toggle layer lock.
    initial_state: mixed-tree
    expect:
      event: LockChanged
accessibility:
  role: tree
  properties:
    - name: name
      value: caller-provided panel label
    - name: treeitem
      value: Each visible group and layer is a treeitem named by its label and exposes one-based level.
    - name: expanded
      value: Groups expose expanded state.
    - name: selected
      value: Active row exposes selected state.
    - name: visible
      value: A labeled toggle button within each layer row exposes visibility state.
    - name: locked
      value: A labeled toggle button within each layer row exposes lock state.
controlled: Owner supplies the full layer tree, active ID, and expanded group IDs. Interaction requests emit typed events and take effect after the owner applies set_layers, set_active, or set_expanded. Uncontrolled mode updates its local state before emitting the same events.
events: [ActiveChanged, VisibilityChanged, LockChanged, GroupExpandedChanged, ReorderRequested]
theme_tokens: [background, surface, text, text_muted, border, accent, accent_text, focus, shadows.medium, spacing.xsmall, spacing.small, spacing.large, spacing.xlarge, controls.small, radii.small, radii.medium, radii.large, borders.hairline, borders.regular, typography.body]
open_questions: [Maintainer review of the public API, event names, keyboard and accessibility contract, and visual treatment., Confirm the provisional E13.2 density decision to keep the 32px rows and existing toggle hit targets.]
---

# Layer panel

## Purpose

Present an editable layer hierarchy for pro-app workflows. The panel supports nested groups,
selection, visibility and lock toggles, small image thumbnails, and sibling reordering.

## Data model

`LayerNode` has a stable string ID, label, and either `Layer` metadata or `Group` metadata with
children. Layers may carry an optional GPUI image source or colour swatch. Group expansion is independent of active
selection. The component renders visible rows in tree order and does not own domain-specific image
data or draw thumbnails beyond the provided swatch pixels.

## State and events

`LayerPanel::new` creates uncontrolled state. It updates locally and emits `ActiveChanged`,
`VisibilityChanged`, `LockChanged`, `GroupExpandedChanged`, or `ReorderRequested` after each accepted
interaction. `LayerPanel::controlled` treats layers, active ID, and expanded group IDs as owner
state: interactions emit the same requests, but rendering changes only after `set_layers`,
`set_active`, or `set_expanded`. A reorder event identifies the moved node and destination parent
and sibling index; it does not encode a speculative mutated tree.

Cross-parent drops are deliberately unsupported in this version. Drag targets must have the same
parent ID as the dragged row; dropping on a group row never reparents into that group. Such a drop
shows no compatible-target treatment and emits no `ReorderRequested`. Reparenting is deferred until
the event contract can state destination semantics and its interaction with expansion and active-row
state.

## Keyboard and pointer behavior

The root tree is focusable, and the `MkitLayerPanel` key context exposes rebindable actions for
visible-row navigation, group expansion/collapse, visibility toggle, lock toggle, and sibling
reorder. Navigation does not wrap. Alt+Up/Down requests a sibling move and never moves a node into
its own descendant. Pointer activation selects a row; dedicated visibility and lock controls toggle
their respective layer properties. Group disclosure toggles expansion. Thumbnails and labels do
not act as controls.

## Accessibility

The root exposes tree role and the caller's accessible label. Each visible row exposes treeitem role,
its label, level, and selected state. Groups expose expanded state. Visibility and
lock controls have button role, descriptive accessible names, and pressed state. Decorative
thumbnails are hidden from the accessibility tree.

## Visual treatment and theme

Rows follow the restyled Tree and Sidebar rows (E7, E13.1): radius-small rows in a bordered card,
an accent fill for the active row, a ghost hover, vector chevrons, and an inside focus outline on the
active row while the panel has keyboard focus. Visibility, lock and reorder controls are ghost icon
buttons with vector glyphs, like the restyled IconButton. Read all colours, radii, border widths,
and spacing from the `mkit_core::theme::Theme` Global. Thumbnails use caller-supplied colours in a
bordered rounded tile; no fixed palette is introduced. The "Theme tokens used" section has the full
table.

## Evidence and review

Focused data tests cover visible-tree flattening, reorder validation, and controlled/uncontrolled
state contracts. GPUI tests exercise keyboard navigation/reorder and pointer selection/toggles. The
generated conformance manifest declares intended keyboard, accessibility, and screenshot cases;
it is not evidence that the harness ran. The six visual states are captured under light, dark, and
high-contrast themes at 1× and 2×. The dragging capture is made after a real pointer drag over a
compatible sibling row. Focused data and GPUI tests cover visible-tree flattening, same-parent and
cross-parent reorder behavior, controlled and uncontrolled requests, keyboard interaction, and
pointer toggles. Native screen-reader traversal and maintainer review of the public API, event names,
keyboard and accessibility contracts, and visual treatment remain required.

## Anatomy

A focusable tree root contains visible group and layer rows. Group rows show disclosure, label, and
expanded state. Layer rows show an optional raster thumbnail or color swatch, label, visibility
button, and lock button. The selected row uses the active fill, with an inside focus outline while
the panel has keyboard focus.

## States

`mixed-tree` is the standard expanded hierarchy, `selected-layer` marks the active layer,
`collapsed-group` hides that group's children, `reordered` shows a sibling order after a move, and
`dragging` highlights a compatible insertion target during a drag, and `focused` shows the active
row's keyboard focus outline after ArrowDown.
Visibility and lock state are per-layer metadata and update through their typed event contracts.

## Props and events

`LayerPanel::new` owns active selection and layer metadata; it applies accepted edits locally before
emitting events. `LayerPanel::controlled` treats the supplied layers and active ID as owner state;
the owner applies requests through setters. Visibility and lock requests identify the target layer.
Reorder requests identify node, destination parent, and sibling index. `with_thumbnail` accepts a
GPUI image resource path, URI, or embedded asset name; `with_swatch` supplies a solid fallback.

## Keyboard map

Arrow Up/Down moves among visible rows. Right expands a collapsed group and Left collapses an
expanded group. Alt+Up/Down requests a sibling reorder. Space toggles visibility and L toggles lock
for the active layer. All bindings are registered under the rebindable `MkitLayerPanel` context.

## Pointer behaviour

Clicking a row selects it; clicking a group row also toggles expansion. Visibility and lock buttons
act on their own layer target. Rows are draggable. Dropping a row on another row in the same sibling
list moves it before that row; dropping on itself or a row under another parent does nothing. A
compatible row shows an accent border while hovered as the drop target. Dragging does not expand
groups and cannot move a node into its own descendants. Move-up/down buttons remain an alternate
pointer operation. Thumbnail images and swatches are decorative.

## Accessibility role and properties

The root has tree role and caller-provided accessible name. Visible rows have treeitem role, accessible
label, nesting level, and selected state; group rows also expose expanded state. Visibility and lock
buttons use button role, action-specific names, and toggled state. Thumbnail images are decorative.

## Theme tokens used

Colours are resolved from the installed `Theme` in three variants, the way Button, Select, Tabs,
Tree and Sidebar do it: `high-contrast` is selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" is shadcn's `accent`/`muted`: `text` mixed 4% (light) or 12% (dark) into
`background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Panel (card) fill | `surface` | `surface` | `surface` |
| Panel border, thumbnail border | `border` | `text` at 10% over `surface` | `border` |
| Row label | `text` | `text` | `text` |
| Chevrons | `text_muted` | `text_muted` | `text` |
| Active row | muted fill, `text` | muted fill, `text` | `accent` fill; label and glyphs in `accent_text` |
| Pointer hover (other rows) | muted fill | muted fill | row outline in `border` |
| Active row while the panel has keyboard focus | `borders.hairline` outline in `focus` inside the row | same | same |
| Compatible drop target while dragging | `borders.hairline` outline in `accent` inside the row plus the muted fill | same | outline in `accent` |
| Toggle glyph, default state (visible, unlocked) | `text_muted` | `text_muted` | `text` |
| Toggle glyph, exceptional state (hidden, locked) and reorder arrows | `text` | `text` | `text` |
| Toggle and reorder hover (ghost) | muted mixed again over the row fill | same | `borders.regular` border in `accent` |
| Swatch without a colour | muted fill | muted fill | `background` |
| Drag preview | `surface` fill, panel border, `shadows.medium`, `text` | same | `background`, `border`, no shadow |

- **Density (maintainer decision, provisional).** Rows keep their `controls.small` (32px) height and
  indentation step, and the toggles keep a compact square hit target: `spacing.xlarge` (24px), about
  the previous glyph-plus-padding target. shadcn's 36px control height is not adopted. Colours,
  borders, radii, shadows, focus, hover and typography follow the everyday components.
- **Geometry.** The panel is a card like Tree: radius `radii.large`, a `borders.hairline` border and
  `spacing.xsmall` padding. Rows have radius `radii.small` and a transparent `borders.hairline`
  border reserved for the focus and drop-target outlines, so they do not shift layout. A row starts
  at `spacing.small` plus one `spacing.large` step per nesting level, then a `spacing.large` (16px)
  disclosure slot that holds the group chevron and stays empty for layers so labels align, then a
  `spacing.small` gap. Labels use `typography.body` and truncate.
- **Glyphs.** Group rows show Lucide `chevron-down` while expanded and `chevron-right` while
  collapsed. Visibility shows Lucide `eye` while visible and `eye-off` while hidden; lock shows
  `lock-open` while unlocked and `lock` while locked; reorder controls show `arrow-up` and
  `arrow-down`. All are 2-unit vector strokes on a 24-unit grid in a `spacing.large` (16px) square,
  like Tree's chevrons, and are decorative. The default state is quieter (`text_muted`) than the
  exceptional state so hidden and locked layers stand out.
- **Toggles.** Visibility, lock and reorder controls are ghost icon buttons: `spacing.xlarge` squares
  with radius `radii.medium`, a transparent `borders.regular` border, and no resting fill. They are
  not keyboard tab stops (the row keys Space, `L` and Alt+Up/Down act on the active row), so they
  have no focus ring; their accessibility nodes are unchanged.
- **Thumbnails.** A `spacing.xlarge` (24px) tile with radius `radii.small` and a `borders.hairline`
  border in the panel border colour; raster thumbnails are clipped to it.
- **Drag preview.** A popover-like surface: radius `radii.medium`, `borders.hairline` border,
  `shadows.medium`, `typography.body`. Its 180 logical-pixel width is fixed to keep its label
  readable while following the pointer.
- **Focus.** The active-row outline follows `:focus-visible`: it shows while the panel root or its
  active row owns focus and the last input came from the keyboard. The outline stays inside the row
  because a ring outside it would overlap neighbouring rows, as in Tree.

## WAI-ARIA pattern reference

[WAI-ARIA APG Tree View Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/) guides tree
navigation, treeitem roles, expanded state, and selection. Layer toggles are buttons with pressed
semantics.

## Platform notes

GPUI's AccessKit role and state APIs provide the semantic tree. The conformance manifest marks
platform snapshots pending until run in the supported harness. GPUI image loading supports resources,
URIs, and embedded asset names; callers own resource loading and image lifetime.

## Open questions

Cross-parent drops and drops onto groups remain unsupported in this version. Review API naming,
keyboard bindings, tree and toggle semantics, and visual treatment.
