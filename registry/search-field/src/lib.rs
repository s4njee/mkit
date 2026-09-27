//! Theme-driven search field that delegates native text editing to TextField.
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
use std::time::Duration;

pub const KEY_CONTEXT: &str = "SearchField";
actions!(search_field, [FocusSearch, ClearSearch, ActivateClear]);

/// Default shortcut for moving focus to the query input. Hosts may rebind `FocusSearch`.
pub fn default_key_bindings() -> [KeyBinding; 4] {
    [
        KeyBinding::new("cmd-f", FocusSearch, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", ClearSearch, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", ActivateClear, Some("SearchFieldClear")),
        KeyBinding::new("space", ActivateClear, Some("SearchFieldClear")),
    ]
}

/// A user edit or clear request. In controlled mode the owner applies the requested query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchChanged {
    pub query: String,
}
impl EventEmitter<SearchChanged> for SearchField {}

/// Entity-backed search input with optional debounced change events.
pub struct SearchField {
    label: String,
    placeholder: String,
    query: String,
    draft: String,
    controlled: bool,
    result_count: Option<usize>,
    debounce: Option<Duration>,
    disabled: bool,
    input: Option<Entity<TextField>>,
    input_subscription: Option<gpui_pre::Subscription>,
    focus: Option<FocusHandle>,
    clear_focus: Option<FocusHandle>,
    generation: u64,
    pending_query: Option<String>,
}

impl SearchField {
    /// Creates an uncontrolled search field with an empty initial query.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            placeholder: "Search".into(),
            query: String::new(),
            draft: String::new(),
            controlled: false,
            result_count: None,
            debounce: None,
            disabled: false,
            input: None,
            input_subscription: None,
            focus: None,
            clear_focus: None,
            generation: 0,
            pending_query: None,
        }
    }

    /// Creates a controlled search field. Call `set_query` with the owner's authoritative value.
    pub fn controlled(label: impl Into<String>, query: impl Into<String>) -> Self {
        let query = query.into();
        Self { query: query.clone(), draft: query, controlled: true, ..Self::new(label) }
    }

    /// Sets the initial query for an uncontrolled field.
    pub fn default_query(mut self, query: impl Into<String>) -> Self {
        assert!(!self.controlled, "default_query cannot be combined with controlled mode");
        let query = query.into();
        self.query = query.clone();
        self.draft = query;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn result_count(mut self, result_count: Option<usize>) -> Self {
        self.result_count = result_count;
        self
    }

    pub fn debounce(mut self, delay: Duration) -> Self {
        self.debounce = Some(delay);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn query(&self) -> &str {
        if self.controlled { &self.query } else { &self.draft }
    }

    pub fn is_controlled(&self) -> bool {
        self.controlled
    }

    /// Applies owner state and cancels any pending debounced user change.
    pub fn set_query(&mut self, query: impl Into<String>, cx: &mut Context<Self>) {
        let query = query.into();
        self.generation = self.generation.wrapping_add(1);
        self.pending_query = None;
        self.query = query.clone();
        self.draft = query.clone();
        if let Some(input) = &self.input {
            input.update(cx, |field, cx| field.set_value(query, cx));
        }
        cx.notify();
    }

    pub fn set_result_count(&mut self, result_count: Option<usize>, cx: &mut Context<Self>) {
        self.result_count = result_count;
        cx.notify();
    }

    fn focus_input(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if let Some(input) = &self.input {
            input.update(cx, |field, cx| field.focus_handle(cx).focus(window, cx));
        }
    }

    fn request_query(&mut self, query: String, cx: &mut Context<Self>) {
        if self.disabled || self.draft == query {
            return;
        }
        self.draft = query.clone();
        if !self.controlled {
            self.query = query.clone();
        }
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.pending_query = Some(query.clone());
        cx.notify();
        if let Some(delay) = self.debounce {
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(delay).await;
                let _ = this.update(cx, |this, cx| {
                    if this.generation == generation
                        && let Some(query) = this.pending_query.take()
                    {
                        cx.emit(SearchChanged { query });
                    }
                });
            })
            .detach();
        } else {
            self.pending_query = None;
            cx.emit(SearchChanged { query });
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.draft.is_empty() {
            return;
        }
        self.draft.clear();
        if !self.controlled {
            self.query.clear();
        }
        self.generation = self.generation.wrapping_add(1);
        self.pending_query = None;
        if let Some(input) = &self.input {
            input.update(cx, |field, cx| field.set_value("", cx));
        }
        cx.emit(SearchChanged { query: String::new() });
        self.focus_input(window, cx);
        cx.notify();
    }

    fn on_focus_search(&mut self, _: &FocusSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_input(window, cx);
    }

    fn on_clear_search(&mut self, _: &ClearSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.clear(window, cx);
    }

    fn on_activate_clear(
        &mut self,
        _: &ActivateClear,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.clear(window, cx);
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key.eq_ignore_ascii_case("escape") {
            self.clear(window, cx);
        }
    }
}

