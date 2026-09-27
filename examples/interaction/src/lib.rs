//! A small list app demonstrating GPUI pointer, focus, keymap, drag, clipboard, and menu APIs.

pub mod e4_overlay;

use gpui_kit::component::menu::{ContextMenuExt, PopupMenuItem};
use gpui_pre::{
    App, AppContext, ClipboardItem, Context, ExternalPaths, FocusHandle, IntoElement, KeyBinding,
    Menu, MenuItem, MouseButton, Render, Window, actions, div, prelude::*,
};

const KEY_CONTEXT: &str = "InteractionList";

// ANCHOR: interaction_actions
actions!(
    interaction,
    [
        MoveSelectionUp,
        MoveSelectionDown,
        ActivateSelection,
        CopySelection,
        PasteFromClipboard,
        AppendRow,
    ]
);

/// Register default bindings and a caller-selected alternate down binding.
/// The alternate must be one valid GPUI keystroke (for example, `ctrl-k`).
pub fn bind_interaction_keys(cx: &mut App, alternate_down_key: &str) -> Result<(), String> {
    let mut parts = alternate_down_key.split_whitespace();
    let key = parts.next().ok_or_else(|| "alternate binding cannot be empty".to_owned())?;
    if parts.next().is_some() {
        return Err("alternate binding must be one keystroke".to_owned());
    }
    gpui_pre::Keystroke::parse(key)
        .map_err(|error| format!("invalid alternate binding `{key}`: {error}"))?;
    cx.bind_keys([
        KeyBinding::new("up", MoveSelectionUp, Some(KEY_CONTEXT)),
        KeyBinding::new("down", MoveSelectionDown, Some(KEY_CONTEXT)),
        KeyBinding::new(key, MoveSelectionDown, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", ActivateSelection, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-c", CopySelection, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-v", PasteFromClipboard, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-n", AppendRow, Some(KEY_CONTEXT)),
    ]);
    Ok(())
}
// ANCHOR_END: interaction_actions

// ANCHOR: interaction_list
/// Stateful list view. Selection and interaction state remain in its GPUI entity.
pub struct InteractionList {
    rows: Vec<String>,
    selected: usize,
    activated: u32,
    clicked: u32,
    hovered_row: Option<usize>,
    hover_enters: u32,
    pressed: bool,
    internal_drops: u32,
    external_paths: Vec<String>,
    external_drop_submitted: bool,
    clipboard_preview: String,
    alternate_down_key: String,
    focus_handle: Option<FocusHandle>,
    help_focus_handle: Option<FocusHandle>,
}

impl Default for InteractionList {
    fn default() -> Self {
        Self {
            rows: vec!["First row".into(), "Second row".into(), "Third row".into()],
            selected: 0,
            activated: 0,
            clicked: 0,
            hovered_row: None,
            hover_enters: 0,
            pressed: false,
            internal_drops: 0,
            external_paths: Vec::new(),
            external_drop_submitted: false,
            clipboard_preview: "(empty)".into(),
            alternate_down_key: "alt-j".into(),
            focus_handle: None,
            help_focus_handle: None,
        }
    }
}

impl InteractionList {
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn with_alternate_down_key(key: impl Into<String>) -> Self {
        Self { alternate_down_key: key.into(), ..Self::default() }
    }
    fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn move_down(&mut self) {
        self.selected = (self.selected + 1).min(self.rows.len().saturating_sub(1));
    }

    fn activate(&mut self) {
        self.activated += 1;
    }

    // ANCHOR: interaction_drag_drop
    fn reorder(&mut self, from: usize, to: usize) {
        if from >= self.rows.len() || to >= self.rows.len() || from == to {
            return;
        }
        let row = self.rows.remove(from);
        self.rows.insert(to, row);
        self.selected = to;
        self.internal_drops += 1;
    }
    // ANCHOR_END: interaction_drag_drop

    // ANCHOR: interaction_clipboard
    fn copy_selected(&mut self, cx: &mut App) {
        let Some(row) = self.rows.get(self.selected).cloned() else { return };
        cx.write_to_clipboard(ClipboardItem::new_string(row.clone()));
        self.clipboard_preview = format!("Copied: {row}");
    }

