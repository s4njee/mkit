# E7.17 — SearchField

Specs: [`registry/search-field/spec.md`](../registry/search-field/spec.md).

`SearchField` is an entity-backed search input that delegates text editing to the existing
`mkit-registry-text_field` crate. That dependency is documented as a draft exception to the
mkit-core/GPUI-only registry convention and still needs maintainer approval. Reuse preserves the
shared input's selection, clipboard, Unicode, and IME behavior.

## API and events

`SearchField::new(label)` creates an uncontrolled field. `SearchField::controlled(label, query)`
creates a controlled field. Builders set placeholder, initial query, optional result count, debounce
duration, and disabled state. `set_query` applies external owner state without emitting an event.

`SearchChanged { query }` reports input edits and clear requests. Without debounce, text edits emit
immediately. With debounce, only the latest edit emits after the quiet interval. Clear-button and
Escape actions cancel a pending edit and emit the empty query immediately. Controlled mode keeps the
owner's query as its public value; edits are proposals, while the child editor may display the draft
until the owner calls `set_query` with the accepted or corrected value.

The exported `FocusSearch` action defaults to Cmd+F and is intended to be rebound by applications.
Escape clears a non-empty query. Enter and Space activate the clear button through a separate named
action context. Ordinary text editing, selection, clipboard, and IME handling remain with TextField.
The decorative icon reserves one `Theme.spacing.medium` leading inset with TextField's draft
`with_leading_inset` API. That inset is applied consistently to text layout, pointer hit testing,
and IME candidate bounds; the TextField API addition is pending maintainer approval.

## Accessibility and theme

The outer control is a named group. Its editor inherits TextField's named text-input semantics; the
clear affordance is a button shown only for an enabled, non-empty query, the decorative search icon
is hidden, and result count is polite status text. The shared TextField API does not yet expose a
searchbox role, so the current editor role is a text input. Disabled state is marked on the group and
editor. The component reads
colors, spacing, control sizes, radii, borders, and typography from the mkit-core Global `Theme`.

## Verification

Five focused GPUI tests cover Escape clearing, controlled clear proposals, actual native TextField
typing, clear-button Enter activation and focus restoration, and debounce coalescing. A gallery
matrix covers the four declared visual states across light, dark, and high-contrast themes at 1× and
2×. The 24-case matrix passed after baseline inspection and compare mode. Active-platform
accessibility snapshots and live status announcement behavior remain to be validated.

Maintainers should review the public API, the TextField dependency exception, the Cmd+F default,
result-count announcements, and the text-input versus searchbox role mapping.
