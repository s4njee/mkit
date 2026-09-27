---
spec_version: 1
component: timeline
states:
  - id: sparse
    description: A few app-supplied tracks and clips are readable across the visible time range.
    fixture: sparse_fixture
  - id: dense
    description: Many clips remain ordered, individually named, and readable in a vertically scrollable track list.
    fixture: dense_fixture
  - id: zoomed
    description: The time range is zoomed while preserving the keyboard or pointer anchor time.
    fixture: zoomed_fixture
  - id: scrolled
    description: The track list has a vertical scroll offset and the active visible tracks remain named.
    fixture: scrolled_fixture
  - id: empty
    description: No valid clips are supplied and a clear empty state is shown.
    fixture: empty_fixture
  - id: controlled
    description: Visible-range proposals emit events while the owner-controlled range remains displayed.
    fixture: controlled_fixture
  - id: playhead
    description: A visible playhead can be sought without starting playback.
    fixture: playhead_fixture
  - id: selected
    description: One track or clip is selected independently of keyboard focus.
    fixture: selected_fixture
  - id: controlled_playhead
    description: Playhead and selection requests emit while owner values remain displayed.
    fixture: controlled_playhead_fixture
  - id: clip_move
    description: A clip move preview shows proposed track and start until commit or cancel.
    fixture: clip_move_fixture
  - id: clip_move_committed
    description: A keyboard-moved clip is committed and the owner-applied track data is displayed.
    fixture: clip_move_committed_fixture
  - id: clip_move_cancelled
    description: Escape cancels a keyboard-moved clip and the original owner data remains visible.
    fixture: clip_move_cancelled_fixture
  - id: keyframe_move
    description: A selected keyframe has a reversible snapped move preview.
    fixture: keyframe_move_fixture
  - id: keyframe_move_committed
    description: A keyboard-moved keyframe is committed and owner-applied keyframe data is displayed.
    fixture: keyframe_move_committed_fixture
  - id: keyframe_move_cancelled
    description: Escape cancels a keyboard-moved keyframe and its original time remains visible.
    fixture: keyframe_move_cancelled_fixture
  - id: keyframe_selection
    description: Keyframes are selected and navigated by stable time and id order.
    fixture: keyframe_selection_fixture
  - id: keyframe_density
    description: Equal-time and tightly clustered keyframes occupy distinct hit targets in stable time/id order.
    fixture: keyframe_density_fixture
keys:
  - key: Equal
    modifiers: []
    when: The labeled timeline group has keyboard focus.
    action: Zoom in by 25 percent around the visible-range center.
    initial_state: sparse
    expect: { event: visible_range_changed }
  - key: Minus
    modifiers: []
    when: The labeled timeline group has keyboard focus.
    action: Zoom out by 25 percent around the visible-range center.
    initial_state: zoomed
    expect: { event: visible_range_changed }
  - key: ArrowLeft
    modifiers: []
    when: The labeled timeline group has keyboard focus.
    action: Pan earlier by ten percent of the visible duration.
    initial_state: sparse
    expect: { event: visible_range_changed }
  - key: ArrowRight
    modifiers: []
    when: The labeled timeline group has keyboard focus.
    action: Pan later by ten percent of the visible duration.
    initial_state: sparse
    expect: { event: visible_range_changed }
  - key: f
    modifiers: []
    when: The labeled timeline group has focus and at least one valid clip exists.
    action: Fit the full clip extent with five percent padding on each side.
    initial_state: sparse
    expect: { event: visible_range_changed }
  - key: ","
    modifiers: []
    when: The labeled timeline group has keyboard focus.
    action: Step the playhead earlier by 0.1 seconds, clamped to the visible range.
    initial_state: playhead
    expect: { event: playhead_changed }
  - key: "."
    modifiers: []
    when: The labeled timeline group has keyboard focus.
    action: Step the playhead later by 0.1 seconds, clamped to the visible range.
    initial_state: playhead
    expect: { event: playhead_changed }
  - key: ArrowUp
    modifiers: []
    when: The labeled timeline group has keyboard focus and objects exist.
    action: Select the previous object in track order then clip time/id order.
    initial_state: selected
    expect: { event: selection_changed }
  - key: ArrowDown
    modifiers: []
    when: The labeled timeline group has keyboard focus and objects exist.
    action: Select the next object in track order then clip time/id order.
    initial_state: selected
    expect: { event: selection_changed }
  - key: Enter
    modifiers: []
    when: A clip or keyframe move is active.
    action: Commit the edit and emit its typed move request when changed.
    initial_state: clip_move
    expect: { event: clip_move_requested }
  - key: Escape
    modifiers: []
    when: A clip or keyframe move is active.
    action: Cancel the reversible edit preview.
    initial_state: keyframe_move
    expect: { state: keyframe_selection }
  - key: ArrowRight
    modifiers: [Shift]
    when: A move preview is active.
    action: Nudge the selected clip or keyframe later by one ruler tick step.
    initial_state: keyframe_move
    expect: { state: keyframe_move }
  - key: ArrowRight
    modifiers: [Control, Shift]
    when: The timeline has focus and keyframes exist.
    action: Select the next keyframe in time/id order.
    initial_state: keyframe_selection
    expect: { event: keyframe_selection_changed }
