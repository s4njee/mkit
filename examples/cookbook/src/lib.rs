//! Short, compiling recipes for common GPUI application tasks.
//!
//! Each `recipe_*` anchor surrounds a small implementation that uses the pinned
//! GPUI API. Native tray icons are not exposed by gpui-pre 0.3.5; the supported
//! dock-menu alternative is shown instead.

use gpui_pre::{
    App, AppContext, AsyncApp, Context, Entity, FocusHandle, Global, IntoElement, KeyBinding, Menu,
    MenuItem, Render, ScrollHandle, Task, Window, WindowOptions, actions, div, prelude::*, px,
    size,
};
use std::{
    collections::VecDeque,
    fs, io,
    path::{Path, PathBuf},
    time::Duration,
};

const COOKBOOK_CONTEXT: &str = "Cookbook";

// ANCHOR: recipe_entity_counter
#[derive(Default)]
pub struct Counter {
    value: u32,
}

impl Counter {
    pub fn increment(&mut self, cx: &mut Context<Self>) {
        self.value += 1;
        cx.emit(CounterChanged(self.value));
        cx.notify();
    }
}
// ANCHOR_END: recipe_entity_counter

// ANCHOR: recipe_typed_event
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CounterChanged(pub u32);

impl gpui_pre::EventEmitter<CounterChanged> for Counter {}
// ANCHOR_END: recipe_typed_event

// ANCHOR: recipe_observe_entity
#[derive(Default)]
pub struct CounterDemo {
    source: Option<Entity<Counter>>,
    observed: u32,
    event_value: u32,
    _observation: Option<gpui_pre::Subscription>,
    _event_subscription: Option<gpui_pre::Subscription>,
}

impl CounterDemo {
    fn initialize(&mut self, cx: &mut Context<Self>) {
        if self.source.is_some() {
            return;
        }
        let source = cx.new(|_| Counter::default());
        // Keep the subscription so the observer remains active for the entity lifetime.
        let observation = cx.observe(&source, |this, source, cx| {
            this.observed = source.read(cx).value;
            cx.notify();
        });
        let event_subscription = cx.subscribe(&source, |this, _, event, cx| {
            this.event_value = event.0;
            cx.notify();
        });
        self.source = Some(source);
        self._observation = Some(observation);
        self._event_subscription = Some(event_subscription);
    }
}

impl Render for CounterDemo {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.initialize(cx);
        let source = self.source.as_ref().expect("initialized counter").clone();
        let colors = cx.global::<gpui_kit::component::Theme>().colors;
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_3()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(
                div().child(format!("Observed: {} · event: {}", self.observed, self.event_value)),
            )
            .child(
                div()
                    .id("increment-counter")
                    .debug_selector(|| "increment-counter".into())
                    .px_2()
                    .py_1()
                    .on_click(move |_, _, cx| source.update(cx, Counter::increment))
                    .child("Increment counter"),
            )
    }
}
// ANCHOR_END: recipe_observe_entity

// ANCHOR: recipe_shared_global
#[derive(Clone)]
pub struct SharedLabel(pub String);

impl Global for SharedLabel {}

pub fn install_shared_label(app: &mut App, label: impl Into<String>) {
    app.set_global(SharedLabel(label.into()));
}
// ANCHOR_END: recipe_shared_global

// ANCHOR: recipe_read_global
pub fn global_label(cx: &App) -> String {
    cx.global::<SharedLabel>().0.clone()
}
// ANCHOR_END: recipe_read_global

// ANCHOR: recipe_stable_ids
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub id: u64,
    pub title: String,
}

pub fn update_row_title(rows: &mut [Row], id: u64, title: String) -> bool {
    let Some(row) = rows.iter_mut().find(|row| row.id == id) else { return false };
    row.title = title;
    true
}
// ANCHOR_END: recipe_stable_ids

// ANCHOR: recipe_derived_state
pub fn completed_count(rows: &[Row], completed: &[u64]) -> usize {
    rows.iter().filter(|row| completed.contains(&row.id)).count()
}
// ANCHOR_END: recipe_derived_state

// ANCHOR: recipe_foreground_executor
pub fn foreground_work(cx: &App) -> Task<&'static str> {
    cx.foreground_executor().spawn(async { "small UI-adjacent work" })
}
// ANCHOR_END: recipe_foreground_executor

// ANCHOR: recipe_background_executor
pub fn background_work(cx: &App) -> Task<Vec<u64>> {
    cx.background_executor().spawn(async { (0..100).map(|n| n * n).collect() })
}
// ANCHOR_END: recipe_background_executor

