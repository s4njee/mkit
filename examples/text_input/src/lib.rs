//! A small single-line field built on GPUI's low-level input-handler contract.

use gpui_pre::{
    Bounds, Context, ElementInputHandler, EntityInputHandler, FocusHandle, Focusable, KeyDownEvent,
    Pixels, Render, UTF16Selection, Window, canvas, div, point, prelude::*, px, size,
};
use std::ops::Range;

const FIELD_TEXT_INSET: f32 = 12.;

// ANCHOR: text_field_handler
/// Minimal state for a single-line field. Selection and marked ranges are UTF-8 byte offsets
/// internally; the platform-facing handler converts UTF-16 code-unit ranges at its boundary.
pub struct TextField {
    text: String,
    selection: Range<usize>,
    reversed: bool,
    marked: Option<Range<usize>>,
    undo: Vec<String>,
    redo: Vec<String>,
    focus: Option<FocusHandle>,
    last_bounds: Option<Bounds<Pixels>>,
}

impl TextField {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            text: String::new(),
            selection: 0..0,
            reversed: false,
            marked: None,
            undo: Vec::new(),
            redo: Vec::new(),
            focus: Some(cx.focus_handle()),
            last_bounds: None,
        }
    }

    /// Construct an unfocused preview view for screenshot capture.
    pub fn new_for_demo() -> Self {
        Self {
            text: String::new(),
            selection: 0..0,
            reversed: false,
            marked: None,
            undo: Vec::new(),
            redo: Vec::new(),
            focus: None,
            last_bounds: None,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn selection_utf16(&self) -> UTF16Selection {
        UTF16Selection { range: self.byte_range_to_utf16(&self.selection), reversed: self.reversed }
    }

    pub fn marked_utf16(&self) -> Option<Range<usize>> {
        self.marked.as_ref().map(|range| self.byte_range_to_utf16(range))
    }

    /// Focus handle used by the screenshot harness to capture the active field state.
    pub fn focus_handle_for_demo(&self) -> Option<FocusHandle> {
        self.focus.clone()
    }

    pub fn set_fixture_state(
        &mut self,
        text: &str,
        selection_utf16: Range<usize>,
        marked_utf16: Option<Range<usize>>,
    ) {
        self.text = text.to_owned();
        self.selection = self.utf16_range_to_bytes(&selection_utf16);
        self.marked = marked_utf16.map(|range| self.utf16_range_to_bytes(&range));
    }

    fn record_undo(&mut self) {
        self.undo.push(self.text.clone());
        self.redo.clear();
    }

    fn utf16_to_byte(&self, offset: usize) -> usize {
        utf16_offset_to_byte(&self.text, offset)
    }

    fn byte_to_utf16(&self, offset: usize) -> usize {
        self.text[..offset.min(self.text.len())].encode_utf16().count()
    }

    fn utf16_range_to_bytes(&self, range: &Range<usize>) -> Range<usize> {
        self.utf16_to_byte(range.start)..self.utf16_to_byte(range.end)
    }

    fn byte_range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.byte_to_utf16(range.start)..self.byte_to_utf16(range.end)
    }

    fn shaped_line(&self, window: &Window) -> gpui_pre::ShapedLine {
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let run = style.to_run(self.text.len());
        window.text_system().shape_line(self.text.as_str().into(), font_size, &[run], None)
    }

    fn replace_bytes(&mut self, range: Range<usize>, inserted: &str) {
        self.record_undo();
        self.text.replace_range(range.clone(), inserted);
        let cursor = range.start + inserted.len();
        self.selection = cursor..cursor;
        self.marked = None;
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let bytes = self.utf16_range_to_bytes(&range);
        *adjusted = Some(self.byte_range_to_utf16(&bytes));
        Some(self.text[bytes].to_owned())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(self.selection_utf16())
    }

    // ANCHOR: text_field_ime
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_utf16()
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.marked = None;
        cx.notify();
    }
    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let bytes = range
            .as_ref()
            .map(|range| self.utf16_range_to_bytes(range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        self.replace_bytes(bytes, &text.replace(['\n', '\r'], " "));
        cx.notify();
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let bytes = range
            .as_ref()
            .map(|range| self.utf16_range_to_bytes(range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        let start = bytes.start;
        self.record_undo();
        self.text.replace_range(bytes, text);
        self.marked = (!text.is_empty()).then_some(start..start + text.len());
        self.selection = selected
            .as_ref()
            .map(|range| {
                start + utf16_offset_to_byte(text, range.start)
                    ..start + utf16_offset_to_byte(text, range.end)
            })
            .unwrap_or_else(|| start + text.len()..start + text.len());
        cx.notify();
    }
    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        element_bounds: Bounds<Pixels>,
        window: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let bytes = self.utf16_range_to_bytes(&range);
        let shaped = self.shaped_line(window);
        let start = shaped.x_for_index(bytes.start);
        let end = shaped.x_for_index(bytes.end);
        let line_height = window.line_height();
        let vertical_inset = ((element_bounds.size.height - line_height) / 2.).max(px(0.));
        let origin = point(
            element_bounds.left() + px(FIELD_TEXT_INSET) + start,
            element_bounds.top() + vertical_inset,
        );
        Some(Bounds::new(origin, size((end - start).max(px(1.)), line_height)))
    }
    // ANCHOR_END: text_field_ime

    fn character_index_for_point(
        &mut self,
        point: gpui_pre::Point<Pixels>,
        window: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let shaped = self.shaped_line(window);
        let x = (point.x - bounds.left() - px(FIELD_TEXT_INSET)).max(px(0.));
        let mut byte = shaped.closest_index_for_x(x).min(self.text.len());
        while !self.text.is_char_boundary(byte) {
            byte -= 1;
        }
        Some(self.byte_to_utf16(byte))
    }

    fn set_selected_text_range(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selection = self.utf16_range_to_bytes(&range);
        self.marked = None;
        cx.notify();
    }

    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.text.encode_utf16().count())
    }
}

