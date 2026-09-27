# Drag and drop

## What you'll build

An in-app row reorder and a status area for file paths delivered by the operating system. Run `cargo run -p mkit-example-interaction --locked` from the workspace root.

![Interaction list with rows and a file drop area below them](../images/interaction.png)

## Concept

An in-app drag passes a typed payload from a source to a target. GPUI's `on_drag`, `on_drag_move`, and `on_drop` support that path. An *external drop* arrives from another application and may carry paths through `FileDropEvent` and `ExternalPaths`. `external_drag_payload` can supply a payload when a drag leaves a GPUI window. These are distinct paths with different platform behavior. See [pinned drag methods](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L570-L669) and [external payload types](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L694-L761).

## Minimal compiling example

This reorder method comes from the compiling `examples/interaction/src/lib.rs`. Its row and file drop handlers are in the same file's `interaction_list` region.

```rust
{{#include ../../../examples/interaction/src/lib.rs:interaction_drag_drop}}
```

Run `cargo test -p mkit-example-interaction --lib --locked` to check row reordering and a synthetic file-path drop. The screenshot above is checked by `cargo test -p mkit-example-interaction --test screenshot --locked` on macOS at 1280 × 1040 and scale 2.0. It shows the idle drop area, not a drag preview or dropped state.

## How it works

Each row's `on_drag` supplies its index; `on_drop` calls `reorder` for a valid target. The test drags row one onto row two and checks the new order. The file area has a typed `on_drop` for `ExternalPaths`; the test sends `FileDropEvent::Entered` and `Submit` at that area and checks the displayed path state. The fixture displays file names only, not the complete paths. It has a file-exit handler that clears state, but no handler that records paths on Entered or Pending, so it cannot show a live file-drag preview. That synthetic event does not establish that a native file manager can drop into the window. The [pinned window event translation](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L5375-L5425) turns Entered/Pending into pointer moves and Submit into a pointer up over the current drop target. Outbound OS dragging is not demonstrated by this fixture: its rows call `on_drag` but do not register `external_drag_payload`, which the [pinned contract](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L625-L666) requires for promoting a drag leaving the window.

## Native verification

On an unlocked desktop, start the example and drag an ordinary local file from the platform file manager into the **Drop files here** area. Release and check that the label changes to **OS drop received:** followed by the file name. Repeat by leaving the window before release; check that no submitted state appears. Release over a row or outside the drop area and record whether the target accepts or ignores it. Repeat with a directory and multiple files if those formats are part of the app you intend to support. Record the OS, file manager, source format, result, and any permission prompt. The fixture does not offer an outbound file payload, so dragging a row into the file manager is not an outbound file-transfer test. These native checks have not been run for this chapter.

## Common mistakes

**A browser URL is treated like a file path.** [Zed issue #52110](https://github.com/zed-industries/zed/issues/52110) reports an external drag limitation in standalone GPUI apps in a later revision. It does not prove the same behavior in this pin. Check the offered external payload format and test each claimed platform and source app; keep the file-path example scoped to paths it actually receives.

## Exercises

Drag the first row onto the second and check the displayed row order. Add a drop-outside-target assertion; the current test does not cover it. Use the native verification steps above before claiming file-manager integration.

## API reference links

- [`on_drag`, `on_drag_move`, `on_drop`, `external_drag_payload`, `FileDropEvent`, `ExternalPaths`](../appendices/api-inventory.md) · [pinned source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L694-L761)
