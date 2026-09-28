---
spec_version: 1
component: combobox
states:
  - id: closed_empty
    description: Popup is closed and the editable value is empty.
    fixture: closed_empty_fixture
  - id: open_empty
    description: Popup is open with all enabled options available and no active option.
    fixture: open_empty_fixture
  - id: open_unfiltered_active
    description: Popup is open for an empty query with the first enabled option active.
    fixture: open_unfiltered_active_fixture
  - id: open_filtered
    description: Popup is open with matches for the current query and an active option.
    fixture: open_filtered_fixture
  - id: open_no_matches
    description: Popup is open and the current query has no matching options.
    fixture: open_no_matches_fixture
  - id: closed_committed
    description: Popup is closed with a committed option value.
    fixture: closed_committed_fixture
  - id: closed_draft
    description: Popup is closed after dismissal and the uncommitted typed draft is preserved.
    fixture: closed_draft_fixture
  - id: disabled
    description: Control is disabled and cannot be focused or changed.
    fixture: disabled_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: Editable input is focused and popup is closed.
    action: Open the popup and make the first enabled matching option active.
    initial_state: closed_empty
    expect:
      state: open_unfiltered_active
      event: open
      focus_target: input
  - key: ArrowDown
    modifiers: []
    when: Popup is open with an active option.
    action: Move the active option to the next enabled matching option, without moving DOM/platform focus from the input.
    initial_state: open_filtered
    expect:
      state: open_filtered
      focus_target: input
  - key: ArrowUp
    modifiers: []
    when: Popup is open with an active option.
    action: Move the active option to the previous enabled matching option, without moving focus from the input.
    initial_state: open_filtered
    expect:
      state: open_filtered
      focus_target: input
  - key: Enter
    modifiers: []
    when: Popup is open with an active option.
    action: Commit the active option and close the popup.
    initial_state: open_filtered
    expect:
      state: closed_committed
      event: commit
      focus_target: input
  - key: Escape
    modifiers: []
    when: Popup is open after editing or navigating suggestions.
    action: Cancel the current popup session, restore the value from before that session, and close the popup.
    initial_state: open_filtered
    expect:
      state: closed_committed
      event: cancel
      focus_target: input
  - key: Tab
    modifiers: []
    when: Popup is open.
    action: Close the popup, preserve the typed draft without committing the active suggestion, and allow normal forward focus traversal.
    initial_state: open_filtered
    expect:
      state: closed_draft
      event: dismiss
      focus_target: next
  - key: Tab
    modifiers:
      - Shift
    when: Popup is open.
    action: Close the popup, preserve the typed draft without committing the active suggestion, and allow normal backward focus traversal.
    initial_state: open_filtered
    expect:
      state: closed_draft
      event: dismiss
      focus_target: previous
  - key: ArrowDown
    modifiers: []
    when: Editable input is focused and popup is closed with a committed value.
    action: Open the popup and activate the first enabled matching option.
    initial_state: closed_committed
    expect:
      state: open_filtered
      event: open
      focus_target: input
  - key: ArrowDown
    modifiers:
      - Alt
    when: Editable input is focused and popup is closed.
    action: Open the popup without changing the text or moving focus from the input.
    initial_state: closed_empty
    expect:
      state: open_empty
      event: open
      focus_target: input
accessibility:
  role: combobox
  properties:
    - name: auto_complete
      value: list
    - name: value
      value: current query text
    - name: name
      value: caller label
    - name: expanded
      value: false
      when: closed_empty
    - name: expanded
      value: true
      when: open_empty
    - name: expanded
      value: true
      when: open_unfiltered_active
    - name: expanded
      value: true
      when: open_filtered
    - name: expanded
      value: true
      when: open_no_matches
    - name: expanded
      value: false
      when: closed_committed
    - name: expanded
      value: false
      when: closed_draft
    - name: expanded
      value: false
      when: disabled
    - name: active-descendant-focus
      value: true
      when: open_unfiltered_active
    - name: active-descendant-focus
      value: true
      when: open_filtered
    - name: disabled
      value: true
      when: disabled
