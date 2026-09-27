# Marked text and IME candidates

## What you'll build

A single-line field that marks composing text and supplies range geometry to GPUI's platform input path. Run `cargo run -p mkit-example-text-input --locked` to try the field with a native input method. The harness screenshot below shows a synthetic marked state on macOS; it does not show an OS candidate window.

![Text field with marked Japanese text and an underline](../images/text-field-marked.png)

## Concept

An *input method editor* (IME) turns key sequences into text candidates. During composition, the pending portion is *marked text*. The pinned [`InputHandler` contract](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1793-L1872) exposes `replace_and_mark_text_in_range`, `marked_text_range`, `unmark_text`, and `bounds_for_range`. Text ranges are UTF-16 offsets. The field shapes its full line with `Window::text_system()` and returns bounds for the requested range relative to the GPUI window. On macOS, [the native input client](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_macos/src/window.rs#L3297-L3332) converts those bounds to screen coordinates for AppKit's `firstRectForCharacterRange` callback.

## Minimal compiling example

```rust
{{#include ../../../examples/text_input/src/lib.rs:text_field_ime}}
```

Run `cargo test -p mkit-example-text-input --locked`. The synthetic handler tests check marked text, UTF-16 range conversion, and returned range geometry with the test window. On macOS, the headless checks compare four field screenshots and assert that shaped `iiii` and `WWWW` widths differ, a surrogate-pair range stays intact, and the field origin is retained. These tests call the handler directly or render fixed states; none launches an OS candidate window.

## How it works

`replace_and_mark_text_in_range` replaces the requested range, or the existing mark or selection when no range is supplied. It stores the inserted text as the new mark and interprets `selected` relative to that inserted text. `unmark_text` clears the mark without changing the text. A subsequent `replace_text_in_range` can commit a candidate by replacing the marked range; committing and unmarking are distinct handler calls.

`bounds_for_range` converts UTF-16 offsets to valid UTF-8 byte indices, shapes the full line, and measures the start and end x positions. It adds the painted field origin and text inset, then returns a window-relative rectangle with at least one pixel of width. [GPUI's candidate-position helper](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1647-L1698) requests a *collapsed* rectangle at the start of the marked text's current visual line; without a mark, it uses the selection endpoint. That is separate from an OS asking `bounds_for_range` for a particular character range. The test window uses `NoopTextSystem`, so its unit test checks range and origin math but cannot distinguish ordinary glyph advances. The macOS headless check uses the platform text system for narrow and wide glyphs.

## Verify a native input method

The automated tests do not prove that the OS sends composition callbacks or places its candidate window correctly. Use this procedure on every supported OS and input method:

1. Run `cargo run -p mkit-example-text-input --locked` in a desktop session. Record the OS version, input method and language, display scale, and whether the app window is on a secondary display.
2. Focus the field. Enter `A😀B`, move the caret between `😀` and `B`, then begin composing with a Japanese IME. Confirm the pending text is visibly marked, updates in place when you change the candidate, and does not duplicate the surrounding text.
3. While the candidate list is open, note its position relative to the beginning of the marked text and the field. Move the application window and repeat. Check that the list follows the field and stays on the correct display. Test near a screen edge, where the OS may reposition the list.
4. Commit one candidate and start another composition, then cancel or unmark it. Record the resulting text, caret position, whether the underline clears, and whether the candidate list closes. Repeat with Chinese and Korean input methods if they are part of the app's supported scope.
5. Save a screenshot or short recording that includes the actual OS candidate UI, and log any offset or stale-position symptom with the exact input method and display setup.

This is a manual **procedure**, not a completed test result. The example has no automated hook that drives a native candidate window, and native IME validation is still pending.

## Common mistakes

**Assuming a passing synthetic composition test proves native candidate behavior.** [Zed issue #62661](https://github.com/zed-industries/zed/issues/62661) reports composition failing in a later macOS nonactivating-panel setup, and [issue #46055](https://github.com/zed-industries/zed/issues/46055) reports candidate placement in a narrow terminal case. Neither establishes a bug in this pinned field. Test actual composition and candidate placement with the chosen native IME.

## Exercises

Update a marked range twice and commit it. Check that the final text appears once and the mark clears. Compare range rectangles for `iiii` and `WWWW`, then for a range spanning a surrogate pair. Run the native input method procedure above and record the actual candidate position.

## API reference links

- [`EntityInputHandler`, `InputHandler`, `UTF16Selection`, and `PlatformInputHandler`](../appendices/api-inventory.md) · [pinned input methods](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/input.rs#L32-L90)