// ANCHOR: recipe_timer
pub fn delay(cx: &App, duration: Duration) -> Task<()> {
    cx.background_executor().timer(duration)
}
// ANCHOR_END: recipe_timer

// ANCHOR: recipe_cancel_task
/// Dropping a task cancels its future when no other task handle owns it.
pub fn cancel_previous(previous: &mut Option<Task<()>>) {
    drop(previous.take());
}
// ANCHOR_END: recipe_cancel_task

// ANCHOR: recipe_debounce_generation
#[derive(Default)]
pub struct Debounce {
    generation: u64,
}

impl Debounce {
    pub fn begin(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.generation
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.generation == generation
    }
}
// ANCHOR_END: recipe_debounce_generation

// ANCHOR: recipe_debounce_task
/// Replace the pending timer on each input event; only the newest query commits.
#[derive(Default)]
pub struct DebouncedQuery {
    generation: u64,
    pending: Option<Task<()>>,
    pub committed_query: String,
}

impl DebouncedQuery {
    pub fn input(&mut self, query: String, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        drop(self.pending.take());
        self.pending = Some(cx.spawn(async move |weak, async_cx: &mut AsyncApp| {
            async_cx.background_executor().timer(Duration::from_millis(250)).await;
            let _ = weak.update(async_cx, |this, cx| {
                if this.generation == generation {
                    this.committed_query = query;
                    this.pending = None;
                    cx.notify();
                }
            });
        }));
    }
}
// ANCHOR_END: recipe_debounce_task

// ANCHOR: recipe_latest_result
pub fn apply_latest<T>(current_generation: u64, result_generation: u64, result: T) -> Option<T> {
    (current_generation == result_generation).then_some(result)
}
// ANCHOR_END: recipe_latest_result

// ANCHOR: recipe_toast_queue
#[derive(Default)]
pub struct ToastQueue(VecDeque<String>);

impl ToastQueue {
    pub fn push(&mut self, message: impl Into<String>) {
        self.0.push_back(message.into());
    }

    pub fn dismiss_current(&mut self) -> Option<String> {
        self.0.pop_front()
    }
}
// ANCHOR_END: recipe_toast_queue

// ANCHOR: recipe_actions
actions!(
    cookbook,
    [MoveUp, MoveDown, Activate, ToggleTheme, ModalTabForward, ModalTabBackward, CloseModal]
);
// ANCHOR_END: recipe_actions

// ANCHOR: recipe_rebindable_keys
pub fn bind_navigation(app: &mut App, down_key: &str) -> Result<(), String> {
    gpui_pre::Keystroke::parse(down_key).map_err(|error| error.to_string())?;
    app.bind_keys([
        KeyBinding::new("up", MoveUp, Some(COOKBOOK_CONTEXT)),
        KeyBinding::new(down_key, MoveDown, Some(COOKBOOK_CONTEXT)),
        KeyBinding::new("enter", Activate, Some(COOKBOOK_CONTEXT)),
    ]);
    Ok(())
}
// ANCHOR_END: recipe_rebindable_keys

// ANCHOR: recipe_focus_handle
pub fn new_focus_handle(cx: &mut Context<Cookbook>) -> FocusHandle {
    cx.focus_handle()
}
// ANCHOR_END: recipe_focus_handle

// ANCHOR: recipe_modal_focus_trap
pub struct ModalFocusTrap {
    scope: Option<FocusHandle>,
    first: Option<FocusHandle>,
    last: Option<FocusHandle>,
    open: bool,
}

impl Default for ModalFocusTrap {
    fn default() -> Self {
        Self { scope: None, first: None, last: None, open: true }
    }
}

impl ModalFocusTrap {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            scope: Some(cx.focus_handle()),
            first: Some(cx.focus_handle().tab_stop(true)),
            last: Some(cx.focus_handle().tab_stop(true)),
            open: true,
        }
    }
}

