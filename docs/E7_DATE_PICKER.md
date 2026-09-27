# E7.11 Date picker draft

DatePicker and DateRangePicker share one implementation backed by the E7.10 Calendar grid and
TextField's native text/IME editing. Applications supply parser and formatter callbacks and
localized Calendar and picker labels; date syntax and presentation remain app policy.

The single picker commits valid text on Enter or a date chosen from the calendar. The range picker
shares one calendar between start and end fields and commits typed endpoints only when both parse
and satisfy constraints. Escape restores the session's committed value. Controlled forms emit
typed change requests; uncontrolled forms update internally and emit change events.

The registry component directly uses `mkit-registry-calendar` and `mkit-registry-text_field`. These
dependencies are implementation dependencies only; public values are `CivilDate`, `DateRange`, and
`DateValue`. Maintainers should review this cross-component dependency and the public parser,
formatter, and event names before treating the API as stable.

The macOS Metal screenshot matrix now covers all nine declared states in light, dark, and
high-contrast themes at 1× and 2×. GPUI interaction tests exercise valid and invalid typed input,
range endpoint completion, Escape rollback, controlled requests, disabled suppression, and Calendar
selection. Native platform accessibility tree snapshots remain pending.
