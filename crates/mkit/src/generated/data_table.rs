//! Basic sortable, selectable data grid with explicit column resizing.
extern crate gpui_pre as gpui;
use gpui_pre::{
    AnyElement, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render, Window, actions, div,
    prelude::*, px,
};
use mkit_core::theme::Theme;
use std::collections::HashSet;

pub const KEY_CONTEXT: &str = "MkitDataTable";
actions!(data_table, [Next, Previous, Left, Right, Select, Activate, First, Last]);
pub fn default_key_bindings() -> [KeyBinding; 8] {
    [
        KeyBinding::new("down", Next, Some(KEY_CONTEXT)),
        KeyBinding::new("up", Previous, Some(KEY_CONTEXT)),
        KeyBinding::new("left", Left, Some(KEY_CONTEXT)),
        KeyBinding::new("right", Right, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Activate, Some(KEY_CONTEXT)),
        KeyBinding::new("space", Select, Some(KEY_CONTEXT)),
        KeyBinding::new("home", First, Some(KEY_CONTEXT)),
        KeyBinding::new("end", Last, Some(KEY_CONTEXT)),
    ]
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub id: String,
    pub label: String,
    pub width: u32,
    pub sortable: bool,
}
impl Column {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), width: 160, sortable: true }
    }
    pub fn width(mut self, width: u32) -> Self {
        self.width = width.max(24);
        self
    }
    pub fn sortable(mut self, value: bool) -> Self {
        self.sortable = value;
        self
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataRow {
    pub id: String,
    pub cells: Vec<String>,
}
impl DataRow {
    pub fn new(id: impl Into<String>, cells: Vec<String>) -> Self {
        Self { id: id.into(), cells }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}
fn next_sort_direction(current: Option<SortDirection>) -> SortDirection {
    if current == Some(SortDirection::Ascending) {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SortChanged {
    pub column: String,
    pub direction: SortDirection,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionChanged(pub Vec<String>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveChanged(pub Option<String>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveCellChanged {
    pub row: Option<String>,
    pub column: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnResized {
    pub column: String,
    pub width: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowActivated(pub String);
impl EventEmitter<SortChanged> for DataTable {}
impl EventEmitter<SelectionChanged> for DataTable {}
impl EventEmitter<ActiveChanged> for DataTable {}
impl EventEmitter<ActiveCellChanged> for DataTable {}
impl EventEmitter<ColumnResized> for DataTable {}
impl EventEmitter<RowActivated> for DataTable {}
type CellRenderer = dyn Fn(&DataRow, &Column, bool, Theme) -> AnyElement;
type HeaderRenderer = dyn Fn(&Column, Option<SortDirection>, Theme) -> AnyElement;

pub struct DataTable {
    label: String,
    columns: Vec<Column>,
    rows: Vec<DataRow>,
    selected: HashSet<String>,
    active: Option<usize>,
    active_column: usize,
    header_active: bool,
    sort: Option<(String, SortDirection)>,
    controlled: bool,
    multi: bool,
    resize_drag: Option<(String, f32, u32)>,
    focus: Option<FocusHandle>,
    cell_renderer: Option<Box<CellRenderer>>,
    header_renderer: Option<Box<HeaderRenderer>>,
    row_height: Option<f32>,
    header_height: Option<f32>,
    column_gap: Option<f32>,
    horizontal_padding: Option<f32>,
    cell_padding: Option<f32>,
    row_divider: bool,
    selection_tint: bool,
    resizable: bool,
    activate_on_enter: bool,
}
impl DataTable {
    pub fn new(label: impl Into<String>, columns: Vec<Column>, rows: Vec<DataRow>) -> Self {
        Self {
            label: label.into(),
            columns,
            rows,
            selected: HashSet::new(),
            active: None,
            active_column: 0,
            header_active: true,
            sort: None,
            controlled: false,
            multi: false,
            resize_drag: None,
            focus: None,
            cell_renderer: None,
            header_renderer: None,
            row_height: None,
            header_height: None,
            column_gap: None,
            horizontal_padding: None,
            cell_padding: None,
            row_divider: false,
            selection_tint: false,
            resizable: true,
            activate_on_enter: false,
        }
    }
    pub fn controlled(
        label: impl Into<String>,
        columns: Vec<Column>,
        rows: Vec<DataRow>,
        selected: HashSet<String>,
        sort: Option<(String, SortDirection)>,
    ) -> Self {
        let mut t = Self::new(label, columns, rows);
        t.selected = selected;
        t.sort = sort;
        t.controlled = true;
        t
    }
    pub fn multi_select(mut self) -> Self {
        self.multi = true;
        self
    }
    pub fn with_cell_renderer(
        mut self,
        render: impl Fn(&DataRow, &Column, bool, Theme) -> AnyElement + 'static,
    ) -> Self {
        self.cell_renderer = Some(Box::new(render));
        self
    }
    pub fn with_header_renderer(
        mut self,
        render: impl Fn(&Column, Option<SortDirection>, Theme) -> AnyElement + 'static,
    ) -> Self {
        self.header_renderer = Some(Box::new(render));
        self
    }
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = Some(height.max(1.0));
        self
    }
    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = Some(height.max(1.0));
        self
    }
    pub fn column_gap(mut self, gap: f32) -> Self {
        self.column_gap = Some(gap.max(0.0));
        self
    }
    pub fn horizontal_padding(mut self, padding: f32) -> Self {
        self.horizontal_padding = Some(padding.max(0.0));
        self
    }
    pub fn cell_padding(mut self, padding: f32) -> Self {
        self.cell_padding = Some(padding.max(0.0));
        self
    }
    pub fn row_divider(mut self, enabled: bool) -> Self {
        self.row_divider = enabled;
        self
    }
    pub fn selection_tint(mut self, enabled: bool) -> Self {
        self.selection_tint = enabled;
        self
    }
    pub fn resizable(mut self, enabled: bool) -> Self {
        self.resizable = enabled;
        self
    }
    pub fn activate_on_enter(mut self, enabled: bool) -> Self {
        self.activate_on_enter = enabled;
        self
    }
    pub fn selection(&self) -> &HashSet<String> {
        &self.selected
    }
    pub fn sort(&self) -> Option<(&str, SortDirection)> {
        self.sort.as_ref().map(|(id, d)| (id.as_str(), *d))
    }
    pub fn set_selection(&mut self, ids: HashSet<String>, cx: &mut Context<Self>) {
        if self.selected == ids {
            return;
        }
        self.selected = ids;
        cx.notify()
    }
    pub fn set_rows(&mut self, rows: Vec<DataRow>, cx: &mut Context<Self>) {
        if self.rows == rows {
            return;
        }
        let active_id =
            self.active.and_then(|index| self.rows.get(index)).map(|row| row.id.clone());
        self.rows = rows;
        if !self.controlled && self.sort.is_some() {
            self.sort_rows();
        }
        let ids: HashSet<&str> = self.rows.iter().map(|row| row.id.as_str()).collect();
        self.selected.retain(|id| ids.contains(id.as_str()));
        self.active = active_id.and_then(|id| self.rows.iter().position(|row| row.id == id));
        if self.active.is_none() && !self.header_active {
            self.active = if self.rows.is_empty() { None } else { Some(0) };
            self.header_active = self.active.is_none();
        }
        self.active_column = self.active_column.min(self.columns.len().saturating_sub(1));
        cx.notify();
    }
    pub fn set_sort(&mut self, sort: Option<(String, SortDirection)>, cx: &mut Context<Self>) {
        self.sort = sort;
        cx.notify()
    }
    pub fn resize_column(&mut self, id: &str, width: u32, cx: &mut Context<Self>) -> bool {
        let Some(c) = self.columns.iter_mut().find(|c| c.id == id) else { return false };
        c.width = width.max(24);
        cx.emit(ColumnResized { column: id.to_owned(), width: c.width });
        cx.notify();
        true
    }
    fn start_resize(&mut self, id: &str, event: &MouseDownEvent) {
        if event.button != MouseButton::Left {
            return;
        }
        if let Some(column) = self.columns.iter().find(|column| column.id == id) {
            self.resize_drag = Some((id.to_owned(), f32::from(event.position.x), column.width));
        }
    }
    fn drag_resize(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some((id, start_x, start_width)) = self.resize_drag.clone() else { return };
        if !event.dragging() {
            return;
        }
        let delta = f32::from(event.position.x) - start_x;
        let width = (start_width as f32 + delta).round().max(24.0) as u32;
        if self.columns.iter().any(|column| column.id == id && column.width != width) {
            self.resize_column(&id, width, cx);
        }
    }
    fn end_resize(&mut self, event: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        if event.button == MouseButton::Left {
            self.resize_drag = None;
        }
    }
    pub fn sort_by(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(col) = self.columns.iter().find(|c| c.id == id && c.sortable) else { return };
        let key = col.id.clone();
        let current = self.sort.as_ref().filter(|(s, _)| s == &key).map(|(_, d)| *d);
        let dir = next_sort_direction(current);
        if !self.controlled {
            self.sort = Some((key.clone(), dir));
            self.sort_rows()
        }
        cx.emit(SortChanged { column: key, direction: dir });
        cx.notify()
    }
    fn sort_rows(&mut self) {
        let Some((id, dir)) = self.sort.clone() else { return };
        let Some(column) = self.columns.iter().position(|c| c.id == id) else { return };
        self.rows.sort_by(|a, b| {
            let x = a.cells.get(column).map(String::as_str).unwrap_or("");
            let y = b.cells.get(column).map(String::as_str).unwrap_or("");
            let ord = x.cmp(y);
            if dir == SortDirection::Ascending { ord } else { ord.reverse() }
        })
    }
    fn move_active(&mut self, step: isize, cx: &mut Context<Self>) {
        if self.rows.is_empty() || self.columns.is_empty() {
            return;
        }
        let was_header = self.header_active;
        let Some(i) = self.active else {
            if was_header && step < 0 {
                return;
            }
            self.header_active = false;
            self.active = Some(if was_header || step >= 0 { 0 } else { self.rows.len() - 1 });
            cx.emit(ActiveChanged(
                self.active.and_then(|i| self.rows.get(i)).map(|r| r.id.clone()),
            ));
            self.emit_active_cell(cx);
            cx.notify();
            return;
        };
        let next = i as isize + step;
        if next < 0 {
            if i == 0 {
                self.header_active = true;
                self.active = None;
            } else {
                self.active = Some(i - 1);
            }
        } else if next >= self.rows.len() as isize {
            self.active = Some(self.rows.len() - 1);
        } else {
            self.active = Some(next as usize);
        }
        cx.emit(ActiveChanged(self.active.and_then(|i| self.rows.get(i)).map(|r| r.id.clone())));
        self.emit_active_cell(cx);
        cx.notify()
    }
    fn emit_active_cell(&self, cx: &mut Context<Self>) {
        let Some(column) = self.columns.get(self.active_column) else { return };
        let row = if self.header_active {
            None
        } else {
            self.active.and_then(|i| self.rows.get(i)).map(|r| r.id.clone())
        };
        cx.emit(ActiveCellChanged { row, column: column.id.clone() });
    }
    fn move_horizontal(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.columns.is_empty() {
            return;
        }
        let old = self.active_column;
        self.active_column = (self.active_column as isize + delta)
            .clamp(0, self.columns.len() as isize - 1) as usize;
        if old != self.active_column {
            self.emit_active_cell(cx);
            cx.notify();
        }
    }
    fn move_edge(&mut self, to_end: bool, cx: &mut Context<Self>) {
        if self.columns.is_empty() {
            return;
        }
        let next = if to_end { self.columns.len() - 1 } else { 0 };
        if self.active_column != next {
            self.active_column = next;
            self.emit_active_cell(cx);
            cx.notify();
        }
    }
    fn select_active(&mut self, cx: &mut Context<Self>) {
        if self.header_active {
            if let Some(column) = self.columns.get(self.active_column).filter(|c| c.sortable) {
                let id = column.id.clone();
                self.sort_by(&id, cx);
            }
            return;
        }
        let Some(id) = self.active.and_then(|i| self.rows.get(i)).map(|r| r.id.clone()) else {
            return;
        };
        let mut selected = self.selected.clone();
        if !self.multi {
            if selected.contains(&id) {
                selected.clear();
            } else {
                selected.clear();
                selected.insert(id.clone());
            }
        } else if !selected.insert(id.clone()) {
            selected.remove(&id);
        }
        if !self.controlled {
            self.selected = selected.clone()
        }
        cx.emit(SelectionChanged(selected.into_iter().collect()));
        cx.notify()
    }
    fn activate_active(&mut self, cx: &mut Context<Self>) {
        if self.header_active || !self.activate_on_enter {
            self.select_active(cx);
        } else if let Some(row) = self.active.and_then(|index| self.rows.get(index)) {
            cx.emit(RowActivated(row.id.clone()));
        }
    }
}
impl Focusable for DataTable {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}
impl Render for DataTable {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let entity = cx.entity();
        let has_focus = self.focus.as_ref().is_some_and(|focus| focus.is_focused(window));
        let table_selector = self.label.clone();
        let mut root = div()
            .id(self.label.clone())
            .debug_selector(move || table_selector)
            .flex()
            .flex_col()
            .w_full()
            .key_context(KEY_CONTEXT)
            .tab_index(0)
            .track_focus(self.focus.get_or_insert_with(|| cx.focus_handle()))
            .role(gpui_pre::accesskit::Role::Grid)
            .aria_label(self.label.clone())
            .aria_row_count(self.rows.len() + 1)
            .aria_column_count(self.columns.len())
            .bg(t.colors.surface)
            .border(px(t.borders.hairline))
            .border_color(t.colors.border)
            .focus_visible(|element| element.border_color(t.colors.focus))
            .on_action(cx.listener(|s, _: &Next, _, cx| s.move_active(1, cx)))
            .on_action(cx.listener(|s, _: &Previous, _, cx| s.move_active(-1, cx)))
            .on_action(cx.listener(|s, _: &Left, _, cx| s.move_horizontal(-1, cx)))
            .on_action(cx.listener(|s, _: &Right, _, cx| s.move_horizontal(1, cx)))
            .on_action(cx.listener(|s, _: &Select, _, cx| s.select_active(cx)))
            .on_action(cx.listener(|s, _: &Activate, _, cx| s.activate_active(cx)))
            .on_action(cx.listener(|s, _: &First, _, cx| s.move_edge(false, cx)))
            .on_action(cx.listener(|s, _: &Last, _, cx| s.move_edge(true, cx)))
            .on_mouse_move(cx.listener(Self::drag_resize))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::end_resize));
        let horizontal_padding = self.horizontal_padding.unwrap_or(0.0);
        let cell_padding = self.cell_padding.unwrap_or(t.spacing.small);
        let mut header = div()
            .flex()
            .items_center()
            .min_h(px(self.header_height.unwrap_or(t.controls.medium)))
            .px(px(horizontal_padding))
            .gap(px(self.column_gap.unwrap_or(0.0)))
            .bg(t.colors.elevated_surface)
            .when(self.row_divider, |element| {
                element.border_b(px(t.borders.hairline)).border_color(t.colors.border)
            });
        for (column_index, c) in self.columns.iter().enumerate() {
            let active_header =
                has_focus && self.header_active && self.active_column == column_index;
            let id = c.id.clone();
            let c2 = id.clone();
            let grip_id = id.clone();
            let header_selector = format!("header-label-{id}");
            let grip_selector = format!("column-resize-{id}");
            let dir = self
                .sort
                .as_ref()
                .filter(|(s, _)| s == &id)
                .map(|(_, d)| if *d == SortDirection::Ascending { " ↑" } else { " ↓" })
                .unwrap_or("");
            let header_label = div()
                .id(format!("header-label-{id}"))
                .debug_selector(move || header_selector)
                .role(gpui_pre::accesskit::Role::ColumnHeader)
                .aria_label(format!(
                    "{}{}",
                    c.label,
                    match self.sort.as_ref().filter(|(s, _)| s == &id).map(|(_, d)| d) {
                        Some(SortDirection::Ascending) => ", sorted ascending",
                        Some(SortDirection::Descending) => ", sorted descending",
                        None => "",
                    }
                ))
                .when(active_header, |e| e.aria_active_descendant())
                .when(active_header, |element| element.text_color(t.colors.accent_text))
                .flex_1()
                .px(px(cell_padding))
                .on_click(cx.listener(move |s, _, _, cx| {
                    if !s.header_active {
                        cx.emit(ActiveChanged(None));
                    }
                    s.header_active = true;
                    s.active = None;
                    s.active_column = column_index;
                    s.emit_active_cell(cx);
                    s.sort_by(&c2, cx);
                }));
            let header_label = if let Some(renderer) = &self.header_renderer {
                let sort = self.sort.as_ref().filter(|(id, _)| id == &c.id).map(|(_, d)| *d);
                header_label.child(renderer(c, sort, t))
            } else {
                header_label.child(format!("{}{dir}", c.label))
            };
            header = header.child(
                div()
                    .id(format!("header-{id}"))
                    .w(px(c.width as f32))
                    .flex()
                    .items_center()
                    .when(active_header, |element| element.bg(t.colors.accent))
                    .when(!active_header, |element| element.text_color(t.colors.text))
                    .child(header_label)
                    .when(self.resizable, |element| {
                        element.child(
                            div()
                                .id(format!("column-resize-{grip_id}"))
                                .debug_selector(move || grip_selector)
                                .role(gpui_pre::accesskit::Role::Splitter)
                                .aria_label(format!("Resize {} column", c.label))
                                .w(px(t.spacing.xsmall))
                                .h(px(self.header_height.unwrap_or(t.controls.medium)))
                                .bg(t.colors.border)
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |s, event, _, _| {
                                        s.start_resize(&grip_id, event)
                                    }),
                                ),
                        )
                    }),
            );
        }
        root = root.child(header);
        for (ri, row) in self.rows.iter().enumerate() {
            let row_entity = entity.clone();
            let row_selector = row.id.clone();
            let selected = self.selected.contains(&row.id);
            let active = self.active == Some(ri);
            let mut line = div()
                .id(row.id.clone())
                .debug_selector(move || row_selector)
                .role(gpui_pre::accesskit::Role::Row)
                .aria_label(row.cells.join(", "))
                .aria_selected(selected)
                .flex()
                .when(self.cell_renderer.is_some(), |element| element.items_center())
                .min_h(px(self.row_height.unwrap_or(t.controls.medium)))
                .px(px(horizontal_padding))
                .gap(px(self.column_gap.unwrap_or(0.0)))
                .when(self.row_divider, |element| {
                    element.border_b(px(t.borders.hairline)).border_color(t.colors.border)
                })
                .bg(if selected || active {
                    if self.selection_tint {
                        t.colors.accent.opacity(0.12)
                    } else {
                        t.colors.accent
                    }
                } else {
                    t.colors.surface
                })
                .on_click(move |event: &gpui_pre::ClickEvent, window, cx| {
                    row_entity.update(cx, |table, cx| {
                        if let Some(focus) = &table.focus {
                            focus.focus(window, cx);
                        }
                        table.active = Some(ri);
                        table.header_active = false;
                        table.active_column = 0;
                        cx.emit(ActiveChanged(table.rows.get(ri).map(|row| row.id.clone())));
                        table.emit_active_cell(cx);
                        if event.click_count() >= 2 {
                            if let Some(row) = table.rows.get(ri) {
                                cx.emit(RowActivated(row.id.clone()));
                            }
                        } else {
                            table.select_active(cx);
                        }
                    });
                });
            for (ci, value) in row.cells.iter().enumerate() {
                let column = self.columns.get(ci);
                let width = column.map(|c| c.width).unwrap_or(160);
                let active_cell =
                    has_focus && active && self.active_column == ci && !self.header_active;
                let cell = div()
                    .id(format!("{}-cell-{ci}", row.id))
                    .role(gpui_pre::accesskit::Role::GridCell)
                    .aria_label(format!(
                        "{}: {}",
                        self.columns.get(ci).map(|column| column.label.as_str()).unwrap_or("Cell"),
                        value
                    ))
                    .when(active_cell, |element| element.aria_active_descendant())
                    .when(active_cell, |element| {
                        element.border(px(t.borders.regular)).border_color(t.colors.focus)
                    })
                    .w(px(width as f32))
                    .px(px(cell_padding))
                    .text_color(if (selected || active) && !self.selection_tint {
                        t.colors.accent_text
                    } else {
                        t.colors.text
                    });
                line = if let (Some(renderer), Some(column)) = (&self.cell_renderer, column) {
                    line.child(cell.child(renderer(row, column, selected, t)))
                } else {
                    line.child(cell.child(value.clone()))
                };
            }
            root = root.child(line);
        }
        root
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Modifiers, TestAppContext, point};

    #[test]
    fn minimum_width_is_enforced() {
        let mut c = Column::new("name", "Name").width(1);
        assert_eq!(c.width, 24);
        c = c.width(90);
        assert_eq!(c.width, 90);
    }
    #[test]
    fn sort_direction_cycles() {
        assert_eq!(next_sort_direction(None), SortDirection::Ascending);
        assert_eq!(next_sort_direction(Some(SortDirection::Ascending)), SortDirection::Descending);
        assert_eq!(next_sort_direction(Some(SortDirection::Descending)), SortDirection::Ascending);
    }

    #[gpui_pre::test]
    fn pointer_selects_sorts_and_resizes_columns(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (table, visual) = cx.add_window_view(|_, _| {
            DataTable::new(
                "Files",
                vec![Column::new("name", "Name"), Column::new("type", "Type")],
                vec![
                    DataRow::new("row-b", vec!["Beta".into(), "File".into()]),
                    DataRow::new("row-a", vec!["Alpha".into(), "File".into()]),
                ],
            )
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));

        let row = visual.debug_bounds("row-a").unwrap().center();
        visual.simulate_click(row, Modifiers::default());
        assert!(table.read_with(visual, |table, _| table.selection().contains("row-a")));

        let header = visual.debug_bounds("header-label-name").unwrap().center();
        visual.simulate_click(header, Modifiers::default());
        table.read_with(visual, |table, _| {
            assert_eq!(table.sort(), Some(("name", SortDirection::Ascending)));
            assert_eq!(table.rows[0].id, "row-a");
        });

        visual.update(|window, cx| window.draw(cx).clear(cx));
        let grip = visual.debug_bounds("column-resize-name").unwrap().center();
        let end = grip + point(px(30.), px(0.));
        visual.simulate_mouse_down(grip, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        assert_eq!(table.read_with(visual, |table, _| table.columns[0].width), 190);
    }

    #[gpui_pre::test]
    fn keyboard_navigates_cells_sorts_headers_and_selects_rows(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (table, visual) = cx.add_window_view(|_, _| {
            DataTable::new(
                "Keyboard files",
                vec![Column::new("name", "Name"), Column::new("type", "Type")],
                vec![
                    DataRow::new("row-b", vec!["Beta".into(), "File".into()]),
                    DataRow::new("row-a", vec!["Alpha".into(), "Folder".into()]),
                ],
            )
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            table.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("enter"); // Initially active header sorts by Name.
        table.read_with(visual, |table, _| {
            assert_eq!(table.sort(), Some(("name", SortDirection::Ascending)));
            assert_eq!(table.rows[0].id, "row-a");
        });
        visual.simulate_keystrokes("down right end enter");
        table.read_with(visual, |table, _| {
            assert!(!table.header_active);
            assert_eq!(table.active, Some(0));
            assert_eq!(table.active_column, 1);
            assert!(table.selection().contains("row-a"));
        });
        visual.simulate_keystrokes("space");
        assert!(!table.read_with(visual, |table, _| table.selection().contains("row-a")));
        visual.update(|window, cx| window.draw(cx).clear(cx));
    }

    #[gpui_pre::test]
    fn rendered_cells_and_row_replacement_keep_table_state(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (table, visual) = cx.add_window_view(|_, _| {
            DataTable::new(
                "Files",
                vec![Column::new("name", "Name")],
                vec![
                    DataRow::new("old", vec!["Old".into()]),
                    DataRow::new("keep", vec!["Keep".into()]),
                ],
            )
            .with_cell_renderer(|row, _, _, _| {
                let selector = format!("rendered-{}", row.id);
                div()
                    .id(selector.clone())
                    .debug_selector(move || selector.clone())
                    .child(row.cells[0].clone())
                    .into_any_element()
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("rendered-keep").is_some());
        let keep = visual.debug_bounds("keep").unwrap().center();
        visual.simulate_click(keep, Modifiers::default());
        table.update(visual, |table, cx| {
            table.set_rows(
                vec![
                    DataRow::new("keep", vec!["Updated".into()]),
                    DataRow::new("new", vec!["New".into()]),
                ],
                cx,
            )
        });
        table.read_with(visual, |table, _| {
            assert_eq!(table.rows[table.active.unwrap()].id, "keep");
            assert!(table.selection().contains("keep"));
            assert!(!table.selection().contains("old"));
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("rendered-new").is_some());
    }
}
