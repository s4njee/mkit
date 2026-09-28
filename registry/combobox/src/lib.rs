//! Draft editable combobox for the component registry.
//!
//! This pilot provides native GPUI text editing, option filtering, active-option
//! keyboard navigation, commit/cancel semantics, and a GPUI-rendered listbox.
//! Popup positioning/dismissal and the complete accessibility relationship
//! remain follow-up work; callers may also feed query changes through `set_query`.

extern crate gpui_pre as gpui;

use gpui_pre::{
    Bounds, Context, ElementInputHandler, EntityInputHandler, EventEmitter, FocusHandle, Focusable,
    IntoElement, KeyBinding, KeyDownEvent, PathBuilder, Pixels, Render, Rgba, ScrollStrategy,
    UTF16Selection, UniformListScrollHandle, Window, actions, canvas, div, point, prelude::*, px,
    size, uniform_list,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
use std::ops::Range;

/// Resolved input and popup colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    high_contrast: bool,
    background: Rgba,
    input_border: Rgba,
    text: Rgba,
    icon: Rgba,
    popup_bg: Rgba,
    popup_border: Rgba,
    active_bg: Rgba,
    active_text: Rgba,
    /// Pointer-hover fill for enabled rows; high contrast keeps rows unchanged.
    hover_bg: Option<Rgba>,
    focus: Rgba,
    ring: Rgba,
    disabled: Rgba,
}

/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}

fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            high_contrast: true,
            background: c.background,
            input_border: c.border,
            text: c.text,
            icon: c.text,
            popup_bg: c.background,
            popup_border: c.border,
            active_bg: c.accent,
            active_text: c.accent_text,
            hover_bg: None,
            focus: c.focus,
            ring: c.focus,
            disabled: c.disabled,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    Look {
        high_contrast: false,
        background: c.background,
        input_border: if dark { c.text.opacity(0.15) } else { c.border },
        text: c.text,
        icon: c.text_muted,
        popup_bg: c.surface,
        popup_border: if dark { c.text.opacity(0.1) } else { c.border },
        active_bg: muted,
        active_text: c.text,
        hover_bg: Some(muted),
        focus: c.focus,
        ring: c.focus.opacity(0.5),
        disabled: c.disabled,
    }
}

/// The web preview's `opacity: .5` applied as one layer: composite over `base`, then mix 50%.
fn dim(color: Rgba, base: Rgba) -> Rgba {
    mix(composite(color, base), base, 0.5)
}

fn box_shadow(shadow: ShadowToken, alpha: f32) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: Rgba { a: shadow.color.a * alpha, ..shadow.color }.into(),
        offset: point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}

/// shadcn/ui focus ring width, drawn outside the input.
const FOCUS_RING_WIDTH: f32 = 3.0;

fn focus_ring(color: Rgba) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: color.into(),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(FOCUS_RING_WIDTH),
        inset: false,
    }
}

/// Horizontal text inset inside the input (shadcn `px-3`), shared by rendering, IME bounds, and
/// pointer hit testing.
fn text_inset(theme: &Theme) -> Pixels {
    px(theme.spacing.medium)
}

/// Decorative Lucide `check` (20,6 → 9,17 → 4,12 on a 24-unit grid, 2-unit stroke) drawn as a
/// vector path so it stays crisp at every scale.
fn check_icon(size: f32, color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let origin = bounds.origin;
            let mut path = PathBuilder::stroke(unit * 2.0);
            path.move_to(origin + point(unit * 20.0, unit * 6.0));
            path.line_to(origin + point(unit * 9.0, unit * 17.0));
            path.line_to(origin + point(unit * 4.0, unit * 12.0));
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
}

/// Key context used by the combobox named actions.
pub const KEY_CONTEXT: &str = "Combobox";

actions!(
    combobox,
    [OpenNext, OpenPrevious, Commit, Cancel, Dismiss, DismissPrevious, OpenExplicit, CloseExplicit]
);

/// Default keyboard bindings. Host applications can replace these bindings.
pub fn default_key_bindings() -> [KeyBinding; 8] {
    [
        KeyBinding::new("down", OpenNext, Some(KEY_CONTEXT)),
        KeyBinding::new("up", OpenPrevious, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Commit, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", Cancel, Some(KEY_CONTEXT)),
        KeyBinding::new("tab", Dismiss, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-tab", DismissPrevious, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-down", OpenExplicit, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-up", CloseExplicit, Some(KEY_CONTEXT)),
    ]
}

/// An available single-selection option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionItem {
    pub id: String,
    pub label: String,
    pub disabled: bool,
}

impl OptionItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), disabled: false }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// User interaction event emitted when the popup's visibility changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenChanged(pub bool);

