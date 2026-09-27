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
theme_tokens: [background, surface, elevated_surface, text, text_muted, border, accent, accent_text, focus, danger, disabled, spacing.xsmall, spacing.small, spacing.medium, spacing.large, radii.small, radii.medium, borders.hairline, borders.regular, borders.strong, typography.body, typography.caption, controls.small, controls.medium]
open_questions: [Review public API and child dependency exception; validate selected-tag and per-tag error semantics with platform accessibility snapshots; confirm whether suggestions require an explicit popup-dismissal action beyond Escape and Tab.]
---

# Tag input

## Purpose

Collect a set of short labels, recipients, or filters through a text editor that appends and removes tags. Use it when a free-form multi-value field is more appropriate than a single selection control.

## Anatomy

A named group contains a wrapping list of tag chips above a labelled TextField editor. Each tag can include an optional validation message and a remove control. An optional filtered suggestion listbox appears for non-empty queries. A maximum count is optional. The editor retains the shared TextField border and sizing; tag chips wrap independently above it.

## States

`empty`, `tags`, `query`, `selected_tag`, `validation_error`, `max_reached`, `suggestions`, and `disabled`. Validation is per tag and may also report a rejected draft candidate through a polite status message. Max reached keeps existing tags removable but rejects further additions. Disabled state prevents editing, selection, and removals while retaining readable tags and validation.

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

Read colors, spacing, borders, radii, typography, and control sizes from mkit-core Global `Theme`. Tag chips use surface/elevated surface, semantic text, border, and accent selection tokens. The active suggestion uses a muted text/elevated-surface-derived hover fill for shadcn light/dark and the theme's semantic accent in high-contrast themes; its row is inset and rounded within the elevated-surface popup. Validation uses danger tokens. No component colors are hard-coded; the shared TextField controls editor sizing and focus rendering.

## WAI-ARIA pattern reference

Use the textbox and listbox/option guidance from the WAI-ARIA Authoring Practices. Tags are represented as a list within a named group; each removal control is a named button. The pattern does not claim a standardized tag-input role.

## Platform notes

TextField handles native editing and IME. Tag selection remains a virtual selection while focus stays in the editor, with `aria-selected` exposed on the selected list item. Suggestion listbox and status announcement timing need active-platform accessibility validation. Comma as a commit separator may differ by locale and needs maintainer review; IME composition must not be committed while marked text is active.

## Open questions

- Review controlled event naming, duplicate comparison (trimmed, case-insensitive), and validator callback API.
- Review comma separator behavior for locales where comma is part of common tag values.
- Validate the listbox/status semantic contract on active accessibility platforms.
