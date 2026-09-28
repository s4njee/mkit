//! Stateful command palette for searching and activating caller-registered actions.

extern crate gpui_pre as gpui;

#[cfg(feature = "mkit-mirror")]
use crate::key_hint::{KeyChord, KeyHint, shortcut_label};
use gpui_pre::{
    Bounds, Context, ElementInputHandler, EntityInputHandler, EventEmitter, FocusHandle, Focusable,
    IntoElement, KeyBinding, Pixels, Render, UTF16Selection, WeakFocusHandle, Window, actions,
    canvas, div, point, prelude::*, px, size,
};
use mkit_core::theme::Theme;
#[cfg(not(feature = "mkit-mirror"))]
use mkit_registry_key_hint::{KeyChord, KeyHint, shortcut_label};
use std::ops::Range;

pub const KEY_CONTEXT: &str = "CommandPalette";
actions!(command_palette, [Next, Previous, Activate, Close, FocusSearch, FocusSearchPrevious]);

pub fn default_key_bindings() -> [KeyBinding; 6] {
    [
        KeyBinding::new("down", Next, Some(KEY_CONTEXT)),
        KeyBinding::new("up", Previous, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Activate, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", Close, Some(KEY_CONTEXT)),
        KeyBinding::new("tab", FocusSearch, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-tab", FocusSearchPrevious, Some(KEY_CONTEXT)),
    ]
}

/// Action metadata owned by the host application. `keybinding` is display-only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandAction {
    pub id: String,
    pub label: String,
    pub group: Option<String>,
    pub keywords: Vec<String>,
    pub keybinding: Option<String>,
    pub disabled: bool,
}

impl CommandAction {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            group: None,
            keywords: Vec::new(),
            keybinding: None,
            disabled: false,
        }
    }
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
    pub fn keywords(mut self, words: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.keywords = words.into_iter().map(Into::into).collect();
        self
    }
    pub fn keybinding(mut self, binding: impl Into<String>) -> Self {
        self.keybinding = Some(binding.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionActivated {
    pub id: String,
}
impl EventEmitter<ActionActivated> for CommandPalette {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenChanged(pub bool);
impl EventEmitter<OpenChanged> for CommandPalette {}

/// Entity-backed command palette. Caller is responsible for mounting it while open.
pub struct CommandPalette {
    actions: Vec<CommandAction>,
    query: String,
    active: Option<usize>,
    open: bool,
    controlled: bool,
    disabled: bool,
    focus: Option<FocusHandle>,
    last_open: bool,
    return_focus_to: Option<WeakFocusHandle>,
    selection: Range<usize>,
    marked: Option<Range<usize>>,
    last_bounds: Option<Bounds<Pixels>>,
}

impl CommandPalette {
    pub fn new(actions: Vec<CommandAction>, default_open: bool) -> Self {
        let active = actions.iter().any(|action| !action.disabled).then_some(0);
        Self {
            actions,
            query: String::new(),
            active,
            open: default_open,
            controlled: false,
            disabled: false,
            focus: None,
            last_open: false,
            return_focus_to: None,
            selection: 0..0,
            marked: None,
            last_bounds: None,
        }
    }
    pub fn controlled(actions: Vec<CommandAction>, open: bool) -> Self {
        Self { controlled: true, ..Self::new(actions, open) }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn active_action(&self) -> Option<&str> {
        self.matches().get(self.active?).map(|a| a.id.as_str())
    }
    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.open = open && !self.disabled;
        if self.open {
            self.query.clear();
            self.selection = 0..0;
            self.active = self.first_match();
        }
        cx.notify();
    }
    pub fn set_actions(&mut self, actions: Vec<CommandAction>, cx: &mut Context<Self>) {
        let active_id = self.active_action().map(str::to_owned);
        self.actions = actions;
        self.active = active_id
            .and_then(|id| self.matches().iter().position(|a| a.id == id))
            .or_else(|| self.first_match());
        cx.notify();
    }
    fn matches(&self) -> Vec<&CommandAction> {
        self.actions.iter().filter(|a| !a.disabled && fuzzy_matches(a, &self.query)).collect()
    }
    fn first_match(&self) -> Option<usize> {
        (!self.matches().is_empty()).then_some(0)
    }
    fn set_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.query = query;
        self.active = self.first_match();
        cx.notify();
    }
    fn move_active(&mut self, backwards: bool, cx: &mut Context<Self>) {
        let len = self.matches().len();
        if len > 0 {
            self.active = Some(match self.active {
                Some(i) if backwards => (i + len - 1) % len,
                Some(i) => (i + 1) % len,
                None if backwards => len - 1,
                None => 0,
            });
        }
        cx.notify();
    }
    fn close(&mut self, cx: &mut Context<Self>) {
        if self.open {
            if !self.controlled {
                self.open = false;
            }
            cx.emit(OpenChanged(false));
            cx.notify();
        }
    }
    fn activate(&mut self, cx: &mut Context<Self>) {
        if !self.open || self.disabled {
            return;
        }
        if let Some(id) = self.matches().get(self.active.unwrap_or(0)).map(|a| a.id.clone()) {
            cx.emit(ActionActivated { id });
            self.close(cx);
        }
    }
    fn on_next(&mut self, _: &Next, _: &mut Window, cx: &mut Context<Self>) {
        self.move_active(false, cx);
    }
    fn on_previous(&mut self, _: &Previous, _: &mut Window, cx: &mut Context<Self>) {
        self.move_active(true, cx);
    }
    fn on_activate(&mut self, _: &Activate, _: &mut Window, cx: &mut Context<Self>) {
        self.activate(cx);
    }
    fn on_close(&mut self, _: &Close, _: &mut Window, cx: &mut Context<Self>) {
        self.close(cx);
    }
    fn on_focus_search(&mut self, _: &FocusSearch, window: &mut Window, cx: &mut Context<Self>) {
        if self.open
            && let Some(focus) = &self.focus
        {
            focus.focus(window, cx);
        }
    }
    fn on_focus_search_previous(
        &mut self,
        _: &FocusSearchPrevious,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open
            && let Some(focus) = &self.focus
        {
            focus.focus(window, cx);
        }
    }
}

fn fuzzy_matches(action: &CommandAction, query: &str) -> bool {
    if query.trim().is_empty() {
        return true;
    }
    let mut haystack = action.label.clone();
    if let Some(group) = &action.group {
        haystack.push(' ');
        haystack.push_str(group);
    }
    for word in &action.keywords {
        haystack.push(' ');
        haystack.push_str(word);
    }
    let lowered = haystack.to_lowercase();
    let mut chars = lowered.chars();
    query.to_lowercase().chars().all(|needle| chars.by_ref().any(|ch| ch == needle))
}

impl Focusable for CommandPalette {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("command palette focus initialized during render")
    }
}

impl EntityInputHandler for CommandPalette {
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
        self.marked.as_ref().map(|r| byte_range_to_utf16(&self.query, r))
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
        if !self.open || self.disabled {
            return;
        }
        let bytes = range
            .as_ref()
            .map(|r| utf16_range_to_bytes(&self.query, r))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        let inserted = text.replace(['\n', '\r'], " ");
        let mut query = self.query.clone();
        query.replace_range(bytes.clone(), &inserted);
        let cursor = bytes.start + inserted.len();
        self.selection = cursor..cursor;
        self.marked = None;
        self.set_query(query, cx);
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.open || self.disabled {
            return;
        }
        let bytes = range
            .as_ref()
            .map(|r| utf16_range_to_bytes(&self.query, r))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection.clone());
        let start = bytes.start;
        let mut query = self.query.clone();
        query.replace_range(bytes, text);
        self.marked = (!text.is_empty()).then_some(start..start + text.len());
        self.selection = selected
            .map(|r| {
                start + utf16_offset_to_byte(text, r.start)
                    ..start + utf16_offset_to_byte(text, r.end)
            })
            .unwrap_or(start + text.len()..start + text.len());
        self.set_query(query, cx);
    }
    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        bounds: Bounds<Pixels>,
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
        let line = window.line_height();
        let y = bounds.top() + ((bounds.size.height - line) / 2.).max(px(0.));
        Some(Bounds::new(
            point(bounds.left() + px(cx.global::<Theme>().spacing.small) + start, y),
            size((end - start).max(px(1.)), line),
        ))
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
        let x = (point.x - bounds.left() - px(cx.global::<Theme>().spacing.small)).max(px(0.));
        Some(
            byte_range_to_utf16(&self.query, &{
                let b = shaped.closest_index_for_x(x).min(self.query.len());
                let b = floor_boundary(&self.query, b);
                b..b
            })
            .start,
        )
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
        self.open && !self.disabled
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
fn utf16_range_to_bytes(text: &str, r: &Range<usize>) -> Range<usize> {
    utf16_offset_to_byte(text, r.start)..utf16_offset_to_byte(text, r.end)
}
fn byte_range_to_utf16(text: &str, r: &Range<usize>) -> Range<usize> {
    text[..r.start.min(text.len())].encode_utf16().count()
        ..text[..r.end.min(text.len())].encode_utf16().count()
}
fn floor_boundary(s: &str, mut b: usize) -> usize {
    b = b.min(s.len());
    while !s.is_char_boundary(b) {
        b -= 1
    }
    b
}

