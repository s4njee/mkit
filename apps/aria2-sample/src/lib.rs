//! Standalone, mock-data consumer of mkit's public widgets.
//! The layout follows `xdl/design_handoff_aria2_download_manager`.

pub mod model;
pub mod theme;
mod views;

use gpui_pre::{
    App, Context, Entity, IntoElement, Render, StatefulInteractiveElement, Subscription, Window,
    div, prelude::*, px,
};
use gpui_pre::{FocusHandle, KeyBinding, actions};
use mkit::{
    button::{Button, Variant as ButtonVariant},
    core::theme::Theme,
    data_table::{
        ActiveChanged, Column, DataRow, DataTable, RowActivated, SelectionChanged, SortDirection,
    },
    dialog::Dialog,
    progress::Progress,
    select::{OptionItem, Select},
    slider::Slider,
    text_field::{InputChanged, TextField},
};
use model::{Category, Download, Scene, Status, fixture_downloads};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

const CATEGORIES: [Category; 5] =
    [Category::All, Category::Downloading, Category::Paused, Category::Completed, Category::Failed];

pub const KEY_CONTEXT: &str = "Aria2Sample";
actions!(aria2_sample, [NextRow, PreviousRow, OpenDetail, Back, AddUrl, Pause, Resume, Remove]);

pub fn install_fonts(cx: &mut App) {
    cx.text_system()
        .add_fonts(vec![
            Cow::Borrowed(include_bytes!("../assets/fonts/IBMPlexSans-Regular.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/IBMPlexSans-Medium.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/IBMPlexSans-SemiBold.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf")),
            Cow::Borrowed(include_bytes!("../assets/fonts/IBMPlexMono-Medium.ttf")),
        ])
        .expect("bundled IBM Plex fonts load");
}

