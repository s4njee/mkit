//! Editable, theme-driven keyboard shortcut list for application settings.
extern crate gpui_pre as gpui;

#[cfg(feature = "mkit-mirror")]
use crate::key_hint::{KeyChord, KeyHint, shortcut_label};
use gpui_pre::{
    App, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyBinding, KeyDownEvent, Keystroke, Render, Window, actions, div, prelude::*, px,
};
use mkit_core::contrast::{composite, relative_luminance};
use mkit_core::theme::Theme;
#[cfg(not(feature = "mkit-mirror"))]
use mkit_registry_key_hint::{KeyChord, KeyHint, shortcut_label};
use std::{collections::BTreeMap, fs, io, path::Path};

pub const KEY_CONTEXT: &str = "MkitShortcutEditor";
actions!(shortcut_editor, [Next, Previous, BeginCapture, Clear]);

pub fn default_key_bindings() -> [KeyBinding; 4] {
    [
        KeyBinding::new("down", Next, Some(KEY_CONTEXT)),
        KeyBinding::new("up", Previous, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", BeginCapture, Some(KEY_CONTEXT)),
        KeyBinding::new("delete", Clear, Some(KEY_CONTEXT)),
    ]
}

/// An action shown in the shortcut editor. IDs must be unique and stable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutAction {
    pub id: String,
    pub label: String,
    pub category: Option<String>,
}

