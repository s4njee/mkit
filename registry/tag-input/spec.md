---
spec_version: 1
component: tag-input
states:
  - id: empty
    description: No tags or query; the labelled editor is ready for input.
    fixture: empty_fixture
  - id: tags
    description: Tags are present while the editor is empty and ready for another value.
    fixture: tags_fixture
  - id: query
    description: Tags are present and the editor contains a query awaiting commit.
    fixture: query_fixture
  - id: selected_tag
    description: The editor is empty and the last tag is selected for removal or navigation.
    fixture: selected_tag_fixture
  - id: validation_error
    description: A tag has a per-tag validation message or the current candidate was rejected.
    fixture: validation_error_fixture
  - id: max_reached
    description: The maximum tag count is reached; additions are rejected while tags remain removable.
    fixture: max_reached_fixture
  - id: suggestions
    description: A non-empty query filters the configured suggestions and one suggestion may be active.
    fixture: suggestions_fixture
  - id: disabled
    description: Tags and validation are readable but the editor and removal controls are disabled.
    fixture: disabled_fixture
  - id: focused
    description: Tags are present and the empty editor owns keyboard focus; the container shows the focus border and ring.
    fixture: focused_fixture
keys:
  - key: Enter
    modifiers: []
    when: Editor is enabled and a non-empty query is present.
    action: Add the active suggestion, or otherwise add the trimmed query as a tag; emit a typed tags-change proposal when accepted.
    initial_state: query
    expect:
      state: tags
      event: TagsChanged
      focus_target: editor
  - key: ArrowDown
    modifiers: []
    when: Editor query has filtered suggestions.
    action: Activate the first enabled suggestion or advance to the next enabled suggestion without moving focus from the editor.
    initial_state: suggestions
    expect:
      state: suggestions
      focus_target: editor
  - key: ArrowUp
    modifiers: []
    when: Editor query has filtered suggestions.
    action: Activate the last enabled suggestion or move to the previous enabled suggestion without moving focus from the editor.
    initial_state: suggestions
    expect:
      state: suggestions
      focus_target: editor
  - key: Comma
    modifiers: []
    when: Editor is enabled and a non-empty query is present.
    action: Add the trimmed query as a tag; comma is not included in the tag value.
    initial_state: query
    expect:
      state: tags
      event: TagsChanged
      focus_target: editor
  - key: Backspace
    modifiers: []
    when: Editor is empty and tags are present.
    action: Select the last tag, or remove the selected tag on a second Backspace; emit TagsChanged only on removal.
    initial_state: tags
    expect:
      state: selected_tag
      focus_target: editor
  - key: ArrowLeft
    modifiers: []
    when: Editor is empty and tags are present or a tag is selected.
    action: Select the last tag or move selection to the preceding tag without moving focus from the editor.
    initial_state: tags
    expect:
      state: selected_tag
      focus_target: editor
  - key: ArrowRight
    modifiers: []
    when: A tag is selected.
    action: Move selection to the next tag; after the last tag, clear tag selection and keep focus in the editor.
    initial_state: selected_tag
    expect:
      state: tags
      focus_target: editor
  - key: Delete
    modifiers: []
    when: A tag is selected and the editor is enabled.
    action: Remove the selected tag and emit TagsChanged.
    initial_state: selected_tag
    expect:
      state: tags
      event: TagsChanged
      focus_target: editor
  - key: Escape
    modifiers: []
    when: A suggestion is active or a tag is selected.
    action: Clear suggestion and tag selection without changing tags or query.
    initial_state: suggestions
    expect:
      state: tags
      focus_target: editor
accessibility:
  role: group
  properties:
    - name: label
      value: caller-provided group name
    - name: editor_role
      value: textbox
    - name: editor_label
      value: caller-provided group name
    - name: tag_collection
      value: list with each tag as list item
    - name: remove_control
      value: named button for each enabled tag
    - name: selected
      value: true
      when: selected_tag
    - name: invalid
      value: true
      when: validation_error
    - name: disabled
      value: true
      when: disabled
    - name: suggestions
      value: listbox with active option when present
      when: suggestions
controlled: Uncontrolled mode applies accepted tag additions/removals immediately. Controlled mode emits a complete proposed tag list in TagsChanged and keeps owner-provided tags until set_tags supplies the accepted or corrected list. Programmatic updates do not emit events. TagRejected reports an empty, duplicate, or over-limit candidate without changing the list. The validator attaches per-tag validation text to accepted tags, including invalid values. TagQueryChanged reports editor text changes. Existing TextField supplies native editing, selection, clipboard, and IME behavior; this is a documented draft dependency exception pending maintainer approval. Suggestions follow Combobox's filtered listbox and enabled active-option conventions, keeping disabled options visible but skipping them during keyboard navigation and pointer activation. The single-value Combobox is not embedded because it owns a single editable query and committing its option replaces that value, while TagInput must append to a collection and preserve one shared native editor; direct composition would create nested editors and conflicting keyboard ownership.
events: [TagsChanged, TagRejected, TagQueryChanged]
theme_tokens: [background, surface, text, text_muted, border, accent, accent_text, focus, danger, disabled, shadows.none, shadows.small, shadows.medium, spacing.xsmall, spacing.small, spacing.xlarge, radii.small, radii.medium, borders.regular, typography.body, typography.caption, controls.small, controls.medium, controls.large]
open_questions: [Review public API and child dependency exception; validate selected-tag and per-tag error semantics with platform accessibility snapshots; confirm whether suggestions require an explicit popup-dismissal action beyond Escape and Tab.]
---