---

# Editable combobox with filtering

## Purpose

An editable, single-value choice control for selecting one option from a supplied collection. Typing filters the suggestions; the user may also leave free-form text when the caller permits it. Use it when suggestions help text entry while the input remains the primary editing surface.

## Anatomy

- Editable text input, with a persistent accessible name supplied by the caller.
- Optional trailing disclosure button, which opens the suggestion list and is separately named when present.
- Popup containing a scrollable listbox virtualized over all filtered option rows. A non-interactive empty-state message may replace the rows when there are no matches.
- Optional caller-provided description and validation message associated with the input.

The popup is an in-window anchored surface, not a menu and not a separate dialog. The option list is single-select; highlight/active navigation is distinct from committed selection.

The editable caret, selected-text highlight, and IME marked-text treatment appear only while the input has keyboard focus. A closed, unfocused committed value renders as ordinary text; disabled input never shows an editing caret. The field keeps the theme's medium control height and vertically centers its text even when no caret or query text is present. The disabled field is drawn at 50% (its colours mixed over the background) so an empty disabled control is visibly different from an enabled empty control even without text.

## States

- `closed_empty`: no committed value; the input is empty and the popup is hidden.
- `open_empty`: popup is visible for an empty query; enabled options are available, with no active option. This is the explicit-open state used by Alt+ArrowDown or the disclosure button.
- `open_unfiltered_active`: popup is visible for an empty query and its first enabled option is active after ArrowDown opens the popup.
- `open_filtered`: popup is visible for a non-empty query; matching enabled options are shown and one is active. The active row is visually highlighted, while committed selection remains unchanged.
- `open_no_matches`: popup is visible for a query with no results; show the configured empty-state message, with no active option.
- `closed_committed`: popup is hidden and a selected option's display text/value is committed.
- `closed_draft`: popup is hidden after dismissal; typed text remains as an uncommitted draft and the prior committed value is unchanged. Its fixture uses committed option `us` and draft text `Cam`.
- `disabled`: input and disclosure control are disabled; popup is closed and the value remains readable.

Typing a non-empty query opens the popup and filters options. Clearing a query while open returns to `open_empty` with no active option until the next navigation key. If a query has no matches, transition to `open_no_matches`. The `open_filtered_fixture` is deterministic: committed session-start option `option-us` (“United States”), draft query `Cam`, active suggestion `option-cameroon` (“Cameroon”), and at least one other matching option. Escape restores `option-us` and closes. Clicking outside closes without committing the active option and keeps the user's typed text; Tab does the same while allowing normal traversal. Focus remains on the input throughout popup navigation.

### Screenshot fixture contract (E7.4)

The screenshot matrix is the Cartesian product of all eight states above, the `light`, `dark`, and `high-contrast` shadcn themes, and 1x/2x output scales. Each fixture uses one persistent `Entity<Combobox>` owned by the gallery root and the following deterministic content:

- Accessible label: `Country`; free-form values disabled; the pilot renders its fixed empty message `No matching options`.
- Options in source order: `us` / `United States`, `ca` / `Canada`, `cm` / `Cameroon`, `kh` / `Cambodia`, `jp` / `Japan`, `xx` / `Example disabled` (disabled).
- The fixture canvas is 480×300 logical pixels, with the control in a 320-pixel-wide column and the label `Combobox` above it. Theme background, spacing, and typography come from the active Global `Theme`.
- `closed_empty`: no value and no focus action.
- `open_empty`: focus the input and dispatch the component's `alt-down` action; require `is_open()` and render every option.
- `open_unfiltered_active`: focus the empty input and dispatch `down`; require the popup open with `us` active.
- `open_filtered`: focus the empty input, call the component's public `set_query("Cam")` edit path, then dispatch `down`; require the popup open, the Cameroon and Cambodia matches visible, and Cameroon active.
- `open_no_matches`: focus the empty input and apply `set_query("Atlantis")`; require the popup open with the configured no-results message and no active option.
- `closed_committed`: initialize with committed value `us`; show `United States` with the popup closed.
- `closed_draft`: initialize with committed value `us`, focus, apply query `Cam`, and dispatch `tab`; require the popup closed, draft `Cam` retained, and committed value still `us`.
- `disabled`: initialize empty with the control disabled; do not focus or dispatch input.

