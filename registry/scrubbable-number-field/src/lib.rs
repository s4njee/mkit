//! Initial stateful numeric control for the scrubbable-number-field registry entry.
//!
//! This slice provides a controlled/uncontrolled numeric value, native text
//! drafts, named keyboard actions, pointer scrubbing, and spinbutton semantics.
//! Disabled visuals, input suppression, and AccessKit disabled state are requested.

extern crate gpui_pre as gpui;

use gpui_pre::{
    Bounds, Context, CursorStyle, DispatchPhase, ElementInputHandler, EntityInputHandler,
    EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PathBuilder, Pixels, Render, Rgba, TextStyle, UTF16Selection,
    Window, actions, canvas, div, point, prelude::*, px, size,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
use std::ops::Range;

/// Resolved field colours; see the spec's "Theme tokens used" table. The chrome is the restyled
/// TextField's, so the two controls are indistinguishable at the same size.
#[derive(Clone, Copy)]
struct Look {
    fill: Rgba,
    text: Rgba,
    /// Unit suffix and scrub affordance.
    muted: Rgba,
    border: Rgba,
    focus_border: Rgba,
    invalid_border: Rgba,
    /// `shadows.small` with its colour adjusted for the state; transparent in high contrast.
    shadow: ShadowToken,
    focus_ring: Rgba,
    /// Ring drawn around an invalid draft whether or not the field is focused; `None` in high
    /// contrast, where invalid is shown by the border and focus keeps its own ring.
    invalid_ring: Option<Rgba>,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight, ..foreground }, Rgba { a: 1.0, ..base })
}
/// A disabled field renders at 50% opacity as one layer. GPUI applies element opacity to each
/// painted part separately, so each colour is composited opaque over `background` and then mixed
/// 50% with it instead.
fn dim(color: Rgba, background: Rgba) -> Rgba {
    mix(composite(color, background), background, 0.5)
}
fn look(t: &Theme, disabled: bool) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        let (text, muted, border) = if disabled {
            (c.disabled, c.disabled, c.disabled)
        } else {
            (c.text, c.text_muted, c.border)
        };
        return Look {
            fill: c.background,
            text,
            muted,
            border,
            focus_border: c.focus,
            invalid_border: c.danger,
            shadow: t.shadows.none,
            focus_ring: c.focus,
            invalid_ring: None,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn "input": the light border, or text at 15% in dark themes.
    let input = if dark { c.text.opacity(0.15) } else { c.border };
    // shadcn `dark:bg-input/30`; light fields are transparent over the page, made opaque.
    let fill = if dark { mix(c.text, c.background, 0.15 * 0.3) } else { c.background };
    let look = Look {
        fill,
        text: c.text,
        muted: c.text_muted,
        border: composite(input, fill),
        focus_border: c.focus,
        invalid_border: c.danger,
        shadow: t.shadows.small,
        focus_ring: c.focus.opacity(0.5),
        invalid_ring: Some(c.danger.opacity(if dark { 0.4 } else { 0.2 })),
    };
    if !disabled {
        return look;
    }
    let bg = c.background;
    Look {
        fill: dim(look.fill, bg),
        text: dim(look.text, bg),
        muted: dim(look.muted, bg),
        border: dim(look.border, bg),
        invalid_border: dim(look.invalid_border, bg),
        shadow: ShadowToken { color: look.shadow.color.opacity(0.5), ..look.shadow },
        ..look
    }
}
fn box_shadow(shadow: ShadowToken) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: shadow.color.into(),
        offset: point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}
/// shadcn/ui focus ring width, drawn outside the field (shared with TextField).
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
/// Decorative Lucide `chevrons-left-right` (9,7 → 4,12 → 9,17 and 15,7 → 20,12 → 15,17 on a
/// 24-unit grid, 2-unit stroke), the scrub affordance, drawn as a vector path.
fn scrub_affordance(size: f32, color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let origin = bounds.origin;
            for (tip, back) in [(4.0, 9.0), (20.0, 15.0)] {
                let mut path = PathBuilder::stroke(unit * 2.0);
                path.move_to(origin + point(unit * back, unit * 7.0));
                path.line_to(origin + point(unit * tip, unit * 12.0));
                path.line_to(origin + point(unit * back, unit * 17.0));
                if let Ok(path) = path.build() {
                    window.paint_path(path, color);
                }
            }
        },
    )
    .size(px(size))
    .flex_none()
}

/// Key context used by this component's named actions.
pub const KEY_CONTEXT: &str = "ScrubbableNumberField";

actions!(
    scrubbable_number_field,
    [
        Increment,
        Decrement,
        PrecisionIncrement,
        PrecisionDecrement,
        PageIncrement,
        PageDecrement,
        SetMinimum,
        SetMaximum,
        CommitDraft,
        CancelDraft
    ]
);

/// Default named bindings. Applications may replace these bindings while keeping the actions.
pub fn default_key_bindings() -> [KeyBinding; 10] {
    [
        KeyBinding::new("up", Increment, Some(KEY_CONTEXT)),
        KeyBinding::new("down", Decrement, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-up", PrecisionIncrement, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-down", PrecisionDecrement, Some(KEY_CONTEXT)),
        KeyBinding::new("pageup", PageIncrement, Some(KEY_CONTEXT)),
        KeyBinding::new("pagedown", PageDecrement, Some(KEY_CONTEXT)),
        KeyBinding::new("home", SetMinimum, Some(KEY_CONTEXT)),
        KeyBinding::new("end", SetMaximum, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", CommitDraft, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", CancelDraft, Some(KEY_CONTEXT)),
    ]
}

