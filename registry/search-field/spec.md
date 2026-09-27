---
spec_version: 1
component: search-field
states:
  - id: empty
    description: Search input has no query and no clear control or count.
    fixture: empty_fixture
  - id: query
    description: A query is present and the clear control is available.
    fixture: query_fixture
  - id: results
    description: A query and optional result count are visible.
    fixture: results_fixture
  - id: disabled
    description: Search field is disabled and the clear control is omitted.
    fixture: disabled_fixture
keys:
  - key: Escape
    modifiers: []
    when: Search field is focused and query is non-empty.
    action: Clear the query immediately and emit SearchChanged.
    initial_state: query
    expect:
      state: empty
      event: SearchChanged
  - key: cmd-f
    modifiers: []
    when: Search field is mounted and enabled.
    action: Focus the search input.
    initial_state: empty
    expect:
      focus_target: search-input
  - key: Enter
    modifiers: []
    when: Clear button is focused and enabled.
    action: Clear the query immediately and focus the search input.
    initial_state: query
    expect:
      state: empty
      event: SearchChanged
  - key: Space
    modifiers: []
    when: Clear button is focused and enabled.
    action: Clear the query immediately and focus the search input.
    initial_state: query
    expect:
      state: empty
      event: SearchChanged
accessibility:
  role: group
  properties:
    - name: label
      value: Search
    - name: input_role
      value: text input
    - name: input_label
      value: Search
    - name: clear_button
      value: named button for enabled non-empty queries, otherwise omitted
    - name: result_count
      value: polite status text when provided
    - name: disabled
      value: true
      when: disabled
controlled: Uncontrolled mode updates the query locally; controlled mode treats user edits and clear requests as SearchChanged proposals while the owner applies authoritative values with set_query. Programmatic query updates emit no events. With debounce configured, typing emits only the latest SearchChanged after the quiet interval; clearing by button or Escape emits immediately and cancels pending work.
events: [SearchChanged]
theme_tokens: [surface, elevated_surface, text, text_muted, border, accent, focus, disabled, spacing.xsmall, spacing.small, spacing.medium, radii.medium, borders.regular, typography.body, controls.medium, controls.small]
open_questions: [Review result-count live announcement behavior and whether the default focus shortcut should be cmd-f/ctrl-f on every host platform.]
---

# SearchField

## Purpose

Provide a consistent single-line search control for filtering lists and panels, with an optional result count and a shortcut that moves focus to the input.

## Anatomy

A labelled input row contains a decorative search icon inside the native single-line input border, an optional result count, and a clear button shown only when the query is non-empty and the field is enabled. Input editing delegates to the existing TextField component so selection, clipboard, Unicode, and IME behavior stay shared. The icon reserves one theme `spacing.medium` leading inset through TextField's `with_leading_inset` builder; the same inset positions text, pointer hit testing, and IME candidate bounds.

## States

Empty, query, results, and disabled. The clear control is hidden for empty queries and omitted while the field is disabled. Debounce is optional; without it, accepted edits emit immediately. With it, typing is coalesced until the quiet interval expires. Clear and Escape bypass debounce and emit immediately.

## Props and events

`SearchField::new(label)` creates an uncontrolled entity; `SearchField::controlled(label, query)` creates a controlled entity. Builders configure placeholder, optional result count, debounce duration, and disabled state. `set_query` applies external state without emitting. `SearchChanged { query }` reports edits and clear requests. Uncontrolled mode applies edits internally. Controlled mode emits proposals and expects the owner to reconcile with `set_query`; the native editor may display a transient edit until the owner supplies its value. Repeated identical values, disabled input, and programmatic updates emit no event.

The implementation depends on `mkit-registry-text_field` to delegate native editing, selection, clipboard, and IME behavior. This is a documented draft exception to the mkit-core/GPUI-only default pending maintainer approval.

## Keyboard map

Tab follows ordinary platform order. Text entry, selection, clipboard, and IME keys are handled by TextField. Escape clears a non-empty query immediately. The named `FocusSearch` action focuses the input; the default binding is Cmd+F and applications can rebind the action for their platform or product.

## Pointer behaviour

The input accepts normal text-field pointer editing. The clear button clears the query and returns focus to the input. The decorative search icon and result count are not interactive.

## Accessibility role and properties

The root is a named group. The child editor exposes a named text-input role using the shared TextField accessibility contract; the clear control is a named button and is present only for an enabled, non-empty query. Result count is polite status content, not a focus stop. Disabled state is exposed on the group and editor. A dedicated searchbox role is not currently available through the shared TextField API; this is an explicit platform mapping limitation for maintainer review.

## Theme tokens used

Read all colors, spacing, radii, borders, typography, and control sizes from mkit-core Global `Theme`. The search icon is a decorative GPUI-drawn lens and handle sized from `Theme.typography.body_emphasis` (lens diameter approximately 0.76 of that token), with stroke from `Theme.borders.strong`; it is hidden from accessibility because the input label already identifies search. The clear control uses semantic text and disabled tokens.

## WAI-ARIA pattern reference

Follow the [WAI-ARIA Textbox Pattern] for the editable query and labelled input behavior. The group and result status use their WAI-ARIA roles; no dedicated searchbox widget interactions are introduced.

## Platform notes

TextField supplies GPUI's native input handler for selection, clipboard, Unicode, and IME composition. The host binds the exported named action. Debounced callbacks are scheduled on GPUI's background timer and canceled by a generation check. Live announcement timing and the current text-input/searchbox role mapping need active-platform accessibility validation.

## Open questions

- Review whether a searchbox role setter should be added to TextField and used here.
- Review Cmd+F / Ctrl+F defaults and result-count announcement behavior across macOS, Windows, and Linux.

[WAI-ARIA Textbox Pattern]: https://www.w3.org/WAI/ARIA/apg/patterns/role-textbox/