impl ShortcutAction {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), category: None }
    }
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionChanged(pub Option<String>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureStarted(pub String);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingChangeRequested {
    pub action_id: String,
    pub chord: Option<String>,
    pub bindings: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingChanged {
    pub action_id: String,
    pub chord: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictDetected {
    pub action_id: String,
    pub conflicting_action_id: String,
    pub chord: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveRequested(pub String);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saved(pub String);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveFailed {
    pub path: String,
    pub error: String,
}
impl EventEmitter<SelectionChanged> for ShortcutEditor {}
impl EventEmitter<CaptureStarted> for ShortcutEditor {}
impl EventEmitter<BindingChangeRequested> for ShortcutEditor {}
impl EventEmitter<BindingChanged> for ShortcutEditor {}
impl EventEmitter<ConflictDetected> for ShortcutEditor {}
impl EventEmitter<SaveRequested> for ShortcutEditor {}
impl EventEmitter<Saved> for ShortcutEditor {}
impl EventEmitter<SaveFailed> for ShortcutEditor {}

/// Stateful shortcut editor. Controlled mode treats `set_bindings` as the source of truth.
pub struct ShortcutEditor {
    actions: Vec<ShortcutAction>,
    bindings: BTreeMap<String, String>,
    controlled: bool,
    selected: Option<usize>,
    capturing: bool,
    conflict: Option<(String, String)>,
    status: String,
    focus: Option<FocusHandle>,
}

impl ShortcutEditor {
    pub fn new(actions: Vec<ShortcutAction>, bindings: BTreeMap<String, String>) -> Self {
        let selected = (!actions.is_empty()).then_some(0);
        Self {
            actions,
            bindings,
            controlled: false,
            selected,
            capturing: false,
            conflict: None,
            status: String::new(),
            focus: None,
        }
    }

    pub fn controlled(actions: Vec<ShortcutAction>, bindings: BTreeMap<String, String>) -> Self {
        Self { controlled: true, ..Self::new(actions, bindings) }
    }

    pub fn actions(&self) -> &[ShortcutAction] {
        &self.actions
    }
    pub fn bindings(&self) -> &BTreeMap<String, String> {
        &self.bindings
    }
    pub fn selected_action(&self) -> Option<&str> {
        self.actions.get(self.selected?).map(|action| action.id.as_str())
    }
    pub fn is_capturing(&self) -> bool {
        self.capturing
    }

    pub fn set_actions(&mut self, actions: Vec<ShortcutAction>, cx: &mut Context<Self>) {
        self.actions = actions;
        self.selected = self
            .selected
            .filter(|index| *index < self.actions.len())
            .or_else(|| (!self.actions.is_empty()).then_some(0));
        cx.notify();
    }

    pub fn set_bindings(&mut self, bindings: BTreeMap<String, String>, cx: &mut Context<Self>) {
        self.bindings = bindings;
        self.conflict = None;
        self.status.clear();
        cx.notify();
    }

    pub fn begin_capture(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.selected_action().map(str::to_owned) else { return };
        self.capturing = true;
        self.conflict = None;
        self.status = format!(
            "Press a key chord for {}. Escape cancels.",
            self.actions[self.selected.unwrap()].label
        );
        cx.emit(CaptureStarted(id));
        cx.notify();
    }

    pub fn set_binding(&mut self, action_id: &str, chord: Option<String>, cx: &mut Context<Self>) {
        if !self.actions.iter().any(|action| action.id == action_id) {
            return;
        }
        let mut next = self.bindings.clone();
        assign(&mut next, action_id, chord.clone());
        if self.controlled {
            cx.emit(BindingChangeRequested { action_id: action_id.into(), chord, bindings: next });
        } else {
            self.bindings = next;
            cx.emit(BindingChanged { action_id: action_id.into(), chord });
        }
        cx.notify();
    }

    pub fn save_to_path(
        &mut self,
        path: impl AsRef<Path>,
        cx: &mut Context<Self>,
    ) -> io::Result<()> {
        let path = path.as_ref();
        let display_path = path.display().to_string();
        cx.emit(SaveRequested(display_path.clone()));
        let result = fs::write(path, keymap_json(&self.bindings));
        match result {
            Ok(()) => {
                self.status = format!("Saved keymap to {display_path}");
                cx.emit(Saved(display_path));
                cx.notify();
                Ok(())
            }
            Err(error) => {
                self.status = format!("Could not save keymap: {error}");
                cx.emit(SaveFailed { path: display_path, error: error.to_string() });
                cx.notify();
                Err(error)
            }
        }
    }

    fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        let len = self.actions.len();
        if len == 0 {
            return;
        }
        let current = self.selected.unwrap_or(0) as isize;
        self.selected = Some((current + delta).rem_euclid(len as isize) as usize);
        self.conflict = None;
        cx.emit(SelectionChanged(self.selected_action().map(str::to_owned)));
        cx.notify();
    }

    fn cancel_capture(&mut self, cx: &mut Context<Self>) {
        self.capturing = false;
        self.conflict = None;
        self.status.clear();
        cx.notify();
    }

    fn capture_key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.capturing {
            return;
        }
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            self.cancel_capture(cx);
            return;
        }
        let chord = normalize_keystroke(&event.keystroke);
        if chord.is_empty() || key == "enter" {
            return;
        }
        let Some(id) = self.selected_action().map(str::to_owned) else { return };
        if let Some(other) = find_conflict(&self.bindings, &id, &chord) {
            self.conflict = Some((chord.clone(), other.clone()));
            self.status = format!(
                "{} is already assigned. Press Enter to move it here, or Escape to cancel.",
                shortcut_label(&chord)
            );
            cx.emit(ConflictDetected { action_id: id, conflicting_action_id: other, chord });
            cx.notify();
            return;
        }
        self.capturing = false;
        self.conflict = None;
        self.status = "Shortcut changed. Save to write the keymap file.".into();
        self.set_binding(&id, Some(chord), cx);
    }

    fn accept_conflict(&mut self, cx: &mut Context<Self>) {
        let Some((chord, other_id)) = self.conflict.take() else { return };
        let Some(id) = self.selected_action().map(str::to_owned) else { return };
        let mut next = self.bindings.clone();
        assign(&mut next, &other_id, None);
        assign(&mut next, &id, Some(chord.clone()));
        self.capturing = false;
        self.status = "Shortcut changed. Save to write the keymap file.".into();
        if self.controlled {
            cx.emit(BindingChangeRequested { action_id: id, chord: Some(chord), bindings: next });
        } else {
            self.bindings = next;
            cx.emit(BindingChanged { action_id: id, chord: Some(chord) });
            cx.emit(BindingChanged { action_id: other_id, chord: None });
        }
        cx.notify();
    }

    fn clear_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_action().map(str::to_owned) {
            self.status = "Shortcut cleared. Save to write the keymap file.".into();
            self.set_binding(&id, None, cx);
        }
    }
    fn next(&mut self, _: &Next, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selection(1, cx);
    }
    fn previous(&mut self, _: &Previous, _: &mut Window, cx: &mut Context<Self>) {
        self.move_selection(-1, cx);
    }
    fn start_capture(&mut self, _: &BeginCapture, _: &mut Window, cx: &mut Context<Self>) {
        if self.conflict.is_some() {
            self.accept_conflict(cx);
        } else {
            self.begin_capture(cx);
        }
    }
    fn clear_action(&mut self, _: &Clear, _: &mut Window, cx: &mut Context<Self>) {
        self.clear_selected(cx);
    }
}