Open fixtures are valid only when the actual popup/list content is in the captured window. The matrix adapter asserts open state, active option, query/value, and the state-specific list or empty-message invariant before capture; a closed-state image is not an acceptable substitute for an open fixture.

## Props and events

Required props: accessible `label`, `options` (stable option IDs, display labels, and disabled flags), and `value`/`default_value` configuration. Optional props: `placeholder`, `description`, `disabled`, `allow_custom_value` (default false), `filter` (default case-insensitive substring match over display labels), `empty_message`, `open_on_focus` (default false), and `on_open_change`.

The component supports exactly one value mode:

- **Controlled:** caller supplies `value` and updates it in response to `on_change`. User edits emit `on_input(query)`; they do not mutate the supplied value. Selecting an option emits `on_change(option_id)` and `on_select(option_id)`. The caller's next value prop is authoritative. Escape emits `on_cancel(session_start_value)` and restores the displayed session value when the caller supplies the rollback value. No event is emitted solely because props changed.
- **Uncontrolled:** caller supplies `default_value` or neither value prop. The component owns draft text and committed selection. `on_input(query)` fires for user text edits; `on_change(option_id_or_custom_text)` fires when a value is committed; `on_select(option_id)` fires only for an option selection. Escape discards uncommitted text and restores the session-start value. Prop changes to options re-filter without emitting selection events.

`on_open_change(open)` fires only when the popup visibility changes due to user interaction. Events are emitted once per action. Disabled options are never active or committable. With `allow_custom_value=false`, Enter with no active option does not commit text. With it enabled, Enter commits the exact current text as a custom value and emits `on_change(text)`; it does not emit `on_select`.

## Keyboard map

The front matter provides executable cases. Additional behavior:

- Text input editing, caret movement, selection, clipboard, undo, and IME composition follow the native text-control convention. Left/Right/Home/End remain text-editing keys.
- Printable input edits the query and opens/updates the filtered popup. Backspace/Delete edit text normally.
- ArrowDown/ArrowUp navigate enabled options and wrap at list ends. From `closed_empty`, ArrowDown opens the unfiltered list and activates its first enabled option (`open_unfiltered_active`); from a closed popup with a committed value it opens filtered to that value. ArrowUp opens and activates the last enabled option. If there are no matches, arrows leave the active option unset.
- Enter commits the active option. If no option is active, it commits custom text only when `allow_custom_value` is enabled; otherwise it does nothing.
- Escape closes the popup and reverts the current editing session to its starting value. A second Escape after closure has no component action.
- Tab and Shift+Tab close without committing an active option and preserve typed draft text; platform focus traversal proceeds normally.
- Alt+ArrowDown opens without changing the active option or text. Alt+ArrowUp closes the popup without reverting typed text.
- The disclosure button toggles the popup; Space and Enter activate it through standard button behavior.

## Pointer behaviour

Clicking the input places the caret at the platform-selected location. Clicking the disclosure button toggles the popup without stealing input focus. Hovering an enabled option previews it as active but does not commit. Clicking an enabled option commits it, closes the popup, and returns/keeps focus on the input. Disabled options cannot activate or commit. Clicking outside closes the popup without committing an active suggestion; typed text remains. Option rows have full-row hit targets. Pointer movement must not force active-descendant changes while a mouse-down is selecting text in the input.

## Accessibility role and properties

