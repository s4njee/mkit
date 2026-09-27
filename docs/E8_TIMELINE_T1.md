# E8.9 T1: readable tracks and zoomable time ruler

`mkit-registry-timeline` renders app-supplied tracks and clips on a seconds-based time axis. It
provides a visible range, ruler ticks, horizontal pan and zoom, vertical track scrolling, and fit to
the supplied clip extent. Tracks and clips remain read-only; the consuming app owns media, playback,
selection, persistence, and edit commands.

Use `=` and `-` to zoom around the visible center, left and right arrows to pan, and `F` to fit the
clip extent. Ctrl or platform modifier plus wheel zooms around the pointer time. A horizontal wheel,
or Shift plus vertical wheel, pans the time range; unmodified vertical scrolling moves through tracks.

Use `Timeline::new` for uncontrolled pan/zoom state or `Timeline::controlled` when the owner approves
the proposed range. Subscribe to `VisibleRangeChanged`, then apply accepted changes with
`set_visible_range`. Stable `TrackId` and `ClipId` values keep semantics tied to app data as viewport
contents change.

The component contract and open questions are in [`registry/timeline/spec.md`](../registry/timeline/spec.md).
T1 does not include the playhead, selection, clip editing, snapping, or keyframes; those belong to
the later timeline stories.