accessibility:
  role: group
  properties:
    - name: name
      value: caller-provided timeline label
    - name: description
      value: Ordered tracks and clips with time spans. Use left and right arrows to pan, plus and minus to zoom, and F to fit all clips.
    - name: edit_preview
      value: Proposed clip/keyframe target and time; absent outside an edit
      when: clip_move
    - name: keyframe_name
      value: Keyframe label, owner relationship, formatted time, and selection state
      when: keyframe_selection
  playhead:
    role: slider
    properties:
      - { name: value, value: formatted playhead time in seconds }
  selected_objects:
    role: group
    properties:
      - { name: description, value: Selected or Not selected }
---

# Timeline T1/T2: readable tracks, time ruler, playhead, and selection

## Purpose

Render app-supplied tracks and clips read-only against a visible range. Provide stable time/pixel
conversion, a labeled time ruler, horizontal pan and zoom, vertical track scrolling, and a fit-range
command. T2 adds playhead movement and single-object selection. Clip edits, snapping, and keyframes
remain out of scope.

## Anatomy

A toolbar names the timeline and offers zoom and Fit. Below it, a time ruler shares the horizontal
range with ordered track rows. Tick labels near the right edge align inward so their text stays in
the ruler. Each row has a text label and a lane of read-only, named clips.

## States

Time values are finite `f64` seconds. `TrackId` and `ClipId` are stable app-supplied `u64` values.
Tracks render in supplied order; each track's valid clips sort by start time and then stable id.
Clips with non-finite values, non-finite duration, or `end <= start` are ignored. Overlap is allowed
and does not change app data. The visible range requires finite `start < end` and finite duration. An empty/invalid clip extent renders an
explicit empty state. Fit uses the minimum clip start and maximum clip end plus five percent padding.

## Props and events

`Timeline` is a GPUI `Entity`. `new(label, tracks, range)` is uncontrolled: a user pan, zoom, or fit
applies the new `TimeRange` before emitting `VisibleRangeChanged { start, end }`. `controlled(...)`
emits the same proposal and keeps showing its supplied range until the owner applies a value using
`set_visible_range`. No event fires for a no-op. `set_tracks` updates app-supplied content without
emitting a user event. `time_to_pixel` and `pixel_to_time` map finite values linearly across the
visible interval and reject invalid width or range inputs.

## Keyboard map

The `MkitTimeline` key context exposes rebindable `ZoomIn`, `ZoomOut`, `PanEarlier`, `PanLater`, and
`FitRange` actions. `=`/`-` zoom by 25 percent around the visible center, left/right pan by ten
percent of visible duration, and `F` fits the content extent.

## Pointer behaviour

On the time surface, Ctrl or platform
modifier plus wheel zooms around the pointer's time. Horizontal wheel deltas pan time; Shift+vertical
wheel deltas pan time; unmodified vertical wheel deltas scroll the track list. Toolbar buttons expose
the same zoom and fit actions. Clips are not draggable in T1.

## Accessibility role and properties

The focusable root requests group role with caller label and instructions. The track viewport
requests scroll-view role and label. Every track is an individually named group. Every clip is an
individually named group whose accessible description includes its track and formatted start/end
time. Ruler ticks are visual labels within a named time-ruler group. Empty data exposes a status
message. Clips are read-only; no fake button or selection semantics are added. Platform snapshots
and announcement order need verification in the active harness.

## Theme tokens used

Read frame, lane, ruler, text, accent, focus, and border colors from `mkit-core`'s GPUI Global
`Theme`. Use theme spacing, radii, border, typography, and control-size tokens for padding, ruler
height, lane height, labels, and clip hit geometry. Track type is always named in text and never
encoded only by color. Lane height is one large control token; ruler height is one small control
token; the scroll viewport shows eight large-token rows before scrolling.
Ruler ticks keep at least 72 logical pixels apart so short second labels remain legible at the
default theme typography; this is a label-collision threshold rather than a control dimension.
Tick generation caps at 512 labels to bound work for extreme ranges.

## WAI-ARIA pattern reference

