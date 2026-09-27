# E8.14 Laika adoption spike

## Spec / migration contract

### Goal

Laika is the first production-style consumer of mkit's slider, histogram, and
segmented-control components. Keep Laika's edit semantics, palette, and current
layout while replacing presentation and interaction with the registry versions.

### Existing state and contracts

- The histogram is display-only, receives 48 normalized luminance bins, and is
  rendered in the Develop and Library views. Its accessible name and summary
  must describe the histogram, and its 74/78 px host heights remain stable.
- Slider rows edit one `ParamDef` in a larger Develop transaction. They support
  drag/scrub, double-click reset, keyboard nudging, typed value editing,
  modified/focus styling, undo history, and multi-photo effective values. All
  mutations flow through `Laika::set_value`/`commit_edit`.
- Segmented controls are view/action pickers. View choices can trigger actions
  such as Compare or Survey, and are refreshed from the active view state.
- Laika appearance/accent are persisted app preferences. Embedded mkit
  controls must follow that palette rather than install a competing theme.

### Controlled/uncontrolled and event mapping

- Histogram: stateless `RenderOnce`; values and accessible summary are supplied
  by the host on each render.
- Slider: use controlled mode with the current effective value and map
  `ChangeRequested(Vec<f64>)` to `Laika::set_value`. Bracket pointer gestures
  and individual keyboard nudges with typed `InteractionStarted`,
  `InteractionEnded`, and `InteractionCancelled` events. The host keeps each
  component entity stable and updates values after presets, resets, undo, and
  selection changes. One start/end pair is one undo transaction; cancellation
  restores the original value vector without committing.
- Segmented control: use controlled mode with the selected Laika view key and
  map `ValueChanged(Option<String>)` to the existing view actions. The host
  must preserve one component entity across renders and keep the selected key
  synchronized after keyboard shortcuts and external view changes.

Laika owns one persistent `Entity<SegmentedControl>` and one `Entity<Slider>`
per Develop parameter. Typed subscriptions map view proposals to existing view
actions and slider proposals to `Laika::set_value`; lifecycle events seed,
commit, or cancel the existing undo gesture. The host refreshes controlled
values after presets, reset, undo/redo, and selection changes. Segmented items
retain their individual hover tips with the optional `Item::tooltip` builder.

### Keyboard, accessibility, theme

Registry slider keyboard behavior and accessible slider roles/properties are
owned by mkit. Registry segmented-control radio-group keyboard behavior,
accessible group label, and item selection are owned by mkit. The histogram is
an image with an accessible name and textual summary. Host shortcuts outside
these controls remain Laika-owned. All three controls read the GPUI Global
`mkit_core::theme::Theme`; Laika maps its existing palette to those semantic
tokens when starting and whenever appearance/accent changes.

### Open questions

1. Should mkit's Slider add a formatter or reset event, or should Laika keep
   formatting and reset affordances in its host row?
2. What is the intended relocatable dependency for local app adoption before
   mkit is published? This spike uses a local absolute path.

## Spike result

The Laika worktree is isolated at `/tmp/laika-e8-mkit-adoption`, branch
`codex/e8-mkit-adoption`. Its committed base was clean; the user's modified
checkout at `/Users/sanjee/projects/Laika` was not edited.

The worktree pins `gpui-kit = 0.6.4`, `gpui-pre = 0.3.5`, and
`gpui-pre-platform = 0.3.5`, matching this mkit workspace. `laika-app` has a
local path dependency on mkit with only the slider, histogram, and
segmented-control features enabled. The lockfile resolved a single GPUI version
for the application and mkit.

Laika's existing histogram adapter now returns the mkit `Histogram`, preserving
the 48-bin luminance input, 74/78 px heights, and app call sites. The app
installs a semantic mkit Theme Global mapped from Laika's background, panel,
text, border, accent, and disabled tokens at startup and when settings change.

The Library view selector now uses one persistent controlled mkit
`SegmentedControl`, preserving typed `ValueChanged` routing for Compare, Survey,
Wall, Timeline, and the basic Grid/Loupe choices. Each item's tooltip is kept
on the radio item and its text is exposed as the accessible description.
Develop rows keep their typed value editor/readout while using persistent mkit
Slider entities for track, focus, stepping, and accessible semantics. Slider
change proposals call `Laika::set_value`; lifecycle events seed one undo
transaction, commit it once at end, and restore the starting scalar on cancel.

## Remaining migration blockers

The mkit Slider does not format host-specific values, so Laika retains its
typed editor and formatted readout beside the slider entity. The slider's
scalar value is synchronized from Laika state after changes; verify multi-photo
selection transitions in an interactive run. Public API naming, spec/event
contract, visual behavior, and book accuracy remain maintainer review gates.

## Checks performed

- `cargo check -p laika-app` — passed on the isolated worktree after adapter
  changes, with existing app warnings.
- `cargo build -p laika-app` — passed on the isolated worktree after adapter
  changes; emitted existing unused/deprecation warnings.
- `cargo clippy -p laika-app --no-deps` — passed in the prior adoption check
  with existing warnings. The
  committed Laika baseline emits 179 Clippy warnings across the app; no
  warning-clean claim is made.
- `cargo clippy -p laika-app --no-deps -- -D warnings` — failed on existing
  warnings throughout the committed Laika app (179 diagnostics), not a clean
  migration gate.
- `cargo test -p laika-app` — passed, 36 tests; this exercises Laika's existing
  unit tests but not an interactive GPUI harness.
- `cargo test -p mkit-registry-slider` — passed, 7 tests, including lifecycle
  sequence, pointer end, and Escape cancellation harness coverage.
- `cargo clippy -p mkit-registry-slider -p mkit-registry-segmented_control --
  -D warnings` — passed.
- `cargo check -p mkit --no-default-features --features
  slider,segmented-control,histogram` — passed.
- Slider and segmented-control conformance manifests checked against their
  specs. The global checker was green before a concurrent property-inspector
  edit; its latest run reports that unrelated component's manifest is stale.
- No Laika interactive GPUI harness, keyboard script, accessibility snapshot,
  or screenshot matrix was run. The mkit Slider GPUI harness verifies keyboard
  and pointer event order plus controlled/uncontrolled Escape cancellation;
  Laika multi-photo state transitions and visual integration still need a
  manual app check.
