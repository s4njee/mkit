# E8.9 timeline — proposed story split

Planning only. E8.9 is `P2`, `XL`, and depends on E8.1 Viewport. These slices cover the tracks, clips, playhead, zoomable time ruler, snapping, and keyframe markers promised in [plan.md](../plan.md). Each slice is small enough to specify, implement, and review separately. Before coding a slice, add its registry spec and conformance fixtures; the decisions below are proposals for maintainer review, not a frozen public API.

## Shared boundary and contract

The timeline is a stateful `Entity<Timeline>` view. The app owns the media, animation, playback clock, persistence, and commands such as splitting or moving a clip between tracks. The component draws app-supplied track, clip, and keyframe data, performs hit testing and time conversion, and emits typed edit requests. It does not start playback or mutate source assets. Reuse E8.1 coordinate-transform ideas, theme tokens, and pan/zoom conventions without making a timeline an app-painted viewport: the timeline must expose individual semantic objects.

Use stable `TrackId`, `ClipId`, and `KeyframeId` values supplied by the app. A clip has a track, start and end time, and label; a keyframe has a time and owner. Invalid times, missing owners, and overlapping edit policy must be specified before implementation. Use finite time values in one documented unit, tentatively seconds. Rendering can window a large data set, but keyboard and accessibility navigation must preserve identity when rows leave the screen.

Proposed typed events are `SelectionChanged { tracks, clips, keyframes }`, `PlayheadChanged { time }`, `VisibleRangeChanged { start, end }`, `ClipMoveRequested { clip_id, track_id, start }`, and `KeyframeMoveRequested { keyframe_id, time }`. The final event names and payloads need maintainer approval. No event fires for a no-op. In uncontrolled mode, accepted interaction updates local selection, playhead, range, or edit preview before emission. In controlled mode, the same event is a request; the displayed committed value changes only after the owner calls the corresponding setter. A rejected request returns to the owner value. Define whether edit events are emitted continuously while dragging or only on commit, and expose a distinct cancel outcome if a preview is public.