There is no APG pattern for a read-only multitrack time surface. T1 follows labeled group and
scroll-view platform semantics so tracks and clips remain individually available. The later
playhead slice should follow the [APG Slider pattern](https://www.w3.org/WAI/ARIA/apg/patterns/slider/).

## Platform notes

Native accessibility snapshots are needed to verify track and clip announcement order. Physical
trackpad scrolling and zooming need manual checks alongside the GPUI interaction tests.

## Open questions

- Is seconds the right shared time unit for frame-based or tempo-mapped applications?
- Should tick labels later support SMPTE/drop-frame or a consumer-supplied formatter?
- Should the visible-range event fire continuously during wheel gestures or be debounced by the host?
- Track ordering, public names, keyboard behavior, and accessibility contracts require maintainer review.

## T2: playhead and selection

Time remains finite seconds. `Timeline::new` initializes an uncontrolled playhead at the visible
range start and an empty selection. `Timeline::controlled` also controls playhead and selection.
`set_playhead` accepts any finite owner playback-clock time; user seeks clamp to the visible range.
`set_selection` accepts zero or one existing `TimelineObject` (track or clip). Tracks and clips are
ordered by supplied track order and normalized clip time/id order for keyboard navigation.

Pointer click or scrub on the ruler seeks. Clicks on track labels and clips select that object.
Comma and period step by 0.1 seconds. Up/down select the previous/next object and do not alter the
playhead. Left/right continue to pan the visible range. Focus and selection are independent.

`PlayheadChanged { time }` and `SelectionChanged { selected }` are typed events. Uncontrolled mode
applies before emission; controlled mode emits a request and waits for owner setters. Setters never
emit. Invalid times, missing objects, and no-op requests emit nothing. The playhead has slider role
and exposes formatted current time. Selected track/clip nodes include selection in their accessible
description because GPUI's generic group node does not support the selected property. Selected
clips also show a check mark and stronger theme border, so selection is not conveyed by color alone.

T2 adds playhead and selected states. Accent/focus and text/border/spacing/control-size values come
from the GPUI `Global` theme. A hairline playhead and small-control ruler hit target are justified
geometry; colors remain theme tokens. The root group and track/clip semantics from T1 remain.

- Confirm whether selection should support multiple objects and modifier-click.
- Confirm the seconds unit, 0.1-second step, seek clamp policy, event names, and accessibility
  contract. Native screen-reader announcement behavior needs manual platform review.

## T3: clip placement and snapping

`TimelineClipMoveRequested { clip_id, track_id, start }` is emitted only on commit when the
proposed target differs from the committed clip position. Move gestures keep a reversible preview
until commit. Uncontrolled mode applies the committed move before emitting; controlled mode waits
for the owner to apply accepted `Track` and `Clip` data through `set_tracks`. `set_tracks` cancels a
preview if its clip disappears. Escape cancels; Enter commits. A pointer release outside every
valid track lane cancels the drag and emits no move request, even if the pointer had visited a valid
lane earlier. A release inside a lane uses that release coordinate and target track, even if no
intermediate move event was delivered there. Invalid programmatic destinations are ignored. No
overlap policy is imposed.

A move snaps within 8 logical pixels to ruler ticks, then the playhead, then other clip edges, with
ties resolved by that priority and then earlier time. The tolerance is screen-space so zoom does not
change the effective target. Alt-S toggles snap bypass. Keyboard actions `PickUpClip`, `MoveEarlier`,
`MoveLater`, `MoveTrackUp`, `MoveTrackDown`, `CommitEdit`, `CancelEdit`, and `ToggleSnapBypass` live
in `MkitTimeline`; Space picks up, Shift-Left/Right nudges by the current ruler tick step, Alt-Up/Down
changes target track, Enter commits, and Escape cancels. A preview exposes proposed track/time and
validity in its accessible description.

## T4: keyframes

`Keyframe { id, owner, time, label }` uses stable app-supplied `KeyframeId` and a `TimelineObject`
owner. Non-finite keyframe times and missing owners are ignored. `KeyframeMoveRequested { keyframe_id,
time }` fires only on commit when time differs. As for clips, both modes show a reversible preview;
uncontrolled mode applies the move before emitting, while controlled mode waits for `set_keyframes`
as the owner commit/rejection path. Pointer release outside every valid track lane cancels without
emitting; release inside a lane uses its final coordinate even without a preceding move event.
Escape cancels and Enter commits. Keyframes use
the T3 snap candidates/tolerance and Alt bypass. Ctrl-Shift-Left/Right navigate stable time/id order;
`n` picks up, Shift-Left/Right move by ruler tick step, Enter commits, Escape cancels. Each marker is a
named group with owner, formatted time, and selected/focused state. Equal-time markers sort by
stable id. Each track reserves a compact three-row marker band above its clip content. Markers
within a close time cluster use those separate vertical slots and fan horizontally around the
cluster's actual time position, keeping their hit targets disjoint without covering clip labels.
Accessible time remains the exact app-supplied time. Keyboard navigation always follows stable
time/id order. Extremely large clusters can fan beyond the visible track width, so keyboard
navigation remains the fallback for markers outside the viewport; marker virtualization is not
currently provided. Keyframe creation and interpolation remain app-owned.

T3/T4 screenshot fixtures additionally cover accepted and cancelled clip/keyframe edits. The focused
gallery matrix applies Space/Shift-Right/Enter or Escape and Ctrl-Shift-Right/N/Shift-Right/Enter or
Escape through GPUI keyboard dispatch; commit fixtures accept the typed request in a host adapter.
Theme colors and dimensions continue to come from GPUI `Global` theme tokens; snap tolerance is a
fixed 8 logical pixels and marker geometry is one small-control token, as justified by interaction
hit-testing. Full native accessibility snapshots and pointer capture evidence remain harness/platform
dependent.
