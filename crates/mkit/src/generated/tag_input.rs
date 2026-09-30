//! Multi-value tag input with native TextField editing and optional suggestions.
extern crate gpui_pre as gpui;

#[cfg(feature = "mkit-mirror")]
use crate::text_field::{InputChanged as TextInputChanged, TextField};
use gpui_pre::{
    Context, Entity, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement, KeyBinding,
    KeyDownEvent, PathBuilder, Render, Rgba, Window, actions, canvas, div, point, prelude::*, px,
};
use mkit_core::{
    a11y::{AccessibilityExt, LiveRegionPriority},
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
#[cfg(not(feature = "mkit-mirror"))]
use mkit_registry_text_field::{InputChanged as TextInputChanged, TextField};
use std::rc::Rc;

/// Resolved container, chip, and suggestion colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    high_contrast: bool,
    /// Container fill; the same colour TextField fills the embedded editor with.
    fill: Rgba,
    border: Rgba,
    /// `shadows.small` with its colour adjusted for the state; transparent in high contrast.
    shadow: ShadowToken,
    focus: Rgba,
    ring: Rgba,
    invalid_border: Rgba,
    /// Ring drawn around an invalid container whether or not it is focused; `None` in high
    /// contrast, where invalid is shown by the border and focus keeps its own ring.
    invalid_ring: Option<Rgba>,
    chip_bg: Rgba,
    chip_border: Rgba,
    chip_text: Rgba,
    selected_bg: Rgba,
    selected_text: Rgba,
    danger: Rgba,
    popup_bg: Rgba,
    popup_border: Rgba,
    text: Rgba,
    active_bg: Rgba,
    active_text: Rgba,
    /// Pointer-hover fill for enabled suggestion rows; high contrast keeps rows unchanged.
    hover_bg: Option<Rgba>,
    disabled_option: Rgba,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}
