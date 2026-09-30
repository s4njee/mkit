---
spec_version: 1
component: calendar
states:
  - id: single
    description: One date is selected and the active day is in the displayed month.
    fixture: single_fixture
  - id: range_start
    description: A range calendar has a start date and awaits its end date.
    fixture: range_start_fixture
  - id: range_complete
    description: A range has a start and end date.
    fixture: range_complete_fixture
  - id: empty
    description: No date is selected and the active day is in the displayed month.
    fixture: empty_fixture
  - id: disabled_day
    description: A disabled date is visible but cannot be focused or selected.
    fixture: disabled_day_fixture
  - id: focused
    description: A single-date calendar whose active day owns keyboard focus after an arrow key moved it off the selected day.
    fixture: focused_fixture
keys:
  - key: ArrowRight
    modifiers: []
    when: Calendar grid has focus.
    action: Move active day forward one civil day, skipping unavailable dates.
    initial_state: empty
    expect: { event: ActiveDateChanged }
  - key: ArrowLeft
    modifiers: []
    when: Calendar grid has focus.
    action: Move active day backward one civil day, skipping unavailable dates.
    initial_state: empty
    expect: { event: ActiveDateChanged }
  - key: ArrowDown
    modifiers: []
    when: Calendar grid has focus.
    action: Move active day forward one week, skipping unavailable dates.
    initial_state: empty
    expect: { event: ActiveDateChanged }
  - key: ArrowUp
    modifiers: []
    when: Calendar grid has focus.
    action: Move active day backward one week, skipping unavailable dates.
    initial_state: empty
    expect: { event: ActiveDateChanged }
  - key: PageDown
    modifiers: []
    when: Calendar grid has focus.
    action: Move displayed month forward, clamping day to month length.
    initial_state: empty
    expect: { event: ViewChanged }
  - key: PageUp
    modifiers: []
    when: Calendar grid has focus.
    action: Move displayed month backward, clamping day to month length.
    initial_state: empty
    expect: { event: ViewChanged }
  - key: PageDown
    modifiers: [Shift]
    when: Calendar grid has focus.
    action: Move displayed year forward, clamping day to month length.
    initial_state: empty
    expect: { event: ViewChanged }
  - key: PageUp
    modifiers: [Shift]
    when: Calendar grid has focus.
    action: Move displayed year backward, clamping day to month length.
    initial_state: empty
    expect: { event: ViewChanged }
  - key: Home
    modifiers: []
    when: Calendar grid has focus.
    action: Move to first selectable date in the current locale-defined week.
    initial_state: empty
    expect: { event: ActiveDateChanged }
  - key: End
    modifiers: []
    when: Calendar grid has focus.
    action: Move to last selectable date in the current locale-defined week.
    initial_state: empty
    expect: { event: ActiveDateChanged }
accessibility:
  role: grid
  properties:
    - name: label
      value: Caller-supplied calendar label
    - name: selected
      value: true on selected date cells
    - name: disabled
      value: true on out-of-bounds or app-disabled dates
    - name: focused
      value: Exactly the active date cell
---

# Calendar

## Purpose

An inline month grid for picking one date or a range. Dates use a new `mkit_core::CivilDate`
value type (Gregorian year/month/day, bounded to years 1 through 9999). This keeps registry
components dependent only on `mkit-core` and GPUI, avoids timezone semantics, and leaves parsing,
formatting, and localization with the application. This dependency decision needs maintainer
review before the public API is considered stable.

## Anatomy

A card containing the month and year heading between previous- and next-month chevrons; seven
localized weekday headings; six rows of date cells. A single active
cell owns keyboard focus. Cells outside the displayed month remain visible to complete the grid,
but are visually muted. The selected date or range is visually distinguished with accent tokens.

## States

Empty, single selected, range start pending, complete range, disabled-day, and keyboard-focused
states. Locale text
is provided by the app. Optional bounds and a disabled-date predicate constrain focus and selection.
Range selection starts on first activation and completes on the second; a subsequent activation
starts a new range. Reversed endpoints are normalized chronologically.

## Props and events

The builder requires an accessible label, active date, seven weekday labels in Monday-based order,
and twelve month names. It accepts a Monday-based first weekday index, optional min/max bounds, and
an application-supplied disabled-date predicate. `range_mode()` selects range behavior;
`default_selection()` initializes uncontrolled state. `controlled()` takes the authoritative
selection.

In controlled mode, selection remains unchanged until the caller updates it and user activation
emits `SelectionRequested`. In uncontrolled mode the component updates and emits
`SelectionChanged`. Navigation emits `ActiveDateChanged`; month or year view movement emits
`ViewChanged`. Programmatic setter changes do not emit user events.

## Keyboard map

One grid tab stop uses roving focus on the active date. Arrows move by day/week; Page Up/Down by
month, Shift+Page Up/Down by year; Home/End move to the locale week boundaries. Enter or Space
selects. Unavailable dates are skipped. Register the `Calendar` key context and named actions so
applications can rebind every navigation operation. The structure follows the WAI-ARIA date picker
grid keyboard convention.

## Pointer behaviour

Clicking an enabled date cell makes it active and selects it. In range mode, the first click starts
a range and the second completes it; a later click starts another range. Disabled dates ignore
pointer activation. Clicking the previous- or next-month chevron moves the view one month, exactly
as Page Up or Page Down does.

## Accessibility role and properties