/// Origin of a value request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChangeSource {
    Keyboard,
    Text,
    Pointer,
}

/// A valid user requested value. Controlled owners accept it by calling `set_value`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValueChanged {
    pub previous: f64,
    pub value: f64,
    pub source: ChangeSource,
}

/// Emitted when the user commits a value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValueCommitted {
    pub value: f64,
    pub source: ChangeSource,
}

/// Entity-backed numeric field. Initial slice supports keyboard stepping only.
pub struct ScrubbableNumberField {
    value: f64,
    controlled: bool,
    min: Option<f64>,
    max: Option<f64>,
    step: f64,
    page_step: Option<f64>,
    precision_step: Option<f64>,
    label: String,
    description: String,
    unit: String,
    disabled: bool,
    focus: Option<FocusHandle>,
    draft: String,
    selection: Range<usize>,
    marked: Option<Range<usize>>,
    last_bounds: Option<Bounds<Pixels>>,
    draft_active: bool,
    scrub_scale: f32,
    pointer_origin: Option<f32>,
    pointer_start_value: f64,
    pointer_current_value: f64,
    pointer_changed: bool,
    pointer_scrubbing: bool,
    pointer_started_focused: bool,
    /// Text size used for rendering, pointer hit testing, and IME bounds.
    text_size: Option<Pixels>,
}

impl EventEmitter<ValueChanged> for ScrubbableNumberField {}
impl EventEmitter<ValueCommitted> for ScrubbableNumberField {}

impl ScrubbableNumberField {
    /// Create an uncontrolled field with a positive finite step (defaults to 1).
    pub fn new(value: f64) -> Self {
        Self {
            value: finite_or(value, 0.0),
            controlled: false,
            min: None,
            max: None,
            step: 1.0,
            page_step: None,
            precision_step: None,
            label: String::new(),
            description: String::new(),
            unit: String::new(),
            disabled: false,
            focus: None,
            draft: format_number(finite_or(value, 0.0)),
            selection: 0..format_number(finite_or(value, 0.0)).len(),
            marked: None,
            last_bounds: None,
            draft_active: false,
            scrub_scale: 4.0,
            pointer_origin: None,
            pointer_start_value: finite_or(value, 0.0),
            pointer_current_value: finite_or(value, 0.0),
            pointer_changed: false,
            pointer_scrubbing: false,
            pointer_started_focused: false,
            text_size: None,
        }
    }

    /// Create a field whose displayed value is supplied by its owner.
    pub fn controlled(value: f64) -> Self {
        let mut field = Self::new(value);
        field.controlled = true;
        field
    }

    pub fn bounds(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.min = min.filter(|v| v.is_finite());
        // A reversed range is treated as an absent maximum rather than creating
        // contradictory clamp behavior; the minimum remains meaningful.
        self.max = max.filter(|v| v.is_finite() && self.min.is_none_or(|min| min <= *v));
        self.value = self.clamp(self.value);
        self.refresh_draft();
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        if step.is_finite() && step > 0.0 {
            self.step = step;
        }
        self
    }

    /// Override the page-key delta. Defaults to ten times `step`.
    pub fn page_step(mut self, step: f64) -> Self {
        if step.is_finite() && step > 0.0 {
            self.page_step = Some(step);
        }
        self
    }

    /// Override the Shift+Arrow delta. Defaults to one tenth of `step`.
    pub fn precision_step(mut self, step: f64) -> Self {
        if step.is_finite() && step > 0.0 {
            self.precision_step = Some(step);
        }
        self
    }