/// User-edited query text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputChanged(pub String);

/// User-requested committed value. A value is an option ID or custom text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueChanged(pub String);

/// User selected an option (custom values do not emit this event).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionSelected(pub String);

/// Editing session was cancelled; contains the session-start option ID, if any.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cancelled(pub Option<String>);

/// Popup was dismissed without committing an active option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dismissed;

impl EventEmitter<OpenChanged> for Combobox {}
impl EventEmitter<InputChanged> for Combobox {}
impl EventEmitter<ValueChanged> for Combobox {}
impl EventEmitter<OptionSelected> for Combobox {}
impl EventEmitter<Cancelled> for Combobox {}
impl EventEmitter<Dismissed> for Combobox {}

/// Stateful editable single-value combobox pilot.
pub struct Combobox {
    label: String,
    options: Vec<OptionItem>,
    committed: Option<String>,
    query: String,
    session_start: Option<Option<String>>,
    active: Option<usize>,
    open: bool,
    controlled: bool,
    disabled: bool,
    allow_custom_value: bool,
    focus: Option<FocusHandle>,
    selection: Range<usize>,
    marked: Option<Range<usize>>,
    last_bounds: Option<Bounds<Pixels>>,
    list_scroll: UniformListScrollHandle,
}

impl Combobox {
    /// Create an uncontrolled combobox. `default_value` is an option ID.
    pub fn new(
        label: impl Into<String>,
        options: Vec<OptionItem>,
        default_value: Option<String>,
    ) -> Self {
        let query = default_value
            .as_deref()
            .and_then(|id| options.iter().find(|option| option.id == id))
            .map_or_else(String::new, |option| option.label.clone());
        let cursor = query.len();
        Self {
            label: label.into(),
            options,
            committed: default_value,
            query,
            session_start: None,
            active: None,
            open: false,
            controlled: false,
            disabled: false,
            allow_custom_value: false,
            focus: None,
            selection: cursor..cursor,
            marked: None,
            last_bounds: None,
            list_scroll: UniformListScrollHandle::new(),
        }
    }

