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

Month and year heading; seven localized weekday headings; six rows of date cells. A single active
cell owns keyboard focus. Cells outside the displayed month remain visible to complete the grid,
but are visually muted. The selected date or range is visually distinguished with accent tokens.

## States

Empty, single selected, range start pending, complete range, and disabled-day states. Locale text
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
pointer activation.

## Accessibility role and properties

The month is exposed as a grid with a caller-supplied accessible name. Date cells have full date
labels, selected state, and disabled state for bounds or application-disabled dates. Only the active
cell participates in the roving focus stop.

## Theme tokens used

Read surface, text, border, accent, accent_text, focus, disabled colors and spacing, radii, border,
control and body typography from `mkit_core::theme::Theme`. No fixed colors. Day cell sizing uses
the small control token as its minimum square size.

## WAI-ARIA pattern reference

[Date Picker Dialog Example: Date Grid](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/examples/datepicker-dialog/)

## Platform notes

APG semantics are expressed with GPUI/AccessKit roles and selected/disabled properties. Verify the
resulting native accessibility tree on supported macOS, Windows, and Linux bridges before release.

## Open questions

Maintainer review: approve `CivilDate` versus an external date type; approve the public locale-label
and selection API; confirm how AccessKit exposes grid cell selected/current semantics on supported
platforms. Harness keyboard, a11y and screenshot coverage is pending where the E0 harness cannot
capture them.
