//! Stateful text editor using GPUI's native text input and IME handler.

extern crate gpui_pre as gpui;

use gpui_pre::{
    Bounds, ClipboardItem, Context, DispatchPhase, ElementInputHandler, EntityInputHandler,
    EventEmitter, FocusHandle, Focusable, KeyBinding, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Render, ScrollHandle, UTF16Selection, Window, actions,
    canvas, div, point, prelude::*, px, size,
};
use std::ops::Range;

const FIELD_TEXT_INSET: f32 = 12.;
pub const KEY_CONTEXT: &str = "TextArea";
actions!(text_area, [SelectAll, Undo, Redo, Copy, Cut, Paste]);

fn caret_belongs_to_visual_row(caret: usize, start: usize, end: usize, is_last_row: bool) -> bool {
    caret >= start && (caret < end || (is_last_row && caret == end))
}

fn visual_row_boundaries(line: &gpui_pre::WrappedLine, line_height: Pixels) -> Vec<usize> {
    let mut boundaries = vec![0];
    let mut previous_y = None;
    for (byte, _) in line.text.char_indices() {
        let y =
            line.position_for_index(byte, line_height).map(|position| position.y).unwrap_or(px(0.));
        if previous_y.is_some_and(|previous| y > previous) {
            boundaries.push(byte);
        }
        previous_y = Some(y);
    }
    boundaries.push(line.text.len());
    boundaries
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputChanged(pub String);
impl EventEmitter<InputChanged> for TextArea {}

pub fn default_key_bindings() -> [KeyBinding; 12] {
    [
        KeyBinding::new("cmd-a", SelectAll, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-a", SelectAll, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-z", Undo, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-z", Undo, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-shift-z", Redo, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-shift-z", Redo, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-c", Copy, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-c", Copy, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-x", Cut, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-x", Cut, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-v", Paste, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-v", Paste, Some(KEY_CONTEXT)),
    ]
}

// ANCHOR: text_field_handler
/// Minimal state for a multi-line field. Selection and marked ranges are UTF-8 byte offsets
/// internally; the platform-facing handler converts UTF-16 code-unit ranges at its boundary.
pub struct TextArea {
    text: String,
    controlled: bool,
    label: String,
    placeholder: Option<String>,
    description: Option<String>,
    validation_message: Option<String>,
    disabled: bool,
    selection: Range<usize>,
    reversed: bool,
    marked: Option<Range<usize>>,
    undo: Vec<String>,
    redo: Vec<String>,
    coalesce_typing: bool,
    focus: Option<FocusHandle>,
    last_bounds: Option<Bounds<Pixels>>,
    scroll: ScrollHandle,
    drag_anchor: Option<usize>,
}

impl TextArea {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            text: String::new(),
            controlled: false,
            label: String::new(),
            placeholder: None,
            description: None,
            validation_message: None,
            disabled: false,
            selection: 0..0,
            reversed: false,
            marked: None,
            undo: Vec::new(),
            redo: Vec::new(),
            coalesce_typing: false,
            focus: Some(cx.focus_handle()),
            last_bounds: None,
            scroll: ScrollHandle::new(),
            drag_anchor: None,
        }
    }

    /// Construct an unfocused preview view for screenshot capture.
    pub fn new_for_demo() -> Self {
        Self {
            text: String::new(),
            controlled: false,
            label: String::new(),
            placeholder: None,
            description: None,
            validation_message: None,
            disabled: false,
            selection: 0..0,
            reversed: false,
            marked: None,
            undo: Vec::new(),
            redo: Vec::new(),
            coalesce_typing: false,
            focus: None,
            last_bounds: None,
            scroll: ScrollHandle::new(),
            drag_anchor: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_validation_message(mut self, message: impl Into<String>) -> Self {
        self.validation_message = Some(message.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn controlled(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self.controlled = true;
        self.selection = self.text.len()..self.text.len();
        self
    }
    pub fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        let value = value.into();
        if value == self.text {
            return;
        }
        self.text = value;
        self.selection = self.text.len()..self.text.len();
        self.marked = None;
        self.undo.clear();
        self.redo.clear();
        self.coalesce_typing = false;
        cx.notify();
    }
    pub fn is_controlled(&self) -> bool {
        self.controlled
    }
    pub fn set_validation_message(&mut self, message: Option<String>) {
        self.validation_message = message;
    }
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
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
        self.coalesce_typing = false;
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

    fn shaped_lines(&self, window: &Window, width: Pixels) -> Vec<(usize, gpui_pre::WrappedLine)> {
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let run = style.to_run(self.text.len());
        window
            .text_system()
            .shape_text(self.text.as_str().into(), font_size, &[run], Some(width), None)
            .unwrap_or_default()
            .into_iter()
            .scan(0usize, |offset, line| {
                let start = *offset;
                *offset += line.len();
                if *offset < self.text.len() {
                    *offset += 1; // GPUI omits the explicit newline from the shaped paragraph.
                }
                Some((start, line))
            })
            .collect()
    }

    fn position_for_byte(
        lines: &[(usize, gpui_pre::WrappedLine)],
        byte: usize,
        line_height: Pixels,
    ) -> gpui_pre::Point<Pixels> {
        let mut paragraph_y = px(0.);
        for (offset, line) in lines {
            let end = offset + line.len();
            if byte <= end {
                let mut position = line
                    .position_for_index(byte.saturating_sub(*offset), line_height)
                    .unwrap_or(point(px(0.), px(0.)));
                position.y += paragraph_y;
                return position;
            }
            paragraph_y += line.size(line_height).height;
        }
        let mut position = lines
            .last()
            .and_then(|(offset, line)| {
                line.position_for_index(line.len().min(byte.saturating_sub(*offset)), line_height)
            })
            .unwrap_or(point(px(0.), px(0.)));
        position.y += paragraph_y;
        position
    }

    fn scroll_caret_into_view(&self, window: &mut Window, cx: &Context<Self>) {
        let bounds = self.scroll.bounds();
        if bounds.size.width == px(0.) || bounds.size.height == px(0.) {
            return;
        }
        let theme = cx.global::<mkit_core::theme::Theme>();
        let inset = px(theme.spacing.small);
        let line_height = window.line_height();
        let lines =
            self.shaped_lines(window, (bounds.size.width - px(FIELD_TEXT_INSET * 2.)).max(px(1.)));
        let cursor = if self.reversed { self.selection.start } else { self.selection.end };
        let caret_top = Self::position_for_byte(&lines, cursor, line_height).y + inset;
        let caret_bottom = caret_top + line_height;
        let offset = self.scroll.offset();
        let visible_top = -f32::from(offset.y);
        let visible_bottom = visible_top + f32::from(bounds.size.height);
        let target_top = if f32::from(caret_top) < visible_top {
            f32::from(caret_top)
        } else if f32::from(caret_bottom) > visible_bottom {
            f32::from(caret_bottom) - f32::from(bounds.size.height)
        } else {
            return;
        };
        let max_offset = f32::from(self.scroll.max_offset().y);
        self.scroll.set_offset(point(offset.x, px(-target_top.clamp(0., max_offset))));
        window.refresh();
    }

    fn replace_bytes(&mut self, range: Range<usize>, inserted: &str) {
        let should_coalesce = self.coalesce_typing && range.is_empty() && !inserted.is_empty();
        if !should_coalesce {
            self.record_undo();
        }
        self.text.replace_range(range.clone(), inserted);
        let cursor = range.start + inserted.len();
        self.selection = cursor..cursor;
        self.marked = None;
        self.coalesce_typing = range.is_empty() && !inserted.is_empty();
    }
}

impl EntityInputHandler for TextArea {
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
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let composing = self.marked.is_some();
        let bytes = range
            .as_ref()
            .map(|range| self.utf16_range_to_bytes(range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        if composing {
            self.text.replace_range(bytes.clone(), text);
            let cursor = bytes.start + text.len();
            self.selection = cursor..cursor;
            self.marked = None;
            self.coalesce_typing = false;
        } else {
            self.replace_bytes(bytes, text);
        }
        cx.emit(InputChanged(self.text.clone()));
        self.scroll_caret_into_view(window, cx);
        cx.notify();
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let bytes = range
            .as_ref()
            .map(|range| self.utf16_range_to_bytes(range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        let start = bytes.start;
        if self.marked.is_none() {
            self.record_undo();
        }
        self.text.replace_range(bytes, text);
        self.marked = (!text.is_empty()).then_some(start..start + text.len());
        self.selection = selected
            .as_ref()
            .map(|range| {
                start + utf16_offset_to_byte(text, range.start)
                    ..start + utf16_offset_to_byte(text, range.end)
            })
            .unwrap_or_else(|| start + text.len()..start + text.len());
        self.coalesce_typing = false;
        self.scroll_caret_into_view(window, cx);
        cx.emit(InputChanged(self.text.clone()));
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
        let line_height = window.line_height();
        let lines = self.shaped_lines(
            window,
            (element_bounds.size.width - px(FIELD_TEXT_INSET * 2.)).max(px(1.)),
        );
        let start = Self::position_for_byte(&lines, bytes.start, line_height);
        let end = Self::position_for_byte(&lines, bytes.end, line_height);
        let origin = point(
            element_bounds.left() + px(FIELD_TEXT_INSET) + start.x.min(end.x),
            element_bounds.top() + start.y.min(end.y),
        );
        Some(Bounds::new(
            origin,
            size((end.x - start.x).abs().max(px(1.)), (end.y - start.y).abs() + line_height),
        ))
    }
    // ANCHOR_END: text_field_ime

    fn character_index_for_point(
        &mut self,
        position: gpui_pre::Point<Pixels>,
        window: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        self.byte_index_for_point(position, window).map(|byte| self.byte_to_utf16(byte))
    }

    fn set_selected_text_range(
        &mut self,
        range: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selection = self.utf16_range_to_bytes(&range);
        self.reversed = false;
        self.marked = None;
        self.coalesce_typing = false;
        self.scroll_caret_into_view(window, cx);
        cx.notify();
    }

    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.text.encode_utf16().count())
    }

    fn accepts_text_input(&self, _: &mut Window, _: &mut Context<Self>) -> bool {
        !self.disabled
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

impl Focusable for TextArea {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("focus handle initialized during render")
    }
}

// ANCHOR: text_field_element
impl Render for TextArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        let show_caret = focus.is_focused(window) && !self.disabled;
        let input = cx.entity();
        let theme = *cx.global::<mkit_core::theme::Theme>();
        let colors = &theme.colors;
        let text = if self.text.is_empty() {
            self.placeholder.as_deref().unwrap_or("")
        } else {
            &self.text
        };
        let is_placeholder = self.text.is_empty();
        // Render one element row per shaped visual line. Keeping newline and wrap boundaries
        // outside the inline marker fragments prevents a suffix after a caret/selection/IME
        // range from carrying an offset into the following visual line.
        let content = if self.text.is_empty() {
            div()
                .w_full()
                .text_color(if is_placeholder { colors.text_muted } else { colors.text })
                .child(text.to_owned())
        } else {
            let bounds = self.last_bounds.unwrap_or_else(|| window.bounds());
            let text_width = (bounds.size.width - px(FIELD_TEXT_INSET * 2.)).max(px(1.));
            let shaped_lines = self.shaped_lines(window, text_width);
            let line_height = window.line_height();
            let marker = self
                .marked
                .clone()
                .or_else(|| (!self.selection.is_empty()).then(|| self.selection.clone()));
            let caret = (marker.is_none() && show_caret).then_some(if self.reversed {
                self.selection.start
            } else {
                self.selection.end
            });
            let mut rows = Vec::new();

            for (paragraph_start, shaped) in shaped_lines {
                // WrappedLine represents a hard-newline paragraph and exposes the exact
                // visual y coordinate for each byte boundary. Use those coordinates to
                // materialize soft-wrapped rows without reimplementing GPUI's line breaking.
                let visual_starts = visual_row_boundaries(&shaped, line_height);
                let row_count = visual_starts.len().saturating_sub(1);
                for (row_index, boundaries) in visual_starts.windows(2).enumerate() {
                    let start = paragraph_start + boundaries[0];
                    let end = paragraph_start + boundaries[1];
                    let mut row = div().flex().w_full().h(line_height).items_start();

                    if let Some(marker) = marker.as_ref() {
                        let marked_start = marker.start.max(start).min(end);
                        let marked_end = marker.end.max(start).min(end);
                        if marked_start < marked_end {
                            if start < marked_start {
                                row = row.child(self.text[start..marked_start].to_owned());
                            }
                            let marked_text = self.text[marked_start..marked_end].to_owned();
                            row = if self.marked.is_some() {
                                row.child(
                                    div()
                                        .border_b_2()
                                        .border_color(colors.accent)
                                        .child(marked_text),
                                )
                            } else {
                                row.child(
                                    div()
                                        .bg(colors.accent)
                                        .text_color(colors.accent_text)
                                        .child(marked_text),
                                )
                            };
                            if marked_end < end {
                                row = row.child(self.text[marked_end..end].to_owned());
                            }
                        } else {
                            row = row.child(self.text[start..end].to_owned());
                        }
                    } else if let Some(caret) = caret.filter(|caret| {
                        caret_belongs_to_visual_row(*caret, start, end, row_index + 1 == row_count)
                    }) {
                        row = row.child(self.text[start..caret].to_owned()).child(
                            div()
                                .w(px(theme.borders.strong))
                                .h(px(theme.typography.heading))
                                .flex_shrink_0()
                                .bg(colors.accent),
                        );
                        if caret < end {
                            row = row.child(self.text[caret..end].to_owned());
                        }
                    } else {
                        row = row.child(self.text[start..end].to_owned());
                    }

                    rows.push(row);
                }
            }

            div().flex().flex_col().w_full().items_start().children(rows)
        };
        div()
            .w_full()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(theme.spacing.xsmall))
            .text_color(colors.text)
            .child(
                div()
                    .id("text-area")
                    .debug_selector(|| "text-area".to_owned())
                    .w_full()
                    .h(px(theme.controls.large * 3.))
                    .px(px(theme.spacing.medium))
                    .py(px(theme.spacing.small))
                    .flex()
                    .items_start()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .border_1()
                    .border_color(if self.validation_message.is_some() {
                        colors.danger
                    } else if self.marked.is_some() || focus.is_focused(window) {
                        colors.focus
                    } else {
                        colors.border
                    })
                    .rounded(px(theme.radii.medium))
                    .when(self.disabled, |e| e.opacity(0.55))
                    .role(gpui_pre::accesskit::Role::MultilineTextInput)
                    .aria_label(self.label.clone())
                    .aria_value(self.text.clone())
                    .when_some(
                        self.validation_message.clone().or_else(|| self.description.clone()),
                        |e, msg| e.aria_description(msg),
                    )
                    .when(self.validation_message.is_some() || self.disabled, |e| {
                        let invalid = self.validation_message.is_some();
                        let disabled = self.disabled;
                        e.a11y_synthetic_children(move |builder| {
                            if invalid {
                                builder
                                    .parent_node()
                                    .set_invalid(gpui_pre::accesskit::Invalid::True);
                            }
                            if disabled {
                                builder.parent_node().set_disabled();
                            }
                        })
                    })
                    .key_context(KEY_CONTEXT)
                    .bg(colors.surface)
                    .when(!self.disabled, |e| e.track_focus(&focus))
                    .on_action(cx.listener(Self::select_all_action))
                    .on_action(cx.listener(Self::undo_action))
                    .on_action(cx.listener(Self::redo_action))
                    .on_action(cx.listener(Self::copy_action))
                    .on_action(cx.listener(Self::cut_action))
                    .on_action(cx.listener(Self::paste_action))
                    .on_key_down(cx.listener(Self::on_key_down))
                    .child(content)
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, (), window, cx| {
                                let down_target = input.clone();
                                window.on_mouse_event(
                                    move |event: &MouseDownEvent, phase, window, cx| {
                                        if phase == DispatchPhase::Capture
                                            && event.button == MouseButton::Left
                                            && bounds.contains(&event.position)
                                        {
                                            down_target.update(cx, |area, cx| {
                                                area.pointer_down(event, window, cx)
                                            });
                                        }
                                    },
                                );
                                let move_target = input.clone();
                                window.on_mouse_event(
                                    move |event: &MouseMoveEvent, phase, window, cx| {
                                        if phase == DispatchPhase::Capture && event.dragging() {
                                            move_target.update(cx, |area, cx| {
                                                area.pointer_move(event, window, cx)
                                            });
                                        }
                                    },
                                );
                                let up_target = input.clone();
                                window.on_mouse_event(
                                    move |event: &MouseUpEvent, phase, window, cx| {
                                        if phase == DispatchPhase::Capture
                                            && event.button == MouseButton::Left
                                        {
                                            up_target.update(cx, |area, cx| {
                                                area.pointer_up(event, window, cx)
                                            });
                                        }
                                    },
                                );
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
            .when_some(self.validation_message.clone(), |e, message| {
                e.child(div().text_color(colors.danger).child(message))
            })
    }
}
// ANCHOR_END: text_field_element

// ANCHOR: text_field_editing
impl TextArea {
    fn selected_text(&self) -> Option<&str> {
        (!self.disabled && !self.selection.is_empty()).then(|| &self.text[self.selection.clone()])
    }

    fn copy_action(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = self.selected_text() {
            cx.write_to_clipboard(ClipboardItem::new_string(text.to_owned()));
        }
    }

    fn cut_action(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        let Some(text) = self.selected_text().map(str::to_owned) else { return };
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        let range = self.selection.clone();
        self.replace_bytes(range, "");
        cx.emit(InputChanged(self.text.clone()));
        self.scroll_caret_into_view(window, cx);
        cx.notify();
    }

    fn paste_action(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else { return };
        let range = self.selection.clone();
        self.replace_bytes(range, &text);
        cx.emit(InputChanged(self.text.clone()));
        self.scroll_caret_into_view(window, cx);
        cx.notify();
    }

    fn byte_index_for_point(
        &self,
        position: gpui_pre::Point<Pixels>,
        window: &mut Window,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let line_height = window.line_height();
        let lines =
            self.shaped_lines(window, (bounds.size.width - px(FIELD_TEXT_INSET * 2.)).max(px(1.)));
        let offset = self.scroll.offset();
        let relative = point(
            (position.x - bounds.left() - px(FIELD_TEXT_INSET)).max(px(0.)),
            (position.y - bounds.top() - px(FIELD_TEXT_INSET) - offset.y).max(px(0.)),
        );
        let mut byte = self.text.len();
        let mut y = relative.y;
        for (line_ix, (offset, line)) in lines.iter().enumerate() {
            let height = line.size(line_height).height;
            if y < height || line_ix + 1 == lines.len() {
                let local = line
                    .closest_index_for_position(point(relative.x, y), line_height)
                    .unwrap_or(line.len());
                byte = offset + local.min(line.len());
                break;
            }
            y -= height;
        }
        let mut byte = byte.min(self.text.len());
        while !self.text.is_char_boundary(byte) {
            byte -= 1;
        }
        Some(byte)
    }

    fn pointer_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || event.button != MouseButton::Left {
            return;
        }
        let Some(byte) = self.byte_index_for_point(event.position, window) else { return };
        self.drag_anchor = Some(byte);
        self.selection = byte..byte;
        self.reversed = false;
        self.marked = None;
        self.coalesce_typing = false;
        self.scroll_caret_into_view(window, cx);
        cx.notify();
    }

    fn pointer_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || !event.dragging() {
            return;
        }
        let Some(anchor) = self.drag_anchor else { return };
        let Some(pointer) = self.byte_index_for_point(event.position, window) else { return };
        self.selection = anchor.min(pointer)..anchor.max(pointer);
        self.reversed = pointer < anchor;
        self.marked = None;
        self.coalesce_typing = false;
        self.scroll_caret_into_view(window, cx);
        cx.notify();
    }

    fn pointer_up(&mut self, event: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        if event.button == MouseButton::Left {
            self.drag_anchor = None;
        }
    }

    pub fn move_cursor_by_utf16(&mut self, delta: isize) {
        let cursor = if self.reversed { self.selection.start } else { self.selection.end };
        let current = self.byte_to_utf16(cursor) as isize;
        let next = (current + delta).clamp(0, self.text.encode_utf16().count() as isize) as usize;
        let byte = self.utf16_to_byte(next);
        self.selection = byte..byte;
        self.reversed = false;
        self.coalesce_typing = false;
    }

    pub fn select_all(&mut self) {
        self.selection = 0..self.text.len();
        self.reversed = false;
        self.coalesce_typing = false;
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else { return false };
        self.redo.push(self.text.clone());
        self.text = previous;
        self.selection = self.text.len()..self.text.len();
        self.marked = None;
        self.coalesce_typing = false;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else { return false };
        self.undo.push(self.text.clone());
        self.text = next;
        self.selection = self.text.len()..self.text.len();
        self.marked = None;
        self.coalesce_typing = false;
        true
    }

    fn select_all_action(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.select_all();
        cx.notify();
    }
    fn undo_action(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if self.undo() {
            cx.emit(InputChanged(self.text.clone()));
            cx.notify();
        }
    }
    fn redo_action(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if self.redo() {
            cx.emit(InputChanged(self.text.clone()));
            cx.notify();
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;
        match (key, modifiers.platform, modifiers.shift) {
            ("left", false, _) if !self.selection.is_empty() => {
                self.coalesce_typing = false;
                let byte = self.selection.start;
                self.selection = byte..byte;
                self.reversed = false;
            }
            ("right", false, _) if !self.selection.is_empty() => {
                self.coalesce_typing = false;
                let byte = self.selection.end;
                self.selection = byte..byte;
                self.reversed = false;
            }
            ("left", false, _) => self.move_cursor_by_utf16(-1),
            ("right", false, _) => self.move_cursor_by_utf16(1),
            _ => return,
        }
        self.scroll_caret_into_view(window, cx);
        cx.notify();
    }
}
// ANCHOR_END: text_field_editing

pub fn new_field(cx: &mut Context<TextArea>) -> TextArea {
    TextArea::new(cx)
}

#[cfg(test)]
mod conformance_unit_tests {
    use super::{caret_belongs_to_visual_row, utf16_offset_to_byte};
    #[test]
    fn utf16_offsets_preserve_surrogate_pair_boundaries() {
        let value = "A😀B";
        assert_eq!(utf16_offset_to_byte(value, 0), 0);
        assert_eq!(utf16_offset_to_byte(value, 1), 1);
        assert_eq!(utf16_offset_to_byte(value, 2), 5);
        assert_eq!(utf16_offset_to_byte(value, 3), 5);
        assert_eq!(utf16_offset_to_byte(value, 4), 6);
    }

    #[test]
    fn soft_wrap_boundary_caret_belongs_to_exactly_one_visual_row() {
        let boundary = 8;
        assert!(!caret_belongs_to_visual_row(boundary, 0, boundary, false));
        assert!(caret_belongs_to_visual_row(boundary, boundary, 15, true));
        assert!(caret_belongs_to_visual_row(15, boundary, 15, true));
    }
}

#[cfg(test)]
mod gpui_tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    fn controlled_sync_echoes_ime_and_resets_history_on_override(cx: &mut TestAppContext) {
        use std::{cell::RefCell, rc::Rc};
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let events = Rc::new(RefCell::new(Vec::new()));
        let (area, visual) = cx.add_window_view(|window, cx| {
            let area = TextArea::new(cx).controlled("A😀B");
            window.focus(area.focus.as_ref().unwrap(), cx);
            area
        });
        let event_log = events.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&area, move |_, event: &InputChanged, _| {
                event_log.borrow_mut().push(event.0.clone());
            })
        });
        area.update(visual, |area, _| area.set_fixture_state("A😀B", 1..3, None));
        visual.update(|window, cx| {
            area.update(cx, |area, cx| {
                area.replace_and_mark_text_in_range(Some(1..3), "かな", Some(1..2), window, cx);
                area.set_value(area.text().to_owned(), cx);
            });
        });
        assert_eq!(area.read_with(visual, |area, _| area.marked_utf16()), Some(1..3));
        assert_eq!(*events.borrow(), vec!["AかなB"]);
        area.update(visual, |area, cx| area.set_value("owner override", cx));
        area.update(visual, |area, _| assert!(!area.undo()));
        assert_eq!(area.read_with(visual, |area, _| area.text().to_owned()), "owner override");
    }

    #[gpui::test]
    fn clipboard_actions_copy_cut_and_paste_with_utf16_selection(cx: &mut TestAppContext) {
        use std::{cell::RefCell, rc::Rc};
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let events = Rc::new(RefCell::new(Vec::new()));
        let (area, visual) = cx.add_window_view(|window, cx| {
            let mut area = TextArea::new(cx).controlled("A😀B");
            area.set_fixture_state("A😀B", 1..3, None);
            window.focus(area.focus.as_ref().unwrap(), cx);
            area
        });
        let event_log = events.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&area, move |_, event: &InputChanged, _| {
                event_log.borrow_mut().push(event.0.clone());
            })
        });

        visual.simulate_keystrokes("cmd-c");
        assert_eq!(visual.read_from_clipboard().and_then(|item| item.text()), Some("😀".into()));
        assert_eq!(area.read_with(visual, |area, _| area.text().to_owned()), "A😀B");
        assert!(events.borrow().is_empty());

        visual.simulate_keystrokes("cmd-x");
        assert_eq!(area.read_with(visual, |area, _| area.text().to_owned()), "AB");
        assert_eq!(*events.borrow(), vec!["AB"]);
        visual.write_to_clipboard(ClipboardItem::new_string("🌱\nnext".into()));
        visual.simulate_keystrokes("cmd-v");
        assert_eq!(area.read_with(visual, |area, _| area.text().to_owned()), "A🌱\nnextB");
        assert_eq!(*events.borrow(), vec!["AB", "A🌱\nnextB"]);
    }

    #[gpui::test]
    fn pointer_drag_selects_across_soft_wrap_and_explicit_newline(cx: &mut TestAppContext) {
        cx.update(|app| mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT));
        let first_line = format!("A😀{}", "wrap this paragraph many times ".repeat(10));
        let text = format!("{first_line}\nsecond paragraph");
        let end_byte = first_line.len() + 1 + "second".len();
        let (area, visual) = cx.add_window_view(|window, cx| {
            let mut area = TextArea::new(cx);
            area.set_value(text.clone(), cx);
            window.focus(area.focus.as_ref().unwrap(), cx);
            area
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (start, end) = visual.update(|window, cx| {
            area.read_with(cx, |area, _| {
                let bounds = area.last_bounds.expect("input canvas should have laid out");
                let lines = area.shaped_lines(
                    window,
                    (bounds.size.width - px(FIELD_TEXT_INSET * 2.)).max(px(1.)),
                );
                let first_row = TextArea::position_for_byte(&lines, 1, window.line_height());
                let last_row = TextArea::position_for_byte(&lines, end_byte, window.line_height());
                assert!(last_row.y > first_row.y, "drag destination crosses visual rows");
                let origin = |position: gpui_pre::Point<Pixels>| {
                    point(
                        bounds.left() + px(FIELD_TEXT_INSET) + position.x,
                        bounds.top() + px(FIELD_TEXT_INSET) + position.y + area.scroll.offset().y,
                    )
                };
                (origin(first_row), origin(last_row))
            })
        });
        visual.simulate_mouse_down(start, MouseButton::Left, Default::default());
        visual.simulate_mouse_move(end, Some(MouseButton::Left), Default::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Default::default());
        let end_utf16 = text[..end_byte].encode_utf16().count();
        assert_eq!(area.read_with(visual, |area, _| area.selection_utf16().range), 1..end_utf16);
        visual.simulate_mouse_down(end, MouseButton::Left, Default::default());
        visual.simulate_mouse_move(start, Some(MouseButton::Left), Default::default());
        visual.simulate_mouse_up(start, MouseButton::Left, Default::default());
        let selection = area.read_with(visual, |area, _| area.selection_utf16());
        assert_eq!(selection.range, 1..end_utf16);
        assert!(selection.reversed);
    }

    #[gpui::test]
    fn disabled_editor_rejects_clipboard_actions(cx: &mut TestAppContext) {
        cx.update(|app| mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT));
        let (area, visual) = cx.add_window_view(|_, cx| {
            let mut area = TextArea::new(cx).disabled(true);
            area.set_fixture_state("A😀B", 1..3, None);
            area
        });
        visual.write_to_clipboard(ClipboardItem::new_string("keep".into()));
        visual.update(|window, cx| {
            area.update(cx, |area, cx| {
                area.copy_action(&Copy, window, cx);
                area.cut_action(&Cut, window, cx);
                area.paste_action(&Paste, window, cx);
            });
        });
        assert_eq!(area.read_with(visual, |area, _| area.text().to_owned()), "A😀B");
        assert_eq!(visual.read_from_clipboard().and_then(|item| item.text()), Some("keep".into()));
    }

    #[gpui::test]
    fn multiline_native_edit_preserves_newlines_and_utf16_selection(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (area, visual) = cx.add_window_view(|window, cx| {
            let area = TextArea::new(cx);
            window.focus(area.focus.as_ref().unwrap(), cx);
            area
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_input("A😀\nB");
        assert_eq!(area.read_with(visual, |area, _| area.text().to_owned()), "A😀\nB");
        area.update(visual, |area, _| area.set_fixture_state("A😀\nB", 1..3, None));
        visual.update(|window, cx| {
            area.update(cx, |area, cx| {
                area.replace_and_mark_text_in_range(Some(1..3), "か", Some(1..1), window, cx);
                area.replace_and_mark_text_in_range(None, "かな", Some(1..2), window, cx);
            });
        });
        assert_eq!(area.read_with(visual, |area, _| area.text().to_owned()), "Aかな\nB");
        assert_eq!(area.read_with(visual, |area, _| area.marked_utf16()), Some(1..3));
        visual.update(|window, cx| {
            area.update(cx, |area, cx| area.unmark_text(window, cx));
        });
        visual.simulate_keystrokes("cmd-z");
        assert_eq!(area.read_with(visual, |area, _| area.text().to_owned()), "A😀\nB");
        area.update(visual, |area, cx| area.set_value("", cx));
        visual.simulate_input("abc");
        visual.simulate_keystrokes("cmd-z");
        assert_eq!(area.read_with(visual, |area, _| area.text().to_owned()), "");
    }

    #[gpui::test]
    fn multiline_shaping_preserves_soft_wrap_and_maps_next_paragraph(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (area, visual) = cx.add_window_view(|_, cx| {
            let mut area = TextArea::new(cx);
            area.set_value(
                "a long paragraph designed to soft wrap across multiple visual rows\nsecond",
                cx,
            );
            area
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            let width = px(80.);
            let lines = area.read_with(cx, |area, _| area.shaped_lines(window, width));
            assert_eq!(lines.len(), 2);
            assert_eq!(lines[0].0, 0);
            let first_paragraph_len =
                "a long paragraph designed to soft wrap across multiple visual rows".len();
            assert_eq!(lines[1].0, first_paragraph_len + 1);
            let first =
                TextArea::position_for_byte(&lines, first_paragraph_len, window.line_height());
            let wrapped = TextArea::position_for_byte(&lines, 20, window.line_height());
            let second =
                TextArea::position_for_byte(&lines, first_paragraph_len + 1, window.line_height());
            assert!(wrapped.y > px(0.), "the long paragraph must exercise a soft wrap");
            assert!(second.y > first.y);
        });
    }

    #[gpui::test]
    fn trailing_newlines_shape_one_empty_row_per_final_paragraph(cx: &mut TestAppContext) {
        cx.update(|app| mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT));
        let (area, visual) = cx.add_window_view(|_, cx| {
            let mut area = TextArea::new(cx);
            area.set_value("first\n\n", cx);
            area
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            let lines = area.read_with(cx, |area, _| area.shaped_lines(window, px(300.)));
            assert_eq!(lines.len(), 3);
            assert_eq!(lines[0].1.text.as_ref(), "first");
            assert!(lines[1].1.text.is_empty());
            assert!(lines[2].1.text.is_empty());
            assert_eq!(
                visual_row_boundaries(&lines[1].1, window.line_height()).windows(2).count(),
                1
            );
            assert_eq!(
                visual_row_boundaries(&lines[2].1, window.line_height()).windows(2).count(),
                1
            );
        });
    }

    #[gpui::test]
    fn keyboard_navigation_keeps_caret_visible_in_long_content(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let text = (0..30).map(|line| format!("line {line}")).collect::<Vec<_>>().join("\n");
        let (area, visual) = cx.add_window_view(|window, cx| {
            let mut area = TextArea::new(cx);
            area.set_value(text, cx);
            window.focus(area.focus.as_ref().unwrap(), cx);
            area
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));

        let navigation = std::iter::repeat_n("right", 220).collect::<Vec<_>>().join(" ");
        visual.simulate_keystrokes(&navigation);
        visual.update(|window, cx| window.draw(cx).clear(cx));

        visual.update(|window, cx| {
            area.read_with(cx, |area, _| {
                let offset = area.scroll.offset();
                assert!(f32::from(offset.y) < 0., "expected the viewport to scroll: {offset:?}");
                let bounds = area.scroll.bounds();
                let lines = area.shaped_lines(
                    window,
                    (bounds.size.width - px(FIELD_TEXT_INSET * 2.)).max(px(1.)),
                );
                let caret =
                    TextArea::position_for_byte(&lines, area.selection.end, window.line_height()).y
                        + px(mkit_core::theme::SHADCN_LIGHT.spacing.small);
                let visible_top = -f32::from(offset.y);
                let visible_bottom = visible_top + f32::from(bounds.size.height);
                assert!(f32::from(caret) >= visible_top - 1.);
                assert!(f32::from(caret) + f32::from(window.line_height()) <= visible_bottom + 1.);
            });
        });
    }
}
