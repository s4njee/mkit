---
spec_version: 1
component: time-field
states:
  - id: empty
    description: No time has been committed.
    fixture: empty_fixture
  - id: focused_hour
    description: The hour spinbutton segment owns focus.
    fixture: focused_hour_fixture
  - id: focused_minute
    description: The minute spinbutton segment owns focus.
    fixture: focused_minute_fixture
  - id: focused_second
    description: Optional seconds segment owns focus.
    fixture: focused_second_fixture
  - id: twelve_hour
    description: A 12-hour display includes a localized AM or PM segment.
    fixture: twelve_hour_fixture
  - id: invalid_bounds
    description: Draft value is outside configured bounds and shows validation.
    fixture: invalid_bounds_fixture
  - id: disabled
    description: Segments are readable but cannot be edited.
    fixture: disabled_fixture
keys:
  - key: ArrowUp
    modifiers: []
    when: An enabled segment is focused.
    action: Increment the focused segment by its step and commit a valid value.
    initial_state: focused_minute
    expect: { event: TimeChanged }
  - key: ArrowDown
    modifiers: []
    when: An enabled segment is focused.
    action: Decrement the focused segment by its step and commit a valid value.
    initial_state: focused_minute
    expect: { event: TimeChanged }
  - key: ArrowRight
    modifiers: []
    when: An enabled segment is focused.
    action: Move focus to the next available segment.
    initial_state: focused_hour
    expect: { focus_target: minute }
  - key: ArrowLeft
    modifiers: []
    when: An enabled segment is focused.
    action: Move focus to the previous available segment.
    initial_state: focused_minute
    expect: { focus_target: hour }
  - key: "0-9"
    modifiers: []
    when: An enabled numeric segment is focused.
    action: Replace the segment draft; commit after a complete valid segment value.
    initial_state: focused_hour
    expect: { event: TimeChanged }
  - key: Enter
    modifiers: []
    when: An enabled segment has a valid draft.
    action: Commit the composed time value.
    initial_state: focused_minute
    expect: { event: TimeChanged }
accessibility:
  role: group
  properties:
    - name: label
      value: Caller-supplied field label
    - name: segment-role
      value: spinbutton for each enabled segment
    - name: value
      value: Current hour/minute/optional second/period
---

# TimeField

## Purpose

TimeField provides precise segmented time entry with spinbutton semantics. `TimeValue` is a
validated 24-hour value; presentation can use a 12- or 24-hour cycle and omit seconds.

## Anatomy

The field contains hour and minute segments, an optional second segment, and an AM/PM segment in
12-hour mode. An application supplies accessible segment names and localized AM/PM labels. A
localized separator is shown between numeric segments.

## States

`empty`, `focused_hour`, `focused_minute`, `focused_second`, `twelve_hour`, `invalid_bounds`, and
`disabled`. The focused segment has a focus ring. A typed value is a draft until completed or
committed with Enter. Bounds violations preserve the draft and show the caller-provided message.
Disabled fields preserve their text but ignore segment focus, typing, and adjustment.

## Props and events

`TimeValue::new(hour, minute, second)` accepts valid civil-time ranges. The builder configures the
label, locale labels, hour cycle, seconds visibility, step, and optional inclusive minimum/maximum.
`set_value` is the owner synchronization API. In controlled mode, committed edits emit
`TimeChangeRequested(Option<TimeValue>)` and leave the displayed committed value to the owner;
uncontrolled mode stores the committed value and emits `TimeChanged(Option<TimeValue>)`. Draft
digits do not emit commit events. Arrow adjustments commit immediately. A step is a minute step
when the minute segment is focused; hour/second segments step by one. Values are clamped to legal
segment and configured bounds; out-of-bound typed candidates remain drafts with validation.

## Keyboard map

Arrow Up/Down increase/decrease the focused segment; with Shift they use the configured larger
step. Arrow Left/Right move between visible segments. Digits replace the focused numeric segment,
and two digits commit it when valid. Enter commits the complete draft. In 12-hour mode, Space or
Arrow Up/Down toggles the period. Home/End move the focused segment to its legal minimum/maximum.
`TimeField` registers its key context and named actions so applications can rebind adjustments.

## Pointer behaviour

Clicking a segment focuses it without changing the value. Disabled fields do not accept pointer
focus. Separators are not hit targets.

## Accessibility role and properties

The field is a named group containing one AccessKit spinbutton for every visible segment, including
the 12-hour period. Each spinbutton exposes its localized name, current numeric value, min/max, and
localized value text. Disabled segments expose disabled state. Draft bounds errors expose invalid
state and the localized validation description.

## Theme tokens used

The component reads Global `Theme` colors for surface, text, muted text, border, focus, accent,
danger, and disabled content; spacing, control sizing, radii, borders, and body typography also use
theme tokens.

## WAI-ARIA pattern reference

Segments follow the WAI-ARIA spinbutton pattern. Segment navigation and editing follow the native
segmented date/time input convention.

## Platform notes

TimeField uses GPUI key bindings and AccessKit roles, with app-supplied locale labels. Platform
screen-reader announcements and accessibility snapshots require supported-platform verification.

## Open questions

Maintainer review is required for public type/event naming, per-segment step behavior, and the
12-hour period interaction. The implementation tests and screenshot matrix should not be treated as
a maintainer approval of the public contract.