impl Focusable for SearchField {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.input
            .as_ref()
            .map(|input| input.read(cx).focus_handle(cx))
            .or_else(|| self.focus.clone())
            .expect("search input focus handle initializes during render")
    }
}

impl Render for SearchField {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        if self.input.is_none() {
            let label = self.label.clone();
            let placeholder = self.placeholder.clone();
            let draft = self.draft.clone();
            let disabled = self.disabled;
            let leading_inset = theme.spacing.medium;
            let input = cx.new(move |cx| {
                let mut field = TextField::new(cx)
                    .with_label(label)
                    .with_placeholder(placeholder)
                    .with_leading_inset(leading_inset)
                    .controlled(draft);
                field.set_disabled(disabled);
                field
            });
            self.focus = Some(input.read(cx).focus_handle(cx));
            self.input_subscription =
                Some(cx.subscribe(&input, |this, _, event: &TextInputChanged, cx| {
                    this.request_query(event.0.clone(), cx);
                }));
            self.input = Some(input);
        }
        let input = self.input.as_ref().expect("initialized input").clone();
        let mut root = div()
            .id("search-field")
            .debug_selector(|| "search-field".into())
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::on_focus_search))
            .on_action(cx.listener(Self::on_clear_search))
            .on_key_down(cx.listener(Self::on_key_down))
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.label.clone())
            .when(self.disabled, |element| {
                element.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .w_full()
            .flex()
            .items_center()
            .gap(px(theme.spacing.small));
        let icon_color = theme.colors.text_muted;
        let icon_size = theme.typography.body_emphasis;
        let lens_size = icon_size * 0.76;
        let stroke = theme.borders.strong;
        root = root.child(
            div().relative().flex_1().child(input).child(
                div()
                    .id("search-field-icon")
                    .debug_selector(|| "search-field-icon".into())
                    .absolute()
                    .left(px(theme.borders.regular))
                    .top(px(theme.spacing.none))
                    .w(px(icon_size))
                    .h(px(theme.controls.medium))
                    .flex()
                    .items_center()
                    .a11y_synthetic_children(|builder| builder.parent_node().set_hidden())
                    .child(
                        div()
                            .relative()
                            .size(px(icon_size))
                            .child(
                                div()
                                    .absolute()
                                    .left(px(theme.spacing.none))
                                    .top(px(theme.spacing.none))
                                    .size(px(lens_size))
                                    .rounded(px(lens_size))
                                    .border(px(stroke))
                                    .border_color(icon_color),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .left(px(lens_size * 0.74))
                                    .top(px(lens_size * 0.74))
                                    .w(px(icon_size * 0.32))
                                    .h(px(stroke))
                                    .rounded(px(stroke))
                                    .bg(icon_color),
                            ),
                    ),
            ),
        );
        if let Some(count) = self.result_count {
            root = root.child(
                div()
                    .id("search-field-result-count")
                    .role(gpui_pre::accesskit::Role::Status)
                    .a11y_live_region(LiveRegionPriority::Polite)
                    .text_color(theme.colors.text_muted)
                    .text_size(px(theme.typography.caption))
                    .child(format!("{count} results")),
            );
        }
        if !self.draft.is_empty() && !self.disabled {
            let clear_focus = self
                .clear_focus
                .get_or_insert_with(|| cx.focus_handle().tab_stop(true))
                .clone()
                .tab_stop(true);
            root = root.child(
                div()
                    .id("search-field-clear")
                    .debug_selector(|| "search-field-clear".into())
                    .key_context("SearchFieldClear")
                    .track_focus(&clear_focus)
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label("Clear search")
                    .tab_stop(true)
                    .tab_index(0)
                    .h(px(theme.controls.small))
                    .px(px(theme.spacing.small))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(theme.radii.medium))
                    .text_color(theme.colors.text_muted)
                    .on_click(cx.listener(|this, _, window, cx| this.clear(window, cx)))
                    .on_action(cx.listener(Self::on_activate_clear))
                    .child("Clear"),
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
        search: Option<Entity<SearchField>>,
        events: Rc<RefCell<Vec<String>>>,
        initial_query: String,
        controlled: bool,
        debounce: Option<Duration>,
        _subscription: Option<gpui_pre::Subscription>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.search.is_none() {
                let query = self.initial_query.clone();
                let controlled = self.controlled;
                let debounce = self.debounce;
                let search = cx.new(move |_| {
                    let mut field = if controlled {
                        SearchField::controlled("Search files", query)
                    } else {
                        SearchField::new("Search files").default_query(query)
                    };
                    field = field.placeholder("Search files...");
                    if let Some(delay) = debounce {
                        field = field.debounce(delay);
                    }
                    field
                });
                let events = self.events.clone();
                let subscription = cx.subscribe(&search, move |_, _, event: &SearchChanged, _| {
                    events.borrow_mut().push(event.query.clone());
                });
                self.search = Some(search);
                self._subscription = Some(subscription);
            }
            div().child(self.search.as_ref().unwrap().clone())
        }
    }

    fn host(
        query: &str,
        controlled: bool,
        debounce: Option<Duration>,
        events: Rc<RefCell<Vec<String>>>,
    ) -> Host {
        Host {
            search: None,
            events,
            initial_query: query.to_owned(),
            controlled,
            debounce,
            _subscription: None,
        }
    }

    #[gpui::test]
    fn clear_and_escape_emit_once_and_restore_focus(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let events = Rc::new(RefCell::new(Vec::new()));
        let (host, visual) = cx.add_window_view({
            let events = events.clone();
            move |_, _| host("alpha", false, None, events)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field = host.read_with(visual, |host, _| host.search.as_ref().unwrap().clone());
        visual.update(|window, cx| field.update(cx, |field, cx| field.focus_input(window, cx)));
        visual.simulate_keystrokes("escape");
        assert_eq!(&*events.borrow(), &[String::new()]);
        assert!(visual.debug_bounds("search-field-clear").is_none());
    }

    #[gpui::test]
    fn native_text_input_emits_search_changed(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let events = Rc::new(RefCell::new(Vec::new()));
        let (host, visual) = cx.add_window_view({
            let events = events.clone();
            move |_, _| host("", false, None, events)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field = host.read_with(visual, |host, _| host.search.as_ref().unwrap().clone());
        visual.update(|window, cx| field.update(cx, |field, cx| field.focus_input(window, cx)));
        visual.simulate_keystrokes("cat");
        assert_eq!(field.read_with(visual, |field, _| field.query().to_owned()), "cat");
        assert_eq!(events.borrow().last().map(String::as_str), Some("cat"));
    }

    #[gpui::test]
    fn clear_button_supports_enter_and_returns_focus(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let events = Rc::new(RefCell::new(Vec::new()));
        let (host, visual) = cx.add_window_view({
            let events = events.clone();
            move |_, _| host("alpha", false, None, events)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field = host.read_with(visual, |host, _| host.search.as_ref().unwrap().clone());
        visual.update(|window, cx| {
            field.update(cx, |field, cx| {
                field.clear_focus.as_ref().expect("clear handle rendered").focus(window, cx)
            })
        });
        visual.simulate_keystrokes("enter");
        assert_eq!(&*events.borrow(), &[String::new()]);
        assert!(visual.debug_bounds("search-field-clear").is_none());
    }

    #[gpui::test]
    fn controlled_query_is_owner_state_and_clear_proposes(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let events = Rc::new(RefCell::new(Vec::new()));
        let (host, visual) = cx.add_window_view({
            let events = events.clone();
            move |_, _| host("alpha", true, None, events)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field = host.read_with(visual, |host, _| host.search.as_ref().unwrap().clone());
        visual.update(|window, cx| field.update(cx, |field, cx| field.clear(window, cx)));
        assert_eq!(field.read_with(visual, |field, _| field.query().to_owned()), "alpha");
        assert_eq!(&*events.borrow(), &[String::new()]);
    }

    #[gpui::test]
    fn debounced_input_coalesces_to_latest_query(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let events = Rc::new(RefCell::new(Vec::new()));
        let (host, visual) = cx.add_window_view({
            let events = events.clone();
            move |_, _| host("", false, Some(Duration::from_millis(10)), events)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field = host.read_with(visual, |host, _| host.search.as_ref().unwrap().clone());
        field.update(visual, |field, cx| field.request_query("c".into(), cx));
        field.update(visual, |field, cx| field.request_query("ca".into(), cx));
        field.update(visual, |field, cx| field.request_query("cat".into(), cx));
        visual.executor().advance_clock(Duration::from_millis(10));
        visual.run_until_parked();
        assert_eq!(&*events.borrow(), &["cat"]);
        assert_eq!(field.read_with(visual, |field, _| field.query().to_owned()), "cat");
    }
}