    /// Configure logical pixels per step used during horizontal scrubbing.
    pub fn scrub_scale(mut self, pixels_per_step: f32) -> Self {
        if pixels_per_step.is_finite() && pixels_per_step > 0.0 {
            self.scrub_scale = pixels_per_step;
        }
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Supplementary accessible description announced after the field's name and value.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Optional display suffix. The suffix is presentation only and is stripped
    /// from typed drafts before parsing.
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    /// The text currently shown in the editing surface (which may be invalid).
    pub fn draft_text(&self) -> &str {
        &self.draft
    }

    /// Select the displayed text, matching the native select-all editing command.
    pub fn select_all(&mut self) {
        self.selection = 0..self.draft.len();
        self.marked = None;
    }

    /// Replace the current value from controlled owner props (or reset uncontrolled state).
    pub fn set_value(&mut self, value: f64, cx: &mut Context<Self>) {
        if value.is_finite() {
            self.value = self.clamp(value);
            if !self.draft_active {
                self.refresh_draft();
            }
            cx.notify();
        }
    }

    fn clamp(&self, value: f64) -> f64 {
        let value = self.min.map_or(value, |min| value.max(min));
        self.max.map_or(value, |max| value.min(max))
    }

    fn change(&mut self, delta: f64, source: ChangeSource, cx: &mut Context<Self>) {
        if self.disabled || self.marked.is_some() || !delta.is_finite() || delta == 0.0 {
            return;
        }
        let previous = self.value;
        let sum = previous + delta;
        // Overflow can still resolve to a finite endpoint when a matching bound
        // exists. With no endpoint, retain the last valid value.
        let target = if sum.is_finite() {
            sum
        } else if delta.is_sign_positive() {
            self.max.unwrap_or(previous)
        } else {
            self.min.unwrap_or(previous)
        };
        let next = self.clamp(target);
        self.change_to(next, source, cx);
    }

    fn change_to(&mut self, target: f64, source: ChangeSource, cx: &mut Context<Self>) {
        if self.disabled || self.marked.is_some() || !target.is_finite() {
            return;
        }
        let previous = self.value;
        let next = self.clamp(target);
        if next == previous {
            return;
        }
        if !self.controlled {
            self.value = next;
            if !self.draft_active {
                self.refresh_draft();
            }
        }
        cx.emit(ValueChanged { previous, value: next, source });
        cx.emit(ValueCommitted { value: next, source });
        cx.notify();
    }

    fn increment(&mut self, _: &Increment, _: &mut Window, cx: &mut Context<Self>) {
        self.change(self.step, ChangeSource::Keyboard, cx);
    }

    fn decrement(&mut self, _: &Decrement, _: &mut Window, cx: &mut Context<Self>) {
        self.change(-self.step, ChangeSource::Keyboard, cx);
    }

    fn precision_increment(
        &mut self,
        _: &PrecisionIncrement,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.change(self.precision_step.unwrap_or(self.step / 10.0), ChangeSource::Keyboard, cx);
    }

    fn precision_decrement(
        &mut self,
        _: &PrecisionDecrement,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.change(-self.precision_step.unwrap_or(self.step / 10.0), ChangeSource::Keyboard, cx);
    }

    fn page_increment(&mut self, _: &PageIncrement, _: &mut Window, cx: &mut Context<Self>) {
        let page_step = self.page_step.unwrap_or_else(|| default_page_step(self.step));
        self.change(page_step, ChangeSource::Keyboard, cx);
    }

    fn page_decrement(&mut self, _: &PageDecrement, _: &mut Window, cx: &mut Context<Self>) {
        let page_step = self.page_step.unwrap_or_else(|| default_page_step(self.step));
        self.change(-page_step, ChangeSource::Keyboard, cx);
    }

    fn set_minimum(&mut self, _: &SetMinimum, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(min) = self.min {
            self.change_to(min, ChangeSource::Keyboard, cx);
        }
    }

    fn set_maximum(&mut self, _: &SetMaximum, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(max) = self.max {
            self.change_to(max, ChangeSource::Keyboard, cx);
        }
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
        self.pointer_origin = Some(event.position.x.as_f32());
        self.pointer_start_value = self.value;
        self.pointer_current_value = self.value;
        self.pointer_changed = false;
        self.pointer_scrubbing = false;
        self.pointer_started_focused =
            self.focus.as_ref().is_some_and(|focus| focus.is_focused(window));
        if !self.pointer_started_focused {
            window.prevent_default();
        }
        cx.notify();
    }

    fn pointer_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || !event.dragging() {
            return;
        }
        let Some(origin_x) = self.pointer_origin else {
            return;
        };
        let x = event.position.x.as_f32();
        if !self.pointer_scrubbing && (x - origin_x).abs() <= 3.0 {
            return;
        }
        // Vertical movement does not affect the numeric value; horizontal displacement
        // alone crosses the scrub threshold.
        let delta_x = x - origin_x;
        if delta_x.abs() <= 3.0 {
            return;
        }
        self.pointer_scrubbing = true;
        let mut factor = 1.0;
        if event.modifiers.shift {
            factor *= 0.1;
        }
        if event.modifiers.alt {
            factor *= 10.0;
        }
        let delta = f64::from(delta_x / self.scrub_scale) * self.step * factor;
        let target = self.pointer_start_value + delta;
        if !target.is_finite() {
            return;
        }
        let previous = self.pointer_current_value;
        let next = self.clamp(target);
        if next == previous {
            return;
        }
        if !self.controlled {
            self.value = next;
        }
        self.pointer_current_value = next;
        self.pointer_changed = next != self.pointer_start_value;
        self.draft = format_number(next);
        self.draft_active = false;
        cx.emit(ValueChanged { previous, value: next, source: ChangeSource::Pointer });
        cx.notify();
    }

    fn pointer_up(&mut self, event: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.button != MouseButton::Left || self.pointer_origin.is_none() {
            return;
        }
        self.pointer_origin = None;
        if self.pointer_changed {
            cx.emit(ValueCommitted {
                value: self.pointer_current_value,
                source: ChangeSource::Pointer,
            });
        } else if !self.pointer_scrubbing
            && !self.pointer_started_focused
            && let Some(focus) = &self.focus
        {
            window.focus(focus, cx);
        }
        self.pointer_changed = false;
        self.pointer_scrubbing = false;
        cx.notify();
    }

    fn cancel_pointer(&mut self, cx: &mut Context<Self>) {
        if self.pointer_origin.take().is_none() {
            return;
        }
        let previous = self.pointer_current_value;
        let restored = self.pointer_start_value;
        if !self.controlled {
            self.value = restored;
        }
        if previous != restored {
            cx.emit(ValueChanged { previous, value: restored, source: ChangeSource::Pointer });
        }
        self.draft = format_number(restored);
        self.pointer_current_value = restored;
        self.draft_active = false;
        self.pointer_changed = false;
        self.pointer_scrubbing = false;
        cx.notify();
    }

