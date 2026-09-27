# Accessibility across the window

## What you'll build

A custom status element with a stable ID, AccessKit `Status` role, name, and description. Run `cargo test -p mkit-example-platform-shipping --lib --locked` to check those properties. The status is noninteractive; its native screen-reader announcement has not been tested.

![Platform example with a green status marker and Ready to ship text](../images/platform-shipping.png)

## Concept

AccessKit is the accessibility tree layer used by pinned GPUI. GPUI builds a tree update during each frame and sends it to a platform adapter. A node needs a stable `GlobalElementId`; otherwise it cannot enter the tree. An interactive element can set a role and label with `StatefulInteractiveElement`. A custom `Element` can write its own node information. See the [pinned a11y design](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/a11y.rs#L35-L95).

## Minimal compiling example

```rust
{{#include ../../../examples/platform_shipping/src/lib.rs:accessible_custom_element}}
```

Run `cargo test -p mkit-example-platform-shipping --locked`. `custom_accessibility_semantics_have_stable_id_and_action` checks the element ID and direct AccessKit node role, name, and description. It inspects `Window::debug_a11y_tree_json` only if an active tree is available. The macOS screenshot test checks the visible marker, not the OS accessibility adapter.

## How it works

`ShippingStatus` returns a stable GPUI element ID and `Role::Status`. `Element::write_a11y_info` sets its label and description. GPUI can derive a stable node ID and include it during prepaint when an accessibility adapter is active. The local test also confirms the node has no Click action. The [pinned custom element hook](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L120-L126) and [a11y tree design](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/a11y.rs#L35-L95) explain the route. This headless test did not activate a native adapter.

An active-adapter check needs a running desktop app and the OS accessibility inspector or screen reader. Activate the adapter, inspect the status node's role, name, and description, and confirm they remain associated with the same element after redraw. Check whether a status change is announced; a `Status` role and static label alone do not establish an announcement. Verify that the noninteractive marker is not exposed as a clickable control. Repeat with the supported assistive technology on each platform being claimed. The optional headless JSON in this example cannot substitute for this check.

## Updating a live status region

`mkit_core::a11y::AccessibilityExt` provides role, name, value, state, and live-region helpers for GPUI interactive elements. The state example's `StatusAnnouncer` keeps one element ID, applies `Role::Status` and a polite live region, and changes its accessible name and value when the message changes. Its [initial](../images/e4-status-initial.png) and [updated](../images/e4-status-updated.png) screenshots verify the visible change. The headless test verifies the entity update and the AccessKit live property, but its accessibility adapter is inactive; a native screen reader must verify that the update is spoken. The pinned AT-SPI adapter has not been verified to announce an existing node's changed value, so this example does not establish Linux announcement behavior.

## Exercises

Change the status label and assert the direct AccessKit node changes without changing its element ID. Then inspect the active tree and announcement with a real screen reader on each claimed OS. Keep the status noninteractive unless its role and keyboard contract change. The visible “Ready to ship” text is sample content, not a release-readiness result.

## API reference links

- [`Element::write_a11y_info`, `Window::debug_a11y_tree_json`, `Role`](../appendices/api-inventory.md) · [pinned a11y source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/a11y.rs#L35-L95)