impl Render for CommandPalette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focus =
            self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(1).tab_stop(true)).clone();
        if self.open && !self.last_open {
            if let Some(previous) = window.focused(cx)
                && previous != focus
            {
                self.return_focus_to = Some(previous.downgrade());
            }
            focus.focus(window, cx);
        } else if !self.open
            && self.last_open
            && let Some(previous) = self.return_focus_to.as_ref().and_then(WeakFocusHandle::upgrade)
        {
            previous.focus(window, cx);
        }
        self.last_open = self.open;
        if !self.open || self.disabled {
            return div();
        }
        let entity = cx.entity();
        let matches = self.matches();
        let active = self.active.unwrap_or(usize::MAX);
        let query = self.query.clone();
        let focused = focus.is_focused(window);
        let marked = self.marked.clone();
        let selection = self.selection.clone();
        let mut text = div().flex().items_center();
        if !focused {
            text = text.child(query.clone());
        } else if let Some(r) = marked {
            text = text
                .child(query[..r.start].to_owned())
                .child(
                    div()
                        .bg(theme.colors.accent)
                        .text_color(theme.colors.accent_text)
                        .child(query[r.clone()].to_owned()),
                )
                .child(query[r.end..].to_owned());
        } else {
            text = text.child(query[..selection.start].to_owned());
            if selection.is_empty() {
                text = text.child(
                    div()
                        .w(px(theme.borders.regular))
                        .h(window.line_height())
                        .bg(theme.colors.accent),
                );
            } else {
                text = text.child(
                    div()
                        .bg(theme.colors.accent)
                        .text_color(theme.colors.accent_text)
                        .child(query[selection.clone()].to_owned()),
                );
            }
            text = text.child(query[selection.end..].to_owned());
        }
        let field_entity = entity.clone();
        let input_focus = focus.clone();
        let field = div()
            .id("command-palette-search")
            .debug_selector(|| "command-palette-search".into())
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .role(gpui_pre::accesskit::Role::TextInput)
            .aria_label("Search commands")
            .aria_value(query.clone())
            .on_action(cx.listener(Self::on_next))
            .on_action(cx.listener(Self::on_previous))
            .on_action(cx.listener(Self::on_activate))
            .on_action(cx.listener(Self::on_close))
            .on_action(cx.listener(Self::on_focus_search))
            .on_action(cx.listener(Self::on_focus_search_previous))
            .flex()
            .items_center()
            .gap(px(theme.spacing.small))
            .h(px(theme.controls.medium))
            .px(px(theme.spacing.medium))
            .border_b(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .text_color(theme.colors.text)
            .text_size(px(theme.typography.body))
            .child(div().text_color(theme.colors.text_muted).child("⌕"))
            .child(if query.is_empty() {
                div().text_color(theme.colors.text_muted).child("Search commands...")
            } else {
                text
            })
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, (), window, cx| {
                        window.handle_input(
                            &input_focus,
                            ElementInputHandler::new(bounds, field_entity.clone()),
                            cx,
                        );
                        field_entity.update(cx, |p, _| p.last_bounds = Some(bounds));
                    },
                )
                .absolute()
                .inset_0(),
            );
        let mut list = div()
            .id("command-palette-results")
            .role(gpui_pre::accesskit::Role::ListBox)
            .p(px(theme.spacing.xsmall));
        if matches.is_empty() {
            list = list.child(
                div()
                    .px(px(theme.spacing.medium))
                    .py(px(theme.spacing.large))
                    .text_color(theme.colors.text_muted)
                    .text_size(px(theme.typography.body))
                    .child("No results found."),
            );
        } else {
            for (index, action) in matches.iter().enumerate().take(8) {
                let id = action.id.clone();
                let label = action.label.clone();
                let group = action.group.clone();
                let key = action.keybinding.clone();
                let target = entity.clone();
                let selected = index == active;
                let mut row = div()
                    .id(format!("command-{id}"))
                    .role(gpui_pre::accesskit::Role::ListBoxOption)
                    .aria_label(option_name(&label, group.as_deref(), key.as_deref()))
                    .aria_selected(selected)
                    .h(px(theme.controls.medium))
                    .w_full()
                    .px(px(theme.spacing.medium))
                    .flex()
                    .items_center()
                    .gap(px(theme.spacing.small))
                    .rounded(px(theme.radii.small))
                    .text_color(theme.colors.text);
                if selected {
                    row = row
                        .aria_active_descendant()
                        .bg(theme.colors.accent)
                        .text_color(theme.colors.accent_text);
                }
                row = row.child(div().flex_1().child(label));
                if let Some(group) = group {
                    row = row.child(
                        div()
                            .text_color(if selected {
                                theme.colors.accent_text
                            } else {
                                theme.colors.text_muted
                            })
                            .text_size(px(theme.typography.caption))
                            .child(group),
                    );
                }
                if let Some(key) = key {
                    row = row.child(
                        div()
                            .text_color(if selected {
                                theme.colors.accent_text
                            } else {
                                theme.colors.text_muted
                            })
                            .text_size(px(theme.typography.caption))
                            .child(shortcut_hint(&key)),
                    );
                }
                list = list.child(row.on_click(move |_, _, cx| {
                    target.update(cx, |p, cx| {
                        if let Some(action) = p.actions.iter().find(|a| a.id == id && !a.disabled) {
                            cx.emit(ActionActivated { id: action.id.clone() });
                            p.close(cx);
                        }
                    })
                }));
            }
        }
        div()
            .size_full()
            .flex()
            .items_start()
            .justify_center()
            .pt(px(theme.spacing.xlarge))
            .bg(theme.colors.background.opacity(0.66))
            .child(
                div()
                    .id("command-palette-dialog")
                    .debug_selector(|| "command-palette-dialog".into())
                    .role(gpui_pre::accesskit::Role::Dialog)
                    .aria_label("Command palette")
                    .a11y_synthetic_children(|builder| builder.parent_node().set_modal())
                    .key_context(KEY_CONTEXT)
                    .track_focus(&focus)
                    .w_full()
                    .max_w(px(560.))
                    .mx(px(theme.spacing.large))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.hairline))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.elevated_surface)
                    .shadow(vec![{
                        let shadow = theme.shadows.medium;
                        gpui_pre::BoxShadow {
                            color: shadow.color.into(),
                            offset: point(px(shadow.x), px(shadow.y)),
                            blur_radius: px(shadow.blur),
                            spread_radius: px(shadow.spread),
                            inset: false,
                        }
                    }])
                    .overflow_hidden()
                    .child(field)
                    .child(list),
            )
    }
}