    /// Parse the text draft, accepting the unit as a suffix.
    fn parse_draft(&self) -> Option<f64> {
        let mut draft = self.draft.trim();
        if !self.unit.is_empty() {
            draft = draft.strip_suffix(self.unit.trim()).unwrap_or(draft).trim_end();
        }
        draft.parse::<f64>().ok().filter(|value| value.is_finite())
    }

    /// The inherited text style at the size the field renders with, so hit testing and IME
    /// bounds match the rendered text wherever the platform callback runs.
    fn text_style(&self, window: &Window) -> TextStyle {
        let mut style = window.text_style();
        if let Some(size) = self.text_size {
            style.font_size = size.into();
        }
        style
    }

    fn refresh_draft(&mut self) {
        self.draft = format_number(self.value);
        self.selection = self.draft.len()..self.draft.len();
        self.marked = None;
        self.draft_active = false;
    }

    fn commit_draft(&mut self, _: &CommitDraft, _: &mut Window, cx: &mut Context<Self>) {
        if self.marked.is_some() {
            return;
        }
        self.commit_text(cx);
    }
    fn cancel_draft(&mut self, _: &CancelDraft, _: &mut Window, cx: &mut Context<Self>) {
        if self.marked.is_some() {
            return;
        }
        if self.pointer_origin.is_some() {
            self.cancel_pointer(cx);
            return;
        }
        if self.draft_active && self.marked.is_none() {
            self.refresh_draft();
            cx.notify();
        }
    }
    fn commit_text(&mut self, cx: &mut Context<Self>) {
        if !self.draft_active || self.marked.is_some() {
            return;
        }
        if let Some(value) = self.parse_draft() {
            let previous = self.value;
            let next = self.clamp(value);
            if next != previous {
                if !self.controlled {
                    self.value = next;
                }
                cx.emit(ValueChanged { previous, value: next, source: ChangeSource::Text });
            }
            self.refresh_draft();
            cx.emit(ValueCommitted { value: next, source: ChangeSource::Text });
        } else {
            self.refresh_draft();
        }
        cx.notify();
    }
}

impl EntityInputHandler for ScrubbableNumberField {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let bytes = utf16_range_to_bytes(&self.draft, &range);
        *adjusted = Some(byte_range_to_utf16(&self.draft, &bytes));
        Some(self.draft[bytes].to_owned())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: byte_range_to_utf16(&self.draft, &self.selection),
            reversed: false,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_ref().map(|r| byte_range_to_utf16(&self.draft, r))
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
        let range = range
            .as_ref()
            .map(|r| utf16_range_to_bytes(&self.draft, r))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        let inserted = text.replace(['\n', '\r'], " ");
        self.draft.replace_range(range.clone(), &inserted);
        self.draft_active = true;
        let caret = range.start + inserted.len();
        self.selection = caret..caret;
        self.marked = None;
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
        let range = range
            .as_ref()
            .map(|r| utf16_range_to_bytes(&self.draft, r))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        let start = range.start;
        self.draft.replace_range(range, text);
        self.draft_active = true;
        self.marked = (!text.is_empty()).then_some(start..start + text.len());
        self.selection = selected
            .map(|r| {
                start + utf16_offset_to_byte(text, r.start)
                    ..start + utf16_offset_to_byte(text, r.end)
            })
            .unwrap_or(start + text.len()..start + text.len());
        cx.notify();
    }
    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let bytes = utf16_range_to_bytes(&self.draft, &range);
        let style = self.text_style(window);
        let inset = px(cx.global::<Theme>().spacing.small);
        let shaped = window.text_system().shape_line(
            self.draft.as_str().into(),
            style.font_size.to_pixels(window.rem_size()),
            &[style.to_run(self.draft.len())],
            None,
        );
        let x0 = shaped.x_for_index(bytes.start);
        let x1 = shaped.x_for_index(bytes.end);
        let line = style.line_height_in_pixels(window.rem_size());
        Some(Bounds::new(
            point(
                bounds.left() + inset + x0,
                bounds.top() + ((bounds.size.height - line) / 2.).max(px(0.)),
            ),
            size((x1 - x0).max(px(1.)), line),
        ))
    }
    fn character_index_for_point(
        &mut self,
        point: gpui_pre::Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let style = self.text_style(window);
        let inset = px(cx.global::<Theme>().spacing.small);
        let shaped = window.text_system().shape_line(
            self.draft.as_str().into(),
            style.font_size.to_pixels(window.rem_size()),
            &[style.to_run(self.draft.len())],
            None,
        );
        let x = (point.x - bounds.left() - inset).max(px(0.));
        Some(byte_to_utf16(&self.draft, shaped.closest_index_for_x(x).min(self.draft.len())))
    }
    fn set_selected_text_range(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selection = utf16_range_to_bytes(&self.draft, &range);
        self.marked = None;
        cx.notify();
    }
    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.draft.encode_utf16().count())
    }
}