fn utf16_offset_to_byte(text: &str, offset: usize) -> usize {
    let mut units = 0;
    for (byte, ch) in text.char_indices() {
        if units >= offset {
            return byte;
        }
        units += ch.len_utf16();
        if units >= offset {
            return byte + ch.len_utf8();
        }
    }
    text.len()
}
// ANCHOR_END: text_field_handler

impl Focusable for TextField {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("focus handle initialized during render")
    }
}

// ANCHOR: text_field_element
impl Render for TextField {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        let input = cx.entity();
        let colors = &gpui_kit::component::ActiveTheme::theme(&**cx).colors;
        let text = if self.text.is_empty() { "Type a name…" } else { &self.text };
        let content = if let Some(marked) = self.marked.clone() {
            div()
                .flex()
                .items_center()
                .child(self.text[..marked.start].to_owned())
                .child(
                    div()
                        .border_b_2()
                        .border_color(colors.primary)
                        .child(self.text[marked.clone()].to_owned()),
                )
                .child(self.text[marked.end..].to_owned())
        } else if self.text.is_empty() {
            div().child(text.to_owned())
        } else {
            div()
                .flex()
                .items_center()
                .child(self.text[..self.selection.start].to_owned())
                .child(if self.selection.is_empty() {
                    div().w(px(2.)).h(px(20.)).flex_shrink_0().bg(colors.primary)
                } else {
                    div()
                        .bg(colors.primary)
                        .text_color(colors.primary_foreground)
                        .child(self.text[self.selection.clone()].to_owned())
                })
                .child(self.text[self.selection.end..].to_owned())
        };
        div()
            .size_full()
            .flex()
            .items_center()
            .p_6()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(
                div()
                    .id("text-field")
                    .debug_selector(|| "text-field".to_owned())
                    .w(px(420.))
                    .h(px(52.))
                    .px_3()
                    .flex()
                    .items_center()
                    .border_1()
                    .border_color(if self.marked.is_some() {
                        colors.primary
                    } else {
                        colors.border
                    })
                    .bg(colors.input)
                    .track_focus(&focus)
                    .on_key_down(cx.listener(Self::on_key_down))
                    .child(content)
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, (), window, cx| {
                                window.handle_input(
                                    &focus,
                                    ElementInputHandler::new(bounds, input.clone()),
                                    cx,
                                );
                                input.update(cx, |field, _| field.last_bounds = Some(bounds));
                            },
                        )
                        .absolute()
                        .inset_0(),
                    ),
            )
    }
}
// ANCHOR_END: text_field_element