impl Render for ModalFocusTrap {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.open {
            return div().child("Dialog closed");
        }
        if self.scope.is_none() {
            self.scope = Some(cx.focus_handle());
            self.first = Some(cx.focus_handle().tab_stop(true));
            self.last = Some(cx.focus_handle().tab_stop(true));
        }
        let scope = self.scope.as_ref().expect("focus scope initialized");
        let first = self.first.as_ref().expect("first action initialized").clone();
        let last = self.last.as_ref().expect("last action initialized").clone();
        let colors = cx.global::<gpui_kit::component::Theme>().colors;
        if !first.is_focused(window) && !last.is_focused(window) {
            window.focus(&first, cx);
        }
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_3()
            .bg(colors.background)
            .text_color(colors.foreground)
            .key_context("ModalFocusTrap")
            .track_focus(scope)
            .tab_group()
            .on_action(cx.listener(|this, _: &ModalTabForward, window, cx| {
                let last = this.last.as_ref().expect("last action initialized");
                let first = this.first.as_ref().expect("first action initialized");
                if last.is_focused(window) {
                    window.focus(first, cx);
                } else {
                    window.focus_next(cx);
                }
            }))
            .on_action(cx.listener(|this, _: &ModalTabBackward, window, cx| {
                let first = this.first.as_ref().expect("first action initialized");
                let last = this.last.as_ref().expect("last action initialized");
                if first.is_focused(window) {
                    window.focus(last, cx);
                } else {
                    window.focus_prev(cx);
                }
            }))
            .on_action(cx.listener(|this, _: &CloseModal, _, cx| {
                this.open = false;
                cx.notify();
            }))
            .child(div().track_focus(&first).tab_stop(true).child("First dialog action"))
            .child(div().track_focus(&last).tab_stop(true).child("Last dialog action"))
    }
}

pub fn install_modal_keys(app: &mut App) {
    app.bind_keys([
        KeyBinding::new("tab", ModalTabForward, Some("ModalFocusTrap")),
        KeyBinding::new("shift-tab", ModalTabBackward, Some("ModalFocusTrap")),
        KeyBinding::new("escape", CloseModal, Some("ModalFocusTrap")),
    ]);
}
// ANCHOR_END: recipe_modal_focus_trap

// ANCHOR: recipe_clipboard
pub fn copy_text(app: &mut App, text: impl Into<String>) {
    app.write_to_clipboard(gpui_pre::ClipboardItem::new_string(text.into()));
}
// ANCHOR_END: recipe_clipboard

// ANCHOR: recipe_drag_reorder
pub fn reorder<T>(items: &mut Vec<T>, from: usize, to: usize) -> bool {
    if from >= items.len() || to >= items.len() || from == to {
        return false;
    }
    let item = items.remove(from);
    items.insert(to, item);
    true
}
// ANCHOR_END: recipe_drag_reorder

// ANCHOR: recipe_file_dialog
pub fn choose_files(app: &App) -> Task<Option<Vec<PathBuf>>> {
    let receiver = app.prompt_for_paths(gpui_pre::PathPromptOptions {
        files: true,
        directories: false,
        multiple: true,
        prompt: Some("Choose files".into()),
    });
    app.background_executor()
        .spawn(async move { receiver.await.ok().and_then(Result::ok).flatten() })
}
// ANCHOR_END: recipe_file_dialog

// ANCHOR: recipe_application_menu
pub fn install_application_menu(app: &mut App) {
    app.set_menus([Menu::new("Cookbook").items([MenuItem::action("Activate", Activate)])]);
}
// ANCHOR_END: recipe_application_menu

// ANCHOR: recipe_dock_menu
/// gpui-pre 0.3.5 exposes a dock menu on macOS, but no general tray-icon API.
pub fn install_dock_menu(app: &mut App) {
    app.set_dock_menu(vec![MenuItem::action("Activate", Activate)]);
}
// ANCHOR_END: recipe_dock_menu

// ANCHOR: recipe_system_notification
pub fn notify(app: &App, title: impl Into<String>, body: impl Into<String>) {
    app.show_system_notification(gpui_pre::SystemNotification {
        tag: "cookbook".into(),
        title: title.into().into(),
        body: body.into().into(),
        actions: Vec::new(),
    });
}
// ANCHOR_END: recipe_system_notification

// ANCHOR: recipe_window_options
pub fn cookbook_window_options(app: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(gpui_pre::WindowBounds::centered(size(px(960.), px(640.)), app)),
        ..WindowOptions::default()
    }
}
// ANCHOR_END: recipe_window_options

// ANCHOR: recipe_open_second_window
pub fn open_another_window(app: &mut App) -> gpui_pre::Result<gpui_pre::WindowHandle<SmallWindow>> {
    app.open_window(WindowOptions::default(), |_, cx| cx.new(|_| SmallWindow))
}

pub struct SmallWindow;

impl Render for SmallWindow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().p_4().child("A second window")
    }
}
// ANCHOR_END: recipe_open_second_window

// ANCHOR: recipe_persist_window_size
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SavedWindowSize {
    pub width: f32,
    pub height: f32,
}

pub fn save_window_size(window: &Window, path: &Path) -> io::Result<SavedWindowSize> {
    let bounds = window.window_bounds().get_bounds();
    let saved = SavedWindowSize {
        width: f32::from(bounds.size.width),
        height: f32::from(bounds.size.height),
    };
    fs::write(path, format!("{} {}", saved.width, saved.height))?;
    Ok(saved)
}

