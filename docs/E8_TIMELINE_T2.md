# E8.9 T2: playhead and selection

T2 adds an app-readable playhead and single-object selection to the T1 timeline. Times remain finite
seconds. A user seek is clamped to the visible time range; owner playback-clock updates may place the
playhead anywhere in finite time. Seeking never starts playback. Pointer seeking starts on the ruler
and scrubs while the left button remains down. Clicking a track label or clip selects that object.

`Timeline::new` owns playhead and selection and applies changes before emitting events.
`Timeline::controlled` emits `PlayheadChanged` and `SelectionChanged` requests while continuing to
display owner state; `set_playhead` and `set_selection` apply owner updates without emitting user
events. An empty selection has no active object; a single selected object is also the active object.
Track order followed by clip time/id order defines selection navigation order.

The `MkitTimeline` context adds rebindable `StepEarlier`, `StepLater`, `PreviousObject`, and
`NextObject` actions. Comma and period step by 0.1 seconds, clamped to the visible range. Up and down
select the previous and next track or clip. Left/right retain T1's range-pan behavior. The playhead
is represented as a slider value; the root remains a labeled group and tracks/clips remain named
objects with selected state. Embedded controls keep their own key handling.

Playhead and selection use theme accent/focus, text, border, spacing, and control-size tokens. The
playhead line is one hairline wide, with a small-control-sized ruler hit area. Media type is never
encoded only by color.

## Acceptance covered by this slice

- Ruler click and drag request/apply one playhead value per distinct time.
- Comma/period step the playhead; up/down navigate stable track/clip order.
- Track and clip clicks update selection; keyboard focus does not imply selection.
- Controlled requests do not drift when the owner rejects them.
- The accessible playhead reports its formatted current time and selected objects expose selected
  state.

## Evidence and gaps

Focused crate tests cover controlled and uncontrolled keyboard changes, selection navigation, and
playhead clamping. The active-platform accessibility bridge is not available in headless tests, so
native screen-reader time announcement and physical trackpad behavior remain manual checks. T2 does
not add screenshot baselines or example/book integration; these require the parent integration
change.

## Open questions for maintainer review

- Should the public time API use seconds or an app-provided frame/tempo time scale?
- Should selection be multi-object, including modifier-click and range selection?
- Should user seeks be clamped to the visible range or allowed outside it?
- Confirm event names, 0.1-second step, keyboard bindings, and slider/group accessibility contract.
