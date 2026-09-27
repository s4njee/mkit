# E8.9 T3: clip placement and snapping

T3 adds a reversible clip-move preview to the T1/T2 timeline. Drag a clip within or between existing
tracks to choose a new start time; duration stays fixed and overlap remains app policy. `Enter`
commits a keyboard move, pointer release commits a drag, and `Escape` cancels. Both emit one
`ClipMoveRequested { clip_id, track_id, start }` request. Uncontrolled mode applies the move
locally before emitting the event. Controlled mode leaves committed data unchanged until the app
accepts the request through `set_tracks`; a rejected request leaves the committed model visible.
No event is emitted for a no-op or invalid destination.
Releasing a pointer outside every valid track lane cancels the drag, including when the pointer
previously crossed a valid lane. A valid release uses its final position and target track even
when the input system delivered no intermediate move event there.

Snapping uses 8 logical pixels and considers ruler ticks first, then the playhead, then clip edges.
Nearest candidate wins; ties use candidate priority and then earlier time. `Alt-S` toggles snap
bypass. While editing a clip, Shift-Left/Shift-Right nudges by the current ruler tick step and
Alt-Up/Alt-Down changes the target track. Space picks up the selected clip. Hosts can rebind these
through `MkitTimeline`.

The source contract is in `registry/timeline/spec.md`. Crate tests cover pure snap calculations,
pointer dragging, invalid drop cancellation, and controlled/uncontrolled commit/cancel behavior. The macOS gallery matrix sends
Space, Shift-Right, Enter, and Escape through GPUI input dispatch and captures clip preview, accepted
commit, and cancellation across light, dark, and high-contrast themes at 1× and 2×. Native pointer
capture beyond the window and active-platform AX snapshots remain pending. Public API, spec, keyboard
contract, and visual baselines need maintainer review.