pub fn load_window_size(path: &Path) -> io::Result<SavedWindowSize> {
    let contents = fs::read_to_string(path)?;
    let mut values = contents.split_whitespace();
    let parse = |value: Option<&str>| {
        value
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?
            .parse::<f32>()
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    };
    let width = parse(values.next())?;
    let height = parse(values.next())?;
    if !width.is_finite()
        || !height.is_finite()
        || width <= 0.
        || height <= 0.
        || values.next().is_some()
    {
        return Err(io::Error::from(io::ErrorKind::InvalidData));
    }
    Ok(SavedWindowSize { width, height })
}

// ANCHOR: recipe_restore_window_size
pub fn options_for_saved_size(saved: SavedWindowSize, app: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(gpui_pre::WindowBounds::centered(
            size(px(saved.width), px(saved.height)),
            app,
        )),
        ..WindowOptions::default()
    }
}
// ANCHOR_END: recipe_restore_window_size
// ANCHOR_END: recipe_persist_window_size

// ANCHOR: recipe_theme_switch
#[derive(Default)]
pub struct ThemeChoice {
    pub alternate: bool,
}

impl ThemeChoice {
    /// Choose contrasting surfaces from the active kit theme tokens for an in-app preview.
    pub fn token_pair(
        &self,
        theme: &gpui_kit::component::Theme,
    ) -> (gpui_pre::Hsla, gpui_pre::Hsla) {
        if self.alternate {
            (theme.colors.primary, theme.colors.primary_foreground)
        } else {
            (theme.colors.background, theme.colors.foreground)
        }
    }
}
// ANCHOR_END: recipe_theme_switch

// ANCHOR: recipe_appearance
pub fn current_appearance(window: &Window) -> gpui_pre::WindowAppearance {
    window.appearance()
}
// ANCHOR_END: recipe_appearance

// ANCHOR: recipe_split_pane
pub fn split_pane(left: impl IntoElement, right: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .size_full()
        .child(div().flex_1().min_w_0().child(left))
        .child(div().w(px(1.)).h_full().bg(gpui_pre::rgb(0x888888)))
        .child(div().flex_1().min_w_0().child(right))
}
// ANCHOR_END: recipe_split_pane

// ANCHOR: recipe_scroll_region
pub fn scroll_region(content: impl IntoElement, handle: &ScrollHandle) -> impl IntoElement {
    div()
        .id("recipe-scroll")
        .debug_selector(|| "recipe-scroll".into())
        .h(px(140.))
        .flex_shrink_0()
        .track_scroll(handle)
        .overflow_y_scroll()
        .child(content)
}
// ANCHOR_END: recipe_scroll_region

// ANCHOR: recipe_rich_text
pub fn rich_text(window: &Window, color: gpui_pre::Hsla) -> impl IntoElement {
    let text = "Inline emphasis uses byte ranges.";
    let emphasis = "emphasis";
    let start = text.find(emphasis).expect("emphasis appears in text");
    let mut runs = Vec::new();
    let mut opening = window.text_style().to_run(start);
    opening.color = color;
    runs.push(opening);
    let mut highlighted = window.text_style().to_run(emphasis.len());
    highlighted.font.weight = gpui_pre::FontWeight(700.);
    highlighted.color = color;
    runs.push(highlighted);
    let mut ending = window.text_style().to_run(text.len() - start - emphasis.len());
    ending.color = color;
    runs.push(ending);
    div().child(gpui_pre::StyledText::new(text).with_runs(runs))
}
// ANCHOR_END: recipe_rich_text

// ANCHOR: recipe_grid_layout
pub fn two_column_grid(first: impl IntoElement, second: impl IntoElement) -> impl IntoElement {
    div().grid().grid_cols(2).gap_3().child(first).child(second)
}
// ANCHOR_END: recipe_grid_layout

// ANCHOR: recipe_conditional_render
pub fn conditional_panel(visible: bool) -> impl IntoElement {
    div().when(visible, |this| this.child("Only rendered while visible"))
}
// ANCHOR_END: recipe_conditional_render

// ANCHOR: recipe_div_component
/// Return a GPUI element from a reusable builder function without creating a view entity.
pub fn status_badge(label: &'static str) -> impl IntoElement {
    div().px_2().py_1().rounded_md().child(label)
}
// ANCHOR_END: recipe_div_component

