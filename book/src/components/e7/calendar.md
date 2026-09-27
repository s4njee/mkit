# Calendar

Calendar presents an inline month grid for choosing a single civil date or a date range. It uses
`mkit_core::CivilDate`, which represents a Gregorian year, month, and day without a time zone.
Applications provide localized month and weekday labels, plus the first weekday for their locale.

The active date is the date that keyboard navigation moves. Selection is separate: activating a
date requests or commits a selection. A range calendar takes two activations and normalizes the
endpoints into chronological order. Minimum and maximum dates and the application's disabled-date
predicate restrict both focus movement and selection.

The grid follows the WAI-ARIA date picker grid keyboard convention: arrows move by day or week,
Page Up/Down changes month, Shift+Page Up/Down changes year, and Home/End moves to the locale
week's boundaries. The grid registers a `Calendar` key context so applications can rebind these
actions. Controlled mode emits a selection request and waits for the caller's updated selection;
uncontrolled mode owns its selected value.

See `registry/calendar/spec.md` for the state, event, accessibility, and theme contract. The
component is a draft pending maintainer review of the public date type and API. The macOS Metal
screenshot matrix covers five selection states across light, dark, and high-contrast themes at 1×
and 2×. GPUI interaction tests cover keyboard selection; platform accessibility tree snapshots are
not yet available.
