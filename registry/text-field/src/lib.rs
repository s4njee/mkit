//! Stateful text editor using GPUI's native text input and IME handler.

extern crate gpui_pre as gpui;

use gpui_pre::{
    Bounds, ClipboardItem, Context, ElementInputHandler, EntityInputHandler, EventEmitter,
    FocusHandle, Focusable, KeyBinding, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent,
    Pixels, Render, UTF16Selection, Window, actions, canvas, div, point, prelude::*, px, size,
};
use std::ops::Range;

pub const KEY_CONTEXT: &str = "TextField";
actions!(text_field, [SelectAll, Copy, Cut, Paste, Undo, Redo]);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputChanged(pub String);
impl EventEmitter<InputChanged> for TextField {}

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
/// Minimal state for a single-line field. Selection and marked ranges are UTF-8 byte offsets
/// internally; the platform-facing handler converts UTF-16 code-unit ranges at its boundary.
pub struct TextField {
    text: String,
    controlled: bool,
    label: String,
    placeholder: Option<String>,
    description: Option<String>,
    validation_message: Option<String>,
    disabled: bool,
    secure: bool,
    selection: Range<usize>,
    reversed: bool,
    marked: Option<Range<usize>>,
    undo: Vec<String>,
    redo: Vec<String>,
    focus: Option<FocusHandle>,
    last_bounds: Option<Bounds<Pixels>>,
    text_inset: Pixels,
    leading_inset: f32,
    drag_anchor: Option<usize>,
}

impl TextField {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            text: String::new(),
            controlled: false,
            label: String::new(),
            placeholder: None,
            description: None,
            validation_message: None,
            disabled: false,
            secure: false,
            selection: 0..0,
            reversed: false,
            marked: None,
            undo: Vec::new(),
            redo: Vec::new(),
            focus: Some(cx.focus_handle()),
            last_bounds: None,
            text_inset: px(0.),
            leading_inset: 0.,
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
            secure: false,
            selection: 0..0,
            reversed: false,
            marked: None,
            undo: Vec::new(),
            redo: Vec::new(),
            focus: None,
            last_bounds: None,
            text_inset: px(0.),
            leading_inset: 0.,
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

    /// Reserve additional logical pixels before the text for a leading adornment.
    /// Pass a theme spacing token so the adornment and editor remain aligned across themes.
    pub fn with_leading_inset(mut self, inset: f32) -> Self {
        self.leading_inset = if inset.is_finite() { inset.max(0.) } else { 0. };
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
    /// Mask the value on screen and in accessibility output, and disable copy and cut.
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
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
        let display = self.display_text(&self.text);
        let run = style.to_run(display.len());
        window.text_system().shape_line(display.into(), font_size, &[run], None)
    }

    fn display_text(&self, text: &str) -> String {
        if self.secure { "•".repeat(text.chars().count()) } else { text.to_owned() }
    }

    fn display_offset(&self, byte: usize) -> usize {
        if self.secure { self.text[..byte].chars().count() * "•".len() } else { byte }
    }