// ANCHOR: text_field_editing
impl TextField {
    pub fn move_cursor_by_utf16(&mut self, delta: isize) {
        let cursor = if self.reversed { self.selection.start } else { self.selection.end };
        let current = self.byte_to_utf16(cursor) as isize;
        let next = (current + delta).clamp(0, self.text.encode_utf16().count() as isize) as usize;
        let byte = self.utf16_to_byte(next);
        self.selection = byte..byte;
        self.reversed = false;
    }

    pub fn select_all(&mut self) {
        self.selection = 0..self.text.len();
        self.reversed = false;
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else { return false };
        self.redo.push(self.text.clone());
        self.text = previous;
        self.selection = self.text.len()..self.text.len();
        self.marked = None;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else { return false };
        self.undo.push(self.text.clone());
        self.text = next;
        self.selection = self.text.len()..self.text.len();
        self.marked = None;
        true
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;
        match (key, modifiers.platform, modifiers.shift) {
            ("left", false, _) if !self.selection.is_empty() => {
                let byte = self.selection.start;
                self.selection = byte..byte;
                self.reversed = false;
            }
            ("right", false, _) if !self.selection.is_empty() => {
                let byte = self.selection.end;
                self.selection = byte..byte;
                self.reversed = false;
            }
            ("left", false, _) => self.move_cursor_by_utf16(-1),
            ("right", false, _) => self.move_cursor_by_utf16(1),
            ("a", true, _) => self.select_all(),
            ("z", true, true) => {
                self.redo();
            }
            ("z", true, false) => {
                self.undo();
            }
            _ => return,
        }
        cx.notify();
    }
}
// ANCHOR_END: text_field_editing

