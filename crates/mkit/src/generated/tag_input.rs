//! Multi-value tag input with native TextField editing and optional suggestions.
extern crate gpui_pre as gpui;

#[cfg(feature = "mkit-mirror")]
use crate::text_field::{InputChanged as TextInputChanged, TextField};
use gpui_pre::{
    Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, KeyDownEvent,
    Render, Window, actions, div, prelude::*, px,
};
use mkit_core::{
    a11y::{AccessibilityExt, LiveRegionPriority},
    theme::Theme,
};
#[cfg(not(feature = "mkit-mirror"))]
use mkit_registry_text_field::{InputChanged as TextInputChanged, TextField};
use std::rc::Rc;

fn active_suggestion_colors(theme: Theme) -> (gpui_pre::Rgba, gpui_pre::Rgba) {
    // Match DropdownMenu's muted active treatment for shadcn themes. Themes
    // designed for stronger contrast continue to use their semantic accent.
    let weight = match theme.name {
        "shadcn-light" => 0.04,
        "shadcn-dark" => 0.12,
        _ => return (theme.colors.accent, theme.colors.accent_text),
    };
    let text = theme.colors.text;
    // The option popup sits on elevated_surface, so derive from that token to
    // preserve visible contrast against the actual row surface in dark mode.
    let background = theme.colors.elevated_surface;
    let mix = |foreground: f32, base: f32| foreground * weight + base * (1.0 - weight);
    (
        gpui_pre::Rgba {
            r: mix(text.r, background.r),
            g: mix(text.g, background.g),
            b: mix(text.b, background.b),
            a: 1.0,
        },
        text,
    )
}

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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            .gap(px(theme.spacing.xsmall));
        let chips = self
            .tags
            .iter()
            .enumerate()
            .map(|(index, tag)| {
                let value = tag.value.clone();
                let remove = entity.clone();
                let mut chip = div()
                    .id(("tag-chip", index))
                    .role(gpui_pre::accesskit::Role::ListItem)
                    .aria_label(value.clone())
                    .aria_selected(selected == Some(index))
                    .px(px(theme.spacing.small))
                    .py(px(theme.spacing.xsmall))
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.regular))
                    .border_color(if tag.validation_message.is_some() {
                        theme.colors.danger
                    } else if selected == Some(index) {
                        theme.colors.focus
                    } else {
                        theme.colors.border
                    })
                    .bg(if selected == Some(index) {
                        theme.colors.accent
                    } else {
                        theme.colors.surface
                    })
                    .text_color(if selected == Some(index) {
                        theme.colors.accent_text
                    } else {
                        theme.colors.text
                    })
                    .flex()
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
                            .text_color(theme.colors.text_muted)
                            .child("×")
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
                            .text_color(theme.colors.danger)
                            .text_size(px(theme.typography.caption))
                            .child(message.clone()),
                    );
                }
                chip
            })
            .collect::<Vec<_>>();
        root = root.child(
            div()
                .id("tag-list")
                .role(gpui_pre::accesskit::Role::List)
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(theme.spacing.xsmall))
                .children(chips),
        );
        root = root.child(div().w_full().child(input).when(disabled, |el| el.opacity(0.6)));
        let suggestions = self.filtered_suggestions();
        if !suggestions.is_empty() && !disabled {
            let active = self.active_suggestion;
            let (active_bg, active_fg) = active_suggestion_colors(theme);
            let rows = suggestions
                .into_iter()
                .enumerate()
                .map(|(index, suggestion)| {
                    let ent = entity.clone();
                    let value = suggestion.value.clone();
                    let suggestion_disabled = suggestion.disabled;
                    let mut option = div()
                        .id(("tag-suggestion", index))
                        .debug_selector(move || format!("tag-suggestion-{index}"))
                        .role(gpui_pre::accesskit::Role::ListBoxOption)
                        .aria_label(value.clone())
                        .aria_selected(active == Some(index))
                        .when(active == Some(index), |el| el.aria_active_descendant())
                        .h(px(theme.controls.medium))
                        .px(px(theme.spacing.small))
                        .rounded(px(theme.radii.small))
                        .bg(if active == Some(index) {
                            active_bg
                        } else {
                            theme.colors.elevated_surface
                        })
                        .text_color(if suggestion.disabled {
                            theme.colors.disabled
                        } else if active == Some(index) {
                            active_fg
                        } else {
                            theme.colors.text
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
                    .p(px(theme.spacing.xsmall))
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.regular))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.elevated_surface)
                    .gap(px(theme.spacing.xsmall))
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