The month is exposed as a grid with a caller-supplied accessible name. Date cells have full date
labels, selected state, and disabled state for bounds or application-disabled dates. Only the active
cell participates in the roving focus stop.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts`, styled by
`.e7-calendar*` in `site/src/demos/e7_expansion.css` with the shadcn token mapping in
`site/src/ui/tokens.ts`), which in turn follows the shadcn/ui Calendar. It is resolved from the
installed `Theme` in three variants, the same way Button, Select, TextField and Tabs do it.
`high-contrast` is selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" below is shadcn's `accent`: `text` mixed 4% (light) or 12% (dark) into
`background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Card fill | `background` | `background` | `background` |
| Card border | `border` | `text` at 10% over the fill | `border` |
| Card shadow | `shadows.small` (shadcn `shadow-xs`) | `shadows.small` | none |
| Month title | `text`, semibold | same | same |
| Previous/next month chevrons | `text` on a transparent ghost button; hover muted fill | same | `text`; hover border `accent` |
| Weekday headings | `text_muted` | `text_muted` | `text_muted` |
| Day in the displayed month | `text` on the card; hover muted fill | same | `text`; hover border `accent` |
| Day outside the displayed month | `text_muted` | `text_muted` | `disabled` (`text_muted` is near-white in this theme) |
| Disabled day | `text_muted` mixed 50% over the card (shadcn `opacity-50`) | same | `disabled` |
| Selected day, range endpoint | `accent` fill (shadcn `primary`), `accent_text` | same | `accent` fill, `accent_text` |
| Range middle | muted band, `text` (shadcn `range_middle`) | same | `accent` border, `text` |
| Active day with keyboard focus | border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |

- **Card.** The calendar draws the web preview's card: radius `radii.medium`, a `borders.regular`
  border (2px in high contrast), and `spacing.small` (8px) padding. Its width is exactly seven day
  columns plus padding and border, so it does not stretch with a parent's cross-axis alignment. GPUI paints drop shadows as
  filled shapes that are not clipped to the element's outside, so the card fill is opaque.
- **Header.** Lucide `chevron-left` (15,18 → 9,12 → 15,6) and `chevron-right` (9,18 → 15,12 → 9,6)
  on a 24-unit grid with a 2-unit stroke, drawn as vector paths in a `spacing.large` (16px) square
  (no icon assets), each centred in a `controls.small` (32px, the web preview's
  `ui-btn--sm ui-btn--icon`) ghost button with radius `radii.medium`. The month and year title sits
  between them in `typography.body` (14px) at semibold weight, the web preview's `font-weight: 600`.
  The header is followed by a `spacing.xsmall` (4px) gap, the nearest token to the preview's 6px.
  The chevrons are pointer affordances for the existing month navigation: a click moves the view
  exactly as Page Up or Page Down does, with the same bounds handling and `ViewChanged` and
  `ActiveDateChanged` events. They are not tab stops and add no accessibility nodes, so the keyboard
  contract and the single grid tab stop are unchanged; see "Open questions".
- **Grid.** Weekday headings are `typography.caption` (12px, the preview's 12px) in cells
  `controls.xsmall` (28px, the preview's weekday height) tall. Day cells are `controls.medium`
  (36px) wide, the preview's column width inside its 280px card, and `controls.small` (32px, the
  preview's day height) tall, with no horizontal gap so a range forms one band like shadcn's. Rows
  are separated by `spacing.xsmall` (4px), the nearest non-zero token to the preview's 2px gap. Day
  numbers use `typography.body` (14px, shadcn `text-sm`; the preview's 13px has no token) and cells
  have radius `radii.small` (the preview's `--ui-radius-sm`). Range endpoints sit on the muted band
  on their inner side, as shadcn's `range_start`/`range_end` do, so the band runs continuously
  behind the endpoint corners; a range with one endpoint shows only the filled day.
- **Focus.** The focus ring follows `:focus-visible`: it shows while the active day owns keyboard
  focus and the last input was from the keyboard. The focused day's fill stays opaque under the
  ring. Days have a transparent `borders.regular` border so the focus border does not shift the
  number. The 3px focus ring is the shadcn/ui ring width, a fixed component value, and may be
  overlapped by filled neighbouring days, which paint later.
- **No "today" marker.** shadcn fills today's date with `accent`. `CivilDate` has no clock and the
  calendar receives no current date, so no today state is drawn; see "Open questions".
- **Hover** gives enabled days and the chevron buttons the muted fill in light and dark themes and an
  `accent` border in high contrast; disabled days have no hover. Pointer hover is suppressed after
  keyboard input, as GPUI does for every hover style.

## WAI-ARIA pattern reference

[Date Picker Dialog Example: Date Grid](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/examples/datepicker-dialog/)

## Platform notes

APG semantics are expressed with GPUI/AccessKit roles and selected/disabled properties. Verify the
resulting native accessibility tree on supported macOS, Windows, and Linux bridges before release.

## Open questions

Maintainer review: approve `CivilDate` versus an external date type; approve the public locale-label
and selection API; confirm how AccessKit exposes grid cell selected/current semantics on supported
platforms. The previous/next month chevrons are pointer-only affordances with no accessible name,
because the builder has no localized "previous month"/"next month" labels; exposing them as named
buttons (as the APG date picker dialog does) needs new label API and an accessibility-tree change,
pending maintainer approval. A "today" highlight likewise needs an application-supplied current
date. Harness keyboard, a11y and screenshot coverage is pending where the E0 harness cannot
capture them.