    /// Create a controlled combobox. User requests emit events; `set_value`
    /// applies the owner's authoritative value.
    pub fn controlled(
        label: impl Into<String>,
        options: Vec<OptionItem>,
        value: Option<String>,
    ) -> Self {
        let mut combobox = Self::new(label, options, value);
        combobox.controlled = true;
        combobox
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn allow_custom_value(mut self, allow: bool) -> Self {
        self.allow_custom_value = allow;
        self
    }

    pub fn value(&self) -> Option<&str> {
        self.committed.as_deref()
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn active_option(&self) -> Option<&str> {
        self.active
            .and_then(|index| self.filtered_indices().get(index).copied())
            .map(|index| self.options[index].id.as_str())
    }

    /// Apply a value prop from a controlled owner (or reset uncontrolled state).
    /// This operation intentionally emits no user interaction events.
    pub fn set_value(&mut self, value: Option<String>, cx: &mut Context<Self>) {
        self.committed = value;
        if !self.open {
            self.query = self.committed_label().unwrap_or_default();
            self.selection = self.query.len()..self.query.len();
            self.marked = None;
        }
        cx.notify();
    }

    /// Replace option props. Filtering and the active row are recomputed without
    /// emitting selection events.
    pub fn set_options(&mut self, options: Vec<OptionItem>, cx: &mut Context<Self>) {
        let active_id = self.active_option().map(str::to_owned);
        let had_active = active_id.is_some();
        self.options = options;
        let filtered = self.filtered_indices();
        self.active = active_id
            .and_then(|id| filtered.iter().position(|index| self.options[*index].id == id))
            .filter(|index| !self.options[filtered[*index]].disabled)
            .or_else(|| if had_active { first_enabled(&self.options, &filtered) } else { None });
        self.scroll_active();
        cx.notify();
    }

    /// Apply a user text edit directly. Native keyboard text entry uses GPUI's
    /// input-handler contract below; this is also useful for external editor state.
    pub fn set_query(&mut self, query: impl Into<String>, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let query = query.into();
        self.selection = query.len()..query.len();
        self.marked = None;
        self.update_query(query, cx);
    }

    fn update_query(&mut self, query: String, cx: &mut Context<Self>) {
        if self.session_start.is_none() {
            self.session_start = Some(self.committed.clone());
        }
        self.query = query.clone();
        self.active = None;
        self.list_scroll.scroll_to_item(0, ScrollStrategy::Top);
        self.set_open(true, cx);
        cx.emit(InputChanged(query));
        cx.notify();
    }

    fn scroll_active(&self) {
        if let Some(index) = self.active {
            self.list_scroll.scroll_to_item(index, ScrollStrategy::Nearest);
        }
    }

    fn filtered_indices(&self) -> Vec<usize> {
        self.options
            .iter()
            .enumerate()
            .filter_map(|(index, option)| {
                (self.query.is_empty()
                    || option.label.to_lowercase().contains(&self.query.to_lowercase()))
                .then_some(index)
            })
            .collect()
    }

    fn committed_label(&self) -> Option<String> {
        self.committed.as_deref().and_then(|id| {
            self.options.iter().find(|option| option.id == id).map(|option| option.label.clone())
        })
    }

    fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.open == open {
            return;
        }
        self.open = open;
        if open {
            self.session_start = Some(self.committed.clone());
        } else {
            self.active = None;
        }
        cx.emit(OpenChanged(open));
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.marked.is_some() {
            return;
        }
        let key = event.keystroke.key.as_str();
        let head = self.selection.start;
        let tail = self.selection.end;
        let next = match key {
            "left" if !self.selection.is_empty() => Some(head..head),
            "right" if !self.selection.is_empty() => Some(tail..tail),
            "left" => {
                let byte = previous_char_boundary(&self.query, tail);
                Some(byte..byte)
            }
            "right" => {
                let byte = next_char_boundary(&self.query, tail);
                Some(byte..byte)
            }
            "home" => Some(0..0),
            "end" => Some(self.query.len()..self.query.len()),
            _ => None,
        };
        if let Some(range) = next {
            self.selection = range;
            cx.notify();
        }
    }

    fn open_next(&mut self, _: &OpenNext, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.marked.is_some() {
            return;
        }
        let filtered = self.filtered_indices();
        if !self.open {
            self.query = self.committed_label().unwrap_or_default();
            self.selection = self.query.len()..self.query.len();
            self.marked = None;
            self.set_open(true, cx);
            let filtered = self.filtered_indices();
            self.active = first_enabled(&self.options, &filtered);
        } else if !filtered.is_empty() {
            self.active = next_enabled(&self.options, &filtered, self.active, false);
        }
        self.scroll_active();
        cx.notify();
    }

    fn open_previous(&mut self, _: &OpenPrevious, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.marked.is_some() {
            return;
        }
        if !self.open {
            self.set_open(true, cx);
        }
        let filtered = self.filtered_indices();
        if !filtered.is_empty() {
            self.active = next_enabled(&self.options, &filtered, self.active, true);
        }
        self.scroll_active();
        cx.notify();
    }

    fn commit(&mut self, _: &Commit, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.marked.is_some() || !self.open {
            return;
        }
        let filtered = self.filtered_indices();
        if let Some(option) = self
            .active
            .and_then(|index| filtered.get(index))
            .filter(|index| !self.options[**index].disabled)
        {
            let option = self.options[*option].clone();
            if !self.controlled {
                self.committed = Some(option.id.clone());
            }
            self.query = option.label;
            self.selection = self.query.len()..self.query.len();
            self.marked = None;
            cx.emit(ValueChanged(option.id.clone()));
            cx.emit(OptionSelected(option.id));
            self.set_open(false, cx);
            if self.controlled {
                self.query = self.committed_label().unwrap_or_default();
            }
            self.selection = self.query.len()..self.query.len();
            self.marked = None;
            self.session_start = None;
        } else if self.allow_custom_value && !self.query.is_empty() {
            if !self.controlled {
                self.committed = Some(self.query.clone());
            }
            cx.emit(ValueChanged(self.query.clone()));
            self.set_open(false, cx);
            if self.controlled {
                self.query = self.committed_label().unwrap_or_default();
            }
            self.selection = self.query.len()..self.query.len();
            self.marked = None;
            self.session_start = None;
        }
        cx.notify();
    }

    fn cancel(&mut self, _: &Cancel, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.marked.is_some() || !self.open {
            return;
        }
        let rollback = self.session_start.take().unwrap_or_else(|| self.committed.clone());
        if !self.controlled {
            self.committed = rollback.clone();
        }
        self.query = self.committed_label().unwrap_or_default();
        self.selection = self.query.len()..self.query.len();
        self.marked = None;
        cx.emit(Cancelled(rollback));
        self.set_open(false, cx);
        cx.notify();
    }

    fn dismiss(&mut self, _: &Dismiss, window: &mut Window, cx: &mut Context<Self>) {
        if self.marked.is_some() {
            return;
        }
        self.dismiss_popup(cx);
        window.focus_next(cx);
    }

    fn dismiss_previous(
        &mut self,
        _: &DismissPrevious,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.marked.is_some() {
            return;
        }
        self.dismiss_popup(cx);
        window.focus_prev(cx);
    }

    fn dismiss_popup(&mut self, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        self.set_open(false, cx);
        self.session_start = None;
        cx.emit(Dismissed);
        cx.notify();
    }

    fn open_explicit(&mut self, _: &OpenExplicit, _: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled && self.marked.is_none() {
            self.active = None;
            self.set_open(true, cx);
            cx.notify();
        }
    }

    fn close_explicit(&mut self, _: &CloseExplicit, _: &mut Window, cx: &mut Context<Self>) {
        if self.open && self.marked.is_none() {
            self.set_open(false, cx);
            self.session_start = None;
            cx.notify();
        }
    }
}

fn first_enabled(options: &[OptionItem], filtered: &[usize]) -> Option<usize> {
    filtered.iter().position(|index| !options[*index].disabled)
}

fn next_enabled(
    options: &[OptionItem],
    filtered: &[usize],
    active: Option<usize>,
    backwards: bool,
) -> Option<usize> {
    if filtered.is_empty() {
        return None;
    }
    for offset in 1..=filtered.len() {
        let candidate = match (active, backwards) {
            (Some(current), false) => (current + offset) % filtered.len(),
            (Some(current), true) => {
                (current + filtered.len() - (offset % filtered.len())) % filtered.len()
            }
            (None, false) => (offset - 1) % filtered.len(),
            (None, true) => filtered.len() - offset,
        };
        if !options[filtered[candidate]].disabled {
            return Some(candidate);
        }
    }
    None
}

impl Focusable for Combobox {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("combobox focus handle initialized during render")
    }
}