pub fn default_key_bindings() -> [KeyBinding; 9] {
    [
        KeyBinding::new("down", NextRow, Some(KEY_CONTEXT)),
        KeyBinding::new("up", PreviousRow, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", OpenDetail, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", Back, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-n", AddUrl, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-n", AddUrl, Some(KEY_CONTEXT)),
        KeyBinding::new("space", Pause, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-space", Resume, Some(KEY_CONTEXT)),
        KeyBinding::new("delete", Remove, Some(KEY_CONTEXT)),
    ]
}

pub struct Aria2Sample {
    downloads: Vec<Download>,
    category: Category,
    selected: Option<usize>,
    table: Option<Entity<DataTable>>,
    table_downloads: Rc<RefCell<HashMap<usize, Download>>>,
    _table_subscriptions: Vec<Subscription>,
    scene: Scene,
    search: Option<Entity<TextField>>,
    search_text: String,
    _search_subscription: Option<Subscription>,
    url_field: Option<Entity<TextField>>,
    save_to_field: Option<Entity<TextField>>,
    queue_field: Option<Entity<Select>>,
    connections_field: Option<Entity<Slider>>,
    split_field: Option<Entity<TextField>>,
    min_split_field: Option<Entity<TextField>>,
    settings_fields: Vec<Entity<TextField>>,
    dialog: Option<Entity<Dialog>>,
    add_open: bool,
    advanced_open: bool,
    notice: Option<String>,
    schedule: [[u8; 24]; 7],
    limiter_on: bool,
    detail_mode: DetailMode,
    captured: [bool; 4],
    focus: Option<FocusHandle>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DetailMode {
    Lanes,
    Blocks,
    Ribbon,
}

impl Default for Aria2Sample {
    fn default() -> Self {
        Self::new()
    }
}

impl Aria2Sample {
    pub fn new() -> Self {
        let mut schedule = [[0; 24]; 7];
        for (day, hours) in schedule.iter_mut().enumerate() {
            for (hour, cell) in hours.iter_mut().enumerate() {
                *cell = if (1..7).contains(&hour) {
                    2
                } else if day >= 5 || hour >= 19 {
                    1
                } else {
                    0
                };
            }
        }
        let downloads = fixture_downloads();
        let table_downloads =
            Rc::new(RefCell::new(downloads.iter().map(|d| (d.id, d.clone())).collect()));
        Self {
            downloads,
            category: Category::All,
            selected: None,
            table: None,
            table_downloads,
            _table_subscriptions: Vec::new(),
            scene: Scene::Downloads,
            search: None,
            search_text: String::new(),
            _search_subscription: None,
            url_field: None,
            save_to_field: None,
            queue_field: None,
            connections_field: None,
            split_field: None,
            min_split_field: None,
            settings_fields: Vec::new(),
            dialog: None,
            add_open: false,
            advanced_open: true,
            notice: None,
            schedule,
            limiter_on: true,
            detail_mode: DetailMode::Lanes,
            captured: [true, false, true, true],
            focus: None,
        }
    }

    pub fn for_scene(scene: Scene) -> Self {
        let mut sample = Self::new();
        sample.scene = scene;
        sample.selected = Some(0);
        sample
    }

    pub fn for_add_dialog() -> Self {
        let mut sample = Self::new();
        sample.add_open = true;
        sample
    }

    pub fn selected_name(&self) -> Option<&str> {
        self.selected
            .and_then(|id| self.downloads.iter().find(|d| d.id == id))
            .map(|d| d.name.as_str())
    }

    pub fn visible_count(&self) -> usize {
        self.visible_downloads().len()
    }

    pub fn captured_count(&self) -> usize {
        self.captured.iter().filter(|selected| **selected).count()
    }

    fn select_relative(&mut self, step: isize, cx: &mut Context<Self>) {
        let ids: Vec<usize> = self.visible_downloads().iter().map(|row| row.id).collect();
        if ids.is_empty() {
            return;
        }
        let next = self
            .selected
            .and_then(|id| ids.iter().position(|candidate| *candidate == id))
            .map_or(if step < 0 { ids.len() - 1 } else { 0 }, |index| {
                (index as isize + step).clamp(0, ids.len() as isize - 1) as usize
            });
        self.selected = Some(ids[next]);
        cx.notify();
    }

    fn visible_downloads(&self) -> Vec<&Download> {
        let search = self.search_text.to_ascii_lowercase();
        self.downloads
            .iter()
            .filter(|d| {
                self.category.matches(d.status)
                    && (search.is_empty()
                        || d.name.to_ascii_lowercase().contains(&search)
                        || d.host.to_ascii_lowercase().contains(&search))
            })
            .collect()
    }

    fn download_table_rows(&self) -> Vec<DataRow> {
        self.visible_downloads()
            .into_iter()
            .map(|download| {
                DataRow::new(
                    format!("download-{}", download.id),
                    vec![
                        "Select".into(),
                        format!("{}, {}", download.name, download.host),
                        download.size.clone(),
                        format!(
                            "{} percent, {} connections",
                            download.percent, download.connections
                        ),
                        download.speed.clone(),
                        download.eta.clone(),
                        download.status.label().into(),
                    ],
                )
            })
            .collect()
    }

    fn ensure_table(&mut self, cx: &mut Context<Self>) {
        *self.table_downloads.borrow_mut() =
            self.downloads.iter().map(|download| (download.id, download.clone())).collect();
        let rows = self.download_table_rows();
        let selected: HashSet<String> =
            self.selected.map(|id| format!("download-{id}")).into_iter().collect();
        if let Some(table) = &self.table {
            table.update(cx, |table, cx| {
                table.set_rows(rows, cx);
                table.set_selection(selected, cx);
            });
            return;
        }
        let source = self.table_downloads.clone();
        let table = cx.new(|_| {
            DataTable::new(
                "Downloads",
                vec![
                    Column::new("select", "").width(22).sortable(false),
                    Column::new("file", "FILE").width(250),
                    Column::new("size", "SIZE").width(73).sortable(false),
                    Column::new("progress", "PROGRESS").width(160).sortable(false),
                    Column::new("speed", "SPEED").width(88).sortable(false),
                    Column::new("eta", "ETA").width(64).sortable(false),
                    Column::new("status", "STATUS").width(98).sortable(false),
                ],
                rows,
            )
            .with_cell_renderer(move |row, column, selected, theme| {
                let Some(download) = row
                    .id
                    .strip_prefix("download-")
                    .and_then(|id| id.parse::<usize>().ok())
                    .and_then(|id| source.borrow().get(&id).cloned())
                else {
                    return div().child("Missing download").into_any_element();
                };
                render_download_cell(&download, column.id.as_str(), selected, theme)
            })
            .with_header_renderer(|column, sort, theme| {
                let direction = match sort {
                    Some(SortDirection::Ascending) => " ↑",
                    Some(SortDirection::Descending) => " ↓",
                    None => "",
                };
                div()
                    .font_family("IBM Plex Mono")
                    .text_size(px(10.0))
                    .text_color(theme::meta(theme))
                    .child(format!("{}{direction}", column.label))
                    .into_any_element()
            })
            .row_height(60.0)
            .header_height(34.0)
            .column_gap(12.0)
            .horizontal_padding(14.0)
            .cell_padding(0.0)
            .row_divider(true)
            .selection_tint(true)
            .resizable(false)
            .activate_on_enter(true)
        });
        self._table_subscriptions.push(cx.subscribe(
            &table,
            |this, _, event: &SelectionChanged, cx| {
                this.selected = event
                    .0
                    .first()
                    .and_then(|id| id.strip_prefix("download-"))
                    .and_then(|id| id.parse().ok());
                cx.notify();
            },
        ));
        self._table_subscriptions.push(cx.subscribe(
            &table,
            |this, _, event: &ActiveChanged, cx| {
                this.selected = event
                    .0
                    .as_deref()
                    .and_then(|id| id.strip_prefix("download-"))
                    .and_then(|id| id.parse().ok());
                cx.notify();
            },
        ));
        self._table_subscriptions.push(cx.subscribe(
            &table,
            |this, _, event: &RowActivated, cx| {
                this.selected = event.0.strip_prefix("download-").and_then(|id| id.parse().ok());
                if this.selected.is_some() {
                    this.scene = Scene::Detail;
                }
                cx.notify();
            },
        ));
        self.table = Some(table);
    }

    fn ensure_search(&mut self, cx: &mut Context<Self>) {
        if self.search.is_some() {
            return;
        }
        let search = cx.new(|cx| {
            TextField::new(cx).with_label("Filter downloads").with_placeholder("Filter downloads")
        });
        self._search_subscription =
            Some(cx.subscribe(&search, |this, _, value: &InputChanged, cx| {
                this.search_text = value.0.clone();
                cx.notify();
            }));
        self.search = Some(search);
    }

    fn ensure_settings_fields(&mut self, cx: &mut Context<Self>) {
        if !self.settings_fields.is_empty() {
            return;
        }
        for (label, value) in [
            ("RPC host", "127.0.0.1"),
            ("RPC port", "6800"),
            ("RPC secret token", ""),
            ("Max concurrent downloads", "3"),
            ("Connections per server", "16"),
            ("Minimum split size", "10M"),
            ("Global download limit", "25M"),
            ("Disk cache", "64M"),
            ("File allocation", "falloc"),
        ] {
            let field = cx.new(|cx| {
                TextField::new(cx)
                    .with_label(label)
                    .secure(label == "RPC secret token")
                    .with_placeholder(if value.is_empty() { "Optional" } else { value })
            });
            if !value.is_empty() {
                field.update(cx, |field, cx| field.set_value(value, cx));
            }
            self.settings_fields.push(field);
        }
    }

    fn open_add(&mut self, cx: &mut Context<Self>) {
        if self.url_field.is_none() {
            self.url_field = Some(cx.new(|cx| {
                TextField::new(cx)
                    .with_label("Download URL")
                    .with_placeholder("https://example.org/file.iso")
            }));
        }
        self.add_open = true;
        self.dialog = None;
        cx.notify();
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        if self.add_open {
            self.add_open = false;
            self.dialog = None;
        } else {
            self.scene = Scene::Downloads;
        }
        cx.notify();
    }

    fn start_download(&mut self, url: String, cx: &mut Context<Self>) {
        let url = url.trim();
        if url.is_empty() {
            self.notice = Some("Enter a URL to add a download.".into());
            cx.notify();
            return;
        }
        let id = self.downloads.iter().map(|row| row.id).max().unwrap_or(0) + 1;
        let mut download = Download::from_url(id, url);
        let queue = self
            .queue_field
            .as_ref()
            .and_then(|field| field.read(cx).value())
            .unwrap_or("main")
            .to_owned();
        let save_to = self
            .save_to_field
            .as_ref()
            .map(|field| field.read(cx).text().trim().to_owned())
            .unwrap_or_default();
        let connections = self
            .connections_field
            .as_ref()
            .and_then(|field| field.read(cx).values().first().copied())
            .unwrap_or(16.0)
            .round() as u8;
        let split_text = self
            .split_field
            .as_ref()
            .map(|field| field.read(cx).text().trim().to_owned())
            .unwrap_or_default();
        let split = if split_text.is_empty() {
            16
        } else if let Ok(value @ 1..=32) = split_text.parse::<u8>() {
            value
        } else {
            self.notice = Some("Split count must be between 1 and 32.".into());
            cx.notify();
            return;
        };
        let min_split = self
            .min_split_field
            .as_ref()
            .map(|field| field.read(cx).text().trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "10M".into());
        download.connections = connections;
        download.queue = queue.clone();
        download.save_to = if save_to.is_empty() { "~/Downloads/ISO".into() } else { save_to };
        download.split = split;
        download.min_split_size = min_split;
        self.notice = Some(format!(
            "Added {} to {} queue · {} connections · sample data only",
            download.name, queue, connections
        ));
        self.downloads.insert(0, download);
        self.category = Category::All;
        self.scene = Scene::Downloads;
        self.add_open = false;
        self.dialog = None;
        self.selected = Some(id);
        if let Some(field) = &self.url_field {
            field.update(cx, |field, cx| field.set_value("", cx));
        }
        cx.notify();
    }

    fn change_selected_status(&mut self, status: Status, cx: &mut Context<Self>) {
        if let Some(row) =
            self.selected.and_then(|id| self.downloads.iter_mut().find(|row| row.id == id))
        {
            row.status = status;
            row.speed =
                if status == Status::Downloading { "4.2 MB/s".into() } else { "—".into() };
            row.eta = if status == Status::Downloading { "03:18".into() } else { "—".into() };
            self.notice = Some(format!("{}: {} · local sample state", row.name, status.label()));
            cx.notify();
        }
    }

    fn remove_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected.take() {
            self.downloads.retain(|row| row.id != id);
            self.notice = Some("Removed from the sample list. No files were changed.".into());
            cx.notify();
        }
    }

    fn mkit_button(
        label: &'static str,
        id: usize,
        variant: ButtonVariant,
        enabled: bool,
        owner: Entity<Self>,
        action: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> Button {
        Button::new(label).id(id).variant(variant).disabled(!enabled).on_activate(move |_, cx| {
            owner.update(cx, |this, cx| action(this, cx));
        })
    }

    fn titlebar(&self, t: Theme) -> gpui_pre::Div {
        let circle = || div().size(px(11.0)).rounded(px(6.0)).bg(theme::meta(t));
        div()
            .h(px(38.0))
            .px(px(12.0))
            .flex()
            .items_center()
            .gap(px(10.0))
            .bg(theme::subtle(t))
            .border_b(px(1.0))
            .border_color(t.colors.border)
            .child(div().flex().gap(px(7.0)).child(circle()).child(circle()).child(circle()))
            .child(
                div()
                    .flex_1()
                    .text_center()
                    .text_size(px(12.0))
                    .text_color(t.colors.text_muted)
                    .child("Aria2 Manager"),
            )
            .child(
                div()
                    .px(px(8.0))
                    .py(px(3.0))
                    .rounded(px(t.radii.pill))
                    .bg(theme::tint(t))
                    .text_size(px(10.5))
                    .text_color(t.colors.accent)
                    .child("●  Sample data"),
            )
    }

    fn toolbar(&self, t: Theme, cx: &mut Context<Self>) -> gpui_pre::Div {
        let owner = cx.entity();
        let selected = self.selected.is_some();
        let divider = || div().w(px(1.0)).h(px(22.0)).mx(px(3.0)).bg(t.colors.border);
        div()
            .h(px(55.0))
            .px(px(12.0))
            .flex()
            .items_center()
            .gap(px(8.0))
            .bg(t.colors.surface)
            .border_b(px(1.0))
            .border_color(t.colors.border)
            .child(Self::mkit_button(
                "＋ Add URL",
                1,
                ButtonVariant::Default,
                true,
                owner.clone(),
                |s, cx| s.open_add(cx),
            ))
            .child(divider())
            .child(Self::mkit_button(
                "▶ Resume",
                2,
                ButtonVariant::Outline,
                selected,
                owner.clone(),
                |s, cx| s.change_selected_status(Status::Downloading, cx),
            ))
            .child(Self::mkit_button(
                "Ⅱ Pause",
                3,
                ButtonVariant::Outline,
                selected,
                owner.clone(),
                |s, cx| s.change_selected_status(Status::Paused, cx),
            ))
            .child(Self::mkit_button(
                "✕ Remove",
                4,
                ButtonVariant::Outline,
                selected,
                owner.clone(),
                |s, cx| s.remove_selected(cx),
            ))
            .child(divider())
            .child(Self::mkit_button(
                "Queues",
                5,
                ButtonVariant::Outline,
                true,
                owner.clone(),
                |s, cx| {
                    s.scene = Scene::Queues;
                    cx.notify();
                },
            ))
            .child(Self::mkit_button(
                "Settings",
                6,
                ButtonVariant::Outline,
                true,
                owner.clone(),
                |s, cx| {
                    s.scene = Scene::Settings;
                    cx.notify();
                },
            ))
            .child(Self::mkit_button("Capture", 7, ButtonVariant::Outline, true, owner, |s, cx| {
                s.scene = Scene::Capture;
                cx.notify();
            }))
            .child(div().flex_1())
            .child(
                div().w(px(210.0)).child(self.search.as_ref().expect("search initialized").clone()),
            )
    }

    fn sidebar(&self, t: Theme, cx: &mut Context<Self>) -> gpui_pre::Div {
        let mut side = div()
            .w(px(196.0))
            .h_full()
            .flex_none()
            .p(px(10.0))
            .pt(px(14.0))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .bg(t.colors.elevated_surface)
            .border_r(px(1.0))
            .border_color(t.colors.border)
            .child(section_label("CATEGORIES", t));
        for category in CATEGORIES {
            let selected = self.category == category;
            let count = self.downloads.iter().filter(|row| category.matches(row.status)).count();
            side = side.child(
                div()
                    .id(format!("category-{}", category.label()))
                    .debug_selector(move || format!("category-{}", category.label()))
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label(format!("{}, {count}", category.label()))
                    .h(px(31.0))
                    .px(px(9.0))
                    .flex()
                    .items_center()
                    .gap(px(9.0))
                    .rounded(px(6.0))
                    .bg(if selected { theme::tint(t) } else { t.colors.elevated_surface })
                    .text_color(if selected { t.colors.accent } else { t.colors.text_muted })
                    .child(div().flex_1().child(category.label()))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(if selected { t.colors.accent } else { theme::meta(t) })
                            .child(count.to_string()),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.category = category;
                        this.scene = Scene::Downloads;
                        cx.notify();
                    })),
            );
        }
        side = side.child(div().h(px(15.0))).child(section_label("QUEUES", t));
        for (i, (label, color)) in [
            ("Main queue", t.colors.accent),
            ("Overnight", t.colors.warning),
            ("Datasets", t.colors.danger),
        ]
        .into_iter()
        .enumerate()
        {
            side = side.child(
                div()
                    .id(format!("queue-{i}"))
                    .h(px(31.0))
                    .px(px(9.0))
                    .flex()
                    .items_center()
                    .gap(px(9.0))
                    .rounded(px(6.0))
                    .text_color(t.colors.text_muted)
                    .child(div().size(px(7.0)).rounded(px(2.0)).bg(color))
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.scene = Scene::Queues;
                        cx.notify();
                    })),
            );
        }
        let limiter_bg = if self.limiter_on { t.colors.accent } else { t.colors.disabled };
        side.child(div().flex_1()).child(
            div()
                .id("speed-limiter")
                .p(px(12.0))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(t.colors.border)
                .bg(t.colors.surface)
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child(div().text_size(px(11.0)).child("Speed limiter"))
                        .child(
                            div()
                                .text_size(px(10.5))
                                .text_color(limiter_bg)
                                .child(if self.limiter_on { "ON" } else { "OFF" }),
                        ),
                )
                .child(
                    div().h(px(5.0)).rounded(px(3.0)).bg(theme::track(t)).child(
                        div()
                            .h_full()
                            .w(px(if self.limiter_on { 95.0 } else { 0.0 }))
                            .rounded(px(3.0))
                            .bg(limiter_bg),
                    ),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(t.colors.text_muted)
                        .child("25 MB/s max · 16 conn/file"),
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.limiter_on = !this.limiter_on;
                    cx.notify();
                })),
        )
    }

    fn footer(&self, t: Theme) -> gpui_pre::Div {
        let activity =
            self.downloads.iter().filter(|row| row.status == Status::Downloading).count();
        div()
            .h(px(43.0))
            .px(px(14.0))
            .flex()
            .items_center()
            .gap(px(8.0))
            .bg(t.colors.elevated_surface)
            .border_t(px(1.0))
            .border_color(t.colors.border)
            .child(
                div()
                    .font_family("IBM Plex Mono")
                    .text_size(px(13.0))
                    .text_color(t.colors.accent)
                    .child("24.5"),
            )
            .child(
                div()
                    .font_family("IBM Plex Mono")
                    .text_size(px(10.5))
                    .text_color(t.colors.text_muted)
                    .child("MB/s down"),
            )
            .child(
                div().w(px(150.0)).h(px(24.0)).flex().items_end().gap(px(5.0)).children(
                    [
                        9.0, 12.0, 10.0, 17.0, 14.0, 21.0, 18.0, 23.0, 20.0, 24.0, 19.0, 23.0,
                        25.0, 21.0, 24.0,
                    ]
                    .into_iter()
                    .map(|height| div().w(px(5.0)).h(px(height)).bg(t.colors.accent.opacity(0.5))),
                ),
            )
            .child(div().flex_1())
            .child(
                div()
                    .font_family("IBM Plex Mono")
                    .text_size(px(11.0))
                    .text_color(t.colors.text_muted)
                    .child(format!("{activity} active · 48 connections · 12.1 GB remaining")),
            )
    }

    fn downloads_view(&self, t: Theme) -> impl IntoElement {
        div()
            .id("downloads-grid")
            .flex_1()
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .bg(t.colors.surface)
            .child(
                div()
                    .id("table-scroll")
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_y_scroll()
                    .child(self.table.as_ref().expect("download table initialized").clone()),
            )
            .child(self.footer(t))
    }

    fn add_dialog(&mut self, t: Theme, cx: &mut Context<Self>) -> Option<Entity<Dialog>> {
        if !self.add_open {
            return None;
        }
        if self.url_field.is_none() {
            self.url_field = Some(cx.new(|cx| {
                TextField::new(cx)
                    .with_label("Download URL")
                    .with_placeholder("https://example.org/file.iso")
            }));
        }
        if self.save_to_field.is_none() {
            self.save_to_field = Some(cx.new(|cx| {
                TextField::new(cx).with_label("Save to").with_placeholder("~/Downloads/ISO")
            }));
        }
        if self.queue_field.is_none() {
            self.queue_field = Some(cx.new(|_| {
                Select::new(
                    "Queue",
                    vec![
                        OptionItem::new("main", "Main"),
                        OptionItem::new("later", "Later"),
                        OptionItem::new("night", "Night"),
                    ],
                    Some("main".into()),
                )
            }));
        }
        if self.connections_field.is_none() {
            self.connections_field =
                Some(cx.new(|_| Slider::new("Connections", 16.0, 1.0, 32.0, 1.0)));
        }
        if self.split_field.is_none() {
            self.split_field = Some(
                cx.new(|cx| TextField::new(cx).with_label("Split count").with_placeholder("16")),
            );
        }
        if self.min_split_field.is_none() {
            self.min_split_field = Some(cx.new(|cx| {
                TextField::new(cx).with_label("Minimum split size").with_placeholder("10M")
            }));
        }
        if self.dialog.is_none() {
            let owner = cx.entity();
            let field = self.url_field.as_ref().expect("URL field initialized").clone();
            let save_to = self.save_to_field.as_ref().expect("save field initialized").clone();
            let queue = self.queue_field.as_ref().expect("queue initialized").clone();
            let connections =
                self.connections_field.as_ref().expect("connections initialized").clone();
            let split = self.split_field.as_ref().expect("split initialized").clone();
            let min_split = self.min_split_field.as_ref().expect("min split initialized").clone();
            let advanced_open = self.advanced_open;
            let dialog = cx.new(|_| Dialog::with_content("Add download", move || {
                let owner_start = owner.clone();
                let owner_cancel = owner.clone();
                let owner_advanced = owner.clone();
                let input = field.clone();
                div().w(px(480.0)).flex().flex_col().gap(px(16.0))
                    .child(div().text_size(px(11.5)).text_color(t.colors.text_muted)
                        .child("Paste one or more URLs. Mirrors on separate lines download in parallel."))
                    .child(div().flex().flex_col().gap(px(6.0))
                        .child(section_label("URL", t))
                        .child(input.clone())
                        .child(div().text_size(px(11.0)).text_color(t.colors.success)
                            .child("Sample mode · network resolution is disabled")))
                    .child(div().flex().gap(px(12.0))
                        .child(div().flex_1().flex().flex_col().gap(px(6.0))
                            .child(section_label("SAVE TO", t))
                            .child(save_to.clone()))
                        .child(div().w(px(98.0)).flex().flex_col().gap(px(6.0))
                            .child(section_label("QUEUE", t))
                            .child(queue.clone())))
                    .child(section_label("CONNECTIONS", t))
                    .child(connections.clone())
                    .child(div().flex().justify_between().font_family("IBM Plex Mono").text_size(px(10.0))
                        .text_color(theme::meta(t)).child("1").child("32 max (server-limited)"))
                    .child(div().id("advanced-toggle").debug_selector(|| "advanced-toggle".into())
                        .border_t(px(1.0)).border_color(t.colors.border).pt(px(12.0))
                        .child(Button::new(if advanced_open { "▾ Advanced (aria2 options)" } else { "▸ Advanced (aria2 options)" })
                            .id(53).variant(ButtonVariant::Outline).on_activate(move |_, cx| {
                                owner_advanced.update(cx, |s, cx| { s.advanced_open = !s.advanced_open; s.dialog = None; cx.notify(); });
                            })))
                    .when(advanced_open, |body| body.child(div().flex().gap(px(12.0))
                        .child(div().flex_1().flex().flex_col().gap(px(6.0)).child(section_label("SPLIT COUNT", t)).child(split.clone()))
                        .child(div().flex_1().flex().flex_col().gap(px(6.0)).child(section_label("MIN SPLIT SIZE", t)).child(min_split.clone()))))
                    .child(div().flex().justify_end().gap(px(10.0)).pt(px(10.0))
                        .child(Button::new("Cancel").id(51).variant(ButtonVariant::Outline).on_activate(move |_, cx| {
                            owner_cancel.update(cx, |s, cx| { s.add_open = false; s.dialog = None; cx.notify(); });
                        }))
                        .child(Button::new("Start download").id(52).on_activate(move |_, cx| {
                            let url = input.read(cx).text().to_owned();
                            owner_start.update(cx, |s, cx| s.start_download(url, cx));
                        })))
            }));
            self.dialog = Some(dialog);
        }
        self.dialog.clone()
    }
}