/// The web preview's `opacity: .5` applied as one layer: composite over `base`, then mix 50%.
/// GPUI element opacity dims each painted part separately, so it is not used.
fn dim(color: Rgba, base: Rgba) -> Rgba {
    mix(composite(color, base), base, 0.5)
}
fn look(t: &Theme, disabled: bool) -> Look {
    let c = t.colors;
    let transparent = c.background.opacity(0.);
    if t.name == "high-contrast" {
        let (border, chip_text, danger) = if disabled {
            (c.disabled, c.disabled, c.disabled)
        } else {
            (c.border, c.text, c.danger)
        };
        return Look {
            high_contrast: true,
            fill: c.background,
            border,
            shadow: t.shadows.none,
            focus: c.focus,
            ring: c.focus,
            invalid_border: danger,
            invalid_ring: None,
            chip_bg: c.background,
            chip_border: border,
            chip_text,
            selected_bg: c.accent,
            selected_text: c.accent_text,
            danger,
            popup_bg: c.background,
            popup_border: c.border,
            text: c.text,
            active_bg: c.accent,
            active_text: c.accent_text,
            hover_bg: None,
            disabled_option: c.disabled,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn "input": the light border, or text at 15% in dark themes.
    let input = if dark { c.text.opacity(0.15) } else { c.border };
    // TextField's fill (shadcn `dark:bg-input/30`), so the embedded editor blends in.
    let fill = if dark { mix(c.text, c.background, 0.15 * 0.3) } else { c.background };
    // shadcn "secondary"/"accent"/"muted": text mixed into the background.
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    let look = Look {
        high_contrast: false,
        fill,
        border: composite(input, fill),
        shadow: t.shadows.small,
        focus: c.focus,
        ring: c.focus.opacity(0.5),
        invalid_border: c.danger,
        invalid_ring: Some(c.danger.opacity(if dark { 0.4 } else { 0.2 })),
        chip_bg: muted,
        chip_border: transparent,
        chip_text: c.text,
        selected_bg: c.accent,
        selected_text: c.accent_text,
        danger: c.danger,
        popup_bg: c.surface,
        popup_border: if dark { c.text.opacity(0.1) } else { c.border },
        text: c.text,
        active_bg: muted,
        active_text: c.text,
        hover_bg: Some(muted),
        disabled_option: dim(c.text, c.surface),
    };
    if !disabled {
        return look;
    }
    let bg = c.background;
    Look {
        fill: dim(look.fill, bg),
        border: dim(look.border, bg),
        shadow: ShadowToken { color: look.shadow.color.opacity(0.5), ..look.shadow },
        chip_bg: dim(look.chip_bg, bg),
        chip_text: dim(look.chip_text, bg),
        danger: dim(look.danger, bg),
        invalid_border: dim(look.invalid_border, bg),
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
/// shadcn/ui focus ring width, drawn outside the container.
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
/// Decorative Lucide icon drawn as a vector stroke so it stays crisp at every scale. Each
/// polyline is a list of points on a 24-unit grid; the stroke is 2 units, Lucide's default.
fn icon(size: f32, lines: &'static [&'static [(f32, f32)]], color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let origin = bounds.origin;
            let mut path = PathBuilder::stroke(unit * 2.0);
            for line in lines {
                for (i, (x, y)) in line.iter().enumerate() {
                    let at = origin + point(unit * *x, unit * *y);
                    if i == 0 { path.move_to(at) } else { path.line_to(at) }
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
}
/// Lucide `x`.
const X: &[&[(f32, f32)]] = &[&[(18., 6.), (6., 18.)], &[(6., 6.), (18., 18.)]];

pub const KEY_CONTEXT: &str = "TagInput";
actions!(
    tag_input,
    [AddTag, AddTagSeparator, RemoveOrSelectTag, RemoveSelectedTag, ClearTagSelection]
);

pub fn default_key_bindings() -> [KeyBinding; 5] {
    [
        KeyBinding::new("enter", AddTag, Some(KEY_CONTEXT)),
        KeyBinding::new(",", AddTagSeparator, Some(KEY_CONTEXT)),
        KeyBinding::new("backspace", RemoveOrSelectTag, Some(KEY_CONTEXT)),
        KeyBinding::new("delete", RemoveSelectedTag, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", ClearTagSelection, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    pub value: String,
    pub validation_message: Option<String>,
}
impl Tag {
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into(), validation_message: None }
    }
    pub fn validation_message(mut self, message: impl Into<String>) -> Self {
        self.validation_message = Some(message.into());
        self
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagsChanged {
    pub tags: Vec<Tag>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagRejectReason {
    Duplicate,
    MaximumReached,
    Empty,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagRejected {
    pub value: String,
    pub reason: TagRejectReason,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagQueryChanged {
    pub query: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
    pub value: String,
    pub disabled: bool,
}
impl Suggestion {
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into(), disabled: false }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
impl EventEmitter<TagsChanged> for TagInput {}
impl EventEmitter<TagRejected> for TagInput {}
impl EventEmitter<TagQueryChanged> for TagInput {}
type Validator = Rc<dyn Fn(&str) -> Option<String>>;

fn next_enabled_suggestion(
    suggestions: &[Suggestion],
    active: Option<usize>,
    backwards: bool,
) -> Option<usize> {
    if suggestions.is_empty() {
        return None;
    }
    for offset in 1..=suggestions.len() {
        let candidate = match (active, backwards) {
            (Some(current), false) => (current + offset) % suggestions.len(),
            (Some(current), true) => {
                (current + suggestions.len() - (offset % suggestions.len())) % suggestions.len()
            }
            (None, false) => (offset - 1) % suggestions.len(),
            (None, true) => suggestions.len() - offset,
        };
        if !suggestions[candidate].disabled {
            return Some(candidate);
        }
    }
    None
}

/// Entity-backed multi-value input. Controlled mode emits proposed tag lists to its owner.
pub struct TagInput {
    label: String,
    placeholder: String,
    tags: Vec<Tag>,
    controlled: bool,
    draft: String,
    max_tags: Option<usize>,
    suggestions: Vec<Suggestion>,
    validator: Option<Validator>,
    disabled: bool,
    selected_tag: Option<usize>,
    active_suggestion: Option<usize>,
    rejected_message: Option<String>,
    input: Option<Entity<TextField>>,
    input_subscription: Option<gpui_pre::Subscription>,
    focus: Option<FocusHandle>,
}

impl TagInput {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            placeholder: "Add a tag".into(),
            tags: Vec::new(),
            controlled: false,
            draft: String::new(),
            max_tags: None,
            suggestions: Vec::new(),
            validator: None,
            disabled: false,
            selected_tag: None,
            active_suggestion: None,
            rejected_message: None,
            input: None,
            input_subscription: None,
            focus: None,
        }
    }
    pub fn controlled(label: impl Into<String>, tags: Vec<Tag>) -> Self {
        Self { tags, controlled: true, ..Self::new(label) }
    }
    pub fn default_tags(mut self, tags: Vec<Tag>) -> Self {
        assert!(!self.controlled, "default_tags cannot be combined with controlled mode");
        self.tags = tags;
        self
    }
    pub fn default_query(mut self, query: impl Into<String>) -> Self {
        assert!(!self.controlled, "default_query cannot be combined with controlled mode");
        self.draft = query.into();
        self
    }
    #[doc(hidden)]
    pub fn with_fixture_selected_tag(mut self, index: usize) -> Self {
        self.selected_tag = (index < self.tags.len()).then_some(index);
        self
    }
    #[doc(hidden)]
    pub fn with_fixture_active_suggestion(mut self, index: usize) -> Self {
        self.active_suggestion = Some(index);
        self
    }
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }
    pub fn max_tags(mut self, maximum: usize) -> Self {
        self.max_tags = Some(maximum);
        self
    }
    pub fn suggestions(mut self, values: Vec<Suggestion>) -> Self {
        self.suggestions = values;
        self
    }
    pub fn validator(mut self, validator: impl Fn(&str) -> Option<String> + 'static) -> Self {
        self.validator = Some(Rc::new(validator));
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }
    pub fn query(&self) -> &str {
        &self.draft
    }
    pub fn is_controlled(&self) -> bool {
        self.controlled
    }
    pub fn set_tags(&mut self, tags: Vec<Tag>, cx: &mut Context<Self>) {
        self.tags = tags;
        self.selected_tag = None;
        self.rejected_message = None;
        cx.notify();
    }

    fn focus_input(&self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled
            && let Some(input) = &self.input
        {
            input.update(cx, |field, cx| field.focus_handle(cx).focus(window, cx));
        }
    }
    fn update_query(&mut self, query: String, cx: &mut Context<Self>) {
        if self.disabled || self.draft == query {
            return;
        }
        self.draft = query.clone();
        self.selected_tag = None;
        self.active_suggestion = None;
        self.rejected_message = None;
        cx.emit(TagQueryChanged { query });
        cx.notify();
    }
    fn filtered_suggestions(&self) -> Vec<Suggestion> {
        if self.draft.trim().is_empty() {
            return Vec::new();
        }
        let needle = self.draft.trim().to_lowercase();
        self.suggestions
            .iter()
            .filter(|s| s.value.to_lowercase().contains(&needle))
            .filter(|s| !self.tags.iter().any(|tag| tag.value.eq_ignore_ascii_case(&s.value)))
            .cloned()
            .collect()
    }
    fn reject(&mut self, value: String, reason: TagRejectReason, cx: &mut Context<Self>) {
        self.rejected_message = Some(
            match reason {
                TagRejectReason::Duplicate => "This tag is already present.",
                TagRejectReason::MaximumReached => "The maximum number of tags has been reached.",
                TagRejectReason::Empty => "Enter a tag value.",
            }
            .into(),
        );
        cx.emit(TagRejected { value, reason });
        cx.notify();
    }
    fn propose_tags(&mut self, tags: Vec<Tag>, cx: &mut Context<Self>) {
        if !self.controlled {
            self.tags = tags.clone();
        }
        self.selected_tag = None;
        self.rejected_message = None;
        cx.emit(TagsChanged { tags });
        cx.notify();
    }
    fn add_value(&mut self, value: &str, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let value = value.trim();
        if value.is_empty() {
            self.reject(String::new(), TagRejectReason::Empty, cx);
            return;
        }
        if self.tags.iter().any(|tag| tag.value.trim().eq_ignore_ascii_case(value)) {
            self.reject(value.into(), TagRejectReason::Duplicate, cx);
            return;
        }
        if self.max_tags.is_some_and(|max| self.tags.len() >= max) {
            self.reject(value.into(), TagRejectReason::MaximumReached, cx);
            return;
        }
        let tag = self
            .validator
            .as_ref()
            .and_then(|validate| validate(value))
            .map_or_else(|| Tag::new(value), |message| Tag::new(value).validation_message(message));
        let mut tags = self.tags.clone();
        tags.push(tag);
        self.draft.clear();
        self.active_suggestion = None;
        if let Some(input) = &self.input {
            input.update(cx, |field, cx| field.set_value("", cx));
        }
        self.propose_tags(tags, cx);
    }
    fn remove_selected(&mut self, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let Some(index) = self.selected_tag.filter(|i| *i < self.tags.len()) else { return };
        let mut tags = self.tags.clone();
        tags.remove(index);
        self.selected_tag = None;
        self.propose_tags(tags, cx);
    }
    fn handle_key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || event.keystroke.is_ime_in_progress() {
            return;
        }
        let key = event.keystroke.key.to_ascii_lowercase();
        if key == "left" && self.draft.is_empty() && !self.tags.is_empty() {
            self.selected_tag =
                Some(self.selected_tag.unwrap_or(self.tags.len()).saturating_sub(1));
            cx.notify();
        } else if key == "right" && self.selected_tag.is_some() {
            let next = self.selected_tag.unwrap() + 1;
            self.selected_tag = (next < self.tags.len()).then_some(next);
            cx.notify();
        } else if key == "down" || key == "up" {
            let suggestions = self.filtered_suggestions();
            self.active_suggestion =
                next_enabled_suggestion(&suggestions, self.active_suggestion, key == "up");
            cx.notify();
        }
    }
}

impl Focusable for TagInput {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.input
            .as_ref()
            .map(|i| i.read(cx).focus_handle(cx))
            .or_else(|| self.focus.clone())
            .expect("TagInput editor initializes during render")
    }
}

impl Render for TagInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        if self.input.is_none() {
            let label = self.label.clone();
            let placeholder = self.placeholder.clone();
            let query = self.draft.clone();
            let disabled = self.disabled;
            let input = cx.new(move |cx| {
                let mut field = TextField::new(cx)
                    .with_label(label)
                    .with_placeholder(placeholder)
                    .controlled(query);
                field.set_disabled(disabled);
                field
            });
            self.focus = Some(input.read(cx).focus_handle(cx));
            self.input_subscription =
                Some(cx.subscribe(&input, |this, _, event: &TextInputChanged, cx| {
                    this.update_query(event.0.clone(), cx)
                }));
            self.input = Some(input);
        }
        let input = self.input.as_ref().unwrap().clone();
        let entity = cx.entity();
        let selected = self.selected_tag;
        let disabled = self.disabled;
        let invalid = self.tags.iter().any(|tag| tag.validation_message.is_some())
            || self.rejected_message.is_some();
        let look = look(&theme, disabled);
        // `:focus-within`: the container shows the ring while the embedded editor owns focus.
        let focused = !disabled && input.read(cx).focus_handle(cx).is_focused(window);
        let mut root = div()
            .id(("tag-input", entity.entity_id()))
            .debug_selector(|| "tag-input".into())
            .key_context(KEY_CONTEXT)
            .on_key_down(cx.listener(Self::handle_key))
            .on_action(cx.listener(|this, _: &AddTag, window, cx| {
                let value = this
                    .active_suggestion
                    .and_then(|i| this.filtered_suggestions().get(i).cloned())
                    .filter(|suggestion| !suggestion.disabled)
                    .map(|suggestion| suggestion.value)
                    .unwrap_or_else(|| this.draft.clone());
                if !value.trim().is_empty() {
                    this.add_value(&value, cx);
                }
                this.focus_input(window, cx)
            }))
            .on_action(cx.listener(|this, _: &AddTagSeparator, window, cx| {
                let value = this.draft.clone();
                if !value.trim().is_empty() {
                    this.add_value(&value, cx);
                }
                this.focus_input(window, cx)
            }))
            .on_action(cx.listener(|this, _: &RemoveOrSelectTag, _, cx| {
                if this.draft.is_empty() {
                    if this.selected_tag.is_some() {
                        this.remove_selected(cx)
                    } else if !this.tags.is_empty() {
                        this.selected_tag = Some(this.tags.len() - 1);
                        cx.notify()
                    }
                }
            }))
            .on_action(cx.listener(|this, _: &RemoveSelectedTag, _, cx| this.remove_selected(cx)))
            .on_action(cx.listener(|this, _: &ClearTagSelection, _, cx| {
                this.selected_tag = None;
                this.active_suggestion = None;
                cx.notify()
            }))
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.label.clone())
            .when(disabled, |el| el.a11y_synthetic_children(|b| b.parent_node().set_disabled()))
            .when(invalid, |el| {
                el.a11y_synthetic_children(|b| {
                    b.parent_node().set_invalid(gpui_pre::accesskit::Invalid::True)
                })
            })
            .w_full()
            .flex()
            .flex_col()
            .gap(px(theme.spacing.small));
        let chips = self
            .tags
            .iter()
            .enumerate()
            .map(|(index, tag)| {
                let value = tag.value.clone();
                let remove = entity.clone();
                let is_selected = selected == Some(index);
                let (chip_bg, chip_text) = if is_selected {
                    (look.selected_bg, look.selected_text)
                } else {
                    (look.chip_bg, look.chip_text)
                };
                let mut chip = div()
                    .id(("tag-chip", index))
                    .role(gpui_pre::accesskit::Role::ListItem)
                    .aria_label(value.clone())
                    .aria_selected(is_selected)
                    .h(px(theme.spacing.xlarge))
                    .px(px(theme.spacing.small))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(if tag.validation_message.is_some() {
                        look.invalid_border
                    } else if is_selected && look.high_contrast {
                        look.selected_bg
                    } else {
                        look.chip_border
                    })
                    .bg(chip_bg)
                    .text_color(chip_text)
                    .text_size(px(theme.typography.caption))
                    .font_weight(FontWeight::MEDIUM)
                    .whitespace_nowrap()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(theme.spacing.xsmall))
                    .child(tag.value.clone());
                if let Some(message) = &tag.validation_message {
                    chip = chip.aria_description(message.clone()).a11y_synthetic_children(|b| {
                        b.parent_node().set_invalid(gpui_pre::accesskit::Invalid::True)
                    });
                }
                if !disabled {
                    let select = entity.clone();
                    let selector_value = value.clone();
                    chip = chip.on_click(move |_, window, cx| {
                        select.update(cx, |this, cx| {
                            this.selected_tag = Some(index);
                            cx.notify();
                            this.focus_input(window, cx);
                        });
                    });
                    chip = chip.child(
                        div()
                            .id(("tag-remove", index))
                            .debug_selector(move || format!("tag-remove-{selector_value}"))
                            .role(gpui_pre::accesskit::Role::Button)
                            .aria_label(format!("Remove {value}"))
                            .tab_stop(true)
                            .flex()
                            .items_center()
                            .child(icon(theme.typography.caption, X, chip_text))
                            .on_click(move |_, window, cx| {
                                cx.stop_propagation();
                                remove.update(cx, |this, cx| {
                                    this.selected_tag = Some(index);
                                    this.remove_selected(cx);
                                    this.focus_input(window, cx)
                                })
                            }),
                    );
                }
                if let Some(message) = &tag.validation_message {
                    chip = chip.child(
                        div()
                            .text_color(look.danger)
                            .font_weight(FontWeight::NORMAL)
                            .child(message.clone()),
                    );
                }
                chip
            })
            .collect::<Vec<_>>();
        // The ring replaces the resting shadow, as in CSS; see the spec's "Theme tokens used".
        let ring = match look.invalid_ring {
            Some(ring) if invalid => Some(ring),
            _ => focused.then_some(look.ring),
        };
        let shadows = match ring {
            Some(color) => vec![focus_ring(color)],
            None if look.shadow.color.a > 0. => vec![box_shadow(look.shadow)],
            None => Vec::new(),
        };
        // The editor is TextField clipped to its inner band: `radii.medium` is cut from every
        // side, which hides the field's own border, corners, shadow, and ring so the container
        // draws them instead; see the spec's geometry notes.
        let clip = theme.radii.medium;
        let editor = div()
            .relative()
            .flex_1()
            .min_w(px(theme.controls.large))
            .h(px(theme.controls.medium - 2. * clip))
            .overflow_hidden()
            .child(div().absolute().top(px(-clip)).left(px(-clip)).right(px(-clip)).child(input));
        root = root.child(
            div()
                .w_full()
                .min_h(px(theme.controls.medium))
                .p(px(theme.spacing.xsmall))
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(theme.spacing.xsmall))
                .border(px(theme.borders.regular))
                .border_color(if invalid {
                    look.invalid_border
                } else if focused {
                    look.focus
                } else {
                    look.border
                })
                .rounded(px(theme.radii.medium))
                .bg(look.fill)
                .shadow(shadows)
                .child(
                    div()
                        .id("tag-list")
                        .role(gpui_pre::accesskit::Role::List)
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(theme.spacing.xsmall))
                        .children(chips),
                )
                .child(editor),
        );
        let suggestions = self.filtered_suggestions();
        if !suggestions.is_empty() && !disabled {
            let active = self.active_suggestion;
            let rows = suggestions
                .into_iter()
                .enumerate()
                .map(|(index, suggestion)| {
                    let ent = entity.clone();
                    let value = suggestion.value.clone();
                    let suggestion_disabled = suggestion.disabled;
                    let is_active = active == Some(index);
                    let hover_bg = look.hover_bg.filter(|_| !suggestion_disabled && !is_active);
                    let mut option = div()
                        .id(("tag-suggestion", index))
                        .debug_selector(move || format!("tag-suggestion-{index}"))
                        .role(gpui_pre::accesskit::Role::ListBoxOption)
                        .aria_label(value.clone())
                        .aria_selected(is_active)
                        .when(is_active, |el| el.aria_active_descendant())
                        .h(px(theme.controls.small))
                        .w_full()
                        .px(px(theme.spacing.small))
                        .flex()
                        .items_center()
                        .rounded(px(theme.radii.small))
                        .when(is_active, |el| el.bg(look.active_bg))
                        .when_some(hover_bg, |el, bg| el.hover(move |s| s.bg(bg)))
                        .text_color(if suggestion.disabled {
                            look.disabled_option
                        } else if is_active {
                            look.active_text
                        } else {
                            look.text
                        })
                        .child(value.clone());
                    if suggestion_disabled {
                        option = option.a11y_synthetic_children(|builder| {
                            builder.parent_node().set_disabled()
                        });
                    }
                    option.on_click(move |_, window, cx| {
                        if suggestion_disabled {
                            return;
                        }
                        let value = value.clone();
                        ent.update(cx, |this, cx| {
                            this.add_value(&value, cx);
                            this.focus_input(window, cx)
                        })
                    })
                })
                .collect::<Vec<_>>();
            root = root.child(
                div()
                    .id("tag-suggestions")
                    .role(gpui_pre::accesskit::Role::ListBox)
                    .aria_label(format!("{} suggestions", self.label))
                    .flex()
                    .flex_col()
                    .p(px(theme.spacing.xsmall))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(look.popup_border)
                    .bg(look.popup_bg)
                    .when(!look.high_contrast, |el| {
                        el.shadow(vec![box_shadow(theme.shadows.medium)])
                    })
                    .text_size(px(theme.typography.body))
                    .children(rows),
            );
        }
        if let Some(message) = self.rejected_message.clone() {
            root = root.child(
                div()
                    .id("tag-rejected-status")
                    .a11y_role(gpui_pre::accesskit::Role::Status)
                    .a11y_live_region(LiveRegionPriority::Polite)
                    .text_color(theme.colors.danger)
                    .text_size(px(theme.typography.caption))
                    .child(message),
            );
        }
        if self.max_tags.is_some_and(|max| self.tags.len() >= max) {
            let max = self.max_tags.unwrap();
            root = root.child(
                div()
                    .id("tag-limit-status")
                    .a11y_role(gpui_pre::accesskit::Role::Status)
                    .a11y_live_region(LiveRegionPriority::Polite)
                    .text_color(theme.colors.text_muted)
                    .text_size(px(theme.typography.caption))
                    .child(format!("Maximum {max} tags")),
            );
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::TestAppContext;
    use std::{cell::RefCell, rc::Rc};

    struct Host {
        input: Option<Entity<TagInput>>,
        initial: Vec<Tag>,
        controlled: bool,
        suggestions: Vec<Suggestion>,
        max_tags: Option<usize>,
        validate_bad: bool,
        events: Rc<RefCell<Vec<Vec<String>>>>,
        _subscription: Option<gpui_pre::Subscription>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.input.is_none() {
                let initial = self.initial.clone();
                let controlled = self.controlled;
                let suggestions = self.suggestions.clone();
                let max_tags = self.max_tags;
                let validate_bad = self.validate_bad;
                let input = cx.new(move |_| {
                    let field = if controlled {
                        TagInput::controlled("Tags", initial)
                    } else {
                        TagInput::new("Tags").default_tags(initial)
                    };
                    let mut field = field.suggestions(suggestions);
                    if let Some(maximum) = max_tags {
                        field = field.max_tags(maximum);
                    }
                    if validate_bad {
                        field = field.validator(|value| {
                            (value == "bad").then(|| "Needs review.".to_owned())
                        });
                    }
                    field
                });
                let events = self.events.clone();
                let subscription = cx.subscribe(&input, move |_, _, event: &TagsChanged, _| {
                    events
                        .borrow_mut()
                        .push(event.tags.iter().map(|tag| tag.value.clone()).collect());
                });
                self.input = Some(input);
                self._subscription = Some(subscription);
            }
            div().child(self.input.as_ref().unwrap().clone())
        }
    }

    fn host(
        initial: Vec<Tag>,
        controlled: bool,
        suggestions: Vec<Suggestion>,
        events: Rc<RefCell<Vec<Vec<String>>>>,
    ) -> Host {
        Host {
            input: None,
            initial,
            controlled,
            suggestions,
            max_tags: None,
            validate_bad: false,
            events,
            _subscription: None,
        }
    }

    fn with_host(
        cx: &mut TestAppContext,
        h: Host,
        test: impl FnOnce(Entity<TagInput>, &mut gpui_pre::VisualTestContext),
    ) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, _| h);
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let input = host.read_with(&*visual, |h, _| h.input.as_ref().unwrap().clone());
        visual.update(|window, cx| input.update(cx, |field, cx| field.focus_input(window, cx)));
        test(input, visual);
    }

    #[gpui::test]
    fn enter_adds_tag_and_emits_complete_list(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        with_host(cx, host(Vec::new(), false, Vec::new(), events.clone()), |input, visual| {
            visual.simulate_keystrokes("rust enter");
            assert_eq!(input.read_with(&*visual, |field, _| field.tags()[0].value.clone()), "rust");
            assert_eq!(&*events.borrow(), &[vec!["rust".to_owned()]]);
        });
    }

    #[gpui::test]
    fn arrows_skip_disabled_suggestions_and_enter_adds_active_option(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        let suggestions = vec![
            Suggestion::new("ruby disabled").disabled(true),
            Suggestion::new("rust language"),
            Suggestion::new("rustacean"),
        ];
        with_host(cx, host(Vec::new(), false, suggestions, events.clone()), |input, visual| {
            visual.simulate_keystrokes("rust down enter");
            assert_eq!(
                input.read_with(&*visual, |field, _| field
                    .tags()
                    .iter()
                    .map(|tag| tag.value.clone())
                    .collect::<Vec<_>>()),
                vec!["rust language"]
            );
            assert_eq!(events.borrow().as_slice(), &[vec!["rust language".to_owned()]]);
        });
    }

    #[gpui::test]
    fn controlled_add_emits_proposal_until_owner_updates(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        with_host(cx, host(Vec::new(), true, Vec::new(), events.clone()), |input, visual| {
            visual.simulate_keystrokes("rust enter");
            assert!(input.read_with(&*visual, |field, _| field.tags().is_empty()));
            assert_eq!(events.borrow().as_slice(), &[vec!["rust".to_owned()]]);
            input.update(visual, |field, cx| field.set_tags(vec![Tag::new("rust")], cx));
            assert_eq!(input.read_with(&*visual, |field, _| field.tags()[0].value.clone()), "rust");
        });
    }

    #[gpui::test]
    fn backspace_selects_then_removes_tag(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        with_host(
            cx,
            host(vec![Tag::new("rust")], false, Vec::new(), events.clone()),
            |input, visual| {
                visual.simulate_keystrokes("backspace");
                assert_eq!(input.read_with(&*visual, |field, _| field.selected_tag), Some(0));
                visual.simulate_keystrokes("backspace");
                assert!(input.read_with(&*visual, |field, _| field.tags().is_empty()));
                assert_eq!(events.borrow().as_slice(), &[Vec::<String>::new()]);
            },
        );
    }

    #[gpui::test]
    fn comma_commits_without_separator(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        with_host(cx, host(Vec::new(), false, Vec::new(), events.clone()), |input, visual| {
            visual.simulate_keystrokes("rust ,");
            assert_eq!(input.read_with(&*visual, |field, _| field.tags()[0].value.clone()), "rust");
            assert_eq!(events.borrow().as_slice(), &[vec!["rust".to_owned()]]);
        });
    }

    #[gpui::test]
    fn validator_attaches_error_to_added_tag(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut h = host(Vec::new(), false, Vec::new(), events);
        h.validate_bad = true;
        with_host(cx, h, |input, visual| {
            visual.simulate_keystrokes("bad enter");
            assert_eq!(
                input.read_with(&*visual, |field, _| field.tags()[0].validation_message.clone()),
                Some("Needs review.".into())
            );
        });
    }

    #[gpui::test]
    fn max_count_rejects_addition_without_mutating_tags(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut h = host(vec![Tag::new("first")], false, Vec::new(), events.clone());
        h.max_tags = Some(1);
        with_host(cx, h, |input, visual| {
            visual.simulate_keystrokes("second enter");
            assert_eq!(input.read_with(&*visual, |field, _| field.tags().len()), 1);
            assert!(input.read_with(&*visual, |field, _| field.rejected_message.is_some()));
            assert!(events.borrow().is_empty());
        });
    }

    #[gpui::test]
    fn pointer_does_not_commit_disabled_suggestion(cx: &mut TestAppContext) {
        let events = Rc::new(RefCell::new(Vec::new()));
        let suggestions = vec![Suggestion::new("rust locked").disabled(true)];
        with_host(cx, host(Vec::new(), false, suggestions, events), |input, visual| {
            visual.simulate_keystrokes("rust");
            let bounds = visual.debug_bounds("tag-suggestion-0").expect("disabled option bounds");
            visual.simulate_click(bounds.center(), gpui_pre::Modifiers::default());
            assert!(input.read_with(&*visual, |field, _| field.tags().is_empty()));
        });
    }
}
