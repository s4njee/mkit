---
spec_version: 1
component: split-pane
states:
  - id: centered
    description: Two panels with a centered divider.
    fixture: centered_fixture
  - id: resized
    description: Divider moved from the initial ratio.
    fixture: resized_fixture
  - id: vertical-centered
    description: Vertical pane divider at the initial 50 percent ratio.
    fixture: vertical_centered_fixture
  - id: focused
    description: Horizontal divider has keyboard focus and uses the focus token.
    fixture: focused_fixture
  - id: disabled
    description: Horizontal divider is unavailable, visually muted, and excluded from focus.
    fixture: disabled_fixture
keys:
  - key: ArrowRight
    modifiers: []
    when: Horizontal split divider has focus.
    action: Increase leading pane size by one step.
    initial_state: centered
    expect:
      state: resized
      event: ratio_changed
  - key: ArrowLeft
    modifiers: []
    when: Horizontal split divider has focus.
    action: Decrease leading pane size by one step.
    initial_state: centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowRight
    modifiers: [Shift]
    when: Horizontal split divider has focus.
    action: Increase leading pane size by one fine step.
    initial_state: centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowLeft
    modifiers: [Shift]
    when: Horizontal split divider has focus.
    action: Decrease leading pane size by one fine step.
    initial_state: centered
    expect: { state: resized, event: ratio_changed }
  - key: Home
    modifiers: []
    when: Horizontal split divider has focus.
    action: Move leading pane to its minimum ratio.
    initial_state: centered
    expect: { state: resized, event: ratio_changed }
  - key: End
    modifiers: []
    when: Horizontal split divider has focus.
    action: Move leading pane to its maximum ratio.
    initial_state: centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowDown
    modifiers: []
    when: Vertical split divider has focus.
    action: Increase leading pane height by one step.
    initial_state: vertical-centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowUp
    modifiers: []
    when: Vertical split divider has focus.
    action: Decrease leading pane height by one step.
    initial_state: vertical-centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowDown
    modifiers: [Shift]
    when: Vertical split divider has focus.
    action: Increase leading pane height by one fine step.
    initial_state: vertical-centered
    expect: { state: resized, event: ratio_changed }
  - key: ArrowUp
    modifiers: [Shift]
    when: Vertical split divider has focus.
    action: Decrease leading pane height by one fine step.
    initial_state: vertical-centered
    expect: { state: resized, event: ratio_changed }
accessibility:
  role: splitter
  properties:
    - name: value-min
      value: 0
    - name: value-max
      value: 100
---

# Split pane

## Purpose

Let users resize two adjacent application panels.

## Anatomy

Leading pane, focusable divider, and trailing pane. Orientation chooses horizontal or vertical placement.

## States

Centered, resized, focused, dragging, vertical-centered, and disabled. The focused divider uses the theme focus token. Disabled uses the disabled token and does not enter keyboard focus. Dragging changes the ratio continuously; the resized fixture represents its stable result.

## Props and events

`ratio` or `default_ratio`, `min_ratio`, `max_ratio`, `orientation`, `disabled`; typed `RatioChanged(f32)` event. Controlled mode emits a requested ratio and waits for `set_ratio`; uncontrolled mode updates itself. `set_ratio` emits no event.
Default bounds are 10–90% of the available space, keeping both panes reachable.

## Keyboard map

Horizontal: Left/Right; vertical: Up/Down. Home/End go to bounds, Shift modifies step. `MkitSplitPane` context exposes named resize actions.
Normal adjustment is 2% of the viewport; Shift adjustment is 0.5%.
Tab reaches the enabled divider in normal focus order; disabled dividers are omitted. Keyboard focus exposes the focus token on the divider.

## Pointer behaviour

Dragging the divider adjusts the ratio within bounds; releasing ends the drag. Disabled blocks drag.
The root and divider expose stable `mkit-split-pane` and `mkit-split-pane-divider` GPUI debug selectors for pointer harness scripts.

## Accessibility role and properties

The divider exposes splitter role, name, orientation, current value, and bounds; children preserve their own semantics.

## Theme tokens used

`Theme.colors.border/focus/disabled/surface`, `Theme.borders.regular`, `Theme.spacing.xsmall/small`, `Theme.radii.small`.

## WAI-ARIA pattern reference

[Window splitter](https://www.w3.org/WAI/ARIA/apg/patterns/windowsplitter/).

## Platform notes

The initial implementation uses pointer coordinates in the rendered split bounds.

## Open questions

Confirm pointer capture behavior when dragging beyond the application window.