impl Render for Aria2Sample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_search(cx);
        if self.scene == Scene::Downloads {
            self.ensure_table(cx);
        }
        if self.scene == Scene::Settings {
            self.ensure_settings_fields(cx);
        }
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        if window.focused(cx).is_none() {
            focus.focus(window, cx);
        }
        let t = *cx.global::<Theme>();
        let body = match self.scene {
            Scene::Downloads => self.downloads_view(t).into_any_element(),
            Scene::Detail => self.detail_view(t, cx).into_any_element(),
            Scene::Queues => self.queues_view(t, cx).into_any_element(),
            Scene::Settings => self.settings_view(t, cx).into_any_element(),
            Scene::Capture => self.capture_view(t, cx).into_any_element(),
        };
        let dialog = self.add_dialog(t, cx);
        div()
            .id("aria2-sample-root")
            .relative()
            .size_full()
            .bg(t.colors.background)
            .font_family("IBM Plex Sans")
            .text_size(px(t.typography.body))
            .text_color(t.colors.text)
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .tab_index(0)
            .on_action(cx.listener(|this, _: &NextRow, _, cx| this.select_relative(1, cx)))
            .on_action(cx.listener(|this, _: &PreviousRow, _, cx| this.select_relative(-1, cx)))
            .on_action(cx.listener(|this, _: &OpenDetail, _, cx| {
                if this.selected.is_some() {
                    this.scene = Scene::Detail;
                    cx.notify();
                }
            }))
            .on_action(cx.listener(|this, _: &Back, _, cx| this.back(cx)))
            .on_action(cx.listener(|this, _: &AddUrl, _, cx| this.open_add(cx)))
            .on_action(
                cx.listener(|this, _: &Pause, _, cx| {
                    this.change_selected_status(Status::Paused, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &Resume, _, cx| {
                this.change_selected_status(Status::Downloading, cx)
            }))
            .on_action(cx.listener(|this, _: &Remove, _, cx| this.remove_selected(cx)))
            .flex()
            .flex_col()
            .child(
                div()
                    .id("aria2-background")
                    .size_full()
                    .flex()
                    .flex_col()
                    .role(gpui_pre::accesskit::Role::Group)
                    .when(self.add_open, |view| {
                        view.a11y_synthetic_children(|builder| builder.parent_node().set_hidden())
                    })
                    .child(self.titlebar(t))
                    .child(self.toolbar(t, cx))
                    .child(
                        div().flex_1().flex().min_h(px(0.0)).child(self.sidebar(t, cx)).child(body),
                    ),
            )
            .when_some(self.notice.clone(), |root, notice| {
                root.child(
                    div()
                        .absolute()
                        .bottom(px(52.0))
                        .right(px(18.0))
                        .p(px(10.0))
                        .rounded(px(6.0))
                        .bg(t.colors.text)
                        .text_color(t.colors.surface)
                        .text_size(px(11.0))
                        .child(notice),
                )
            })
            .when_some(dialog, |root, dialog| root.child(dialog))
    }
}