impl Focusable for ShortcutEditor {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        let _ = cx;
        self.focus.clone().expect("focus initialized during render")
    }
}

impl Render for ShortcutEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(0)).clone();
        let mut root = div()
            .id("mkit-shortcut-editor")
            .track_focus(&focus)
            .key_context(KEY_CONTEXT)
            .role(gpui_pre::accesskit::Role::Grid)
            .aria_label("Keyboard shortcuts")
            .on_key_down(cx.listener(Self::capture_key))
            .on_action(cx.listener(Self::next))
            .on_action(cx.listener(Self::previous))
            .on_action(cx.listener(Self::start_capture))
            .on_action(cx.listener(Self::clear_action))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.xsmall))
            .p(px(theme.spacing.medium))
            .rounded(px(theme.radii.small))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .text_color(theme.colors.text);
        let selected_bg = selected_row_bg(&theme);
        for (index, action) in self.actions.iter().enumerate() {
            let selected = self.selected == Some(index);
            let binding = self.bindings.get(&action.id);
            let chord = binding.map_or_else(|| "Unassigned".into(), |value| shortcut_label(value));
            let conflict = self.bindings.get(&action.id).is_some_and(|value| {
                self.bindings.iter().any(|(id, other)| {
                    id != &action.id && normalize_chord(value) == normalize_chord(other)
                })
            });
            let row_label = format!(
                "{}, {}{}",
                action.label,
                chord,
                if conflict { ", shortcut conflict" } else { "" }
            );
            let id = action.id.clone();
            let mut row = div()
                .id(format!("shortcut-row-{}", action.id))
                .role(gpui_pre::accesskit::Role::Row)
                .aria_label(row_label)
                .aria_selected(selected)
                .flex()
                .items_center()
                .justify_between()
                .gap(px(theme.spacing.medium))
                .px(px(theme.spacing.small))
                .py(px(theme.spacing.xsmall))
                .relative()
                .rounded(px(theme.radii.small))
                // Flat rows like shadcn menu and command rows. Every row reserves the strong
                // border so selection never shifts content. Selection adds the accent fill and
                // keeps a strong focus-coloured bar as a non-colour cue.
                .border(px(theme.borders.strong))
                .border_color(gpui_pre::transparent_black())
                .when(selected, |el| match selected_bg {
                    Some(bg) => el.bg(bg).child(
                        div()
                            .absolute()
                            .left(px(-theme.borders.strong))
                            .top_0()
                            .bottom_0()
                            .w(px(theme.borders.strong))
                            .bg(theme.colors.focus),
                    ),
                    None => el.border_color(theme.colors.focus),
                });
            if let Some(category) = &action.category {
                row = row.child(
                    div()
                        // A fixed column so action labels line up across categories.
                        .w(px(theme.controls.large))
                        .flex_none()
                        .text_color(theme.colors.text_muted)
                        .text_size(px(theme.typography.caption))
                        .child(category.clone()),
                );
            }
            row = row.child(div().flex_1().child(action.label.clone())).child(
                div()
                    .text_color(if conflict {
                        theme.colors.danger
                    } else {
                        theme.colors.text_muted
                    })
                    .child(match binding {
                        Some(value) => shortcut_hint(value),
                        None => chord.into_any_element(),
                    }),
            );
            root = root.child(row.on_click(cx.listener(move |this, _, _, cx| {
                if let Some(index) = this.actions.iter().position(|action| action.id == id) {
                    this.selected = Some(index);
                    this.conflict = None;
                    cx.emit(SelectionChanged(Some(id.clone())));
                    this.begin_capture(cx);
                    cx.notify();
                }
            })));
        }
        if let Some((chord, _)) = &self.conflict {
            root = root.child(
                div()
                    .id("shortcut-editor-conflict-status")
                    .role(gpui_pre::accesskit::Role::Status)
                    .text_color(theme.colors.danger)
                    .child(format!(
                        "{} is already assigned. Press Enter to move it here, or Escape to cancel.",
                        shortcut_label(chord)
                    )),
            );
        } else if !self.status.is_empty() {
            root = root.child(
                div()
                    .id("shortcut-editor-status")
                    .role(gpui_pre::accesskit::Role::Status)
                    .text_color(theme.colors.text_muted)
                    .child(self.status.clone()),
            );
        }
        root
    }
}

