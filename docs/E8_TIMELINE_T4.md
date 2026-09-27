# E8.9 T4: keyframe markers and navigation

T4 adds app-owned keyframe markers with stable `KeyframeId`, a `TimelineObject` owner, finite time,
and an accessible label. Markers navigate in time/id order with Ctrl-Shift-Left and Ctrl-Shift-Right.
`N` picks up the selected marker, drag moves its time with the pointer, Shift-Left/Right nudges by
the ruler tick step, Enter commits a `KeyframeMoveRequested { keyframe_id, time }` request, and Escape
cancels. Uncontrolled mode applies the change locally before emitting. Controlled mode waits for
the owner to apply an accepted change through `set_keyframes`; rejected requests restore the
committed time. Releasing outside every track lane cancels the move; a valid release uses its final
position even without a preceding move event. Keyframe movement uses T3 snap candidates and the
same snap-bypass toggle.

The marker's accessible name and description include its label, owning track or clip, formatted
time, and selection state. Creation, interpolation, and animation evaluation stay app-owned.

Equal-time markers retain stable ID order. Dense and close-time markers use a compact three-row
band above clip content, with their pointer targets fanned horizontally around the cluster time.
The band remains bounded; unusually large clusters can extend beyond the visible track width, so
keyboard navigation is the fallback for those markers. Marker virtualization is not provided.

The source contract is in `registry/timeline/spec.md`. The focused crate suite passes 17 tests,
including move commit/cancel, snapping, pointer-release behavior, stable equal-time navigation, and
distinct pointer targets for dense markers. The macOS gallery matrix compares 48 T3/T4 cases:
eight states across light, dark, and high-contrast themes at 1× and 2×. Candidate images were
inspected before accepting the baselines; the dense dark 1× view keeps clip labels readable and
shows individually targetable markers. Native screen-reader traversal and synthesized pointer
drag remain pending.