// ANCHOR: recipe_entity_task_update
pub fn update_weak_entity(
    cx: &mut Context<Cookbook>,
    task: impl Future<Output = String> + Send + 'static,
) -> Task<()> {
    cx.spawn(async move |weak, async_cx: &mut AsyncApp| {
        let result = task.await;
        let _ = weak.update(async_cx, |this, cx| {
            this.message = result;
            cx.notify();
        });
    })
}
// ANCHOR_END: recipe_entity_task_update

#[derive(Default)]
pub struct Cookbook {
    message: String,
    selected: usize,
    activations: usize,
    focus: Option<FocusHandle>,
    alternate_palette: bool,
}

impl Cookbook {
    pub fn set_alternate_palette(&mut self, dark: bool, cx: &mut Context<Self>) {
        self.alternate_palette = dark;
        cx.notify();
    }
}

impl Render for Cookbook {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus.is_none() {
            let focus = cx.focus_handle();
            window.focus(&focus, cx);
            self.focus = Some(focus);
        }
        let kit_theme = cx.global::<gpui_kit::component::Theme>();
        let theme = kit_theme.colors;
        let (surface, foreground) =
            ThemeChoice { alternate: self.alternate_palette }.token_pair(kit_theme);
        let recipes = [
            ("State", "Entities · events · stable IDs"),
            ("Async", "Executors · debounce · toast queue"),
            ("Input", "Actions · focus · drag reorder"),
            ("Windows", "Dialogs · bounds · dock menu"),
            ("Rendering", "Split panes · scroll · rich text"),
            ("Platform note", "No general tray icon API in this GPUI pin"),
        ];
        let mut recipe_grid = div().grid().grid_cols(2).gap_3();
        for (title, detail) in recipes {
            recipe_grid = recipe_grid.child(
                div()
                    .border_1()
                    .border_color(theme.border)
                    .rounded_lg()
                    .p_4()
                    .child(div().font_weight(gpui_pre::FontWeight(650.)).child(title))
                    .child(div().text_color(theme.muted_foreground).child(detail)),
            );
        }
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_3()
            .bg(surface)
            .text_color(foreground)
            .key_context(COOKBOOK_CONTEXT)
            .track_focus(self.focus.as_ref().expect("focus initialized"))
            .tab_group()
            .on_action(cx.listener(|this, _: &MoveUp, _, cx| {
                this.selected = this.selected.saturating_sub(1);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &MoveDown, _, cx| {
                this.selected = (this.selected + 1).min(2);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &Activate, _, cx| {
                this.activations += 1;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleTheme, _, cx| {
                this.alternate_palette = !this.alternate_palette;
                cx.notify();
            }))
            .child(div().text_2xl().child("GPUI Cookbook"))
            .child(
                div().child(
                    "Small, compiling recipes for state, tasks, input, windows, and rendering.",
                ),
            )
            .child(div().child(format!("Latest task result: {}", self.message)))
            .child(
                div().child(format!(
                    "Selection: {} · activations: {}",
                    self.selected, self.activations
                )),
            )
            .child(status_badge("Pinned GPUI 0.3.5"))
            .child(recipe_grid)
            .child(rich_text(window, foreground))
            .child(split_pane(div().child("Editor"), div().child("Preview")))
            .child(conditional_panel(true))
    }
}

// ANCHOR: recipe_scroll_view
#[derive(Default)]
pub struct ScrollRecipeView {
    scroll_handle: ScrollHandle,
}

impl ScrollRecipeView {
    pub fn set_demo_offset(&mut self, y_offset: gpui_pre::Pixels) {
        self.scroll_handle.set_offset(gpui_pre::point(px(0.), y_offset));
    }
}

impl Render for ScrollRecipeView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<gpui_kit::component::Theme>().colors;
        let notes = div().flex().flex_col().gap_2().children((1..=14).map(|index| {
            div()
                .border_1()
                .border_color(colors.border)
                .p_3()
                .child(format!("Scrollable recipe note {index}"))
        }));
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_3()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(div().text_2xl().child("Scroll region"))
            .child(scroll_region(notes, &self.scroll_handle))
    }
}
// ANCHOR_END: recipe_scroll_view

// ANCHOR: recipe_drag_reorder_ui
pub struct ReorderRecipeView {
    items: Vec<&'static str>,
    move_count: usize,
}

impl Default for ReorderRecipeView {
    fn default() -> Self {
        Self { items: vec!["First recipe", "Second recipe", "Third recipe"], move_count: 0 }
    }
}

impl ReorderRecipeView {
    pub fn ordered_items(&self) -> &[&'static str] {
        &self.items
    }

