# E7.14 Time and date-time fields draft

`TimeField` is a segmented time editor with localized segment names and 12/24-hour display.
`TimeValue` stores a validated 24-hour time. Optional seconds, minute step, and inclusive time
bounds are configured on the field. Controlled commits emit `TimeChangeRequested`; uncontrolled
commits update the field and emit `TimeChanged`.

`DateTimeField` is a passive composition of caller-owned E7.11 `DatePicker` and E7.14 `TimeField`
entities. Callers observe the date and time child events separately and decide when a combined
date-time is valid. Its dependency on `mkit-registry-date-picker` is a documented registry
exception pending maintainer approval.

Both specs and conformance manifests are drafts in `registry/time-field/spec.md` and
`registry/date-time-field/spec.md`. Maintainer review is required for the public segment/event API,
step behavior, period control, and DateTimeField's child-entity composition. Focused tests and
macOS visual matrices are recorded in the component book pages; native platform accessibility tree
snapshots remain pending.
