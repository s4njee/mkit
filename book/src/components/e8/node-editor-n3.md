# Node editor: connection editing

N3 adds connection creation and removal requests to the [graph surface](node-editor-n1.md) and [movement and box selection](node-editor-n2.md). The app remains the authority for graph data: a user gesture emits a typed request, and accepted changes appear after the app supplies updated `GraphData`.

The preview uses the same compiling graph example as N1, with visible connected endpoints and their accessible connection summaries.

```rust
{{#include ../../../../examples/e8_components/src/lib.rs:node_editor_n3_preview}}
```

## Create a connection

Drag from an output port to an input port. For keyboard use, focus an output port with Enter and the arrow actions, press `c` to choose it as the source, use `[` and `]` to move among compatible input candidates, then press `Ctrl+Enter`. `Escape` cancels the source preview. Both routes emit `ConnectionCreateRequested { from_port_id, to_port_id }`. The component rejects missing ports, wrong direction, mismatched declared types, same-node endpoints, and repeated endpoint pairs. Graph cycles and application-specific validation remain with the host.

The focused source or candidate port is named with its direction and declared type. Current endpoint names appear in the Graph connections list. If the host rejects a request, no graph state was changed locally; it can report rejection in surrounding application UI.

## Select and remove a connection

Connection summaries are keyboard-focusable controls. Press `g` or `Shift+g` to focus the next or previous connection, then `Delete` to emit `ConnectionRemoveRequested { connection_id }`. Clicking a summary focuses it as well. Connection focus is independent of the selected node set. `ConnectionFocusChanged` reports keyboard focus changes.

The manifest-driven screenshot matrix contains 13 states × 3 themes × 2 scales. It includes connected, selected, port-navigation, keyboard-connection-preview, selected-connection, invalid-target, and real pointer drag-preview states. The invalid-target fixture asserts the typed rejection status before capture, and the image shows the resulting status text. Native screen-reader announcements, pointer capture outside the editor, and maintainer review of public event names, key bindings, and accessibility details remain open.
