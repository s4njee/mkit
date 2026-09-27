# E5.5 pilot measurement log

This log records the observed draft work for the two pilot components. It is
not a full spec → code → conformance → docs → registry measurement: both
components still need complete pointer behavior, visual and accessibility
conformance, rendered book examples, full gallery state coverage,
installation, and maintainer review.

| Slice | Observed wall time | Result | Token count |
|---|---:|---|---:|
| Combobox initial source and review fixes | 18:54:37–19:00:51 UTC, 6m 14s | Draft source, 2 focused GPUI tests, isolated Clippy pass | Unavailable |
| Number-field keyboard extension | About 8 minutes, agent reported | Eight named key bindings, bounds and overflow tests | Unavailable |
| Number-field generated adapter and native input | Not recorded | 10/10 keyboard cases pass, including real draft entry, Enter commit, and Escape rollback | Unavailable |
| Number-field pointer scrub draft | Not recorded | Drag threshold, modifier steps, clamping, continuous change, release commit, and Escape rollback have focused GPUI tests; full pointer contract remains open | Unavailable |
| Number-field screenshot baseline | Not recorded | Idle/light/1× Metal capture shows readable value text and matches the committed baseline; human visual review remains open | Unavailable |
| Combobox generated adapter and native input | Not recorded | 9/9 keyboard cases pass, including Tab and Shift+Tab focus traversal; native query entry has a focused GPUI test | Unavailable |
| Pilot book chapters and compiling examples | Not recorded | Two draft pages, constructor examples in a tested crate; rendered book screenshots pending | Unavailable |
| Pilot gallery previews | Not recorded | Standalone gallery opens both components and switches among three token themes; default light/1× Metal baseline shows readable values, with the visual matrix pending | Unavailable |

The initial number-field source was started before this measurement log, so
the eight-minute extension is not its total implementation time. The two
adapter and later implementation durations were not captured. These slices ran partly in parallel;
their times must not be summed as total elapsed work.

The committed manifests currently contain 10 keyboard, 6 accessibility, and
36 screenshot cases for the number field, and 9 keyboard, 8 accessibility,
and 48 screenshot cases for the combobox. CI runs both complete keyboard maps.
Accessibility capture is still inactive in the pinned headless GPUI window;
those cases cannot be counted as passing. One focused number-field screenshot
case passes on macOS. The remaining screenshot cases are pending, and neither
pilot screenshot matrix has passed.

Agent review found and corrected a number-field test expectation after
`End` followed by `PageDown`, added an explicit typography token for its
rendered text, and required the combobox to leave disabled matches visible
while skipping them during navigation and omitting disabled focus tracking.
This is agent rework, not measured human maintainer rework. Maintainer review
has not happened, so the latter remains unknown.

The agent environment did not expose per-task token usage. The project
estimates in §9 of `plan.md` therefore cannot yet be replaced with measured
numbers. Future pilot runs need start/end timestamps, model/token usage from
the task runner, and a record of maintainer edits after review.