The rendered text entry uses AccessKit's `EditableComboBox` role, caller-provided accessible name, current query as its accessible value, and expanded state. The popup uses listbox semantics; each visible suggestion uses option semantics, a label, and selected state. Disabled controls and disabled options expose AccessKit's disabled state. When an option is active, GPUI's active-descendant focus marker identifies that option while actual keyboard focus remains on the input. This is GPUI's AccessKit focus representation; it is not a literal web `aria-activedescendant` relationship.

AccessKit models list autocomplete, and this component sets `AutoComplete::List` through GPUI's public accessibility subtree callback. The pinned GPUI API exposes no way to obtain the popup node ID needed to add a controls relationship, so `aria-controls` is not promised. GPUI's accessibility bridge also provides no live result-count/no-results announcement contract here; descriptions/errors and a disclosure button are not currently rendered. Those behaviors remain unsupported until the adapter and component provide explicit mechanisms. A disabled control is omitted from keyboard focus order and exposes AccessKit disabled state.

## Theme tokens used

The look follows the shadcn/ui input and command list as drawn by the docs-site web previews
(`site/src/demos/e7.ts`, styled by `.ui-input`, `.ui-popover`, and `.ui-menu*` in
`site/src/ui/ui.css` with the shadcn token mapping in `site/src/ui/tokens.ts`), and matches the
Select trigger and popup. It is resolved from the installed `Theme` in three variants, the same way
Button, Checkbox, and Tabs do it. `high-contrast` is selected by theme name; every other theme is
dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light.
Derived colours use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no
mkit-core API or tokens are added. "Muted" below is shadcn's `accent`: `text` mixed 4% (light) or
12% (dark) into `background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Input fill | `background` | `background` | `background` |
| Input border ("input") | `border` | `text` at 15% | `border` |
| Input shadow | `shadows.small` | `shadows.small` | none |
| Query text | `text` | `text` | `text` |
| Caret, text selection, IME marked text | `accent` caret; `accent` fill with `accent_text` | same | same |
| Popup fill ("popover") | `surface` | `surface` | `background` |
| Popup border | `border` | `text` at 10% | `border` |
| Popup shadow | `shadows.medium` | `shadows.medium` | none |
| Active option | muted fill, `text` | muted fill, `text` | `accent` fill, `accent_text` (check too) |
| Committed option check | `text_muted` | `text_muted` | `text` |
| Disabled option | `text` 50% over the popup fill | same | `disabled` |
| Empty-state message | `text_muted` | `text_muted` | `text_muted` |
| Focus | input border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Disabled control | every input colour mixed 50% over `background`, shadow alpha halved | same | `background` fill, `disabled` text and border |

- **Focus** follows the web preview's `.ui-input:focus` (a text field shows its ring for any focus,
  not only keyboard focus) and stays while the popup is open, because focus remains on the input.
  The ring replaces the resting shadow. GPUI paints drop shadows as filled shapes that are not
  clipped to the element's outside, so the input and popup fills are always opaque (the web input's
  transparent fill composites to `background`).
- **Disabled** matches the web preview's `opacity: .5` as one layer, the way Checkbox does it: each
  colour is composited over `background` and mixed 50% with it. GPUI element opacity is not used
  because it dims each painted part separately. High contrast keeps solid colours instead.
- **Geometry**: the input is `controls.medium` (36px, shadcn `h-9`) tall with `spacing.medium`
  (12px, `px-3`) horizontal padding, radius `radii.medium`, a `borders.regular` border, and
  `typography.body` (14px) text; IME bounds and pointer hit testing use the same 12px text inset.
  The popup keeps its inline placement a `spacing.xsmall` gap below the input and has radius
  `radii.medium`, a `borders.regular` border, and `spacing.xsmall` (4px, `p-1`) padding. Option rows
  are `controls.small` (32px) tall, matching shadcn's `py-1.5` around a 20px line, with
  `spacing.small` (8px, `px-2`) horizontal padding and radius `radii.small`. The committed option
  shows Lucide `check` (20,6 → 9,17 → 4,12 on a 24-unit grid, 2-unit stroke) drawn as a vector path
  in a `spacing.large` (16px) square at the row's end; there are no icon assets. Pointer hover gives
  enabled rows the muted fill (not in high contrast). The empty-state message is centred with `spacing.xlarge` (24px,
  shadcn `py-6`) vertical padding. The 3px focus ring is the shadcn/ui ring width, a fixed component
  value; it fits inside the 4px popup gap.

## WAI-ARIA pattern reference

Use the interaction guidance in the [WAI-ARIA APG Combobox pattern](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/) and [Listbox popup](https://www.w3.org/WAI/ARIA/apg/patterns/listbox/). This desktop pilot exposes the supported AccessKit role, name, value, list autocomplete, expanded and selected states, and GPUI active-descendant focus marker. It does not claim a popup controls relationship.

## Platform notes

Preserve native text entry, selection, clipboard, undo, and IME behavior on macOS and Windows. Do not consume Left/Right or composition events for option navigation. Use platform-standard key labels and key modifiers in any optional shortcut hints. Alt+ArrowDown/Up is the documented popup convention, but it must remain available to apps to rebind or disable where it conflicts with platform or application behavior. Popup placement, clipping, and dismissal must respect the owning window and scroll container. AccessKit/platform accessibility mappings must expose equivalent role, name, expanded state, active descendant, and selection even where literal ARIA properties are not available. Screen-reader behavior must be checked on supported desktop platforms before declaring the public spec final.

## Open questions

- Human review required for public event names and the precise controlled Escape rollback contract.
- Confirm whether the default filter should use Unicode-aware case folding and locale-sensitive matching.
- Confirm whether option collections need async loading/loading announcements in the E5.5 pilot; this draft covers synchronous options only.
- Confirm the available Global token names for popup elevation, list-row states, and control sizing before implementation.

## Pilot implementation notes

- The registry source currently provides the editable text surface, filtering, keyboard navigation, commit/cancel/dismiss events, and a virtualized listbox. The GPUI adapter executes the generated keyboard cases; active-platform accessibility snapshots and theme/scale screenshot comparisons remain pending until the desktop harness can capture them.
- Controlled Escape cancellation reports the session-start option ID and emits `Cancelled`; the owner must apply that value through `set_value` to make the rollback authoritative. Until then, the controlled value prop remains authoritative.
- Pointer selection is covered by the component behavior tests. Outside-click dismissal, a disclosure button, popup anchoring/clipping, live result announcements, and a popup controls relationship are not implemented by this pilot and must not be reported as conformant. The active option uses GPUI's supported AccessKit active-descendant focus marker. List autocomplete and disabled control/option states are written through the public accessibility subtree callback; disabled options remain visibly styled and behaviorally guarded.
- Disabled matching options remain visible in the list but are skipped by active-option navigation and cannot be committed.

For large synchronous option collections, the popup is a real scroll viewport over the complete filtered option sequence. Use uniform-height virtualization with a persistent scroll handle; the viewport shows up to eight option rows (fewer when fewer options match) using the small control-height token, and only the virtualizer's rendered range is mounted while every filtered option remains reachable by scrolling and keyboard navigation. The active item is tracked by its stable option identity within the current filtered mapping, and the scroll handle moves it into view whenever keyboard navigation changes the active option. Filtering rebuilds the mapping in source order and clamps/clears stale active state before rendering. Pointer hit testing maps each rendered virtual row back to its source option and commits the clicked enabled option. Controlled and uncontrolled value/event semantics remain those above; scrolling and virtualization emit no value, selection, or open-change events.

For a large filtered result set, wheel scrolling to a later viewport and clicking a mounted row must commit the option represented by that filtered index, close the popup, and preserve the query. This interaction is covered by a GPUI test.
