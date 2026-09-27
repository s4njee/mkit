# E8.10 node editor — N3 and N4 implementation notes

N3 implements connection editing requests; N4 adds optional minimap navigation. The registry contract is in [the component spec](../registry/node-editor/spec.md), and the user-facing examples are [N3](../book/src/components/e8/node-editor-n3.md) and [N4](../book/src/components/e8/node-editor-n4.md).

## N3 connection editing

- Pointer users drag an output port onto an input port. Keyboard users focus an output, press `c`, cycle compatible candidates with `[` and `]`, and commit with `Ctrl+Enter`.
- The component rejects missing ports, wrong endpoint direction, mismatched declared types, same-node pairs, and duplicate endpoint pairs. Cycles and application-specific validation remain host-owned.
- `ConnectionCreateRequested` and `ConnectionRemoveRequested` are proposal-only; owners confirm graph updates through `set_graph`.
- Connection summaries are focusable and clickable. `g`/`Shift+g` navigate connections; `Delete` requests removal; `ConnectionFocusChanged` reports focus.
- Accessibility describes endpoints in a text list, including unresolved graph references, and port labels include direction and declared type.

## N4 minimap

- The minimap is hidden by default. Toolbar and `m` controls toggle it; visibility is uncontrolled by default, or request-only with `with_controlled_minimap` until the host applies `set_minimap_visible`.
- The overview maps graph bounds uniformly, shows finite nodes and the viewport, and clicking it requests a centered view. Negative coordinates use the same relative-bounds mapping.
- `Alt+Arrow` continues to pan. Reading-order arrow navigation plus `v` to center the focused node gives a keyboard route to distant nodes. The minimap has a name and a world-coordinate viewport description and does not trap focus.
- The overview's 168 by 104 logical-pixel size is a documented compact navigation target. Theme colors, border, focus, spacing, and typography use GPUI Global theme tokens.

## Evidence

The conformance manifest drives 78 screenshots: 13 states × light/dark/high-contrast × 1×/2×. It includes real pointer movement and box-selection previews, keyboard connection preview and selection, minimap visibility, and a rejected-target interaction. The rejected-target fixture checks the typed `connection_status()` error before capture, so a viewport pan cannot satisfy the fixture. All 78 fresh candidate captures were inspected at representative interaction states; the images were copied byte-for-byte to both the book and registry baselines.

Focused verification:

```text
cargo test -p mkit-registry-node-editor
cargo clippy -p mkit-registry-node-editor -- -D warnings
cargo check -p mkit
cargo check -p mkit-example-e8-components
E8_SNAPSHOT_ONLY=e8-node-editor-matrix cargo test -p mkit-example-e8-components --test screenshots
```

Native screen-reader traversal, pointer capture outside the editor, and trackpad behavior remain manual checks. Maintainer review is required for the public API, event names and timing, key bindings, accessibility contract, screenshot baselines, and book accuracy.
