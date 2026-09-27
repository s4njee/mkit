---
spec_version: 1
component: overlay-dismissal-smoke
states:
  - id: nested_open
    description: Outer and inner overlays are open, with the inner layer on top.
    fixture: OverlayDismissalFixture::nested_open
  - id: outer_open
    description: The inner overlay has been dismissed while the outer overlay remains open.
    fixture: OverlayDismissalFixture after one top-layer dismissal
keys:
  - key: Escape
    modifiers: []
    when: Nested overlays are open.
    action: Dismiss only the topmost overlay.
    initial_state: nested_open
    expect:
      state: outer_open
      event: dismiss_top
  - key: E
    modifiers:
      - Control
    when: Nested overlays are open and the app has rebound dismissal to Ctrl+E.
    action: Dismiss only the topmost overlay.
    initial_state: nested_open
    expect:
      state: outer_open
      event: dismiss_top
accessibility:
  role: GenericContainer
  properties: []
---

# Overlay dismissal conformance smoke fixture

## Purpose

Exercise the existing E4 nested overlay example through generated conformance
cases. The fixture verifies that one key action dismisses only the top layer.

## Anatomy

The fixture has an outer overlay, an inner overlay, and an underlying control.
Its `nested_open` fixture starts with both overlays visible.

## States

- `nested_open`: outer and inner overlays are open.
- `outer_open`: only the outer overlay remains after one dismissal.

## Props and events

This smoke fixture has no public component props. A successful dismissal
records one Escape dismissal in the fixture state.

## Keyboard map

Escape dismisses the top overlay. In the rebinding case, Escape remains
unhandled and Ctrl+E dismisses the top overlay.

## Pointer behaviour

Pointer behavior is covered by the E4 example tests; this generated smoke
fixture focuses on keyboard dispatch.

## Accessibility role and properties

The current example uses GPUI's generic container role and does not establish
a reusable overlay accessibility contract. GPUI headless accessibility capture
is inactive, so generated accessibility cases remain unsupported pending
platform capture. A shipped dialog or popover needs its own reviewed role and
focus contract.

## Theme tokens used

The example reads surface, text, accent, border, and spacing values from the
GPUI Global theme token set.

## WAI-ARIA pattern reference

The overlay is a generic non-modal surface and does not claim a specialized
WAI-ARIA interaction pattern.

## Platform notes

Keyboard cases use GPUI's test input dispatcher. The screenshot comparison
runs on macOS through the GPUI headless Metal renderer.

## Open questions

Accessibility snapshots require an active platform accessibility tree, which
is unavailable in the current headless harness.