impl Focusable for ScrubbableNumberField {
    fn focus_handle(&self, _: &gpui_pre::App) -> gpui_pre::FocusHandle {
        // GPUI's default entity focus handle is initialized by `cx.focus_handle()`
        // on first render, retained on the entity, and used by the element below.
        self.focus.clone().expect("focus handle initialized during render")
    }
}

impl Render for ScrubbableNumberField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        let focused = focus.is_focused(window);
        let input = cx.entity();
        let theme = *cx.global::<Theme>();
        self.text_size = Some(px(theme.typography.body));
        let look = look(&theme, self.disabled);
        let composing = self.marked.is_some();
        // Keyboard focus or IME composition draws the ring; see the spec's "Theme tokens used".
        let active = !self.disabled && (focused || composing);
        let invalid = self.draft_active && !composing && self.parse_draft().is_none();
        let scrubbing = !self.disabled && self.pointer_scrubbing;
        let ring = match look.invalid_ring {
            Some(ring) if invalid => Some(ring),
            _ => active.then_some(look.focus_ring),
        };
        let shadows = match ring {
            Some(color) => vec![focus_ring(color)],
            None if look.shadow.color.a > 0. => vec![box_shadow(look.shadow)],
            None => Vec::new(),
        };
        let border = if invalid {
            look.invalid_border
        } else if active || scrubbing {
            look.focus_border
        } else {
            look.border
        };
        let content = self.content(&theme, look, active);
        div()
            .id("scrubbable-number-field")
            .debug_selector(|| "scrubbable-number-field".to_owned())
            .key_context(KEY_CONTEXT)
            .when(!self.disabled, |element| element.track_focus(&focus))
            .role(gpui_pre::accesskit::Role::SpinButton)
            .aria_label(self.label.clone())
            .when(self.disabled, |element| {
                element.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .when(!self.description.is_empty(), |element| {
                element.aria_description(self.description.clone())
            })
            .aria_numeric_value(self.value)
            .aria_numeric_value_step(self.step)
            .when(!self.unit.is_empty(), |element| {
                element.aria_value(format!("{} {}", self.value, self.unit))
            })
            .when_some(self.min, |element, min| element.aria_min_numeric_value(min))
            .when_some(self.max, |element, max| element.aria_max_numeric_value(max))
            .on_action(cx.listener(Self::increment))
            .on_action(cx.listener(Self::decrement))
            .on_action(cx.listener(Self::precision_increment))
            .on_action(cx.listener(Self::precision_decrement))
            .on_action(cx.listener(Self::page_increment))
            .on_action(cx.listener(Self::page_decrement))
            .on_action(cx.listener(Self::set_minimum))
            .on_action(cx.listener(Self::set_maximum))
            .on_action(cx.listener(Self::commit_draft))
            .on_action(cx.listener(Self::cancel_draft))
            .when(!self.disabled, |element| {
                element.cursor(if focused && !scrubbing {
                    CursorStyle::IBeam
                } else {
                    CursorStyle::ResizeLeftRight
                })
            })
            .px(px(theme.spacing.small))
            .py(px(theme.spacing.xsmall))
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.regular))
            .border_color(border)
            .shadow(shadows)
            .bg(look.fill)
            .text_size(px(theme.typography.body))
            .text_color(look.text)
            .flex()
            .items_center()
            .h(px(theme.controls.small))
            .child(content)
            .when(!self.unit.is_empty(), |element| {
                element.child(
                    div()
                        .ml(px(theme.spacing.xsmall))
                        .text_color(look.muted)
                        .child(self.unit.clone()),
                )
            })
            .child(div().ml_auto().pl(px(theme.spacing.xsmall)).flex().items_center().child(
                scrub_affordance(
                    theme.spacing.large,
                    if scrubbing { theme.colors.accent } else { look.muted },
                ),
            ))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, (), window, cx| {
                        let down_target = input.clone();
                        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                            if phase == DispatchPhase::Capture
                                && event.button == MouseButton::Left
                                && bounds.contains(&event.position)
                            {
                                down_target
                                    .update(cx, |field, cx| field.pointer_down(event, window, cx));
                            }
                        });
                        let move_target = input.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                            if phase == DispatchPhase::Capture && event.dragging() {
                                move_target
                                    .update(cx, |field, cx| field.pointer_move(event, window, cx));
                            }
                        });
                        let up_target = input.clone();
                        window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                            if phase == DispatchPhase::Capture && event.button == MouseButton::Left
                            {
                                up_target
                                    .update(cx, |field, cx| field.pointer_up(event, window, cx));
                            }
                        });
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
            )
    }
}

impl ScrubbableNumberField {
    /// The draft with its caret, selection, or IME underline. Marks render only while the enabled
    /// field owns focus or is composing; the range is clamped to the draft.
    fn content(&self, theme: &Theme, look: Look, active: bool) -> gpui_pre::Div {
        let draft = self.draft.as_str();
        let caret = theme.colors.accent;
        let piece = |range: Range<usize>| {
            let range = clamp_range(draft, &range);
            draft[range].to_owned()
        };
        if let Some(marked) = self.marked.clone().filter(|_| active) {
            let marked = clamp_range(draft, &marked);
            return div()
                .flex()
                .items_center()
                .child(piece(0..marked.start))
                .child(
                    div()
                        .border_b(px(theme.borders.strong))
                        .border_color(caret)
                        .child(piece(marked.clone())),
                )
                .child(piece(marked.end..draft.len()));
        }
        if !active {
            return div().text_color(look.text).child(draft.to_owned());
        }
        let selection = clamp_range(draft, &self.selection);
        div()
            .flex()
            .items_center()
            .child(piece(0..selection.start))
            .child(if selection.is_empty() {
                div()
                    .w(px(theme.borders.hairline))
                    .h(px(theme.typography.heading))
                    .flex_shrink_0()
                    .bg(caret)
            } else {
                div()
                    .bg(theme.colors.accent)
                    .text_color(theme.colors.accent_text)
                    .child(piece(selection.clone()))
            })
            .child(piece(selection.end..draft.len()))
    }
}