    fn paste_clipboard(&mut self, cx: &mut App) {
        self.clipboard_preview = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .unwrap_or_else(|| "(clipboard has no text)".into());
    }
    // ANCHOR_END: interaction_clipboard

    fn append_row(&mut self) {
        self.rows.push(format!("New row {}", self.rows.len() + 1));
        self.selected = self.rows.len() - 1;
    }
}

impl Render for InteractionList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // ANCHOR: interaction_focus
        if self.focus_handle.is_none() {
            let handle = cx.focus_handle();
            window.focus(&handle, cx);
            self.focus_handle = Some(handle);
            self.help_focus_handle = Some(cx.focus_handle().tab_stop(true));
        }
        // ANCHOR_END: interaction_focus

        let colors = cx.global::<gpui_kit::component::Theme>().colors;
        let focus_handle = self.focus_handle.as_ref().expect("focus initialized").clone();

        // ANCHOR: interaction_pointer
        let mut rows = div().flex().flex_col().gap_1();
        for (index, label) in self.rows.iter().cloned().enumerate() {
            let selected = index == self.selected;
            let row_colors = colors;
            let selector = format!("row-{index}");
            let row = div()
                .id(selector.clone())
                .debug_selector(move || selector.clone())
                .role(gpui_pre::Role::ListBoxOption)
                .aria_label(label.clone())
                .aria_selected(selected)
                .px_3()
                .py_2()
                .rounded_md()
                .bg(if selected { row_colors.primary } else { row_colors.background })
                .text_color(if selected {
                    row_colors.primary_foreground
                } else {
                    row_colors.foreground
                })
                .hover({
                    let color = row_colors.list_hover;
                    move |style| style.bg(color)
                })
                .active({
                    let color = row_colors.primary_active;
                    move |style| style.bg(color)
                })
                .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                    if *hovered {
                        this.hovered_row = Some(index);
                        this.hover_enters += 1;
                    } else if this.hovered_row == Some(index) {
                        this.hovered_row = None;
                    }
                    cx.notify();
                }))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.pressed = true;
                        cx.notify();
                    }),
                )
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.pressed = false;
                        cx.notify();
                    }),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.selected = index;
                    this.clicked += 1;
                    cx.notify();
                }))
                .on_drag(index, |_, _, _, cx| cx.new(|_| gpui_pre::Empty))
                .on_drop(cx.listener(move |this, from: &usize, _, cx| {
                    this.reorder(*from, index);
                    cx.notify();
                }))
                .context_menu(move |menu, _, _| {
                    menu.item(
                        PopupMenuItem::new("Activate selected row")
                            .action(Box::new(ActivateSelection)),
                    )
                })
                .child(label);
            rows = rows.child(row);
        }
        // ANCHOR_END: interaction_pointer

        // This target reports file drag exit as a distinct lifecycle signal.
        let file_drop_exit = div()
            .id("file-drop-target")
            .debug_selector(|| "file-drop-target".into())
            .px_3()
            .py_2()
            .border_1()
            .border_color(colors.border)
            .on_file_drop_exit(cx.listener(|this, _, _, cx| {
                this.external_paths.clear();
                this.external_drop_submitted = false;
                cx.notify();
            }))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                this.external_paths = paths
                    .paths()
                    .iter()
                    .filter_map(|path| path.file_name())
                    .map(|name| name.to_string_lossy().into_owned())
                    .collect();
                this.external_drop_submitted = true;
                cx.notify();
            }))
            .child(if self.external_drop_submitted {
                format!("OS drop received: {}", self.external_paths.join(", "))
            } else if self.external_paths.is_empty() {
                "Drop files here".to_owned()
            } else {
                format!("OS drag: {}", self.external_paths.join(", "))
            });

        let selected = self.rows.get(self.selected).cloned().unwrap_or_default();
        let hovered = self.hovered_row.map_or_else(|| "none".into(), |index| index.to_string());
        div()
            .id("interaction-list")
            .size_full()
            .p_5()
            .flex()
            .flex_col()
            .gap_3()
            .bg(colors.background)
            .text_color(colors.foreground)
            .key_context(KEY_CONTEXT)
            .track_focus(&focus_handle)
            .tab_group()
            .focus_visible({
                let color = colors.ring;
                move |style| style.border_2().border_color(color)
            })
            .role(gpui_pre::Role::ListBox)
            .aria_label("Keyboard driven list")
            .on_action(cx.listener(|this, _: &MoveSelectionUp, _, cx| {
                this.move_up();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &MoveSelectionDown, _, cx| {
                this.move_down();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ActivateSelection, _, cx| {
                this.activate();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &CopySelection, _, cx| {
                this.copy_selected(cx);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &PasteFromClipboard, _, cx| {
                this.paste_clipboard(cx);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &AppendRow, _, cx| {
                this.append_row();
                cx.notify();
            }))
            .child(div().text_2xl().child("Interaction list"))
            .child(div().child(format!(
                "↑ / ↓ move · {} also moves down · Enter activates · ⌘C / ⌘V copies and pastes · ⌘N adds",
                self.alternate_down_key
            )))
            .child(rows)
            .child(
                div()
                    .id("keyboard-help")
                    .debug_selector(|| "keyboard-help".into())
                    .track_focus(self.help_focus_handle.as_ref().expect("help focus initialized"))
                    .focus_visible({
                        let color = colors.ring;
                        move |style| style.border_2().border_color(color)
                    })
                    .role(gpui_pre::Role::Button)
                    .aria_label("Keyboard help focus target")
                    .px_2()
                    .py_1()
                    .child("Keyboard help"),
            )
            .child(file_drop_exit)
            .child(div().child(format!("Selected: {selected} · Activated: {} · Clicks: {}", self.activated, self.clicked)))
            .child(div().child(format!("Hover row: {hovered} · Hover entries: {} · Pressed: {} · Internal drops: {}", self.hover_enters, self.pressed, self.internal_drops)))
            .child(div().child(format!("Clipboard: {}", self.clipboard_preview)))
    }
}
// ANCHOR_END: interaction_list

// ANCHOR: interaction_menus
/// Register the desktop menu; the row context menu is attached in the list render above.
pub fn set_interaction_menus(cx: &mut App) {
    cx.set_menus([Menu::new("List").items([
        MenuItem::action("Activate selected row", ActivateSelection),
        MenuItem::action("Add row", AppendRow),
    ])]);
}
// ANCHOR_END: interaction_menus

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{ExternalPaths, FileDropEvent, Modifiers, MouseDownEvent, TestAppContext};
    use std::path::PathBuf;

    fn setup(cx: &mut TestAppContext, alternate: &str) {
        cx.update(gpui_kit::init);
        cx.update(|app| bind_interaction_keys(app, alternate).expect("test binding is valid"));
        let menu_count = cx.update(|app| {
            set_interaction_menus(app);
            app.get_menus().map_or(0, |menus| menus.len())
        });
        assert_eq!(menu_count, 1, "application menu is registered");
    }

    // ANCHOR: interaction_keyboard_test
    #[gpui_pre::test]
    fn caller_supplied_alternate_binding_dispatches_the_typed_action(cx: &mut TestAppContext) {
        setup(cx, "ctrl-k");
        let (view, visual) =
            cx.add_window_view(|_, _| InteractionList::with_alternate_down_key("ctrl-k"));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("down ctrl-k enter");
        let state = view.read_with(visual, |list, _| {
            (list.selected, list.activated, list.alternate_down_key.clone())
        });
        assert_eq!(state, (2, 1, "ctrl-k".into()));
    }
    // ANCHOR_END: interaction_keyboard_test

    #[gpui_pre::test]
    fn focus_handle_tab_next_reaches_second_tab_stop(cx: &mut TestAppContext) {
        setup(cx, "alt-j");
        let (view, visual) = cx.add_window_view(|_, _| InteractionList::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(
            visual.update(|window, cx| view
                .read(cx)
                .focus_handle
                .as_ref()
                .unwrap()
                .is_focused(window)),
            "the list receives initial focus"
        );
        visual.update(|window, cx| window.focus_next(cx));
        assert!(
            visual.update(|window, cx| view
                .read(cx)
                .help_focus_handle
                .as_ref()
                .unwrap()
                .is_focused(window)),
            "Tab moves focus to the second tab stop"
        );
    }

    #[gpui_pre::test]
    fn pointer_hit_testing_routes_hover_click_and_active_callbacks(cx: &mut TestAppContext) {
        setup(cx, "alt-j");
        let (view, visual) = cx.add_window_view(|_, _| InteractionList::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let row = visual.debug_bounds("row-1").expect("second row is rendered");
        visual.simulate_mouse_move(row.center(), None, Modifiers::default());
        visual.simulate_mouse_down(row.center(), gpui_pre::MouseButton::Left, Modifiers::default());
        assert!(view.read_with(visual, |list, _| list.pressed));
        visual.simulate_mouse_up(row.center(), gpui_pre::MouseButton::Left, Modifiers::default());
        let state = view.read_with(visual, |list, _| {
            (list.selected, list.clicked, list.hover_enters, list.pressed)
        });
        assert_eq!(state, (1, 1, 1, false));
    }

    #[gpui_pre::test]
    fn internal_drag_reorders_rows_after_a_hit_tested_drop(cx: &mut TestAppContext) {
        setup(cx, "alt-j");
        let (view, visual) = cx.add_window_view(|_, _| InteractionList::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let source = visual.debug_bounds("row-0").unwrap().center();
        let target = visual.debug_bounds("row-1").unwrap().center();
        visual.simulate_mouse_down(source, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(target, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(target, MouseButton::Left, Modifiers::default());

        let result = view.read_with(visual, |list, _| (list.rows[1].clone(), list.internal_drops));
        assert_eq!(result, ("First row".into(), 1));
    }

    #[gpui_pre::test]
    fn synthetic_platform_file_drop_event_records_path_and_submit(cx: &mut TestAppContext) {
        setup(cx, "alt-j");
        let (view, visual) = cx.add_window_view(|_, _| InteractionList::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let target = visual.debug_bounds("file-drop-target").unwrap().center();
        visual.simulate_event(FileDropEvent::Entered {
            position: target,
            paths: ExternalPaths([PathBuf::from("/tmp/example.txt")].into_iter().collect()),
        });
        visual.simulate_event(FileDropEvent::Submit { position: target });

        let result = view.read_with(visual, |list, _| {
            (list.external_paths.clone(), list.external_drop_submitted)
        });
        assert_eq!(result, (vec!["example.txt".into()], true));
    }

    #[gpui_pre::test]
    fn context_menu_keyboard_confirm_dispatches_activate_action(cx: &mut TestAppContext) {
        setup(cx, "alt-j");
        let (view, visual) = cx.add_window_view(|_, _| InteractionList::default());
        visual.run_until_parked();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let row = visual.debug_bounds("row-0").expect("first row is rendered");
        visual.simulate_event(MouseDownEvent {
            button: MouseButton::Right,
            position: row.center(),
            modifiers: Modifiers::default(),
            click_count: 1,
            first_mouse: false,
        });
        visual.run_until_parked();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("down");
        assert_eq!(
            view.read_with(visual, |list, _| list.selected),
            0,
            "the open popup consumes Down instead of moving list selection"
        );
        visual.simulate_keystrokes("enter");
        visual.run_until_parked();
        assert_eq!(view.read_with(visual, |list, _| (list.selected, list.activated)), (0, 1));
    }

    #[gpui_pre::test]
    fn clipboard_actions_roundtrip_text_and_handle_empty_clipboard(cx: &mut TestAppContext) {
        setup(cx, "alt-j");
        let (view, visual) = cx.add_window_view(|_, _| InteractionList::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("cmd-v");
        assert_eq!(
            view.read_with(visual, |list, _| list.clipboard_preview.clone()),
            "(clipboard has no text)"
        );
        visual.write_to_clipboard(ClipboardItem::new_string("pasted text".into()));
        visual.simulate_keystrokes("cmd-c cmd-v");
        assert_eq!(visual.read_from_clipboard().unwrap().text(), Some("First row".into()));
        assert_eq!(view.read_with(visual, |list, _| list.clipboard_preview.clone()), "First row");
        visual.write_to_clipboard(ClipboardItem::new_string(String::new()));
        visual.simulate_keystrokes("cmd-v");
        assert_eq!(
            view.read_with(visual, |list, _| list.clipboard_preview.clone()),
            "(clipboard has no text)"
        );
    }
}
