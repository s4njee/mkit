# E7.10 Calendar draft

Calendar now has a registry spec, source, and conformance manifest. The spec decides on a small
timezone-free `mkit_core::CivilDate` with a proleptic Gregorian calendar and years 1–9999. This
avoids pulling `chrono` or `jiff` into every component; maintainers should approve that public API
decision.

The component accepts localized month and weekday names and a Monday-based first-weekday index.
It supports a single date or a two-step date range, chronological endpoint normalization, bounds,
and an application-supplied disabled-date predicate. Keyboard movement follows the APG date picker
grid pattern through a rebindable `Calendar` key context.

Review items: public naming and date type; exact labels/formatting API; AccessKit selected/current
semantics across platforms; and the generated code's registry and `mkit` crate registration. The
current E0 harness has no Calendar fixture adapter. The conformance manifest therefore marks
keyboard, accessibility, and screenshot runs pending; no harness result is claimed.