/// Clamp a byte range to `text`, snapping both ends down to character boundaries.
fn clamp_range(text: &str, range: &Range<usize>) -> Range<usize> {
    let snap = |mut offset: usize| {
        offset = offset.min(text.len());
        while !text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    };
    let start = snap(range.start);
    start..snap(range.end).max(start)
}

fn finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() { value } else { fallback }
}

fn default_page_step(step: f64) -> f64 {
    let page_step = step * 10.0;
    if page_step.is_finite() { page_step } else { f64::MAX }
}

fn format_number(value: f64) -> String {
    value.to_string()
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
fn byte_to_utf16(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].encode_utf16().count()
}
fn utf16_range_to_bytes(text: &str, range: &Range<usize>) -> Range<usize> {
    utf16_offset_to_byte(text, range.start)..utf16_offset_to_byte(text, range.end)
}
fn byte_range_to_utf16(text: &str, range: &Range<usize>) -> Range<usize> {
    byte_to_utf16(text, range.start)..byte_to_utf16(text, range.end)
}

#[cfg(test)]
mod tests {
    use super::{ScrubbableNumberField, ValueChanged, ValueCommitted, default_key_bindings};
    use gpui_pre::{
        AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
        TestAppContext, Window, div,
    };
    use std::{cell::RefCell, rc::Rc};

    struct Harness {
        controlled: bool,
        keyboard_config: bool,
        field: Option<Entity<ScrubbableNumberField>>,
        changes: Rc<RefCell<Vec<ValueChanged>>>,
        commits: Rc<RefCell<Vec<ValueCommitted>>>,
        subscriptions: Vec<Subscription>,
    }

    impl Harness {
        fn new(controlled: bool) -> Self {
            Self {
                controlled,
                keyboard_config: false,
                field: None,
                changes: Rc::new(RefCell::new(Vec::new())),
                commits: Rc::new(RefCell::new(Vec::new())),
                subscriptions: Vec::new(),
            }
        }