    fn replace_bytes(&mut self, range: Range<usize>, inserted: &str) {
        if self.marked.is_none() {
            self.record_undo();
        }
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
        if self.disabled {
            return;
        }
        let composing = self.marked.is_some();
        let bytes = range
            .as_ref()
            .map(|range| self.utf16_range_to_bytes(range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        let inserted = text.replace(['\n', '\r'], " ");
        if composing {
            self.text.replace_range(bytes.clone(), &inserted);
            let cursor = bytes.start + inserted.len();
            self.selection = cursor..cursor;
            self.marked = None;
        } else {
            self.replace_bytes(bytes, &inserted);
        }
        cx.emit(InputChanged(self.text.clone()));
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
        if self.disabled {
            return;
        }
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
        let shaped = self.shaped_line(window);
        let start = shaped.x_for_index(self.display_offset(bytes.start));
        let end = shaped.x_for_index(self.display_offset(bytes.end));
        let line_height = window.line_height();
        let vertical_inset = ((element_bounds.size.height - line_height) / 2.).max(px(0.));
        let origin = point(
            element_bounds.left() + self.text_inset + start,
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
        let x = (point.x - bounds.left() - self.text_inset).max(px(0.));
        let mut byte = if self.secure {
            let glyph = shaped.closest_index_for_x(x) / "•".len();
            self.text.char_indices().nth(glyph).map_or(self.text.len(), |(index, _)| index)
        } else {
            shaped.closest_index_for_x(x).min(self.text.len())
        };
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
        if self.disabled {
            return;
        }
        self.selection = self.utf16_range_to_bytes(&range);
        self.marked = None;
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

impl Focusable for TextField {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("focus handle initialized during render")
    }
}

// ANCHOR: text_field_element
impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus =
            self.focus.get_or_insert_with(|| cx.focus_handle()).clone().tab_stop(!self.disabled);
        let show_caret = focus.is_focused(window) && !self.disabled;
        let input = cx.entity();
        let theme = *cx.global::<mkit_core::theme::Theme>();
        self.text_inset = px(theme.spacing.medium + self.leading_inset);
        let colors = &theme.colors;
        let text = if self.text.is_empty() {
            self.placeholder.as_deref().unwrap_or("")
        } else {
            &self.text
        };
        let is_placeholder = self.text.is_empty();
        let content = if let Some(marked) = self.marked.clone() {
            div()
                .flex()
                .items_center()
                .child(self.display_text(&self.text[..marked.start]))
                .child(
                    div()
                        .border_b_2()
                        .border_color(colors.accent)
                        .child(self.display_text(&self.text[marked.clone()])),
                )
                .child(self.display_text(&self.text[marked.end..]))
        } else if self.text.is_empty() {
            div()
                .text_color(if is_placeholder { colors.text_muted } else { colors.text })
                .child(text.to_owned())
        } else {
            div()
                .flex()
                .items_center()
                .child(self.display_text(&self.text[..self.selection.start]))
                .child(if self.selection.is_empty() {
                    if show_caret {
                        div()
                            .w(px(theme.borders.strong))
                            .h(px(theme.typography.heading))
                            .flex_shrink_0()
                            .bg(colors.accent)
                    } else {
                        div()
                    }
                } else {
                    div()
                        .bg(colors.accent)
                        .text_color(colors.accent_text)
                        .child(self.display_text(&self.text[self.selection.clone()]))
                })
                .child(self.display_text(&self.text[self.selection.end..]))
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
                    .id("text-field")
                    .debug_selector(|| "text-field".to_owned())
                    .w_full()
                    .h(px(theme.controls.medium))
                    .pl(self.text_inset)
                    .pr(px(theme.spacing.medium))
                    .flex()
                    .items_center()
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
                    .role(if self.secure {
                        gpui_pre::accesskit::Role::PasswordInput
                    } else {
                        gpui_pre::accesskit::Role::TextInput
                    })
                    .aria_label(self.label.clone())
                    .aria_value(self.display_text(&self.text))
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
                    .on_action(cx.listener(Self::copy_action))
                    .on_action(cx.listener(Self::cut_action))
                    .on_action(cx.listener(Self::paste_action))
                    .on_action(cx.listener(Self::undo_action))
                    .on_action(cx.listener(Self::redo_action))
                    .on_key_down(cx.listener(Self::on_key_down))
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::pointer_down))
                    .on_mouse_move(cx.listener(Self::pointer_move))
                    .on_mouse_up(MouseButton::Left, cx.listener(Self::pointer_up))
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
            .when_some(self.validation_message.clone(), |e, message| {
                e.child(div().text_color(colors.danger).child(message))
            })
    }
}
// ANCHOR_END: text_field_element

// ANCHOR: text_field_editing
impl TextField {
    fn copy_action(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.secure || self.selection.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(
            self.text[self.selection.clone()].to_owned(),
        ));
    }