pub fn new_field(cx: &mut Context<TextField>) -> TextField {
    TextField::new(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::TestAppContext;

    #[gpui_pre::test]
    fn synthetic_input_uses_handler_and_utf16_boundaries(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (field, visual) = cx.add_window_view(|window, cx| {
            let field = TextField::new(cx);
            window.focus(field.focus.as_ref().unwrap(), cx);
            field
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_input("A😀B");
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "A😀B");
        field.update(visual, |field, _| field.select_all());
        assert_eq!(field.read_with(visual, |field, _| field.selection_utf16().range), 0..4);
        field.update(visual, |field, _| field.move_cursor_by_utf16(-1));
        assert_eq!(field.read_with(visual, |field, _| field.selection_utf16().range), 3..3);
        visual.simulate_keystrokes("cmd-a left right");
        assert_eq!(field.read_with(visual, |field, _| field.selection_utf16().range), 1..1);

        field.update(visual, |field, _| field.set_fixture_state("A😀B", 0..0, None));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let bounds = visual.debug_bounds("text-field").expect("field is rendered");
        let index = visual.update(|window, cx| {
            field.update(cx, |field, cx| {
                let x_after_emoji = field.shaped_line(window).x_for_index("A😀".len());
                field.character_index_for_point(
                    point(bounds.left() + px(FIELD_TEXT_INSET) + x_after_emoji, bounds.top()),
                    window,
                    cx,
                )
            })
        });
        assert_eq!(index, Some(3), "the point is after the surrogate-pair character");
    }

    #[gpui_pre::test]
    fn replacement_and_undo_restore_previous_value(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (field, visual) = cx.add_window_view(|window, cx| {
            let field = TextField::new(cx);
            window.focus(field.focus.as_ref().unwrap(), cx);
            field
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_input("abname");
        field.update(visual, |field, _| field.set_fixture_state("abname", 2..6, None));
        // The public test API is intentionally direct: native IME dispatch is not exposed by
        // TestWindow in this pinned GPUI release.
        visual.update(|window, cx| {
            field.update(cx, |field, cx| {
                field.replace_and_mark_text_in_range(Some(2..6), "かな", Some(1..2), window, cx);
            });
        });
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "abかな");
        assert_eq!(field.read_with(visual, |field, _| field.marked_utf16()), Some(2..4));
        assert_eq!(field.read_with(visual, |field, _| field.selection_utf16().range), 3..4);
        let element_bounds = Bounds::new(point(px(30.), px(40.)), size(px(300.), px(32.)));
        let (candidate_bounds, expected_x, expected_width) = visual.update(|window, cx| {
            field.update(cx, |field, cx| {
                let shaped = field.shaped_line(window);
                let bytes = field.utf16_range_to_bytes(&(2..4));
                let expected_x =
                    element_bounds.left() + px(FIELD_TEXT_INSET) + shaped.x_for_index(bytes.start);
                let expected_width =
                    shaped.x_for_index(bytes.end) - shaped.x_for_index(bytes.start);
                (
                    field.bounds_for_range(2..4, element_bounds, window, cx),
                    expected_x,
                    expected_width,
                )
            })
        });
        let candidate_bounds = candidate_bounds.expect("marked text has a candidate rectangle");
        assert_eq!(candidate_bounds.origin.x, expected_x);
        assert_eq!(candidate_bounds.size.width, expected_width);
        visual.update(|window, cx| {
            field.update(cx, |field, cx| field.unmark_text(window, cx));
        });
        assert_eq!(field.read_with(visual, |field, _| field.marked_utf16()), None);
        visual.simulate_keystrokes("cmd-z");
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "abname");
        visual.simulate_keystrokes("cmd-shift-z");
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "abかな");
    }

    #[gpui_pre::test]
    fn test_window_candidate_bounds_preserve_utf16_ranges_and_field_origin(
        cx: &mut TestAppContext,
    ) {
        cx.update(gpui_kit::init);
        let (field, visual) = cx.add_window_view(|window, cx| {
            let field = TextField::new(cx);
            window.focus(field.focus.as_ref().unwrap(), cx);
            field
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let element_bounds = Bounds::new(point(px(37.), px(83.)), size(px(260.), px(48.)));

        let mut measure = |text: &str, range: Range<usize>| {
            visual.update(|window, cx| {
                field.update(cx, |field, cx| {
                    field.set_fixture_state(text, range.clone(), None);
                    let shaped = field.shaped_line(window);
                    let bytes = field.utf16_range_to_bytes(&range);
                    if text == "iW😀X" {
                        assert_eq!(bytes, 2..6, "UTF-16 range 2..4 covers the full surrogate pair");
                    }
                    let start = shaped.x_for_index(bytes.start);
                    let end = shaped.x_for_index(bytes.end);
                    let measured = field
                        .bounds_for_range(range, element_bounds, window, cx)
                        .expect("range geometry is available");
                    (shaped.width(), start, end - start, measured)
                })
            })
        };

        let (narrow_line, narrow_start, narrow_width, narrow_bounds) = measure("iiii", 0..4);
        let (wide_line, wide_start, wide_width, wide_bounds) = measure("WWWW", 0..4);
        // Pinned TestWindow uses NoopTextSystem, which assigns one fixed advance to ordinary
        // characters. It can verify range math and origin preservation, but not font shaping.
        assert_eq!(wide_line, narrow_line);
        assert_eq!(wide_width, narrow_width);
        assert_eq!(
            narrow_bounds.origin.x,
            element_bounds.left() + px(FIELD_TEXT_INSET) + narrow_start
        );
        assert_eq!(narrow_bounds.size.width, narrow_width);
        assert_eq!(wide_bounds.origin.x, element_bounds.left() + px(FIELD_TEXT_INSET) + wide_start);
        assert_eq!(wide_bounds.size.width, wide_width);

        let (_, emoji_start, emoji_width, emoji_bounds) = measure("iW😀X", 2..4);
        assert!(emoji_width > px(0.));
        assert_eq!(
            emoji_bounds.origin.x,
            element_bounds.left() + px(FIELD_TEXT_INSET) + emoji_start
        );
        assert_eq!(emoji_bounds.size.width, emoji_width);
        assert!(
            emoji_bounds.origin.x > element_bounds.left(),
            "the field's nonzero origin must be retained"
        );
    }
}
