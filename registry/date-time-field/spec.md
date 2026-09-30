---
spec_version: 1
component: date-time-field
states:
  - id: empty
    description: Date and time values are both empty.
    fixture: empty_fixture
  - id: date_value
    description: A committed date is displayed beside an empty time.
    fixture: date_value_fixture
  - id: time_value
    description: An empty date is displayed beside a committed time.
    fixture: time_value_fixture
  - id: complete
    description: Committed date and time values are displayed.
    fixture: complete_fixture
  - id: date_picker_open
    description: The composed DatePicker calendar is open.
    fixture: date_picker_open_fixture
  - id: time_segment_focused
    description: A time segment in the composition owns focus.
    fixture: time_segment_focused_fixture
  - id: disabled
    description: Both child fields are disabled.
    fixture: disabled_fixture
keys:
  - key: ArrowDown
    modifiers: [Alt]
    when: Date input is focused.
    action: Open the composed DatePicker calendar.
    initial_state: complete
    expect: { state: date_picker_open }
  - key: ArrowUp
    modifiers: []
    when: A time segment is focused.
    action: Increment the focused time segment.
    initial_state: time_segment_focused
    expect: { event: TimeChanged }
accessibility:
  role: group
  properties:
    - name: label
      value: Caller-supplied date and time label
    - name: children
      value: DatePicker and TimeField with their native semantics
---

# DateTimeField

## Purpose

DateTimeField composes the E7.11 DatePicker and E7.14 TimeField so applications can collect a
civil date and time using one localized group. It does not own a second calendar or duplicate
time-segment behavior.

## Anatomy

The group contains a DatePicker followed by a locale-labeled separator and a TimeField. The
application supplies the already-configured child entities, group label, and localized separator.

## States

`empty`, `date_value`, `time_value`, `complete`, `date_picker_open`, `time_segment_focused`, and
`disabled`. Each child retains its own focus, validation, open, and selection state. The composed
view lays out these child states without copying them.

## Props and events

`DateTimeField::new(label, date_picker, time_field, separator)` accepts a DatePicker entity and a
TimeField entity. The DatePicker is the explicit E7.11 dependency; TimeField is the sibling registry
component. This is a passive composition: the caller retains the two child entities and subscribes
to their independent typed events (`ChangeRequested`/`Changed` and
`TimeChangeRequested`/`TimeChanged`). Date and time remain separate child values. DateTimeField
does not subscribe to child events, validate combined values, emit a synthetic cross-field event,
or provide an atomic combined value setter.

This registry package depends on `mkit-registry-date-picker` and `mkit-registry-time-field`. The
dependency is a documented draft exception to the registry rule that components normally depend
only on `mkit-core` and GPUI, pending maintainer approval. The generated `mkit` mirror imports `crate::date_picker` and
`crate::time_field` under its `mkit-mirror` feature to preserve one public child-component type.
Maintainer review is required for this dependency and the child-entity API.

## Keyboard map

DatePicker keeps its Alt+ArrowDown, Escape, Enter, and Calendar APG grid bindings. TimeField keeps
its Arrow, digit, Enter, and 12-hour period behavior. The composition adds no intercepting key
handler and registers no competing actions; applications can rebind either child key context.

## Pointer behaviour

Pointer input is delegated to the child fields. Clicking a date segment opens and operates the
DatePicker calendar according to E7.11. Clicking time segments focuses and edits the corresponding
TimeField segment according to E7.14.

## Accessibility role and properties

The wrapper exposes a named group. Its children preserve DatePicker textbox/button/grid semantics
and TimeField segment spinbutton semantics. Focus order follows the child render order. Disabled
state and validation are owned and exposed by the child fields.

## Theme tokens used

The wrapper uses Global `Theme` spacing and body typography for its localized separator and layout.
Child fields render with their own theme tokens: the DatePicker's shadcn input with the calendar
disclosure inside its right end and its popover-wrapped Calendar card, and the TimeField's
input-style segments with the accent fill on the focused segment (see their specs).

The docs-site web preview (`site/src/demos/e7_expansion.ts`, `date-time-field`) draws one text input
holding both date and time, with the calendar disclosure inside it. DateTimeField is a passive
composition of two independent child components, so matching that single input would need a combined
editor and a new value contract; the visual parity pass instead keeps the two restyled children side
by side, which gives the same input, disclosure, and focus-ring look. The children are vertically
centred on each other, `spacing.small` (8px) apart, with the localized separator in `text_muted` at
`typography.body` (14px). The wrapper does not know the children's disabled state, so the separator
is not dimmed when both children are disabled.

## WAI-ARIA pattern reference

The composition follows the group pattern and preserves the WAI-ARIA Date Picker and spinbutton
patterns implemented by the child components.

## Platform notes

Child keyboard handling, text editing, date grid, and AccessKit semantics remain delegated to
DatePicker and TimeField. Verify combined focus order and native accessibility grouping on
supported platforms.

## Open questions

Maintainer review is required for whether the child-entity composition and the non-atomic combined
date-time contract are the right public API. Native accessibility snapshots are pending.