    fn cut_action(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.secure || self.marked.is_some() || self.selection.is_empty() {
            return;
        }
        let selected = self.text[self.selection.clone()].to_owned();
        cx.write_to_clipboard(ClipboardItem::new_string(selected));
        let range = self.selection.clone();
        self.replace_bytes(range, "");
        cx.emit(InputChanged(self.text.clone()));
        cx.notify();
    }

    fn paste_action(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.marked.is_some() {
            return;
        }
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        let text = text.replace(['\n', '\r'], " ");
        let range = self.selection.clone();
        self.replace_bytes(range, &text);
        cx.emit(InputChanged(self.text.clone()));
        cx.notify();
    }

    fn pointer_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let Some(bounds) = self.last_bounds else {
            return;
        };
        let shaped = self.shaped_line(window);
        let x = (event.position.x - bounds.left() - self.text_inset).max(px(0.));
        let mut byte = shaped.closest_index_for_x(x).min(self.text.len());
        while !self.text.is_char_boundary(byte) {
            byte -= 1;
        }
        self.drag_anchor = Some(byte);
        self.selection = byte..byte;
        self.reversed = false;
        self.marked = None;
        cx.notify();
    }

    fn pointer_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(anchor) = self.drag_anchor else {
            return;
        };
        if self.disabled || !event.dragging() {
            return;
        }
        let Some(bounds) = self.last_bounds else {
            return;
        };
        let shaped = self.shaped_line(window);
        let x = (event.position.x - bounds.left() - self.text_inset).max(px(0.));
        let mut byte = shaped.closest_index_for_x(x).min(self.text.len());
        while !self.text.is_char_boundary(byte) {
            byte -= 1;
        }
        self.selection = anchor.min(byte)..anchor.max(byte);
        self.reversed = byte < anchor;
        cx.notify();
    }

    fn pointer_up(&mut self, _: &gpui_pre::MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.drag_anchor = None;
    }
    pub fn move_cursor_by_utf16(&mut self, delta: isize) {
        if self.disabled {
            return;
        }
        let cursor = if self.reversed { self.selection.start } else { self.selection.end };
        let current = self.byte_to_utf16(cursor) as isize;
        let next = (current + delta).clamp(0, self.text.encode_utf16().count() as isize) as usize;
        let byte = self.utf16_to_byte(next);
        self.selection = byte..byte;
        self.reversed = false;
    }

    pub fn select_all(&mut self) {
        if self.disabled {
            return;
        }
        self.selection = 0..self.text.len();
        self.reversed = false;
    }

    pub fn undo(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        let Some(previous) = self.undo.pop() else { return false };
        self.redo.push(self.text.clone());
        self.text = previous;
        self.selection = self.text.len()..self.text.len();
        self.marked = None;
        true
    }

    pub fn redo(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        let Some(next) = self.redo.pop() else { return false };
        self.undo.push(self.text.clone());
        self.text = next;
        self.selection = self.text.len()..self.text.len();
        self.marked = None;
        true
    }

    fn select_all_action(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.select_all();
        cx.notify();
    }
    fn undo_action(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        if self.undo() {
            cx.emit(InputChanged(self.text.clone()));
            cx.notify();
        }
    }
    fn redo_action(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        if self.redo() {
            cx.emit(InputChanged(self.text.clone()));
            cx.notify();
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let key = event.keystroke.key.as_str();
        match (key, event.keystroke.modifiers.platform) {
            ("left", false) if !self.selection.is_empty() => {
                let byte = self.selection.start;
                self.selection = byte..byte;
                self.reversed = false;
            }
            ("right", false) if !self.selection.is_empty() => {
                let byte = self.selection.end;
                self.selection = byte..byte;
                self.reversed = false;
            }
            ("left", false) => self.move_cursor_by_utf16(-1),
            ("right", false) => self.move_cursor_by_utf16(1),
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
mod conformance_unit_tests {
    use super::{TextField, utf16_offset_to_byte};
    #[test]
    fn secure_display_masks_unicode_and_maps_cursor_offsets() {
        let mut field = TextField::new_for_demo().secure(true);
        field.set_fixture_state("A😀B", 1..3, None);
        assert_eq!(field.display_text(field.text()), "•••");
        assert_eq!(field.display_offset(1), 3);
        assert_eq!(field.display_offset(5), 6);
        assert_eq!(field.display_offset(6), 9);
    }
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
    fn disabled_field_ignores_selection_and_history_actions() {
        let mut field = TextField::new_for_demo();
        field.set_fixture_state("entry", 1..3, None);
        field.set_disabled(true);

        field.select_all();
        field.move_cursor_by_utf16(1);
        assert_eq!(field.selection_utf16().range, 1..3);
        assert!(!field.undo());
        assert!(!field.redo());
        assert_eq!(field.text(), "entry");
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
        let (field, visual) = cx.add_window_view(|window, cx| {
            let field = TextField::new(cx).controlled("A😀B");
            window.focus(field.focus.as_ref().unwrap(), cx);
            field
        });
        let event_log = events.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&field, move |_, event: &InputChanged, _| {
                event_log.borrow_mut().push(event.0.clone());
            })
        });
        field.update(visual, |field, _| field.set_fixture_state("A😀B", 1..3, None));
        visual.update(|window, cx| {
            field.update(cx, |field, cx| {
                field.replace_and_mark_text_in_range(Some(1..3), "かな", Some(1..2), window, cx);
                field.set_value(field.text().to_owned(), cx);
            });
        });
        assert_eq!(field.read_with(visual, |field, _| field.marked_utf16()), Some(1..3));
        assert_eq!(*events.borrow(), vec!["AかなB"]);
        field.update(visual, |field, cx| field.set_value("owner override", cx));
        field.update(visual, |field, _| assert!(!field.undo()));
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "owner override");
    }

    #[gpui::test]
    fn native_edit_and_ime_keep_utf16_selection_and_undo(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let (field, visual) = cx.add_window_view(|window, cx| {
            let field = TextField::new(cx);
            window.focus(field.focus.as_ref().unwrap(), cx);
            field
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_input("A😀B");
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "A😀B");
        assert_eq!(field.read_with(visual, |field, _| field.text().encode_utf16().count()), 4);
        field.update(visual, |field, _| field.set_fixture_state("A😀B", 1..3, None));
        visual.update(|window, cx| {
            field.update(cx, |field, cx| {
                field.replace_and_mark_text_in_range(Some(1..3), "かな", Some(1..2), window, cx);
            });
        });
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "AかなB");
        assert_eq!(field.read_with(visual, |field, _| field.marked_utf16()), Some(1..3));
        visual.update(|window, cx| {
            field.update(cx, |field, cx| field.unmark_text(window, cx));
        });
        visual.simulate_keystrokes("cmd-z");
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "A😀B");
    }

    #[gpui::test]
    fn disabled_field_rejects_native_input_callbacks(cx: &mut TestAppContext) {
        cx.update(|app| mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT));
        let (field, visual) = cx.add_window_view(|window, cx| {
            let field = TextField::new(cx);
            window.focus(field.focus.as_ref().unwrap(), cx);
            field
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        field.update(visual, |field, _| {
            field.set_fixture_state("safe", 4..4, None);
            field.set_disabled(true);
        });
        visual.update(|window, cx| {
            field.update(cx, |field, cx| {
                assert!(!field.accepts_text_input(window, cx));
                field.replace_text_in_range(Some(0..4), "changed", window, cx);
                field.replace_and_mark_text_in_range(Some(0..4), "marked", Some(1..2), window, cx);
                field.set_selected_text_range(0..0, window, cx);
            });
        });
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "safe");
        assert_eq!(field.read_with(visual, |field, _| field.selection_utf16().range), 4..4);
        assert_eq!(field.read_with(visual, |field, _| field.marked_utf16()), None);
    }

    #[gpui::test]
    fn clipboard_actions_copy_cut_paste_and_emit_once(cx: &mut TestAppContext) {
        use std::{cell::RefCell, rc::Rc};
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
        });
        let events = Rc::new(RefCell::new(Vec::new()));
        let (field, visual) = cx.add_window_view(|window, cx| {
            let field = TextField::new(cx).controlled("A😀B");
            window.focus(field.focus.as_ref().unwrap(), cx);
            field
        });
        let log = events.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&field, move |_, event: &InputChanged, _| {
                log.borrow_mut().push(event.0.clone())
            })
        });
        field.update(visual, |field, _| field.set_fixture_state("A😀B", 1..3, None));
        visual.simulate_keystrokes("cmd-c");
        assert_eq!(
            visual.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text())),
            Some("😀".to_owned())
        );
        visual.simulate_keystrokes("cmd-x");
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "AB");
        assert_eq!(*events.borrow(), vec!["AB"]);
        visual.update(|_, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string("paste\ntext".to_owned()))
        });
        visual.simulate_keystrokes("cmd-v");
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "Apaste textB");
        assert_eq!(*events.borrow(), vec!["AB", "Apaste textB"]);
        visual.update(|window, cx| {
            field.update(cx, |field, cx| {
                field.set_fixture_state("safe", 0..4, None);
                field.set_disabled(true);
                field.cut_action(&Cut, window, cx);
                field.paste_action(&Paste, window, cx);
            });
        });
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "safe");
        assert_eq!(*events.borrow(), vec!["AB", "Apaste textB"]);
    }

    #[gpui::test]
    fn secure_field_does_not_copy_or_cut_its_value(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys(default_key_bindings());
            app.write_to_clipboard(ClipboardItem::new_string("sentinel".to_owned()));
        });
        let (field, visual) = cx.add_window_view(|window, cx| {
            let field = TextField::new(cx).secure(true).controlled("sample-token");
            window.focus(field.focus.as_ref().unwrap(), cx);
            field
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        field.update(visual, |field, _| field.set_fixture_state("sample-token", 0..12, None));
        visual.simulate_keystrokes("cmd-c cmd-x");
        assert_eq!(field.read_with(visual, |field, _| field.text().to_owned()), "sample-token");
        assert_eq!(
            visual.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text())),
            Some("sentinel".to_owned())
        );
    }

    #[gpui::test]
    fn pointer_drag_selects_utf16_range_and_disabled_state_is_ignored(cx: &mut TestAppContext) {
        cx.update(|app| mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT));
        let (field, visual) = cx.add_window_view(|window, cx| {
            let field = TextField::new(cx).with_leading_inset(20.);
            window.focus(field.focus.as_ref().unwrap(), cx);
            field
        });
        field.update(visual, |field, _| field.set_fixture_state("A😀B", 0..0, None));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| {
            field.update(cx, |field, cx| {
                let bounds = field.last_bounds.expect("rendered input bounds");
                let shaped = field.shaped_line(window);
                let inset = field.text_inset;
                assert_eq!(inset, px(cx.global::<mkit_core::theme::Theme>().spacing.medium + 20.));
                let x_at = |byte| bounds.left() + inset + shaped.x_for_index(byte);
                let x1 = x_at(1);
                let x6 = x_at(6);
                let x0 = x_at(0);
                let down =
                    MouseDownEvent { position: point(x1, bounds.center().y), ..Default::default() };
                field.pointer_down(&down, window, cx);
                let mut moved = MouseMoveEvent {
                    position: point(x6, bounds.center().y),
                    pressed_button: Some(MouseButton::Left),
                    ..Default::default()
                };
                field.pointer_move(&moved, window, cx);
                assert_eq!(field.selection_utf16().range, 1..4);
                field.set_disabled(true);
                moved.position = point(x0, bounds.center().y);
                field.pointer_move(&moved, window, cx);
                assert_eq!(field.selection_utf16().range, 1..4);
            });
        });
    }
}
