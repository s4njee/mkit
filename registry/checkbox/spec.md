---
spec_version: 1
component: checkbox
states:
  - id: unchecked
    description: Checkbox is off.
    fixture: unchecked_fixture
  - id: checked
    description: Checkbox is on.
    fixture: checked_fixture
  - id: indeterminate
    description: Mixed selection.
    fixture: indeterminate_fixture
  - id: disabled
    description: Checkbox is disabled.
    fixture: disabled_fixture
keys:
  - key: Space
    modifiers: []
    when: Checkbox has focus and is enabled.
    action: Toggle checked state.
    initial_state: unchecked
    expect:
      state: checked
      event: change
accessibility:
  role: checkbox
  properties:
    - name: aria-checked
      value: false
      when: unchecked
    - name: aria-checked
      value: true
      when: checked
    - name: aria-checked
      value: mixed
      when: indeterminate
    - name: aria-disabled
      value: true
      when: disabled
    - name: description
      value: Unavailable
      when: disabled
---

# Checkbox

## Purpose

A single binary choice that may also represent a mixed aggregate state.

## Anatomy

A compact square indicator and caller-provided persistent label. The label is the hit target.

## States

Unchecked, checked, indeterminate, and disabled. Activating indeterminate requests checked. A disabled control never changes.

## Props and events

`label`, `checked`/`default_checked`, `indeterminate`, and `disabled`. `is_checked()` and `is_indeterminate()` expose current local state for host logic and tests. Controlled mode emits `ChangeRequested(bool)` and waits for `set_checked(value, cx)`; uncontrolled mode updates internally and emits the same event. `set_checked` clears a mixed state and requests a redraw without emitting. Props never emit events.

## Keyboard map

Space toggles when focused. Register the `Checkbox` key context and named `Toggle` action; apps may replace its default `space` binding. An enabled checkbox is one tab stop; disabled is omitted from tab order.

## Pointer behaviour

Clicking the indicator or label toggles once. Disabled suppresses pointer actions.

## Accessibility role and properties

Checkbox role, accessible name from label, and checked boolean or mixed state. The disabled state sets the AccessKit disabled property, supplies an "Unavailable" description, and omits the tab stop. Label and indicator share one focus stop. On the pinned macOS bridge, AccessKit's mixed state appears as `AXValue=1`; the component adds a "Mixed" description so the state remains distinguishable from checked when this bridge is used.

## Theme tokens used

Global surface, text, border, accent, accent_text, focus, disabled colors; spacing.small, radii.small, border.hairline, controls.xsmall, and typography.body. The indicator is 0.62 times the xsmall control token so it sits inside the row's text line; maintainers should review this fixed ratio.

## WAI-ARIA pattern reference

[Checkbox](https://www.w3.org/WAI/ARIA/apg/patterns/checkbox/).

## Platform notes

Space activation follows desktop checkbox conventions; preserve system focus visibility.

## Open questions

Confirm public event naming, the `set_checked(value, cx)` redraw contract, and whether indeterminate activation should be configurable. Review disabled accessibility semantics on an active platform.
