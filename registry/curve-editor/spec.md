---
spec_version: 1
component: curve-editor
states:
  - id: linear
    description: Selected channel displays a straight-segment curve.
    fixture: linear_fixture
  - id: smooth
    description: Selected channel displays a smooth cubic interpolation.
    fixture: smooth_fixture
  - id: selected-point
    description: A control point is focused for keyboard adjustment.
    fixture: selected_point_fixture
  - id: multiple-channels
    description: Channel tabs select independent point sets.
    fixture: multiple_channels_fixture
  - id: disabled
    description: Editor is non-interactive.
    fixture: disabled_fixture
keys:
  - key: ArrowRight
    modifiers: []
    when: Curve editor is focused with a selected point.
    action: Nudge selected point right by 0.01 normalized units, preserving point order.
    initial_state: selected-point
    expect: { event: curve_changed }
  - key: ArrowUp
    modifiers: []
    when: Curve editor is focused with a selected point.
    action: Nudge selected point up by 0.01 normalized units.
    initial_state: selected-point
    expect: { event: curve_changed }
  - key: Delete
    modifiers: []
    when: Curve editor is focused with a selected non-endpoint.
    action: Delete selected point.
    initial_state: selected-point
    expect: { event: curve_changed }
  - key: Tab
    modifiers: []
    when: Curve editor has multiple channels.
    action: Select next channel.
    initial_state: multiple-channels
    expect: { event: channel_changed }
accessibility:
  role: group
  properties:
    - name: name
      value: caller-provided label
    - name: description
      value: Select a channel, add points by clicking the curve, drag points to edit, delete selected point.
---

# Curve editor

## Purpose

Edit a normalized tone curve through sorted control points, channel selection, and either linear or
smooth interpolation. The app owns meaning, persistence, and mapping between normalized points and
its domain data.

## Anatomy

A responsive graph with a subtle grid, diagonal reference, rendered curve, and draggable control
points. Its height is six large control-height tokens. Channel selectors sit above the graph; a
linear/smooth mode toggle sits alongside them. A selected point receives a focus ring.

## States

Linear, smooth, selected point, multiple channels, and disabled. Curves have endpoint anchors at
`(0, 0)` and `(1, 1)`. Interior points have normalized x/y coordinates in `[0, 1]`, stay sorted by x,
and cannot cross one another. Duplicate x coordinates are prevented. Smooth interpolation uses
monotone cubic segments to avoid overshoot.

## Conformance fixtures

The dedicated screenshot matrix renders linear, smooth, selected-point, multiple-channels, and
disabled states at shadcn light, shadcn dark, and high contrast, each at 1× and 2×. Linear and
smooth use identical noncollinear points to expose interpolation differences. The selected-point
fixture uses a real pointer click on an interior marker and asserts selection before capture.
Multiple-channels selects a distinct second channel through the public setter. All fixtures use
the same graph width and window dimensions. The separate controlled-mode GPUI test verifies typed
proposal behavior; active-platform accessibility remains a distinct check.

## Props and events

`CurveEditor` is an `Entity` with a caller-provided label and a set of named channels, each with its
own point list. Every channel starts with the two endpoint anchors; supplied points are normalized
and sorted on construction. The selected channel and interpolation mode are readable and can be
changed by the owner. Uncontrolled edits update before emitting `CurveChanged`; controlled mode emits
the proposed channel points and waits for `set_channel_points`. Channel selection emits
`ChannelChanged`. Changing interpolation emits `InterpolationChanged`. No event is emitted for a
no-op. Interior points may be added up to the configured point limit and deleted; endpoint anchors
cannot be deleted.

A focused controlled-mode conformance test must dispatch an actual keyboard nudge and deletion,
verify that each `CurveChanged` payload contains normalized proposed points for the active channel,
verify the displayed points remain owner-supplied until `set_channel_points`, and verify that an
equal-value owner echo does not emit another change. This test complements the visual state matrix.
Another focused GPUI test must dispatch channel and interpolation keyboard actions and verify their
typed event payloads, state changes, and no-op behavior when a selection is repeated.

## Keyboard map

The editor registers a `CurveEditor` key context. Arrow keys nudge the selected point by 0.01 in
normalized graph space; Shift+arrows nudge by 0.001. Delete removes the selected interior point.
Tab and Shift+Tab cycle channels. `L` and `S` choose linear and smooth interpolation. Bindings are
public defaults and can be rebound by the host. Point x is clamped between adjacent points with a
small minimum gap; y is clamped to `[0, 1]`.

## Pointer behaviour

Left click near an existing point selects it and starts dragging. Left click elsewhere adds a point
at the pointer's normalized coordinate and selects it. Right click near an interior point deletes
it; endpoints are protected. Dragging clamps x between adjacent points and y to `[0, 1]`. Pointer
window-level mouse event listeners keep drag updates active as the cursor leaves the graph; the GPUI
test moves a point beyond the graph and confirms its clamped update. Disabled editors ignore pointer
and keyboard input.

## Accessibility role and properties

The root is a labeled group with a concise keyboard and pointer description. Channel and interpolation
controls request button semantics and expose their selected state in their accessible description.
The graph is not a single numeric slider; its description explains pointer and keyboard operation.
Point markers request button role and carry channel and normalized-coordinate names. The root group
receives keyboard focus and actions operate on the currently selected point. Native screen-reader
output and point-marker tab navigation need verification.

## Theme tokens used

Use GPUI Global `Theme` tokens for background, surface, border, accent, focus, text, and disabled
colors. Use theme spacing, border widths, control sizing, and radius tokens for chrome and point
handles. The look follows shadcn's quiet panel, thin grid, and clear selected ring without fixed
component colors.

## WAI-ARIA pattern reference

No APG pattern covers a curve editor. The graph is modeled as a labeled group with keyboard actions;
channel and interpolation controls follow the Button pattern.

## Platform notes

The component uses GPUI pointer and keyboard input only. Touch and native screen-reader interaction
need platform verification.

## Open questions

- Should channel color distinguish all channel names, or should selected channel remain the only
  theme-accent curve?
- What point count limit and minimum horizontal gap suit expected consumer apps?
- Should add/delete expose explicit toolbar buttons in addition to graph pointer actions?
- The public constructors, event payloads, controlled contract, and accessibility model need
  maintainer review.