        fn keyboard_config() -> Self {
            let mut harness = Self::new(false);
            harness.keyboard_config = true;
            harness
        }
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.field.is_none() {
                let controlled = self.controlled;
                let keyboard_config = self.keyboard_config;
                let field = cx.new(|_| {
                    if controlled {
                        ScrubbableNumberField::controlled(12.5)
                    } else if keyboard_config {
                        ScrubbableNumberField::new(12.5)
                            .bounds(Some(0.0), Some(100.0))
                            .step(2.0)
                            .page_step(20.0)
                            .precision_step(0.2)
                    } else {
                        ScrubbableNumberField::new(12.5).bounds(Some(0.0), Some(100.0))
                    }
                });
                let changes = self.changes.clone();
                let change_subscription = cx.subscribe(&field, move |_, _, event, _| {
                    changes.borrow_mut().push(*event);
                });
                let commits = self.commits.clone();
                let commit_subscription = cx.subscribe(&field, move |_, _, event, _| {
                    commits.borrow_mut().push(*event);
                });
                self.field = Some(field);
                self.subscriptions = vec![change_subscription, commit_subscription];
            }
            div().child(self.field.as_ref().expect("initialized").clone())
        }
    }

    #[test]
    fn rendered_ranges_clamp_to_the_draft_and_character_boundaries() {
        assert_eq!(super::clamp_range("9.5", &(4..4)), 3..3);
        assert_eq!(super::clamp_range("12.5", &(1..3)), 1..3);
        assert_eq!(super::clamp_range("é1", &(1..9)), 0..3);
        let reversed = std::ops::Range { start: 2, end: 1 };
        assert_eq!(super::clamp_range("12", &reversed), 2..2);
    }

    #[gpui_pre::test]
    fn named_step_actions_change_uncontrolled_value_and_controlled_value_waits_for_owner(
        cx: &mut TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (uncontrolled, visual) = cx.add_window_view(|_, _| Harness::new(false));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field = uncontrolled.read_with(visual, |host, _| host.field.as_ref().unwrap().clone());
        visual.update(|window, cx| field.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("up");
        assert_eq!(field.read_with(visual, |field, _| field.value()), 13.5);
        assert_eq!(field.read_with(visual, |field, _| field.draft.clone()), "13.5");
        assert_eq!(
            uncontrolled.read_with(visual, |host, _| host.changes.borrow().clone()),
            [ValueChanged { previous: 12.5, value: 13.5, source: super::ChangeSource::Keyboard }]
        );
        assert_eq!(
            uncontrolled.read_with(visual, |host, _| host.commits.borrow().clone()),
            [ValueCommitted { value: 13.5, source: super::ChangeSource::Keyboard }]
        );

        let (controlled, second_visual) = cx.add_window_view(|_, _| Harness::new(true));
        second_visual.update(|window, cx| window.draw(cx).clear(cx));
        let field =
            controlled.read_with(second_visual, |host, _| host.field.as_ref().unwrap().clone());
        second_visual.update(|window, cx| field.focus_handle(cx).focus(window, cx));
        second_visual.simulate_keystrokes("up");
        assert_eq!(field.read_with(second_visual, |field, _| field.value()), 12.5);
        assert_eq!(
            controlled.read_with(second_visual, |host, _| host.changes.borrow().clone()),
            [ValueChanged { previous: 12.5, value: 13.5, source: super::ChangeSource::Keyboard }]
        );
        assert_eq!(
            controlled.read_with(second_visual, |host, _| host.commits.borrow().clone()),
            [ValueCommitted { value: 13.5, source: super::ChangeSource::Keyboard }]
        );
        field.update(second_visual, |field, cx| field.set_value(13.5, cx));
        assert_eq!(field.read_with(second_visual, |field, _| field.value()), 13.5);
        assert_eq!(field.read_with(second_visual, |field, _| field.draft.clone()), "13.5");
    }

    #[gpui_pre::test]
    fn disabled_field_ignores_keyboard_and_pointer_scrub(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (field, visual) = cx.add_window_view(|_, _| {
            ScrubbableNumberField::new(12.5).bounds(Some(0.0), Some(100.0)).disabled(true)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let center = visual.debug_bounds("scrubbable-number-field").unwrap().center();
        visual.update(|window, cx| field.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("up");
        visual.simulate_mouse_down(center, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_move(
            center + gpui_pre::point(gpui_pre::px(8.), gpui_pre::px(0.)),
            Some(gpui_pre::MouseButton::Left),
            Default::default(),
        );
        visual.simulate_mouse_up(
            center + gpui_pre::point(gpui_pre::px(8.), gpui_pre::px(0.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );
        assert_eq!(field.read_with(visual, |field, _| field.value()), 12.5);
    }

    #[gpui_pre::test]
    fn precision_page_and_bound_actions_clamp_and_emit_only_real_changes(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, _| Harness::keyboard_config());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field = host.read_with(visual, |host, _| host.field.as_ref().unwrap().clone());
        visual.update(|window, cx| field.focus_handle(cx).focus(window, cx));

        visual.simulate_keystrokes("up shift-up shift-down pageup pagedown home end up pagedown");

        assert_eq!(field.read_with(visual, |field, _| field.value()), 80.0);
        let requested = host.read_with(visual, |host, _| {
            host.changes.borrow().iter().map(|event| event.value).collect::<Vec<_>>()
        });
        let expected = [14.5, 14.7, 14.5, 34.5, 14.5, 0.0, 100.0, 80.0];
        assert_eq!(requested.len(), expected.len());
        for (actual, expected) in requested.into_iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
        }
        assert_eq!(host.read_with(visual, |host, _| host.commits.borrow().len()), 8);
    }

    #[gpui_pre::test]
    fn arithmetic_overflow_never_commits_a_non_finite_value(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let field = ScrubbableNumberField::new(f64::MAX).step(f64::MAX);
        let (view, visual) = cx.add_window_view(|_, _| field);
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| view.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("up");
        assert_eq!(view.read_with(visual, |field, _| field.value()), f64::MAX);
    }

    #[gpui_pre::test]
    fn pointer_scrub_changes_continuously_then_commits_once_and_click_does_nothing(
        cx: &mut TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) = cx.add_window_view(|_, _| Harness::new(false));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field = host.read_with(visual, |host, _| host.field.as_ref().unwrap().clone());
        let start = visual.debug_bounds("scrubbable-number-field").unwrap().center();
        visual.simulate_mouse_down(start, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_move(
            start + gpui_pre::point(gpui_pre::px(2.), gpui_pre::px(40.)),
            Some(gpui_pre::MouseButton::Left),
            Default::default(),
        );
        assert_eq!(field.read_with(visual, |field, _| field.value()), 12.5);
        visual.simulate_mouse_move(
            start + gpui_pre::point(gpui_pre::px(4.), gpui_pre::px(40.)),
            Some(gpui_pre::MouseButton::Left),
            Default::default(),
        );
        visual.simulate_mouse_move(
            start + gpui_pre::point(gpui_pre::px(8.), gpui_pre::px(80.)),
            Some(gpui_pre::MouseButton::Left),
            Default::default(),
        );
        assert_eq!(field.read_with(visual, |field, _| field.value()), 14.5);
        visual.simulate_mouse_up(
            start + gpui_pre::point(gpui_pre::px(8.), gpui_pre::px(80.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );
        assert_eq!(host.read_with(visual, |host, _| host.changes.borrow().len()), 2);
        assert_eq!(
            host.read_with(visual, |host, _| host.commits.borrow().clone()),
            [ValueCommitted { value: 14.5, source: super::ChangeSource::Pointer }]
        );
        visual.update(|window, cx| assert!(!field.focus_handle(cx).is_focused(window)));

        let click = start;
        visual.simulate_mouse_down(click, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_up(click, gpui_pre::MouseButton::Left, Default::default());
        assert_eq!(host.read_with(visual, |host, _| host.changes.borrow().len()), 2);
        assert_eq!(host.read_with(visual, |host, _| host.commits.borrow().len()), 1);
        visual.update(|window, cx| assert!(field.focus_handle(cx).is_focused(window)));
    }

    #[gpui_pre::test]
    fn pointer_scrub_applies_modifiers_clamps_and_escape_restores_start(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, _| Harness::new(false));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field = host.read_with(visual, |host, _| host.field.as_ref().unwrap().clone());
        visual.update(|window, cx| field.focus_handle(cx).focus(window, cx));
        let start = visual.debug_bounds("scrubbable-number-field").unwrap().center();
        visual.simulate_mouse_down(start, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_move(
            start + gpui_pre::point(gpui_pre::px(4.), gpui_pre::px(0.)),
            Some(gpui_pre::MouseButton::Left),
            gpui_pre::Modifiers { shift: true, ..Default::default() },
        );
        assert!((field.read_with(visual, |field, _| field.value()) - 12.6).abs() < 1e-9);
        visual.simulate_mouse_up(
            start + gpui_pre::point(gpui_pre::px(4.), gpui_pre::px(0.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );

        visual.simulate_mouse_down(start, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_move(
            start + gpui_pre::point(gpui_pre::px(8.), gpui_pre::px(0.)),
            Some(gpui_pre::MouseButton::Left),
            gpui_pre::Modifiers { alt: true, ..Default::default() },
        );
        assert!((field.read_with(visual, |field, _| field.value()) - 32.6).abs() < 1e-9);
        visual.simulate_mouse_up(
            start + gpui_pre::point(gpui_pre::px(8.), gpui_pre::px(0.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );

        visual.simulate_mouse_down(start, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_move(
            start + gpui_pre::point(gpui_pre::px(400.), gpui_pre::px(0.)),
            Some(gpui_pre::MouseButton::Left),
            Default::default(),
        );
        assert_eq!(field.read_with(visual, |field, _| field.value()), 100.0);
        assert!(field.read_with(visual, |field, _| field.pointer_origin.is_some()));
        visual.update(|window, cx| assert!(field.focus_handle(cx).is_focused(window)));
        visual.simulate_keystrokes("escape");
        assert!((field.read_with(visual, |field, _| field.value()) - 32.6).abs() < 1e-9);
        assert_eq!(host.read_with(visual, |host, _| host.commits.borrow().len()), 2);
    }

    #[gpui_pre::test]
    fn native_text_draft_enter_commits_valid_and_escape_discards_invalid(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, _| Harness::new(false));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field = host.read_with(visual, |host, _| host.field.as_ref().unwrap().clone());
        visual.update(|window, cx| field.focus_handle(cx).focus(window, cx));
        field.update(visual, |field, _| {
            field.draft = "13.5".into();
            field.draft_active = true;
            field.marked = Some(0..4);
        });
        visual.simulate_keystrokes("enter escape");
        assert_eq!(field.read_with(visual, |field, _| field.value()), 12.5);
        assert_eq!(host.read_with(visual, |host, _| host.commits.borrow().len()), 0);
        field.update(visual, |field, _| {
            field.marked = None;
            field.select_all();
        });
        field.update(visual, |field, _| field.select_all());
        visual.simulate_input("13.5");
        assert_eq!(field.read_with(visual, |field, _| field.value()), 12.5);
        visual.simulate_keystrokes("enter");
        assert_eq!(field.read_with(visual, |field, _| field.value()), 13.5);
        assert_eq!(
            host.read_with(visual, |host, _| host.changes.borrow().last().unwrap().source),
            super::ChangeSource::Text
        );
        assert_eq!(
            host.read_with(visual, |host, _| host.commits.borrow().last().unwrap().source),
            super::ChangeSource::Text
        );

        field.update(visual, |field, _| field.select_all());
        visual.simulate_input("-");
        visual.simulate_keystrokes("enter");
        assert_eq!(field.read_with(visual, |field, _| field.value()), 13.5);
        assert_eq!(host.read_with(visual, |host, _| host.commits.borrow().len()), 1);
        field.update(visual, |field, _| {
            field.draft = "-".into();
            field.draft_active = true;
        });
        visual.simulate_keystrokes("escape");
        assert_eq!(field.read_with(visual, |field, _| field.draft_text().to_owned()), "13.5");
        assert_eq!(field.read_with(visual, |field, _| field.value()), 13.5);
    }

    #[gpui_pre::test]
    fn click_places_caret_and_unit_suffix_is_display_only_and_parseable(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let field = ScrubbableNumberField::new(12.5).unit("%").label("Opacity");
        let (view, visual) = cx.add_window_view(|_, _| field);
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let bounds = visual.debug_bounds("scrubbable-number-field").unwrap();
        let click = gpui_pre::point(bounds.left() + gpui_pre::px(9.), bounds.center().y);
        visual.simulate_mouse_down(click, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_up(click, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_input("9");
        assert!(view.read_with(visual, |field, _| field.draft_text().starts_with('9')));
        view.update(visual, |field, _| field.select_all());
        visual.simulate_input("14 %");
        visual.simulate_keystrokes("enter");
        assert_eq!(view.read_with(visual, |field, _| field.value()), 14.0);
    }
}