fn section_label(text: &'static str, t: Theme) -> gpui_pre::Div {
    div().font_family("IBM Plex Mono").text_size(px(10.0)).text_color(theme::meta(t)).child(text)
}

fn render_download_cell(
    row: &Download,
    column: &str,
    selected: bool,
    t: Theme,
) -> gpui_pre::AnyElement {
    match column {
        "select" => div()
            .size(px(16.0))
            .rounded(px(4.0))
            .border(px(1.0))
            .border_color(if selected { t.colors.accent } else { t.colors.border })
            .bg(if selected { t.colors.accent } else { t.colors.surface })
            .text_color(t.colors.accent_text)
            .text_size(px(11.0))
            .text_center()
            .child(if selected { "✓" } else { "" })
            .into_any_element(),
        "file" => div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(
                div()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_size(px(12.5))
                    .text_color(t.colors.text)
                    .child(row.name.clone()),
            )
            .child(
                div()
                    .overflow_hidden()
                    .text_ellipsis()
                    .font_family("IBM Plex Mono")
                    .text_size(px(10.5))
                    .text_color(theme::meta(t))
                    .child(row.host.clone()),
            )
            .into_any_element(),
        "progress" => {
            let mut segments =
                div().h(px(6.0)).w_full().flex().gap(px(1.0)).rounded(px(3.0)).bg(theme::track(t));
            for i in 0..row.connections {
                let threshold = (i as f32 + 0.5) / row.connections as f32 * 100.0;
                let fill = if threshold < row.percent as f32 - 100.0 / row.connections as f32 {
                    t.colors.accent
                } else if threshold < row.percent as f32 + 100.0 / row.connections as f32 {
                    if row.status == Status::Paused {
                        t.colors.warning
                    } else {
                        t.colors.accent.opacity(0.48)
                    }
                } else {
                    theme::track(t)
                };
                segments = segments.child(div().flex_1().h_full().bg(fill));
            }
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(segments)
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .font_family("IBM Plex Mono")
                        .text_size(px(10.0))
                        .text_color(theme::meta(t))
                        .child(format!("{}%", row.percent))
                        .child(if row.status == Status::Downloading {
                            format!("{} conn", row.connections)
                        } else {
                            String::new()
                        }),
                )
                .into_any_element()
        }
        "status" => {
            let status_color = match row.status {
                Status::Downloading => t.colors.accent,
                Status::Completed => t.colors.success,
                Status::Paused => t.colors.text_muted,
                Status::Queued => t.colors.warning,
                Status::Error => t.colors.danger,
            };
            div()
                .px(px(8.0))
                .py(px(3.0))
                .rounded(px(5.0))
                .bg(if row.status == Status::Downloading {
                    theme::tint(t)
                } else {
                    t.colors.elevated_surface
                })
                .text_size(px(10.0))
                .font_family("IBM Plex Mono")
                .text_color(status_color)
                .child(row.status.label())
                .into_any_element()
        }
        _ => div()
            .font_family("IBM Plex Mono")
            .text_size(px(11.0))
            .text_color(if column == "speed" && row.status == Status::Downloading {
                t.colors.accent
            } else {
                t.colors.text_muted
            })
            .child(match column {
                "size" => row.size.clone(),
                "speed" => row.speed.clone(),
                "eta" => row.eta.clone(),
                _ => String::new(),
            })
            .into_any_element(),
    }
}

