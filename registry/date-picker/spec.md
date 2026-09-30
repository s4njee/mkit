---
spec_version: 1
component: date-picker
states:
  - id: closed_empty
    description: No committed date; text input is empty and calendar is closed.
    fixture: closed_empty_fixture
  - id: closed_value
    description: A committed date is formatted in the input and calendar is closed.
    fixture: closed_value_fixture
  - id: editing_valid
    description: User text parses as a date but has not been committed.
    fixture: editing_valid_fixture
  - id: editing_invalid
    description: User text cannot be parsed or violates bounds and has not been committed.
    fixture: editing_invalid_fixture
  - id: open_single
    description: The shared Calendar surface is open for single-date selection.
    fixture: open_single_fixture
  - id: range_start
    description: The shared Calendar is open and the start endpoint is being selected.
    fixture: range_start_fixture
  - id: range_end
    description: The shared Calendar is open and the end endpoint is being selected.
    fixture: range_end_fixture
  - id: disabled
    description: The input and calendar trigger cannot be edited or opened.
    fixture: disabled_fixture
  - id: no_available_dates
    description: Constraints exclude every date; opening shows the localized empty message.
    fixture: no_available_dates_fixture
keys:
  - key: ArrowDown
    modifiers: [Alt]
    when: Date input is focused.
    action: Open the calendar and focus its active date cell.
    initial_state: closed_value
    expect: { state: open_single, event: OpenChanged }
  - key: Enter
    modifiers: []
    when: Input text parses and is valid, or a calendar date is active.
    action: Commit the parsed input date or active calendar date and close the calendar.
    initial_state: editing_valid
    expect: { event: ChangeRequested }
  - key: Escape
    modifiers: []
    when: Calendar is open or the input contains an uncommitted draft.
    action: Close the calendar, restore the prior committed text and selection, and clear validation.
    initial_state: editing_invalid
    expect: { event: Cancelled }
  - key: Tab
    modifiers: []
    when: Calendar is open.
    action: Close the calendar and continue normal forward focus traversal without committing a draft.
    initial_state: open_single
    expect: { state: closed_value, focus_target: next }
  - key: Tab
    modifiers: [Shift]
    when: Calendar is open.
    action: Close the calendar and continue normal backward focus traversal without committing a draft.
    initial_state: open_single
    expect: { state: closed_value, focus_target: previous }
accessibility:
  role: textbox
  properties:
    - name: label
      value: Caller-supplied field label
    - name: invalid
      value: true
      when: editing_invalid
    - name: description
      value: Caller-supplied validation message
      when: editing_invalid
    - name: expanded
      value: true
      when: open_single
    - name: expanded
      value: true
      when: range_start
    - name: expanded
      value: true
      when: range_end
    - name: expanded
      value: true
      when: no_available_dates
    - name: disabled
      value: true
      when: disabled
---

# Date picker and date range picker

## Purpose

DatePicker combines a text field with a Calendar in an anchored, nonmodal popover. DateRangePicker
provides start and end fields that share the same Calendar. The picker uses `mkit_core::CivilDate`;
applications supply parse/format behavior so the component does not impose a locale or date syntax.

## Anatomy

Single picker: labeled text field with the calendar disclosure button inside its right end, inline
validation message, and an anchored calendar surface. Range picker: two labeled text fields and the same disclosure/calendar
surface. In range mode, activating a field selects whether the next calendar choice edits the start
or end endpoint.

## States

Closed empty, closed with a committed value, valid uncommitted draft, invalid draft, open single
selection, range start selection, range end selection, and disabled. A valid draft is not committed
until Enter or a calendar selection. Invalid drafts remain visible with an associated validation
message until corrected, cancelled, or committed.

## Props and events

The constructors accept an accessible field label, an initial active date, caller parser and
formatter, `CalendarLabels` (localized weekday/month names and first weekday), and `DatePickerLabels`
(localized placeholders, field suffixes, disclosure names, and validation messages). The single
form is `DatePicker::new`; `DateRangePicker` aliases the same shared implementation and uses
`DatePicker::range`. Optional
min/max and disabled-date rules constrain both typing and calendar selection. The parser returns
either a `CivilDate` or a caller-visible validation error. A parsed date outside constraints is
invalid. The formatter is used for committed values only. Text edits remain drafts and do not emit
a value-change event. If the preferred active date is unavailable, the picker searches within its
bounds for an available date; when none exists it opens a localized no-dates message instead of a
calendar grid.

Controlled mode receives `ChangeRequested(DateValue::Single(...))` or
`RangeChangeRequested(Option<DateRange>)` on commit and waits for the owner to update the value.
Uncontrolled mode commits internally and emits `Changed(DateValue::Single(...))` or
`RangeChanged(Option<DateRange>)`. Both emit `OpenChanged`
for user-driven calendar visibility and `Cancelled` when Escape restores the session-start value.
Prop updates and programmatic setters do not emit user events. A range calendar first requests a
start endpoint and then an end endpoint; endpoints are normalized chronologically.

The component directly depends on `mkit-core`, GPUI, `mkit-registry-calendar`, and
`mkit-registry-text_field`. `mkit` must include these two registry crates as implementation
dependencies for the generated source mirror. The picker owns an anchored calendar surface with GPUI primitives because the
current Popover API accepts text content rather than a component child. It does not copy or reimplement
the Calendar's date grid.

## Keyboard map

