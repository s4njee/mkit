# E5 Accessibility Capture

## Current result

Headless capture of the real AccessKit tree now works on macOS without
patching GPUI. `mkit_harness::AccessibilitySession` opens a view in a
harness-owned platform window. That window keeps the activation callbacks GPUI
passes to `PlatformWindow::a11y_init`, activates them like a platform adapter
does, and records every `TreeUpdate` that GPUI sends to
`PlatformWindow::a11y_tree_update`. `AccessibilityTree::from_tree_update`
normalizes the recorded update into a deterministic snapshot. The snapshot
contains the role, the accessible name, and the states and values GPUI exposed.
It also contains relations and supported actions. The harness does not build
or infer any node.

`button`, `checkbox`, and `slider` use this path for their generated
`accessibility_cases`. Their per-state snapshots are in
`registry/<component>/tests/baselines/a11y/<state>.txt`. No other component
has an accessibility adapter yet.

`AccessibilitySnapshot::capture(&Window)`, which reads GPUI's debug JSON, is
unchanged. It now returns a tree for windows opened by `AccessibilitySession`,
because accessibility is active in those windows.

## Why the plain test window cannot capture

The workspace pins `gpui-pre` 0.3.5. GPUI builds an AccessKit tree only while
the window's accessibility flag is active. The flag is set by the activation
callback in `A11yCallbacks`, which `Window::new` passes to
`PlatformWindow::a11y_init` (`src/window.rs`, around lines 1591–1635). The
crate-private `TestWindow` behind `TestAppContext`, `TestApp`, and
`HeadlessAppContext` uses the default no-op `a11y_init`
(`src/platform.rs`, around line 1066). These contexts construct
`TestPlatform` internally. No public API reaches that callback. The regression
test `gpui_test_window_does_not_expose_a_captured_accessibility_tree` still
confirms the `Inactive` result for that path.

## Supported route used

`Platform`, `PlatformWindow`, `A11yCallbacks`, and `PlatformAtlas` are public
traits and types. `VisualTestAppContext::new(Rc<dyn Platform>)` is public under
`test-support` on macOS. It wraps any supplied platform in
`VisualTestPlatform`, which provides deterministic `TestDispatcher` executors,
the clipboard, and credentials.

`crates/mkit-harness/src/a11y_capture.rs` provides a minimal `CapturePlatform`
and `CaptureWindow`:

- Text shaping uses the real macOS text system from
  `gpui_platform::current_platform(true)`. Layout-dependent semantics, such as
  virtualized rows, therefore match the application.
- The window has no displays, cursor, or pixels. It uses fake atlas tiles and
  ignores `draw`. It stores `A11yCallbacks` in `a11y_init` and the last
  `TreeUpdate` in `a11y_tree_update`.
- The session calls GPUI's activation callback. It then runs the refresh task
  that GPUI scheduled and draws a frame. `update` and `dispatch_keystroke` draw
  again after each change. `deactivate` calls GPUI's deactivation callback.

GPUI's own frame lifecycle produces the recorded payload. It is the same
payload a platform adapter receives. For example, the synthetic disabled flag
set by `a11y_synthetic_children` appears in this payload, but GPUI's debug JSON
omits it.

Rejected alternatives:

- `bench_platform` from the `bench-support` feature returns a `TestPlatform`.
  It would still need a wrapper, and the feature adds `criterion` and the
  profiler to the shared GPUI build.
- Vendoring or patching `gpui-pre` would copy about 2.9 MB of source to change
  three test-only methods.
- Reconstructing nodes from rendered elements would test a second, fabricated
  tree.

## Snapshot format

The snapshot has one line per node, in preorder, indented by depth. Each line
has the AccessKit role and optional `#N` and `[focused]` markers. The markers
are followed by `name=` and then by fields present on the node, in a fixed
order:

1. Text fields
2. Flags such as `disabled`
3. `selected`, `expanded`, and enum states such as `toggled`
4. Numeric values and set or table positions
5. Relations, such as `labelled_by=[#1]`
6. `actions=[...]`

A relation refers to its target by the target's preorder index, not by the
GPUI `NodeId`. A node has a `#N` marker only when a relation refers to it. The
name is the label; if there is no label, it is the text of the
`labelled_by` targets. The normalizer returns an error for any of these update
problems:

- Duplicate nodes
- Missing nodes
- Unreachable nodes
- Cyclic or multiply parented nodes
- Dangling relation targets

The conformance adapter reports the first node below the window root.
`AccessibilityNode::aria_role` and `aria_properties` project that node onto ARIA
names for `run_conformance.py`. The keys include `name`, `description`, and
`value`. They also include `aria-checked` or `aria-pressed` from `toggled`,
`aria-valuenow`, `aria-valuemin`, `aria-valuemax`, `aria-disabled` and the
`disabled` alias, and the relation attributes.

## Conformance wiring

- `crates/mkit/tests/a11y_conformance.rs` (`harness = false`) reads one case,
  builds its fixture, captures the tree, and compares it with the state
  baseline. `MKIT_UPDATE_A11Y_BASELINES=1` writes a changed baseline. On
  platforms other than macOS, it reports `unsupported`, so those cases stay
  pending.
- The `button`, `checkbox`, and `slider` `tests/adapter.py` scripts send
  `accessibility_cases` to that test.
- `scripts/run_conformance.py` checks role and properties as before. When an
  adapter reports `snapshot_baseline`, the script also requires the component
  and state's baseline path and `snapshot_matched: true`.

## Remaining gaps

- Capture is macOS-only, because `VisualTestAppContext` is macOS-gated in GPUI
  0.3.5. The session must be created on the main thread because of the macOS
  text system, so test targets need `harness = false`. On Linux and Windows,
  capture needs `Application::with_platform` plus a public way to reach the
  `App`, or the upstream hook below.
- 63 components still need fixtures in `a11y_conformance.rs` and reviewed
  baselines. Baselines must not be mass-generated.
- This checks the tree GPUI hands to AccessKit. It does not check AccessKit's
  macOS or Windows adapter mapping or screen-reader output. Those still need
  the desktop AX check (`scripts/assert_macos_e7_ax.py`) or a manual
  VoiceOver or NVDA check.
- Recommended upstream change, which would make the harness platform
  unnecessary: a `test-support` hook on `TestWindow` or `TestAppContext` that
  retains `A11yCallbacks`, exposes activate and deactivate methods, and returns
  the last `TreeUpdate`.

## Checks

- `cargo test -p mkit-harness`: the unit tests cover `TreeUpdate`
  normalization, the ARIA projection, and rejection of malformed trees. The
  `a11y_capture` target checks capture, state changes, agreement with GPUI's
  debug JSON, and deactivation. The existing inactive-path and JSON tests also
  pass.
- `python3 scripts/run_conformance.py registry/<c>/tests/conformance.json --adapter "python3 registry/<c>/tests/adapter.py" --kind accessibility`
  passes 3/3 cases for `button`, 4/4 for `checkbox`, and 5/5 for `slider`. An
  edited baseline fails with `snapshot ... not matched`.