    pub fn set_preview_order(&mut self, items: Vec<&'static str>) {
        self.items = items;
        self.move_count = 1;
    }
}

impl Render for ReorderRecipeView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<gpui_kit::component::Theme>().colors;
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_2()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(div().text_2xl().child("Drag to reorder"))
            .child(self.items.iter().enumerate().fold(
                div().flex().flex_col().gap_2(),
                |rows, (index, title)| {
                    rows.child(
                        div()
                            .id(format!("cookbook-row-{index}"))
                            .debug_selector(move || format!("cookbook-row-{index}"))
                            .px_3()
                            .py_2()
                            .border_1()
                            .border_color(colors.border)
                            .on_drag(index, |_, _, _, cx| cx.new(|_| gpui_pre::Empty))
                            .on_drop(cx.listener(move |this, from: &usize, _, cx| {
                                if reorder(&mut this.items, *from, index) {
                                    this.move_count += 1;
                                    cx.notify();
                                }
                            }))
                            .child(*title),
                    )
                },
            ))
            .child(div().child(format!("Moved: {}", self.move_count)))
    }
}
// ANCHOR_END: recipe_drag_reorder_ui

pub const fn recipe_count() -> usize {
    40
}

pub fn install_cookbook_keys(app: &mut App) {
    app.bind_keys([
        KeyBinding::new("up", MoveUp, Some(COOKBOOK_CONTEXT)),
        KeyBinding::new("down", MoveDown, Some(COOKBOOK_CONTEXT)),
        KeyBinding::new("enter", Activate, Some(COOKBOOK_CONTEXT)),
        KeyBinding::new("t", ToggleTheme, Some(COOKBOOK_CONTEXT)),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipes_cover_more_than_thirty_named_examples() {
        assert_eq!(recipe_count(), 40);
    }

    #[test]
    fn stable_id_updates_do_not_depend_on_list_position() {
        let mut rows = vec![Row { id: 10, title: "A".into() }, Row { id: 20, title: "B".into() }];
        assert!(update_row_title(&mut rows, 20, "Renamed".into()));
        assert_eq!(rows[1].title, "Renamed");
        assert!(!update_row_title(&mut rows, 30, "Missing".into()));
    }

    #[test]
    fn debounce_discards_old_generation_and_toasts_are_fifo() {
        let mut debounce = Debounce::default();
        let first = debounce.begin();
        let latest = debounce.begin();
        assert!(!debounce.is_current(first));
        assert!(debounce.is_current(latest));

        let mut toasts = ToastQueue::default();
        toasts.push("saved");
        toasts.push("synced");
        assert_eq!(toasts.dismiss_current().as_deref(), Some("saved"));
        assert_eq!(toasts.dismiss_current().as_deref(), Some("synced"));
    }

    #[test]
    fn drag_reorder_preserves_the_item_and_rejects_bad_indices() {
        let mut items = vec!["A", "B", "C"];
        assert!(reorder(&mut items, 0, 2));
        assert_eq!(items, ["B", "C", "A"]);
        assert!(!reorder(&mut items, 8, 0));
    }

    #[test]
    fn stale_async_result_is_ignored() {
        assert_eq!(apply_latest(4, 3, "old"), None);
        assert_eq!(apply_latest(4, 4, "new"), Some("new"));
    }

    #[gpui_kit::test]
    fn app_global_is_available_to_later_recipe_code(cx: &mut gpui_pre::TestAppContext) {
        cx.update(|app| install_shared_label(app, "Shared across app views"));
        assert_eq!(cx.update(|app| global_label(app)), "Shared across app views");
    }

    #[gpui_kit::test]
    async fn foreground_and_background_executor_recipes_complete(
        cx: &mut gpui_pre::TestAppContext,
    ) {
        let foreground = cx.update(|app| foreground_work(app));
        let background = cx.update(|app| background_work(app));
        cx.run_until_parked();
        assert_eq!(foreground.await, "small UI-adjacent work");
        assert_eq!(background.await, (0..100).map(|n| n * n).collect::<Vec<_>>());
    }

    #[gpui_kit::test]
    fn timer_completion_and_drop_cancellation_are_deterministic(cx: &mut gpui_pre::TestAppContext) {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };

        let timer = cx.update(|app| delay(app, Duration::from_millis(40)));
        assert!(!timer.is_ready());
        cx.executor().advance_clock(Duration::from_millis(40));
        cx.run_until_parked();
        assert!(timer.is_ready());

        let completed = Arc::new(AtomicBool::new(false));
        let completed_by_task = completed.clone();
        let cancelled = cx.update(|app| {
            let executor = app.background_executor();
            let timer_executor = executor.clone();
            executor.spawn(async move {
                timer_executor.timer(Duration::from_secs(1)).await;
                completed_by_task.store(true, Ordering::SeqCst);
            })
        });
        cx.run_until_parked();
        let mut pending = Some(cancelled);
        cancel_previous(&mut pending);
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        assert!(!completed.load(Ordering::SeqCst));
    }

    #[gpui_kit::test]
    fn clipboard_recipe_round_trips_text(cx: &mut gpui_pre::TestAppContext) {
        cx.update(|app| copy_text(app, "copy me"));
        assert_eq!(
            cx.read_from_clipboard().and_then(|item| item.text()).as_deref(),
            Some("copy me")
        );
    }

    #[gpui_kit::test]
    fn app_menu_recipe_registers_a_native_menu(cx: &mut gpui_pre::TestAppContext) {
        cx.update(install_application_menu);
        assert_eq!(cx.update(|app| app.get_menus().map_or(0, |menus| menus.len())), 1);
    }

    #[gpui_kit::test]
    fn notification_recipe_records_the_tagged_local_notification(
        cx: &mut gpui_pre::TestAppContext,
    ) {
        cx.update(|app| {
            app.set_app_identity("com.example.cookbook", "Cookbook");
            notify(app, "Recipe complete", "Local test notification");
        });
        let shown = cx.shown_system_notifications();
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].tag.as_ref(), "cookbook");
    }

    #[gpui_kit::test]
    fn window_options_and_second_window_are_applied(cx: &mut gpui_pre::TestAppContext) {
        let options = cx.update(|app| cookbook_window_options(app));
        let bounds = options.window_bounds.expect("explicit window bounds").get_bounds();
        assert_eq!(f32::from(bounds.size.width), 960.);
        assert_eq!(f32::from(bounds.size.height), 640.);
        let count_before = cx.windows().len();
        cx.update(open_another_window).expect("open second cookbook window");
        assert_eq!(cx.windows().len(), count_before + 1);
    }

    #[gpui_kit::test]
    fn caller_rebinds_navigation_to_a_valid_keystroke(cx: &mut gpui_pre::TestAppContext) {
        cx.update(gpui_kit::init);
        cx.update(|app| bind_navigation(app, "ctrl-j").unwrap());
        let (view, visual) = cx.add_window_view(|_, _| Cookbook::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("ctrl-j");
        assert_eq!(view.read_with(visual, |cookbook, _| cookbook.selected), 1);
    }

    #[gpui_kit::test]
    async fn file_picker_returns_the_platform_selected_paths(cx: &mut gpui_pre::TestAppContext) {
        let response = cx.update(|app| choose_files(app));
        assert!(cx.did_prompt_for_paths());
        let expected = vec![PathBuf::from("/tmp/cookbook.txt")];
        cx.simulate_path_prompt_response({
            let expected = expected.clone();
            move |options| {
                assert!(options.files && !options.directories && options.multiple);
                Some(expected)
            }
        });
        assert_eq!(response.await, Some(expected));
    }

    #[test]
    fn saved_window_size_round_trips_and_rejects_invalid_values() {
        let path =
            std::env::temp_dir().join(format!("cookbook-window-size-{}.txt", std::process::id()));
        fs::write(&path, "960 640").unwrap();
        assert_eq!(load_window_size(&path).unwrap(), SavedWindowSize { width: 960., height: 640. });
        fs::write(&path, "-1 0").unwrap();
        assert!(load_window_size(&path).is_err());
        fs::remove_file(path).unwrap();
    }

    #[gpui_kit::test]
    fn current_window_bounds_can_be_saved_to_a_local_config_file(
        cx: &mut gpui_pre::TestAppContext,
    ) {
        cx.update(gpui_kit::init);
        let (view, visual) = cx.add_window_view(|_, _| Cookbook::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let path =
            std::env::temp_dir().join(format!("cookbook-window-bounds-{}.txt", std::process::id()));
        let saved = visual.update(|window, _| save_window_size(window, &path).unwrap());
        assert_eq!(load_window_size(&path).unwrap(), saved);
        fs::remove_file(path).unwrap();
        drop(view);
    }

    #[gpui_kit::test]
    fn keyboard_actions_update_state_through_the_rendered_view(cx: &mut gpui_pre::TestAppContext) {
        cx.update(gpui_kit::init);
        cx.update(install_cookbook_keys);
        let (view, visual) = cx.add_window_view(|_, _| Cookbook::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("down enter");
        let result =
            view.read_with(visual, |cookbook, _| (cookbook.selected, cookbook.activations));
        assert_eq!(result, (1, 1));
    }

    #[gpui_kit::test]
    fn counter_updates_observer_and_typed_event_subscriber(cx: &mut gpui_pre::TestAppContext) {
        cx.update(gpui_kit::init);
        let (demo, visual) = cx.add_window_view(|_, _| CounterDemo::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let button = visual.debug_bounds("increment-counter").unwrap();
        visual.simulate_click(button.center(), gpui_pre::Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let values = demo.read_with(visual, |demo, _| (demo.observed, demo.event_value));
        assert_eq!(values, (1, 1));
    }

    #[gpui_kit::test]
    fn theme_action_changes_the_rendered_palette_selection(cx: &mut gpui_pre::TestAppContext) {
        cx.update(gpui_kit::init);
        cx.update(install_cookbook_keys);
        let (view, visual) = cx.add_window_view(|_, _| Cookbook::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("t");
        assert!(view.read_with(visual, |cookbook, _| cookbook.alternate_palette));
    }

    #[gpui_kit::test]
    fn rendered_drag_target_reorders_stable_recipe_rows(cx: &mut gpui_pre::TestAppContext) {
        cx.update(gpui_kit::init);
        let (view, visual) = cx.add_window_view(|_, _| ReorderRecipeView::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let first = visual.debug_bounds("cookbook-row-0").unwrap().center();
        let second = visual.debug_bounds("cookbook-row-1").unwrap().center();
        visual.simulate_mouse_down(
            first,
            gpui_pre::MouseButton::Left,
            gpui_pre::Modifiers::default(),
        );
        visual.simulate_mouse_move(
            second,
            Some(gpui_pre::MouseButton::Left),
            gpui_pre::Modifiers::default(),
        );
        visual.simulate_mouse_up(
            second,
            gpui_pre::MouseButton::Left,
            gpui_pre::Modifiers::default(),
        );
        let result =
            view.read_with(visual, |recipe, _| (recipe.ordered_items()[0], recipe.move_count));
        assert_eq!(result, ("Second recipe", 1));
    }

    #[gpui_kit::test]
    fn modal_tab_actions_wrap_focus_at_both_ends(cx: &mut gpui_pre::TestAppContext) {
        cx.update(gpui_kit::init);
        cx.update(install_modal_keys);
        let (view, visual) = cx.add_window_view(|_, cx| ModalFocusTrap::new(cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(
            visual.update(|window, cx| view.read(cx).first.as_ref().unwrap().is_focused(window))
        );
        visual.simulate_keystrokes("tab tab");
        assert!(
            visual.update(|window, cx| view.read(cx).first.as_ref().unwrap().is_focused(window))
        );
        visual.simulate_keystrokes("shift-tab");
        assert!(
            visual.update(|window, cx| view.read(cx).last.as_ref().unwrap().is_focused(window))
        );
        visual.simulate_keystrokes("escape");
        assert!(!view.read_with(visual, |modal, _| modal.open));
    }

    #[gpui_kit::test]
    fn debounce_timer_commits_only_the_latest_input(cx: &mut gpui_pre::TestAppContext) {
        cx.update(gpui_kit::init);
        let query = cx.update(|app| app.new(|_| DebouncedQuery::default()));
        query.update(cx, |this, cx| this.input("first".into(), cx));
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(100));
        query.update(cx, |this, cx| this.input("latest".into(), cx));
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(250));
        cx.run_until_parked();
        assert_eq!(query.read_with(cx, |state, _| state.committed_query.clone()), "latest");
    }

    #[gpui_kit::test]
    fn scroll_recipe_has_extent_and_moves_on_wheel_input(cx: &mut gpui_pre::TestAppContext) {
        cx.update(gpui_kit::init);
        let (view, visual) = cx.add_window_view(|_, _| ScrollRecipeView::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let maximum = view.read_with(visual, |recipe, _| recipe.scroll_handle.max_offset());
        assert!(maximum.y > px(0.));
        let bounds = visual.debug_bounds("recipe-scroll").expect("scroll recipe is rendered");
        visual.simulate_mouse_move(bounds.center(), None, gpui_pre::Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_event(gpui_pre::ScrollWheelEvent {
            position: bounds.center(),
            delta: gpui_pre::ScrollDelta::Pixels(gpui_pre::point(px(0.), px(-120.))),
            modifiers: gpui_pre::Modifiers::default(),
            touch_phase: gpui_pre::TouchPhase::Moved,
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let offset = view.read_with(visual, |recipe, _| recipe.scroll_handle.offset());
        assert!(offset.y < px(0.));
    }
}