# Tag input

## Purpose

Collect a set of short labels, recipients, or filters through a text editor that appends and removes tags. Use it when a free-form multi-value field is more appropriate than a single selection control.

## Anatomy

A named group contains one bordered input container holding a wrapping list of tag chips followed by a labelled TextField editor on the same line, which wraps to a new line when fewer than `controls.large` (40px, the web preview's `min-width: 40px`) remain. Each tag can include an optional validation message and a remove control. An optional filtered suggestion listbox appears below the container for non-empty queries. A maximum count is optional.

## States

`empty`, `tags`, `query`, `selected_tag`, `validation_error`, `max_reached`, `suggestions`, `disabled`, and `focused` (the editor owns keyboard focus). Validation is per tag and may also report a rejected draft candidate through a polite status message. Max reached keeps existing tags removable but rejects further additions. Disabled state prevents editing, selection, and removals while retaining readable tags and validation.

## Props and events

`TagInput::new(label)` creates an uncontrolled entity; `TagInput::controlled(label, tags)` creates a controlled entity. Builders configure placeholder, initial tags, max count, optional suggestions, validation callback, and disabled state. `set_tags` applies owner state without emitting. A `Tag` contains its display value and optional validation message. `TagsChanged { tags }` contains the complete proposed list. In controlled mode tags remain owner-supplied until `set_tags` acknowledges or corrects a proposal. `TagRejected { value, reason }` reports empty, duplicate, or over-limit additions. The validator attaches an optional validation message to accepted tags, including invalid values. `TagQueryChanged { query }` reports text edits. Programmatic updates emit no events.

The editor delegates to the existing TextField to preserve selection, clipboard, Unicode, and IME behavior. This is a documented draft exception to the mkit-core/GPUI-only registry dependency convention pending maintainer approval. Suggestion filtering and active-option behavior follow the existing Combobox conventions, including disabled-option skipping. The single-value Combobox is not embedded because it owns its editor and replaces one committed value, which conflicts with TagInput's collection append and single-editor interaction model.

## Keyboard map

Enter adds the active suggestion, or the trimmed query when no suggestion is active. Comma always adds the trimmed query without including the separator. When the query is empty, Backspace selects the last tag and a second Backspace removes it; Delete removes a selected tag. ArrowLeft selects the last tag or moves selection left; ArrowRight moves selection right and returns to the editor after the last tag. Escape clears suggestion/tag selection. Tab uses normal traversal and does not add a draft. Commit, separator, removal, and clear actions use the `TagInput` key context with named actions so applications can rebind them; arrow navigation is handled when the editor is empty, while native text caret movement remains TextField behavior.

## Pointer behaviour

Clicking the editor places its native caret. Each enabled tag remove button removes that tag and returns focus to the editor. Selecting a chip by click focuses the editor and updates the selected-tag state. Suggestion options use pointer activation to append the chosen value.

## Accessibility role and properties

The root is a named group containing a list of named tag list items, a labelled textbox, optional named remove buttons, and an optional listbox with active-option semantics. Invalid tags expose their validation text and invalid state. The rejected-candidate message is a polite status; max count is exposed as status text. Disabled state is exposed for the editor and remove controls. This is a draft semantic contract requiring active-platform accessibility snapshots and maintainer review.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts` key `tag-input`,
styled by `.e7-tags` in `site/src/demos/e7_expansion.css` and `.ui-badge--secondary`,
`.ui-popover`, and `.ui-menu*` in `site/src/ui/ui.css`, with the shadcn token mapping in
`site/src/ui/tokens.ts`). It is resolved from the installed `Theme` in three variants, the same way
Button, Select, and TextField do it. `high-contrast` is selected by theme name; every other theme is
dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light.
Derived colours use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no
mkit-core API or tokens are added. "Muted" below is shadcn's `secondary`/`accent`/`muted`: `text`
mixed 4% (light) or 12% (dark) into `background`. "Input" is shadcn's `--input`: `border` in light
themes and `text` at 15% in dark themes.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Container fill | `background` | `text` at 4.5% over `background` (TextField's fill) | `background` |
| Container border | input (`border`) | input (`text` at 15%) | `border` |
| Container shadow | `shadows.small` (shadcn `shadow-xs`) | `shadows.small` | none |
| Editor focused (`:focus-within`) | container border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Invalid (a tag has a validation message, or a candidate was rejected) | container border `danger` plus a 3px ring of `danger` at 20% | same with the ring at 40% | border `danger`; focus still adds the opaque `focus` ring |
| Tag chip (secondary badge) | muted fill, `text`, transparent border | same | `background` fill, `text`, `border` border |
| Selected chip (default badge) | `accent` fill, `accent_text` | same | `accent` fill and border, `accent_text` |
| Invalid chip | border `danger`; message `danger` | same | same |
| Chip remove icon | the chip's text colour (`currentColor`) | same | same |
| Suggestion popup ("popover") | `surface` fill, `border` border, `shadows.medium` | `surface` fill, `text` at 10% border, `shadows.medium` | `background` fill, `border` border, no shadow |
| Active suggestion | muted fill, `text` | same | `accent` fill, `accent_text` |
| Disabled suggestion | `text` 50% over the popup fill | same | `disabled` |
| Disabled control | container fill, border, chip fill, chip text, and danger colours mixed 50% over `background`; shadow alpha halved | same | `background` fill, `disabled` border, chip text, chip border, and messages |

- **Container** matches `.e7-tags`: radius `radii.medium`, a `borders.regular` border, and
  `spacing.xsmall` (4px) padding and gaps, the nearest token to the web preview's 4–6px padding
  and 5px gap. Its minimum height is `controls.medium` (36px, the field height shared with
  TextField and Select) rather than the web preview's 38px. The ring replaces the resting shadow,
  as in CSS; GPUI fills the inside of drop shadows, so the container fill is always opaque. The 3px
  focus ring is the shadcn/ui ring width, a fixed component value.
- **Editor.** TextField always draws its own border, shadow, and focus ring, and its look is shared
  with other components, so TagInput does not change it. Instead the editor slot is
  `controls.medium - 2 × radii.medium` tall (24px in the shadcn themes, the web preview's inline
  input height) and clips a TextField that is offset by `radii.medium` on the top, left, and right.
  That cuts the field's border, rounded corners, shadow, and ring away, so only its fill, text,
  caret, and selection show inside the container; the container fill equals TextField's fill in
  every state so the two read as one surface. Text, pointer hit testing, and IME bounds still use
  TextField's own geometry. The visible text starts `spacing.medium - radii.medium` (6px) inside
  the slot. The slot is `flex_1` with a `controls.large` (40px) minimum width.
- **Chips** match `.ui-badge.ui-badge--secondary` inside `.e7-tags`: `spacing.xlarge` (24px) tall,
  `spacing.small` (8px, `px-2`) horizontal padding, radius `radii.medium`, a `borders.regular`
  border that is transparent unless the chip is invalid (or selected in high contrast),
  `typography.caption` (12px) medium-weight text, and a `spacing.xsmall` (4px) gap. The remove
  control is Lucide `x` (18,6 → 6,18 and 6,6 → 18,18 on a 24-unit grid, 2-unit stroke) drawn as a
  vector path in a `typography.caption` (12px, the badge's `svg` size) square, with butt caps
  because GPUI's path builder does not expose cap styles. The web preview has no selected-chip
  state; the selected chip uses the default (primary) badge colours so the virtual selection is
  unambiguous. The web preview's destructive hover on the remove icon is not drawn, because the
  icon colour is fixed when the vector path is built.
- **Suggestions** match `.ui-popover.ui-menu` and Select's popup: radius `radii.medium`, a
  `borders.regular` border, `spacing.xsmall` (4px, `p-1`) padding, and rows `controls.small`
  (32px) tall with `spacing.small` (8px) horizontal padding and radius `radii.small`, text
  `typography.body`. Pointer hover gives enabled rows the muted fill (not in high contrast). The
  popup sits `spacing.small` (8px, the web preview's column gap) below the container; the rejected
  and maximum status messages use the same gap.
- **Disabled** matches the web preview's `opacity: .5` as one layer, the way TextField does it:
  each colour is composited over `background` and mixed 50% with it; GPUI element opacity is not
  used because it dims each painted part separately. High contrast keeps solid colours.

## WAI-ARIA pattern reference

Use the textbox and listbox/option guidance from the WAI-ARIA Authoring Practices. Tags are represented as a list within a named group; each removal control is a named button. The pattern does not claim a standardized tag-input role.

## Platform notes

TextField handles native editing and IME. Tag selection remains a virtual selection while focus stays in the editor, with `aria-selected` exposed on the selected list item. Suggestion listbox and status announcement timing need active-platform accessibility validation. Comma as a commit separator may differ by locale and needs maintainer review; IME composition must not be committed while marked text is active.

## Open questions

- Review controlled event naming, duplicate comparison (trimmed, case-insensitive), and validator callback API.
- Review comma separator behavior for locales where comma is part of common tag values.
- Validate the listbox/status semantic contract on active accessibility platforms.