Alt+ArrowDown opens the calendar and moves focus to its active date. Calendar navigation follows
the Calendar component's APG grid bindings. Enter commits valid typed dates when the calendar is
closed, or selects its active date when the grid is focused. Range text entry commits only when both
endpoints parse and pass constraints; an incomplete range remains a draft with a localized error.
Escape closes and restores the session-start text/value. Tab and Shift+Tab close without committing
and continue ordinary focus traversal. Native text editing, caret movement, IME, and clipboard keys
remain those of the TextField.

## Pointer behaviour

Activating the disclosure opens the calendar. Selecting an enabled date commits it and closes the
surface. Selecting either range field makes it the active typed endpoint. Outside pointer down dismisses
without committing a draft; Escape restores the original value. Disabled pickers ignore input and
disclosure activation.

## Accessibility role and properties

Each date input exposes textbox role, caller-provided name, current text value, and invalid state
with its validation description when parsing or constraint checks fail. The disclosure exposes a
button name and expanded state. The open calendar preserves the Calendar grid role and date-cell
semantics. Closing returns focus to the input; Tab dismissal preserves normal traversal.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts`, styled by
`.ui-input-wrap`, `.e7-date-trigger` and `.e7-date-pop` in `site/src/demos/e7_expansion.css` and
`.ui-popover` in `site/src/ui/ui.css`, with the shadcn token mapping in `site/src/ui/tokens.ts`). The
date inputs are TextFields and keep TextField's look, including its focus ring, invalid ring and
validation message; the calendar grid is the Calendar component with its own card. The picker adds
the disclosure button and the popover surface, resolved from the installed `Theme` in three
variants, the same way Button, Select, TextField and Tabs do it. `high-contrast` is selected by theme
name; every other theme is dark when `mkit_core::contrast::relative_luminance(colors.background) <
0.5`, otherwise light. Derived colours use a crate-local `color-mix` helper built on
`mkit_core::contrast::composite`; no mkit-core API or tokens are added. "Muted" below is shadcn's
`accent`: `text` mixed 4% (light) or 12% (dark) into `background`.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Disclosure fill | the TextField fill (`background`) | the TextField fill (`text` at 4.5% over `background`) | `background` |
| Disclosure calendar icon | `text_muted` | `text_muted` | `text` |
| Disclosure hover | muted fill, `text` icon | same | border `accent` |
| Disclosure keyboard focus | border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Disabled disclosure | icon and fill mixed 50% over `background` | same | `disabled` icon |
| Popover fill ("popover") | `surface` | `surface` | `background` |
| Popover border | `border` | `text` at 10% over the fill | `border` |
| Popover shadow | `shadows.medium` (shadcn `shadow-md`) | `shadows.medium` | none |
| Empty-calendar message | `text_muted` | `text_muted` | `text` |

- **Disclosure.** As in the preview, the disclosure sits inside the right end of the last date
  input: a `controls.xsmall` (28px, the preview's 28px) square ghost button with radius
  `radii.small`, inset by half the difference between `controls.medium` and `controls.xsmall` (4px,
  the preview's `right: 4px`) so it is centred in the 36px input. Its fill is the input fill, so a
  long value that reaches the button is covered instead of drawn under the icon; it blocks pointer
  input to the field beneath it. The icon is Lucide `calendar` (a rounded 18×18 rectangle at 3,4
  with radius 2, a rule at y=10, and two 4-unit pins at x=8 and x=16 on a 24-unit grid, 2-unit
  stroke) drawn as vector paths in a `spacing.large` (16px) square; there are no icon assets.
- **Field width.** TextField has no trailing inset, so the input row has a minimum width equal to
  the Calendar card (seven `controls.medium` columns plus `spacing.small` padding and
  `borders.regular` border on each side, 270px). The preview uses a 240px column; the calendar width
  keeps the popover aligned with the inputs and leaves room for typical formatted dates before the
  disclosure. In range mode the two inputs share the row equally.
- **Popover.** The calendar surface has radius `radii.medium`, a `borders.regular` border, and
  `spacing.xsmall` (4px) padding, the nearest token to the preview's 6px, around the Calendar card.
  It opens `spacing.small` (8px) below the inputs. GPUI paints drop shadows as filled shapes, so the
  popover fill is opaque. The no-dates message uses `typography.body` with `spacing.medium` (12px)
  padding.
- **Focus.** The disclosure's ring follows `:focus-visible`: it shows while the disclosure owns focus
  and the last input was from the keyboard. The ring width is the shadcn/ui 3px ring, a fixed
  component value. The keyboard-focused active day in the open calendar is drawn by Calendar.
- **Disabled.** TextField dims the inputs. The disclosure mixes its colours 50% over `background`
  instead of GPUI element opacity, which dims each painted part separately; high contrast keeps
  solid colours.

## WAI-ARIA pattern reference

[WAI-ARIA APG Date Picker Dialog Example](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/examples/datepicker-dialog/)
informs the text field, disclosure, calendar grid, and Escape behavior. The surface is nonmodal and
does not trap focus.

## Platform notes

Text editing and IME are delegated to the TextField component. The calendar grid is the Calendar
component, including its AccessKit semantics. Verify popover dismissal, focus restoration, and the
native accessibility tree on supported platforms before release.

## Open questions

Maintainer review is required for the public parser/formatter callback types, range change event
names, endpoint editing policy, and the direct registry dependencies. GPUI interaction tests and the
macOS screenshot matrix are implemented; native platform accessibility tree snapshots remain
pending. The disclosure overlays the last input because TextField has no trailing inset; a
formatted value wider than about 226px reaches the disclosure and is covered by it. A TextField
trailing-inset builder would remove the minimum width and the overlap but is new public API pending
maintainer approval. The screenshot matrix has no state for the disclosure's own keyboard focus
ring; the open states show the Calendar's keyboard-focused active day.