impl EntityInputHandler for Combobox {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let bytes = utf16_range_to_bytes(&self.query, &range);
        *adjusted = Some(byte_range_to_utf16(&self.query, &bytes));
        Some(self.query[bytes].to_owned())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: byte_range_to_utf16(&self.query, &self.selection),
            reversed: false,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_ref().map(|range| byte_range_to_utf16(&self.query, range))
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
        let range = range
            .as_ref()
            .map(|range| utf16_range_to_bytes(&self.query, range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        let inserted = text.replace(['\n', '\r'], " ");
        let mut query = self.query.clone();
        query.replace_range(range.clone(), &inserted);
        let caret = range.start + inserted.len();
        self.selection = caret..caret;
        self.marked = None;
        self.update_query(query, cx);
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
        let range = range
            .as_ref()
            .map(|range| utf16_range_to_bytes(&self.query, range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        let start = range.start;
        let mut query = self.query.clone();
        query.replace_range(range, text);
        self.marked = (!text.is_empty()).then_some(start..start + text.len());
        self.selection = selected
            .map(|range| {
                start + utf16_offset_to_byte(text, range.start)
                    ..start + utf16_offset_to_byte(text, range.end)
            })
            .unwrap_or(start + text.len()..start + text.len());
        self.update_query(query, cx);
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        element_bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let bytes = utf16_range_to_bytes(&self.query, &range);
        let style = window.text_style();
        let shaped = window.text_system().shape_line(
            self.query.as_str().into(),
            style.font_size.to_pixels(window.rem_size()),
            &[style.to_run(self.query.len())],
            None,
        );
        let start = shaped.x_for_index(bytes.start);
        let end = shaped.x_for_index(bytes.end);
        let line_height = window.line_height();
        let vertical_inset = ((element_bounds.size.height - line_height) / 2.).max(px(0.));
        let inset = text_inset(cx.global::<Theme>());
        let origin =
            point(element_bounds.left() + inset + start, element_bounds.top() + vertical_inset);
        Some(Bounds::new(origin, size((end - start).max(px(1.)), line_height)))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui_pre::Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let style = window.text_style();
        let shaped = window.text_system().shape_line(
            self.query.as_str().into(),
            style.font_size.to_pixels(window.rem_size()),
            &[style.to_run(self.query.len())],
            None,
        );
        let inset = text_inset(cx.global::<Theme>());
        let x = (point.x - bounds.left() - inset).max(px(0.));
        let byte = shaped.closest_index_for_x(x).min(self.query.len());
        Some(byte_range_to_utf16(&self.query, &(byte..byte)).start)
    }

    fn set_selected_text_range(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selection = utf16_range_to_bytes(&self.query, &range);
        self.marked = None;
        cx.notify();
    }

    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.query.encode_utf16().count())
    }

    fn accepts_text_input(&self, _: &mut Window, _: &mut Context<Self>) -> bool {
        !self.disabled
    }
}

fn utf16_offset_to_byte(text: &str, offset: usize) -> usize {
    let mut units = 0;
    for (byte, character) in text.char_indices() {
        if units >= offset {
            return byte;
        }
        units += character.len_utf16();
        if units >= offset {
            return byte + character.len_utf8();
        }
    }
    text.len()
}

fn utf16_range_to_bytes(text: &str, range: &Range<usize>) -> Range<usize> {
    utf16_offset_to_byte(text, range.start)..utf16_offset_to_byte(text, range.end)
}

fn byte_range_to_utf16(text: &str, range: &Range<usize>) -> Range<usize> {
    text[..range.start.min(text.len())].encode_utf16().count()
        ..text[..range.end.min(text.len())].encode_utf16().count()
}

fn previous_char_boundary(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].char_indices().next_back().map_or(0, |(index, _)| index)
}

