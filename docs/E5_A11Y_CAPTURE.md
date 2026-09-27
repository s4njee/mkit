# E5 Accessibility Capture

## Current result

`mkit_harness::AccessibilitySnapshot::capture(&Window)` reads the serialized
AccessKit tree that GPUI retained for the last active accessibility frame. It
does not construct semantics from GPUI element state. It returns `Inactive`
when GPUI has not activated the accessibility adapter and `NoFrame` when the
adapter is active but has not produced a tree yet. The JSON serializer only
normalizes a tree returned by GPUI; malformed or inconsistent node references
are errors.

This capture path is usable in an application window when its platform
accessibility adapter has activated. It cannot currently capture a component
tree in the workspace's headless `TestAppContext`.

## Evidence from pinned GPUI

The workspace pins `gpui-pre` 0.3.5. Its public `Window` API exposes
`is_a11y_active()` and `debug_a11y_tree_json()` (`src/window.rs`, around lines
6674–6680). The debug method serializes the last `TreeUpdate` retained by the
internal `A11yDebug` object (`src/window/a11y/debug.rs`). It does not build a
tree on demand.

The `A11y` object creates its `active_flag` as false. During window creation,
GPUI passes activation and deactivation callbacks to
`PlatformWindow::a11y_init` (`src/window.rs`, around lines 1591–1635). The
activation callback flips the flag and schedules a fresh frame; that active
frame builds and retains the actual AccessKit `TreeUpdate`. The public
`A11yCallbacks` type is the platform adapter contract. The default
`PlatformWindow::a11y_init` implementation does nothing (`src/platform.rs`,
around line 1066).

The `TestWindow` used by `TestAppContext` is crate-private and does not
override that no-op `a11y_init` (`src/platform/test/window.rs`). The public
test context has no method to request accessibility activation. GPUI also
exposes `Application::new_inaccessible`, which explicitly disables
accessibility; it is not an activation mechanism. The harness integration test
`gpui_test_window_does_not_expose_a_captured_accessibility_tree` confirms that
the headless test window reports inactive, has no debug tree, and yields the
explicit `Inactive` result.

## API barrier and upstream proposal

There is no supported way for a downstream harness to flip a `TestWindow`
active or supply an activation callback. The active flag and `A11yDebug` are
private GPUI state, and `TestWindow` itself is private. Reconstructing
AccessKit nodes from the rendered GPUI tree would test a second, fabricated
tree and is not a valid substitute.

The smallest useful GPUI change is a test-support-only accessibility activation
hook on `TestAppContext` (or its test platform):

1. Retain the `A11yCallbacks` passed to `TestWindow::a11y_init`.
2. Expose a method that invokes the activation callback and returns control
   after the scheduled frame has run.
3. Expose a matching deactivation method so tests can cover inactive behavior.

This keeps activation and tree creation inside GPUI's existing callback and
frame lifecycle. It does not require a public production API or a second tree
serializer. Once available, the harness can add a real capture assertion for
role, accessible name, value, and child order while retaining its current
inactive and malformed-tree checks.

## Checks

`cargo test -p mkit-harness --test accessibility --locked` passes all four
tests. Those results cover normalization, invalid references, and the explicit
headless `Inactive` path. They do not constitute a captured component-tree
result; that remains blocked on the GPUI test-support hook described above.