/// Result option accessible name: label, optional group, and the KeyHint-formatted
/// keybinding, so the name matches the visible row text.
fn option_name(label: &str, group: Option<&str>, keybinding: Option<&str>) -> String {
    let mut name = label.to_owned();
    if let Some(group) = group {
        name.push(' ');
        name.push_str(group);
    }
    if let Some(keybinding) = keybinding {
        name.push(' ');
        name.push_str(&shortcut_label(keybinding));
    }
    name
}

/// Renders a display-only shortcut through KeyHint's shared platform
/// formatter, keeping text KeyHint cannot parse verbatim.
fn shortcut_hint(shortcut: &str) -> gpui_pre::AnyElement {
    match KeyChord::parse(shortcut) {
        Some(chord) => KeyHint::new(chord).inline().into_any_element(),
        None => shortcut.to_owned().into_any_element(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{AppContext, Entity, ParentElement, Subscription, TestAppContext};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn fuzzy_search_matches_subsequences_across_action_metadata() {
        let action = CommandAction::new("save-as", "Save As")
            .group("File")
            .keywords(["export", "document"])
            .keybinding("⌘⇧S");
        assert!(fuzzy_matches(&action, "svas"));
        assert!(fuzzy_matches(&action, "fie"));
        assert!(fuzzy_matches(&action, "doc"));
        assert!(!fuzzy_matches(&action, "xyz"));
        assert!(fuzzy_matches(&action, "  "));
    }

    #[test]
    fn option_names_use_the_shared_key_hint_label() {
        assert_eq!(
            option_name("Save As", Some("File"), Some("⌘⇧S")),
            format!("Save As File {}", shortcut_label("cmd-shift-s"))
        );
        assert_eq!(option_name("Open", None, Some("cmd-k cmd-o")), "Open cmd-k cmd-o");
        assert_eq!(option_name("Open", None, None), "Open");
        #[cfg(target_os = "macos")]
        assert_eq!(option_name("Save As", None, Some("⌘⇧S")), "Save As ⇧⌘S");
    }

    #[gpui_pre::test]
    fn keybindings_render_through_the_shared_key_hint(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (_, visual) = cx.add_window_view(|_, _| {
            CommandPalette::new(vec![CommandAction::new("save", "Save").keybinding("cmd-s")], true)
        });
        visual.run_until_parked();
        assert!(visual.debug_bounds("mkit-key-hint").is_some());
    }

    #[test]
    fn disabled_actions_are_not_search_results() {
        let palette = CommandPalette::new(
            vec![
                CommandAction::new("enabled", "Open").disabled(false),
                CommandAction::new("disabled", "Open recent").disabled(true),
            ],
            true,
        );
        assert_eq!(
            palette.matches().iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
            ["enabled"]
        );
        assert_eq!(palette.active_action(), Some("enabled"));
    }

    struct TestHost {
        palette: Option<Entity<CommandPalette>>,
        opener: FocusHandle,
        activated: Rc<RefCell<Vec<String>>>,
        _subscriptions: Vec<Subscription>,
    }

    impl Render for TestHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.palette.is_none() {
                let palette = cx.new(|_| {
                    CommandPalette::new(
                        vec![
                            CommandAction::new("save-file", "Save File")
                                .group("File")
                                .keybinding("⌘S"),
                            CommandAction::new("save-project", "Save Project")
                                .group("Project")
                                .keywords(["workspace"])
                                .keybinding("⌘⇧S"),
                            CommandAction::new("open-file", "Open File").group("File"),
                        ],
                        false,
                    )
                });
                let activated = self.activated.clone();
                let subscription =
                    cx.subscribe(&palette, move |_, _, event: &ActionActivated, _| {
                        activated.borrow_mut().push(event.id.clone());
                    });
                self.palette = Some(palette);
                self._subscriptions.push(subscription);
            }
            div()
                .track_focus(&self.opener)
                .tab_stop(true)
                .child("Palette test host")
                .child(self.palette.as_ref().unwrap().clone())
        }
    }

    #[gpui_pre::test]
    fn search_navigation_activation_and_escape_restore_focus(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let activated = Rc::new(RefCell::new(Vec::new()));
        let (host, visual) = cx.add_window_view({
            let activated = activated.clone();
            move |_, cx| TestHost {
                palette: None,
                opener: cx.focus_handle().tab_stop(true),
                activated,
                _subscriptions: Vec::new(),
            }
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let palette = host.read_with(visual, |host, _| host.palette.as_ref().unwrap().clone());
        let opener = host.read_with(visual, |host, _| host.opener.clone());
        visual.update(|window, cx| opener.focus(window, cx));

        palette.update(visual, |palette, cx| palette.set_open(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let search_focus = palette.read_with(visual, |palette, cx| palette.focus_handle(cx));
        assert!(visual.update(|window, _| search_focus.is_focused(window)));
        visual.simulate_keystrokes("tab");
        assert!(visual.update(|window, _| search_focus.is_focused(window)));
        visual.simulate_keystrokes("shift-tab");
        assert!(visual.update(|window, _| search_focus.is_focused(window)));

        visual.simulate_input("sav");
        assert_eq!(palette.read_with(visual, |palette, _| palette.query().to_owned()), "sav");
        assert_eq!(
            palette.read_with(visual, |palette, _| palette.active_action().map(str::to_owned)),
            Some("save-file".to_owned())
        );
        visual.simulate_keystrokes("down");
        assert_eq!(
            palette.read_with(visual, |palette, _| palette.active_action().map(str::to_owned)),
            Some("save-project".to_owned())
        );
        visual.simulate_keystrokes("enter");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(&*activated.borrow(), &["save-project"]);
        assert!(!palette.read_with(visual, |palette, _| palette.is_open()));
        assert!(visual.update(|window, _| opener.is_focused(window)));

        palette.update(visual, |palette, cx| palette.set_open(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_input("no-such-command");
        assert_eq!(palette.read_with(visual, |palette, _| palette.matches().len()), 0);
        visual.simulate_keystrokes("escape");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(!palette.read_with(visual, |palette, _| palette.is_open()));
        assert_eq!(&*activated.borrow(), &["save-project"]);
        assert!(visual.update(|window, _| opener.is_focused(window)));
    }
}