fn next_char_boundary(text: &str, offset: usize) -> usize {
    text[offset.min(text.len())..]
        .chars()
        .next()
        .map_or(text.len(), |character| offset.min(text.len()) + character.len_utf8())
}

impl Render for Combobox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus =
            self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(1).tab_stop(true)).clone();
        let input = cx.entity();
        let theme = *cx.global::<Theme>();
        let filtered = self.filtered_indices();
        let disabled = self.disabled;
        let focused = !disabled && focus.is_focused(window);
        let look = look(&theme);
        let (input_border, text) = if !disabled {
            (look.input_border, look.text)
        } else if look.high_contrast {
            (look.disabled, look.disabled)
        } else {
            (dim(look.input_border, look.background), dim(look.text, look.background))
        };
        let field_input = input.clone();
        let mut content = div().flex().items_center();
        if !focused {
            content = content.child(self.query.clone());
        } else if let Some(marked) = self.marked.clone() {
            content = content
                .child(self.query[..marked.start].to_owned())
                .child(
                    div()
                        .bg(theme.colors.accent)
                        .text_color(theme.colors.accent_text)
                        .child(self.query[marked.clone()].to_owned()),
                )
                .child(self.query[marked.end..].to_owned());
        } else {
            content = content.child(self.query[..self.selection.start].to_owned());
            if self.selection.is_empty() {
                content = content.child(
                    div()
                        .w(px(theme.borders.regular))
                        .h(window.line_height())
                        .flex_shrink_0()
                        .bg(theme.colors.accent),
                );
            } else {
                content = content.child(
                    div()
                        .bg(theme.colors.accent)
                        .text_color(theme.colors.accent_text)
                        .child(self.query[self.selection.clone()].to_owned()),
                );
            }
            content = content.child(self.query[self.selection.end..].to_owned());
        }
        let field = div()
            .id("combobox-input")
            .debug_selector(|| "combobox-input".to_owned())
            .key_context(KEY_CONTEXT)
            .when(!self.disabled, |element| element.track_focus(&focus))
            .role(gpui_pre::accesskit::Role::EditableComboBox)
            .aria_label(self.label.clone())
            .aria_value(self.query.clone())
            .aria_expanded(self.open)
            // GPUI 0.3.5 has no convenience setters for these AccessKit properties.
            .a11y_synthetic_children(move |builder| {
                let node = builder.parent_node();
                node.set_auto_complete(gpui_pre::accesskit::AutoComplete::List);
                if disabled {
                    node.set_disabled();
                }
            })
            .on_action(cx.listener(Self::open_next))
            .on_action(cx.listener(Self::open_previous))
            .on_action(cx.listener(Self::commit))
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::dismiss))
            .on_action(cx.listener(Self::dismiss_previous))
            .on_action(cx.listener(Self::open_explicit))
            .on_action(cx.listener(Self::close_explicit))
            .on_key_down(cx.listener(Self::on_key_down))
            .h(gpui_pre::px(theme.controls.medium))
            .px(text_inset(&theme))
            .flex()
            .items_center()
            .rounded(gpui_pre::px(theme.radii.medium))
            .border(gpui_pre::px(theme.borders.regular))
            .border_color(if focused { look.focus } else { input_border })
            .bg(look.background)
            .shadow(if focused {
                vec![focus_ring(look.ring)]
            } else if look.high_contrast {
                Vec::new()
            } else {
                vec![box_shadow(theme.shadows.small, if disabled { 0.5 } else { 1.0 })]
            })
            .text_color(text)
            .text_size(gpui_pre::px(theme.typography.body))
            .min_w(gpui_pre::px(theme.spacing.xxlarge))
            .child(content)
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, (), window, cx| {
                        window.handle_input(
                            &focus,
                            ElementInputHandler::new(bounds, field_input.clone()),
                            cx,
                        );
                        field_input.update(cx, |combobox, _| combobox.last_bounds = Some(bounds));
                    },
                )
                .absolute()
                .inset_0(),
            );
        let mut root = div().child(field);
        if self.open {
            let mut list = div()
                .id("combobox-popup")
                .debug_selector(|| "combobox-popup".to_owned())
                .role(gpui_pre::accesskit::Role::ListBox)
                .mt(gpui_pre::px(theme.spacing.xsmall))
                .p(gpui_pre::px(theme.spacing.xsmall))
                .rounded(gpui_pre::px(theme.radii.medium))
                .border(gpui_pre::px(theme.borders.regular))
                .border_color(look.popup_border)
                .bg(look.popup_bg)
                .when(!look.high_contrast, |element| {
                    element.shadow(vec![box_shadow(theme.shadows.medium, 1.0)])
                })
                .text_size(gpui_pre::px(theme.typography.body));
            if filtered.is_empty() {
                list = list.child(
                    div()
                        .py(gpui_pre::px(theme.spacing.xlarge))
                        .flex()
                        .justify_center()
                        .text_color(theme.colors.text_muted)
                        .child("No matching options"),
                );
            } else {
                let filtered_mapping = filtered.clone();
                let options = self.options.clone();
                let committed = self.committed.clone();
                let active = self.active;
                let scroll = self.list_scroll.clone();
                let row_height = theme.controls.small;
                let rows = filtered_mapping.len().min(8) as f32;
                let input = input.clone();
                list = list.child(
                    uniform_list(
                        "combobox-options",
                        filtered_mapping.len(),
                        move |range, _window, _cx| {
                            range
                                .filter_map(|filtered_index| {
                                    let option_index = *filtered_mapping.get(filtered_index)?;
                                    let option = &options[option_index];
                                    let id = option.id.clone();
                                    let debug_id = id.clone();
                                    let label = option.label.clone();
                                    let query_label = label.clone();
                                    let disabled = option.disabled;
                                    let selected = committed.as_deref() == Some(id.as_str());
                                    let is_active = active == Some(filtered_index);
                                    let (foreground, check) = if disabled {
                                        if look.high_contrast {
                                            (look.disabled, look.disabled)
                                        } else {
                                            (
                                                dim(look.text, look.popup_bg),
                                                dim(look.icon, look.popup_bg),
                                            )
                                        }
                                    } else if is_active {
                                        let check = if look.high_contrast {
                                            look.active_text
                                        } else {
                                            look.icon
                                        };
                                        (look.active_text, check)
                                    } else {
                                        (look.text, look.icon)
                                    };
                                    let hover_bg =
                                        look.hover_bg.filter(|_| !disabled && !is_active);
                                    let input = input.clone();
                                    Some(
                                        div()
                                            .id(id.clone())
                                            .debug_selector(move || {
                                                format!("combobox-option-{debug_id}")
                                            })
                                            .role(gpui_pre::accesskit::Role::ListBoxOption)
                                            .aria_label(label.clone())
                                            .aria_selected(selected)
                                            .a11y_synthetic_children(move |builder| {
                                                if disabled {
                                                    builder.parent_node().set_disabled();
                                                }
                                            })
                                            .h(px(row_height))
                                            .w_full()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .gap(px(theme.spacing.small))
                                            .px(px(theme.spacing.small))
                                            .rounded(px(theme.radii.small))
                                            .when(is_active, |element| {
                                                element.aria_active_descendant().bg(look.active_bg)
                                            })
                                            .when_some(hover_bg, |element, bg| {
                                                element.hover(move |style| style.bg(bg))
                                            })
                                            .text_color(foreground)
                                            .child(div().flex_1().min_w_0().truncate().child(label))
                                            .when(selected, |element| {
                                                element
                                                    .child(check_icon(theme.spacing.large, check))
                                            })
                                            .on_click(move |_, _, cx| {
                                                input.update(cx, |combobox, cx| {
                                                    if combobox.disabled || disabled {
                                                        return;
                                                    }
                                                    let next = Some(id.clone());
                                                    if !combobox.controlled {
                                                        combobox.committed = next.clone();
                                                        cx.notify();
                                                    }
                                                    combobox.query = query_label.clone();
                                                    combobox.selection =
                                                        combobox.query.len()..combobox.query.len();
                                                    combobox.marked = None;
                                                    cx.emit(ValueChanged(id.clone()));
                                                    cx.emit(OptionSelected(id.clone()));
                                                    combobox.set_open(false, cx);
                                                    combobox.session_start = None;
                                                    if combobox.controlled {
                                                        combobox.query = combobox
                                                            .committed_label()
                                                            .unwrap_or_default();
                                                        combobox.selection = combobox.query.len()
                                                            ..combobox.query.len();
                                                    }
                                                    cx.notify();
                                                });
                                            }),
                                    )
                                })
                                .collect()
                        },
                    )
                    .h(px(row_height * rows))
                    .track_scroll(&scroll),
                );
            }
            root = root.child(list);
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{AppContext, Entity, ParentElement, Subscription, TestAppContext, div};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn large_filtered_collections_remain_fully_navigable() {
        let options = (0..128)
            .map(|index| OptionItem::new(format!("item-{index}"), format!("Result {index}")))
            .collect::<Vec<_>>();
        let mut combobox = Combobox::new("Results", options, None);
        combobox.query = "result".into();
        let filtered = combobox.filtered_indices();

        assert_eq!(filtered.len(), 128);
        let mut active = first_enabled(&combobox.options, &filtered);
        for index in 0..128 {
            assert_eq!(active, Some(index));
            active = next_enabled(&combobox.options, &filtered, active, false);
        }
        assert_eq!(active, Some(0), "navigation wraps across the complete filtered mapping");
    }

    #[test]
    fn filtering_keeps_disabled_matches_visible_but_navigation_skips_them() {
        let options = vec![
            OptionItem::new("ca", "Cameroon").disabled(true),
            OptionItem::new("can", "Canada"),
            OptionItem::new("cal", "California").disabled(true),
            OptionItem::new("cm", "Cambodia"),
        ];
        let mut combobox = Combobox::new("Location", options, None);
        combobox.query = "Ca".into();
        let filtered = combobox.filtered_indices();

        assert_eq!(filtered, [0, 1, 2, 3], "matching disabled options remain visible");
        assert_eq!(first_enabled(&combobox.options, &filtered), Some(1));
        assert_eq!(next_enabled(&combobox.options, &filtered, Some(1), false), Some(3));
        assert_eq!(next_enabled(&combobox.options, &filtered, Some(3), false), Some(1));
        assert_eq!(next_enabled(&combobox.options, &filtered, Some(1), true), Some(3));

        let all_disabled = vec![
            OptionItem::new("x", "Café").disabled(true),
            OptionItem::new("y", "Cacao").disabled(true),
        ];
        let all_disabled_indices = [0, 1];
        assert_eq!(first_enabled(&all_disabled, &all_disabled_indices), None);
        assert_eq!(next_enabled(&all_disabled, &all_disabled_indices, None, false), None);
    }

    struct TestHost {
        combobox: Option<Entity<Combobox>>,
        initial_value: Option<String>,
        inputs: Rc<RefCell<Vec<String>>>,
        changes: Rc<RefCell<Vec<String>>>,
        selected: Rc<RefCell<Vec<String>>>,
        _subscriptions: Vec<Subscription>,
    }

    impl Render for TestHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.combobox.is_none() {
                let initial_value = self.initial_value.clone();
                let combobox = cx.new(move |_| {
                    Combobox::new(
                        "Country",
                        vec![
                            OptionItem::new("us", "United States"),
                            OptionItem::new("cm", "Cameroon"),
                            OptionItem::new("ca", "Canada"),
                        ],
                        initial_value,
                    )
                });
                let inputs = self.inputs.clone();
                let input_sub = cx.subscribe(&combobox, move |_, _, event: &InputChanged, _| {
                    inputs.borrow_mut().push(event.0.clone());
                });
                let changes = self.changes.clone();
                let change_sub = cx.subscribe(&combobox, move |_, _, event: &ValueChanged, _| {
                    changes.borrow_mut().push(event.0.clone());
                });
                let selected = self.selected.clone();
                let selected_sub =
                    cx.subscribe(&combobox, move |_, _, event: &OptionSelected, _| {
                        selected.borrow_mut().push(event.0.clone());
                    });
                self.combobox = Some(combobox);
                self._subscriptions = vec![input_sub, change_sub, selected_sub];
            }
            div().child(self.combobox.as_ref().unwrap().clone())
        }
    }

    #[gpui_pre::test]
    fn filters_navigates_and_commits_enabled_options(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let changes = Rc::new(RefCell::new(Vec::new()));
        let selected = Rc::new(RefCell::new(Vec::new()));
        let inputs = Rc::new(RefCell::new(Vec::new()));
        let (host, visual) = cx.add_window_view({
            let changes = changes.clone();
            let selected = selected.clone();
            let inputs = inputs.clone();
            move |_, _| TestHost {
                combobox: None,
                initial_value: Some("us".into()),
                inputs,
                changes,
                selected,
                _subscriptions: Vec::new(),
            }
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let combobox = host.read_with(visual, |host, _| host.combobox.as_ref().unwrap().clone());
        visual.update(|window, cx| combobox.focus_handle(cx).focus(window, cx));
        combobox.update(visual, |combobox, cx| combobox.set_query("Cam", cx));
        visual.simulate_keystrokes("down enter");
        assert_eq!(
            combobox.read_with(visual, |combobox, _| combobox.value().map(str::to_owned)),
            Some("cm".to_owned())
        );
        assert_eq!(&*changes.borrow(), &["cm"]);
        assert_eq!(&*selected.borrow(), &["cm"]);
    }

    #[gpui_pre::test]
    fn native_text_input_filters_and_commits_with_keyboard(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let changes = Rc::new(RefCell::new(Vec::new()));
        let selected = Rc::new(RefCell::new(Vec::new()));
        let inputs = Rc::new(RefCell::new(Vec::new()));
        let (host, visual) = cx.add_window_view({
            let changes = changes.clone();
            let selected = selected.clone();
            let inputs = inputs.clone();
            move |_, _| TestHost {
                combobox: None,
                initial_value: None,
                inputs,
                changes,
                selected,
                _subscriptions: Vec::new(),
            }
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let combobox = host.read_with(visual, |host, _| host.combobox.as_ref().unwrap().clone());
        visual.update(|window, cx| combobox.focus_handle(cx).focus(window, cx));

        visual.simulate_input("Cam");
        assert_eq!(combobox.read_with(visual, |combobox, _| combobox.query().to_owned()), "Cam");
        assert_eq!(&*inputs.borrow(), &["C", "Ca", "Cam"]);
        assert!(combobox.read_with(visual, |combobox, _| combobox.is_open()));
        assert_eq!(combobox.read_with(visual, |combobox, _| combobox.filtered_indices()), [1]);

        visual.simulate_keystrokes("down enter");
        assert_eq!(
            combobox.read_with(visual, |combobox, _| combobox.value().map(str::to_owned)),
            Some("cm".to_owned())
        );
        assert_eq!(&*changes.borrow(), &["cm"]);
        assert_eq!(&*selected.borrow(), &["cm"]);
    }

    #[gpui_pre::test]
    fn large_popup_scrolls_to_the_last_active_option(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, _| TestHost {
            combobox: None,
            initial_value: None,
            inputs: Rc::new(RefCell::new(Vec::new())),
            changes: Rc::new(RefCell::new(Vec::new())),
            selected: Rc::new(RefCell::new(Vec::new())),
            _subscriptions: Vec::new(),
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let combobox = host.read_with(visual, |host, _| host.combobox.as_ref().unwrap().clone());
        combobox.update(visual, |combobox, cx| {
            combobox.options = (0..128)
                .map(|index| OptionItem::new(format!("item-{index}"), format!("Result {index}")))
                .collect();
            cx.notify();
        });
        visual.update(|window, cx| combobox.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("up");
        visual.update(|window, cx| window.draw(cx).clear(cx));

        combobox.read_with(visual, |combobox, _| {
            assert!(combobox.open);
            assert_eq!(combobox.active, Some(127));
            assert!(combobox.list_scroll.is_scrollable());
            assert_eq!(combobox.list_scroll.is_scrolled_to_end(), Some(true));
        });
    }

    #[gpui_pre::test]
    fn pointer_click_after_filtered_virtual_scroll_commits_mapped_option(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) = cx.add_window_view(|_, _| TestHost {
            combobox: None,
            initial_value: None,
            inputs: Rc::new(RefCell::new(Vec::new())),
            changes: Rc::new(RefCell::new(Vec::new())),
            selected: Rc::new(RefCell::new(Vec::new())),
            _subscriptions: Vec::new(),
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let combobox = host.read_with(visual, |host, _| host.combobox.as_ref().unwrap().clone());
        combobox.update(visual, |combobox, cx| {
            combobox.options = (0..128)
                .map(|index| OptionItem::new(format!("item-{index}"), format!("Result {index}")))
                .collect();
            combobox.set_query("Result", cx);
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let popup = visual.debug_bounds("combobox-popup").expect("popup bounds");
        visual.simulate_event(gpui_pre::ScrollWheelEvent {
            position: popup.center(),
            delta: gpui_pre::ScrollDelta::Pixels(gpui_pre::point(
                gpui_pre::px(0.),
                gpui_pre::px(-10000.),
            )),
            modifiers: Default::default(),
            touch_phase: gpui_pre::TouchPhase::Moved,
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let row = visual
            .debug_bounds("combobox-option-item-127")
            .expect("last filtered virtual row")
            .center();
        visual.simulate_mouse_down(row, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_up(row, gpui_pre::MouseButton::Left, Default::default());
        assert_eq!(
            combobox.read_with(visual, |combobox, _| combobox.value().map(str::to_owned)),
            Some("item-127".into())
        );
        assert!(!combobox.read_with(visual, |combobox, _| combobox.is_open()));
    }
}