/// shadcn's `accent`: text mixed into the background. High contrast has no fill and
/// outlines the selected row instead.
fn selected_row_bg(t: &Theme) -> Option<gpui_pre::Rgba> {
    let c = t.colors;
    if t.name == "high-contrast" {
        return None;
    }
    let dark = relative_luminance(c.background) < 0.5;
    let weight = if dark { 0.12 } else { 0.04 };
    Some(composite(
        gpui_pre::Rgba { a: weight, ..c.text },
        gpui_pre::Rgba { a: 1.0, ..c.background },
    ))
}

fn assign(bindings: &mut BTreeMap<String, String>, id: &str, chord: Option<String>) {
    if let Some(chord) = chord.filter(|chord| !chord.is_empty()) {
        bindings.insert(id.into(), chord);
    } else {
        bindings.remove(id);
    }
}

fn normalize_keystroke(value: &Keystroke) -> String {
    let mut parts = Vec::with_capacity(6);
    if value.modifiers.control {
        parts.push("ctrl".to_string());
    }
    if value.modifiers.alt {
        parts.push("alt".to_string());
    }
    if value.modifiers.shift {
        parts.push("shift".to_string());
    }
    if value.modifiers.platform {
        parts.push("cmd".to_string());
    }
    if value.modifiers.function {
        parts.push("fn".to_string());
    }
    parts.push(value.key.to_lowercase());
    parts.join("-")
}
fn normalize_chord(value: &str) -> String {
    value.trim().split('-').map(str::to_lowercase).collect::<Vec<_>>().join("-")
}

fn find_conflict(
    bindings: &BTreeMap<String, String>,
    action_id: &str,
    chord: &str,
) -> Option<String> {
    let chord = normalize_chord(chord);
    bindings
        .iter()
        .find(|(other_id, value)| other_id.as_str() != action_id && normalize_chord(value) == chord)
        .map(|(other_id, _)| other_id.clone())
}

fn keymap_json(bindings: &BTreeMap<String, String>) -> String {
    let mut output = String::from("{\n  \"version\": 1,\n  \"bindings\": {");
    for (index, (id, chord)) in bindings.iter().enumerate() {
        output.push_str(if index == 0 { "\n" } else { ",\n" });
        output.push_str("    ");
        output.push_str(&json_string(id));
        output.push_str(": ");
        output.push_str(&json_string(chord));
    }
    if !bindings.is_empty() {
        output.push('\n');
    }
    output.push_str("  }\n}\n");
    output
}

fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '\"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c < ' ' => output.push_str(&format!("\\u{:04x}", c as u32)),
            c => output.push(c),
        }
    }
    output.push('\"');
    output
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
    use gpui_pre::TestAppContext;

    #[test]
    fn chord_comparison_ignores_case_and_outer_whitespace() {
        assert_eq!(normalize_chord(" Ctrl-Shift-P "), normalize_chord("ctrl-shift-p"));
    }

    #[test]
    fn keymap_serializes_stably_and_escapes_strings() {
        let mut bindings = BTreeMap::new();
        bindings.insert("a\"ction".into(), "cmd-k".into());
        bindings.insert("z".into(), "ctrl-\\".into());
        let json = keymap_json(&bindings);
        assert!(json.contains("\\\"ction"));
        assert!(json.find("a\\\"ction").unwrap() < json.find("\"z\"").unwrap());
        assert!(json.contains("ctrl-\\\\"));
    }

    #[test]
    fn binding_assignment_removes_empty_values() {
        let mut bindings = BTreeMap::from([("save".into(), "cmd-s".into())]);
        assign(&mut bindings, "save", None);
        assert!(bindings.is_empty());
    }

    #[test]
    fn conflict_lookup_ignores_the_selected_action_and_normalizes_chords() {
        let bindings =
            BTreeMap::from([("save".into(), "CMD-S".into()), ("open".into(), "ctrl-o".into())]);
        assert_eq!(find_conflict(&bindings, "save", " cmd-s "), None);
        assert_eq!(find_conflict(&bindings, "open", "CTRL-O"), None);
        assert_eq!(find_conflict(&bindings, "open", "cmd-s"), Some("save".into()));
    }

    #[gpui_pre::test]
    fn chords_display_through_the_shared_key_hint_formatter(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (editor, visual) = cx.add_window_view(|_, _| {
            ShortcutEditor::new(
                vec![ShortcutAction::new("open", "Open"), ShortcutAction::new("save", "Save")],
                BTreeMap::from([("open".into(), "cmd-o".into()), ("save".into(), "cmd-s".into())]),
            )
        });
        visual.run_until_parked();
        assert!(visual.debug_bounds("mkit-key-hint").is_some());
        visual.update(|window, cx| editor.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("down enter cmd-o");
        let status = editor.read_with(visual, |view, _| view.status.clone());
        assert_eq!(
            status,
            format!(
                "{} is already assigned. Press Enter to move it here, or Escape to cancel.",
                shortcut_label("cmd-o")
            )
        );
        #[cfg(target_os = "macos")]
        assert!(status.starts_with("⌘O "));
        let bindings = editor.read_with(visual, |view, _| view.bindings.clone());
        assert_eq!(bindings.get("open"), Some(&"cmd-o".to_string()), "storage keeps GPUI text");
    }

    #[gpui_pre::test]
    fn keyboard_capture_resolves_conflict_and_saves_keymap(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (editor, visual) = cx.add_window_view(|_, _| {
            ShortcutEditor::new(
                vec![ShortcutAction::new("open", "Open"), ShortcutAction::new("save", "Save")],
                BTreeMap::from([("open".into(), "cmd-o".into()), ("save".into(), "cmd-s".into())]),
            )
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| editor.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("down");
        assert_eq!(
            editor.read_with(visual, |view, _| view.selected_action().map(str::to_owned)),
            Some("save".into())
        );
        visual.simulate_keystrokes("enter");
        assert!(editor.read_with(visual, |view, _| view.is_capturing()));
        visual.simulate_keystrokes("cmd-o");
        assert_eq!(
            editor.read_with(visual, |view, _| view.conflict.clone()),
            Some(("cmd-o".into(), "open".into()))
        );
        visual.simulate_keystrokes("enter");
        let bindings = editor.read_with(visual, |view, _| view.bindings.clone());
        assert_eq!(bindings.get("save"), Some(&"cmd-o".to_string()));
        assert!(!bindings.contains_key("open"));

        let path = std::env::temp_dir().join(format!(
            "mkit-shortcut-editor-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        visual.update(|_, cx| editor.update(cx, |view, cx| view.save_to_path(&path, cx).unwrap()));
        let saved = std::fs::read_to_string(&path).expect("keymap file written");
        assert!(saved.contains("\"save\": \"cmd-o\""));
        assert!(!saved.contains("\"open\": \"cmd-o\""));
        std::fs::remove_file(path).expect("remove test keymap");
    }
}