Register a `MkitTimeline` key context and named actions so the host can rebind every default below. Focus entry is a single timeline stop; internal navigation moves among tracks, clips, keyframes, and the playhead. Tab reaches ordinary toolbar controls and exits the composite. An active text editor keeps its native caret and IME keys. A labeled timeline group contains a track list and time surface. The playhead follows the [APG Slider](https://www.w3.org/WAI/ARIA/apg/patterns/slider/) value model. Use [APG Grid](https://www.w3.org/WAI/ARIA/apg/patterns/grid/) semantics only if the final row/cell navigation model matches; the time surface otherwise needs a documented custom collection model. Follow the [component pattern map](component-pattern-map.md#planned-component-mapping). Every meaningful clip/keyframe needs an accessible name, time, track relationship, selection state, and keyboard path; a labeled raster canvas alone is insufficient.

All paint and sizing reads GPUI `Global` theme values: `background` for the lane field, `surface` for headers/ruler, text and muted text for labels, `accent` and `focus` for selection/focus/playhead, border and spacing tokens for dividers/insets, and control-size tokens for hit targets. Do not encode track or media type solely by color. Any minimum row height, ruler height, marker size, time grid density, or snap tolerance must become a token or be justified in the slice spec. Respect light/dark themes and 1×/2× scale.

## T1 — Readable tracks and zoomable time ruler

**Scope.** Render ordered, labeled tracks and non-editable clips against a visible time range. Provide a ruler with stable tick labels, horizontal time pan and zoom centered on pointer or keyboard anchor, vertical track scrolling, and a fit-range command. Establish finite time-to-pixel and pixel-to-time conversion. Do not add clip edits, playhead dragging, snapping, or keyframe interaction in this slice.

**Acceptance.** Given the same data and range, tracks and clips render in deterministic order and position. Zoom keeps the anchor time under the pointer; fit contains the supplied extent; invalid or empty extents produce an explicit empty state. Wheel/trackpad behavior distinguishes horizontal scroll, vertical scroll, and zoom according to a documented modifier. Keyboard actions `ZoomIn`, `ZoomOut`, `PanEarlier`, `PanLater`, and `FitRange` work when the timeline has focus and emit `VisibleRangeChanged` under both state modes. Track and clip labels and time spans are available in the accessibility tree. The active track/clip remains identifiable after scrolling.

**Evidence.** Unit checks for conversion and tick spacing; harness keyboard scripts for all five actions and controlled rejection; accessibility snapshots for populated and empty states; screenshots of sparse and dense tracks in light/dark at 1×/2×, including zoomed and scrolled states. Inspect candidate diffs before accepting baselines.

## T2 — Playhead and selection

**Scope.** Add a visible playhead and selection of tracks or clips. Pointer click or scrub seeks the playhead; keyboard commands step time by a documented unit and move active selection between neighboring tracks/clips. A host-driven playback clock may call `set_playhead` without producing a user event. Keep range/selection/playhead state separate.

**Acceptance.** Seek and step clamp or extend according to the approved time-bound policy; one user action emits one `PlayheadChanged` request and does not start playback. The playhead exposes its current time as an accessible value with meaningful formatted text. Selection names the active object and selected objects; focus and selection remain distinct. Pointer and keyboard reach the same positions, and controlled mode displays an owner-rejected seek without drift. No shortcuts fire from an embedded text field.

**Evidence.** Keyboard scripts for seek, step, selection, focus entry/exit, and disabled/read-only behavior; accessibility snapshots for selected clip and focused playhead; screenshots for playhead, hover, selected, and focus states across both themes/scales. Manually verify physical trackpad and native screen-reader time announcements until the harness covers them.

## T3 — Clip placement with snapping

**Scope.** Move a clip within or between tracks by pointer and keyboard. Show a reversible drag preview and snap indication against ruler grid, playhead, and adjacent clip edges. Supply a way to temporarily bypass snapping. This slice moves whole clips; trimming, ripple edits, and overlap resolution remain app policy unless later approved.

**Acceptance.** Pointer and keyboard placement produce the same proposed track and start time. Snap candidates use a documented priority and time-space tolerance; zoom changes do not change the intended target at a fixed screen tolerance. A move can be cancelled with Escape and leaves committed data untouched. Invalid destinations are visibly rejected and emit no committed move. On accept, emit a typed `ClipMoveRequested`; controlled mode waits for owner data, and uncontrolled mode applies the allowed update once. Announce new track/time after a keyboard move. Keyboard actions cover pick up, move earlier/later, move track up/down, commit, cancel, and snap bypass.

**Evidence.** Pure snap tests for ties, zoom, negative/zero time, and invalid destinations; interaction scripts for pointer move, keyboard move, cancel, controlled accept/reject; accessibility snapshots for drag preview and committed move; screenshots for snap, invalid target, and focus/selection in light/dark at 1×/2×. Manual pointer capture check beyond the window if the harness cannot simulate it.

## T4 — Keyframe markers and navigation

**Scope.** Display app-supplied keyframe markers on tracks or clips, select and navigate among them, and move a keyframe in time using the same snapping policy. Creation, interpolation curves, multi-keyframe transforms, and animation evaluation stay with the app or a later story.

**Acceptance.** Stable marker identity survives zoom and virtualization. Keyboard actions move focus to previous/next keyframe, nudge a selected marker, commit, and cancel; pointer selection and drag offer the same edits. Every keyframe has an accessible name containing owner and formatted time plus selected/focused state. A changed marker emits `KeyframeMoveRequested` only when a real proposed time differs. Controlled owner rejection restores the committed position; snap bypass and invalid-time behavior match T3.

**Evidence.** Keyboard and pointer scripts over sparse/dense markers; controlled/uncontrolled event assertions; accessibility snapshots for marker navigation and move; screenshots of marker overlap, selection, snap, and high zoom in both themes/scales. Manually inspect screen-reader traversal if the platform tree omits off-screen objects.

## Review gates and open questions

Each slice needs a registry spec (states, events, keyboard map, role/properties, tokens, platform notes), compiling example with mdBook `{{#include}}` anchors, relevant build/tests/lint, book build, and coexistence check. Record commands and actual results in its PR. Harness evidence above is required where adapters exist; record any missing adapter or inactive accessibility bridge and the manual check instead of claiming a pass. Request human review of the spec, public names and event timing, keyboard/AX contract, screenshots/baseline changes, and book accuracy before closing a slice.

- Should the timeline use seconds, rational frame time, or an app-supplied time scale? How are drop-frame labels and tempo maps handled?
- Are clips allowed to overlap or cross tracks, and who resolves collisions?
- Does clip/keyframe movement emit during drag or only at commit? What should undo group as one edit?
- What is the exact snap priority and tolerance, and should apps supply custom snap points?
- Which objects are exposed as individual accessibility nodes when a long timeline is virtualized?
- Should the playhead and selection be separate Tab stops, or one composite focus path? Confirm with native screen-reader testing.