#[cfg(test)]
mod form_tests {
    use super::*;
    use gpui_pre::{
        Focusable, Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext,
    };

    #[gpui_pre::test]
    fn add_dialog_applies_mkit_control_values_to_mock_download(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::base::init(cx);
            install_fonts(cx);
            theme::install(cx, theme::LIGHT);
        });
        let (sample, visual) = cx.add_window_view(|_, _| Aria2Sample::for_add_dialog());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        sample.update(visual, |sample, cx| {
            sample
                .save_to_field
                .as_ref()
                .unwrap()
                .update(cx, |field, cx| field.set_value("~/Downloads/Test", cx));
            sample
                .queue_field
                .as_ref()
                .unwrap()
                .update(cx, |field, cx| field.set_value(Some("night".into()), cx));
            sample
                .connections_field
                .as_ref()
                .unwrap()
                .update(cx, |field, cx| field.set_value(vec![8.0], cx));
            sample.split_field.as_ref().unwrap().update(cx, |field, cx| field.set_value("4", cx));
            sample
                .min_split_field
                .as_ref()
                .unwrap()
                .update(cx, |field, cx| field.set_value("20M", cx));
            sample.start_download("https://example.org/test.iso".into(), cx);
            let added = &sample.downloads[0];
            assert_eq!(added.queue, "night");
            assert_eq!(added.save_to, "~/Downloads/Test");
            assert_eq!(added.connections, 8);
            assert_eq!(added.split, 4);
            assert_eq!(added.min_split_size, "20M");
        });
    }

    #[gpui_pre::test]
    fn add_dialog_tabs_into_mkit_fields(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::base::init(cx);
            install_fonts(cx);
            theme::install(cx, theme::LIGHT);
            cx.bind_keys(mkit::dialog::default_key_bindings());
            cx.bind_keys(mkit::text_field::default_key_bindings());
        });
        let (sample, visual) = cx.add_window_view(|_, _| Aria2Sample::for_add_dialog());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("tab");
        visual.update(|window, cx| {
            let field = sample.read(cx).url_field.as_ref().unwrap().read(cx).focus_handle(cx);
            assert!(field.is_focused(window), "first Tab should focus the URL field");
        });
        visual.simulate_keystrokes("tab");
        visual.update(|window, cx| {
            let field = sample.read(cx).save_to_field.as_ref().unwrap().read(cx).focus_handle(cx);
            assert!(field.is_focused(window), "second Tab should focus Save to");
        });
        for _ in 0..12 {
            visual.simulate_keystrokes("tab");
            visual.update(|window, cx| {
                let dialog = sample.read(cx).dialog.as_ref().unwrap().read(cx).focus_handle(cx);
                assert!(dialog.contains_focused(window, cx), "Tab must remain in the dialog");
            });
        }
    }

    #[gpui_pre::test]
    fn double_click_activates_a_download_row(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::base::init(cx);
            install_fonts(cx);
            theme::install(cx, theme::LIGHT);
        });
        let (sample, visual) = cx.add_window_view(|_, _| Aria2Sample::new());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let position = visual.debug_bounds("download-0").unwrap().center();
        visual.simulate_event(MouseDownEvent {
            position,
            modifiers: Modifiers::default(),
            button: MouseButton::Left,
            click_count: 2,
            first_mouse: false,
        });
        visual.simulate_event(MouseUpEvent {
            position,
            modifiers: Modifiers::default(),
            button: MouseButton::Left,
            click_count: 2,
        });
        sample.read_with(visual, |sample, _| assert_eq!(sample.scene, Scene::Detail));
    }

    #[gpui_pre::test]
    fn advanced_form_disclosure_toggles(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::base::init(cx);
            install_fonts(cx);
            theme::install(cx, theme::LIGHT);
        });
        let (sample, visual) = cx.add_window_view(|_, _| Aria2Sample::for_add_dialog());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let toggle = visual.debug_bounds("advanced-toggle").unwrap().center();
        visual.simulate_click(toggle, Modifiers::default());
        sample.read_with(visual, |sample, _| assert!(!sample.advanced_open));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let toggle = visual.debug_bounds("advanced-toggle").unwrap().center();
        visual.simulate_click(toggle, Modifiers::default());
        sample.read_with(visual, |sample, _| assert!(sample.advanced_open));
    }
}
