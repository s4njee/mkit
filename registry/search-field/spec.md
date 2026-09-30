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
  - id: focused
    description: The empty query input owns keyboard focus and shows the TextField focus border and ring around the inset search icon.
    fixture: focused_fixture
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
theme_tokens: [background, text, text_muted, accent, focus, disabled, spacing.none, spacing.small, spacing.medium, spacing.large, radii.medium, borders.regular, typography.caption, controls.medium, controls.small]
open_questions: [Review result-count live announcement behavior and whether the default focus shortcut should be cmd-f/ctrl-f on every host platform.]
---

# SearchField

## Purpose

Provide a consistent single-line search control for filtering lists and panels, with an optional result count and a shortcut that moves focus to the input.

## Anatomy

A labelled input row contains a decorative search icon inside the native single-line input border, an optional result count, and a clear button shown only when the query is non-empty and the field is enabled. Input editing delegates to the existing TextField component so selection, clipboard, Unicode, and IME behavior stay shared. The icon reserves a `spacing.large + spacing.small` leading inset (icon plus gap) through TextField's `with_leading_inset` builder; the same inset positions text, pointer hit testing, and IME candidate bounds.

## States

Empty, query, results, disabled, and focused (the input owns keyboard focus). The clear control is hidden for empty queries and omitted while the field is disabled. Debounce is optional; without it, accepted edits emit immediately. With it, typing is coalesced until the quiet interval expires. Clear and Escape bypass debounce and emit immediately.

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

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts` key `search-field`,
styled by `.ui-input-wrap` and `.ui-input` in `site/src/ui/ui.css` with the shadcn token mapping in
`site/src/ui/tokens.ts`). The field itself is TextField and keeps TextField's look (fill, input
border, shadow, focus and disabled treatment); see `registry/text-field/spec.md`. SearchField draws
only the icon and the clear button, resolved from the installed `Theme` in three variants, the same
way Button, Select, and TextField do it. `high-contrast` is selected by theme name; every other theme
is dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light.
Derived colours use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no
mkit-core API or tokens are added. "Muted" below is shadcn's `accent`: `text` mixed 4% (light) or
12% (dark) into `background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Search icon | `text_muted` | `text_muted` | `text_muted` |
| Search icon, disabled | `text_muted` 50% over `background` | same | `disabled` |
| Clear icon | `text_muted` | `text_muted` | `text` |
| Clear button fill and border | none (transparent) | same | same |
| Clear button pointer hover | muted fill | muted fill | border `accent` |
| Clear button keyboard focus | border `focus`, opaque `background` fill, 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Result count | `text_muted` | `text_muted` | `text_muted` |

- **Icons** are Lucide `search` (a radius-8 circle at 11,11 and a line from 21,21 to 16.7,16.7) and
  Lucide `x` (18,6 → 6,18 and 6,6 → 18,18) on a 24-unit grid with a 2-unit stroke, drawn as GPUI
  vector paths in a `spacing.large` (16px) square, like Select's chevron; there are no icon assets.
  The circle is two arcs. Lucide's round line caps are drawn as butt caps because GPUI's path
  builder does not expose cap styles. The search icon is decorative and hidden from accessibility
  because the input label already identifies search.
- **Icon placement.** The web preview places a 16px icon 10px inside the input and pads the text to
  34px. The icon sits `spacing.medium` (12px) from the field's left edge, vertically centred in
  the `controls.medium` field height, and TextField's leading inset is `spacing.large +
  spacing.small` (the 16px icon plus an 8px gap), so text starts 36px from the edge. Both are the
  nearest token sums to the web values and keep a symmetric 12px margin before the icon.
- **Clear button** is shadcn's small ghost icon button (`size-8`): a `controls.small` (32px) square
  with radius `radii.medium` and a transparent `borders.regular` border that carries the focus
  colour. It stays in the row after the field and the result count, `spacing.small` apart, so the
  accessibility order and the field's text area are unchanged. Focus follows `:focus-visible`
  (GPUI `focus_visible`); the fill under the ring is opaque because GPUI fills the inside of drop
  shadows. The 3px focus ring is the shadcn/ui ring width, a fixed component value.
- **Disabled** matches the web preview's `opacity: .5` as one layer, the way TextField does it: the
  icon colour is composited over `background` and mixed 50% with it rather than using GPUI element
  opacity. High contrast keeps the solid `disabled` colour. The clear button is omitted while
  disabled.
- **Result count** uses `typography.caption` (12px).

## WAI-ARIA pattern reference

Follow the [WAI-ARIA Textbox Pattern] for the editable query and labelled input behavior. The group and result status use their WAI-ARIA roles; no dedicated searchbox widget interactions are introduced.

## Platform notes

TextField supplies GPUI's native input handler for selection, clipboard, Unicode, and IME composition. The host binds the exported named action. Debounced callbacks are scheduled on GPUI's background timer and canceled by a generation check. Live announcement timing and the current text-input/searchbox role mapping need active-platform accessibility validation.

## Open questions

- Review whether a searchbox role setter should be added to TextField and used here.
- The screenshot matrix captures input focus only. The clear button's `:focus-visible` ring needs
  keyboard-modality focus that the fixture cannot yet set deterministically, so it is checked
  manually by tabbing to the button in the gallery.
- Review Cmd+F / Ctrl+F defaults and result-count announcement behavior across macOS, Windows, and Linux.

[WAI-ARIA Textbox Pattern]: https://www.w3.org/WAI/ARIA/apg/patterns/role-textbox/
