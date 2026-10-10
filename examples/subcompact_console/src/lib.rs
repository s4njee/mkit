//! A dense, terminal-style operations console under the subcompact design system.
//!
//! Every control is a registry component reading the installed [`Theme`]; the console adds no
//! sizes or colours of its own beyond layout widths. The palette control switches between the
//! three subcompact palettes at runtime.

use gpui_pre::{
    AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, Subscription, Window, div,
    prelude::*, px,
};
use mkit::{
    badge::{Badge, Status},
    breadcrumbs::{Breadcrumbs, Crumb},
    button::{Button, Size as ButtonSize, Variant as ButtonVariant},
    checkbox::{self, Checkbox},
    core::{
        subcompact::{self, AMBER, PAPER, PHOSPHOR},
        theme::{Theme, set_theme},
    },
    data_table::{self, Column, DataRow, DataTable, SortDirection},
    form_layout::{FormLayout, FormRow},
    key_hint::{KeyChord, KeyHint},
    progress::Progress,
    segmented_control::{self, Item as SegmentItem, SegmentedControl, ValueChanged},
    slider::{self, Slider},
    status_bar::{StatusBar, StatusBarItem},
    switch::{self, Switch},
    tabs::{self, Item as TabItem, Tabs},
    text_field::{self, TextField},
    tree::{self, Tree, TreeNode},
};
use std::collections::HashSet;

/// Install the default, rebindable keys for every component the console composes.
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys(checkbox::default_key_bindings());
    cx.bind_keys(data_table::default_key_bindings());
    cx.bind_keys(segmented_control::default_key_bindings());
    cx.bind_keys(slider::default_key_bindings());
    cx.bind_keys(switch::default_key_bindings());
    cx.bind_keys(tabs::default_key_bindings());
    cx.bind_keys(text_field::default_key_bindings());
    cx.bind_keys(tree::default_key_bindings());
}

/// The palette keys the console's palette control offers, in order.
pub const PALETTE_KEYS: [&str; 3] = ["phosphor", "amber", "paper"];

/// The subcompact palette for a palette-control key.
pub fn palette(key: &str) -> Option<Theme> {
    match key {
        "phosphor" => Some(PHOSPHOR),
        "amber" => Some(AMBER),
        "paper" => Some(PAPER),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum Level {
    Info,
    Ok,
    Warn,
    Error,
}

const LOG: [(&str, Level, &str); 7] = [
    ("12:04:01.112", Level::Info, "scheduler: 14 jobs queued on worker pool `edge-3`"),
    ("12:04:01.287", Level::Ok, "deploy: build 7f3c9a2 promoted to canary (3/24 nodes)"),
    ("12:04:02.004", Level::Info, "net: 1,284 open sockets, 0.4% retransmit"),
    ("12:04:02.519", Level::Warn, "disk: /var/log at 81% on ingest-02"),
    ("12:04:03.770", Level::Info, "cache: hit ratio 97.2% over the last 60 s"),
    ("12:04:04.031", Level::Error, "auth: 3 failed logins for `svc-backup` from 10.4.2.19"),
    ("12:04:04.588", Level::Ok, "health: all probes green on cluster `fra-1`"),
];

const PROCESSES: [[&str; 6]; 9] = [
    ["4021", "ingestd", "svc", "38.2", "1.9G", "run"],
    ["4188", "postgres", "db", "21.7", "6.4G", "run"],
    ["3310", "edge-proxy", "net", "12.4", "412M", "run"],
    ["5102", "indexer", "svc", "9.8", "2.2G", "run"],
    ["2207", "metricsd", "obs", "4.1", "188M", "sleep"],
    ["6650", "backup", "ops", "2.6", "96M", "wait"],
    ["1093", "sshd", "root", "0.3", "12M", "sleep"],
    ["7731", "cron", "root", "0.1", "4M", "sleep"],
    ["8012", "rotate-logs", "ops", "0.0", "2M", "stop"],
];

/// The console view. It owns every stateful component so their state survives a palette switch.
pub struct SubcompactConsole {
    palette: Entity<SegmentedControl>,
    tabs: Entity<Tabs>,
    tree: Entity<Tree>,
    table: Entity<DataTable>,
    host: Entity<TextField>,
    verbose: Entity<Switch>,
    follow: Entity<Checkbox>,
    rate: Entity<Slider>,
    _palette_changed: Subscription,
}

impl SubcompactConsole {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let current = cx.try_global::<Theme>().map(|theme| theme.name).unwrap_or(PHOSPHOR.name);
        let selected = PALETTE_KEYS
            .into_iter()
            .find(|key| palette(key).is_some_and(|theme| theme.name == current))
            .unwrap_or("phosphor");
        let palette = cx.new(|_| {
            SegmentedControl::new(
                "Palette",
                PALETTE_KEYS.iter().map(|key| SegmentItem::new(*key, *key)).collect(),
                Some(selected.into()),
            )
        });
        let palette_changed = cx.subscribe(&palette, |this, _, event: &ValueChanged, cx| {
            if let Some(key) = event.0.as_deref() {
                this.select_palette(key, cx);
            }
        });
        let tabs = cx.new(|_| {
            Tabs::new(
                "Console views",
                vec![
                    TabItem::new("procs", "procs"),
                    TabItem::new("sockets", "sockets"),
                    TabItem::new("env", "env"),
                ],
                Some("procs".into()),
            )
        });
        let tree = cx.new(|_| {
            Tree::new(
                "Hosts",
                vec![
                    TreeNode::branch(
                        "fra-1",
                        "fra-1",
                        vec![
                            TreeNode::leaf("edge-3", "edge-3"),
                            TreeNode::leaf("ingest-02", "ingest-02"),
                            TreeNode::leaf("db-01", "db-01"),
                        ],
                    ),
                    TreeNode::branch(
                        "iad-2",
                        "iad-2",
                        vec![TreeNode::leaf("edge-7", "edge-7"), TreeNode::leaf("db-04", "db-04")],
                    ),
                    TreeNode::leaf("bastion", "bastion"),
                ],
            )
        });
        tree.update(cx, |tree, cx| tree.set_expanded(HashSet::from(["fra-1".to_owned()]), cx));
        let table = cx.new(|_| {
            DataTable::new(
                "Processes",
                vec![
                    Column::new("pid", "PID").width(56),
                    Column::new("name", "NAME").width(120),
                    Column::new("user", "USER").width(56),
                    Column::new("cpu", "CPU%").width(56),
                    Column::new("mem", "RSS").width(56),
                    Column::new("state", "STATE").width(64),
                ],
                PROCESSES
                    .iter()
                    .map(|row| {
                        DataRow::new(row[0], row.iter().map(|cell| (*cell).into()).collect())
                    })
                    .collect(),
            )
            .multi_select()
        });
        table.update(cx, |table, cx| {
            table.set_selection(HashSet::from(["4188".to_owned()]), cx);
            table.set_sort(Some(("cpu".into(), SortDirection::Descending)), cx);
        });
        let host = cx.new(|cx| {
            let mut field = TextField::new(cx).with_label("Host");
            field.set_value("ingest-02.fra-1", cx);
            field
        });
        Self {
            palette,
            tabs,
            tree,
            table,
            host,
            verbose: cx.new(|_| Switch::new("Verbose", true)),
            follow: cx.new(|_| Checkbox::new("Follow tail", true)),
            rate: cx.new(|_| Slider::new("Sample rate", 250.0, 50.0, 1000.0, 50.0)),
            _palette_changed: palette_changed,
        }
    }

    // ANCHOR: subcompact_palette_switch
    /// Install the palette for `key` and redraw. Unknown keys are ignored.
    pub fn select_palette(&mut self, key: &str, cx: &mut Context<Self>) {
        if let Some(theme) = palette(key) {
            set_theme(cx, theme);
            cx.notify();
        }
    }
    // ANCHOR_END: subcompact_palette_switch
}

/// An uppercase pane heading drawn as a ruled line, like a terminal multiplexer's pane title.
fn pane_title(t: &Theme, title: &'static str) -> impl IntoElement {
    div()
        .px(px(t.spacing.small))
        .py(px(t.spacing.xsmall))
        .border_b(px(t.borders.hairline))
        .border_color(t.colors.border)
        .text_size(px(t.typography.caption))
        .text_color(t.colors.text_muted)
        .font_weight(FontWeight::BOLD)
        .child(title)
}

fn pane(t: &Theme) -> gpui_pre::Div {
    div()
        .flex()
        .flex_col()
        .bg(t.colors.surface)
        .border(px(t.borders.hairline))
        .border_color(t.colors.border)
}

fn log_line(t: &Theme, (time, level, message): (&str, Level, &str)) -> AnyElement {
    let (tag, color) = match level {
        Level::Info => ("INFO", t.colors.text_muted),
        Level::Ok => (" OK ", t.colors.success),
        Level::Warn => ("WARN", t.colors.warning),
        Level::Error => ("ERR ", t.colors.danger),
    };
    div()
        .flex()
        .gap(px(t.spacing.medium))
        .child(div().text_color(t.colors.text_muted).child(time.to_owned()))
        .child(div().text_color(color).child(format!("[{tag}]")))
        .child(message.to_owned())
        .into_any_element()
}

impl Render for SubcompactConsole {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let header = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.medium))
            .px(px(t.spacing.small))
            .h(px(t.controls.large))
            .border_b(px(t.borders.hairline))
            .border_color(t.colors.border)
            .child(
                div().text_color(t.colors.accent).font_weight(FontWeight::BOLD).child("mkit://ops"),
            )
            .child(Breadcrumbs::new(
                "Location",
                vec![Crumb::new("fra-1"), Crumb::new("ingest-02"), Crumb::new("procs")],
            ))
            .child(div().flex_1())
            .child(Badge::status(Status::Success, "12 up"))
            .child(Badge::status(Status::Warning, "1 degraded"))
            .child(self.palette.clone())
            .child(Button::new("Restart").size(ButtonSize::Small).variant(ButtonVariant::Outline))
            .child(Button::new("Deploy").size(ButtonSize::Small));

        let hosts = pane(&t).w(px(150.0)).child(pane_title(&t, "HOSTS")).child(self.tree.clone());

        let processes = pane(&t)
            .flex_1()
            .min_w(px(0.0))
            .child(div().px(px(t.spacing.small)).child(self.tabs.clone()))
            .child(self.table.clone());

        let tuning = pane(&t)
            .w(px(240.0))
            .child(pane_title(&t, "TUNING"))
            .child(
                div().p(px(t.spacing.small)).child(
                    FormLayout::new()
                        .label_width(64.0)
                        .row(FormRow::new("Host", self.host.clone()))
                        .row(FormRow::new("Logs", self.verbose.clone()))
                        .row(FormRow::new("Tail", self.follow.clone()))
                        .row(FormRow::new("Rate", self.rate.clone())),
                ),
            )
            .child(pane_title(&t, "LOAD"))
            .child(
                div()
                    .p(px(t.spacing.small))
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.small))
                    .child(Progress::new("CPU", 0.62))
                    .child(Progress::new("Memory", 0.81))
                    .child(Progress::new("Disk", 0.44)),
            );

        let log = pane(&t).child(pane_title(&t, "LOG  tail -f /var/log/ops.log")).child(
            div()
                .px(px(t.spacing.small))
                .py(px(t.spacing.xsmall))
                .flex()
                .flex_col()
                .children(LOG.into_iter().map(|line| log_line(&t, line))),
        );

        let status = StatusBar::new("Console status")
            .leading(StatusBarItem::text("mode", "NORMAL"))
            .leading(StatusBarItem::text("host", "ingest-02.fra-1"))
            .leading(StatusBarItem::progress("sync", "sync", 0.72))
            .trailing(StatusBarItem::text("latency", "rtt 3.1 ms"))
            .trailing(StatusBarItem::text("clock", "12:04:04 UTC"));

        let keys = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.medium))
            .px(px(t.spacing.small))
            .text_size(px(t.typography.caption))
            .text_color(t.colors.text_muted)
            .child(KeyHint::new(KeyChord::new("K").command()))
            .child("palette")
            .child(KeyHint::new(KeyChord::new("/")))
            .child("filter")
            .child(KeyHint::new(KeyChord::new("R").control()))
            .child("restart");

        // ANCHOR: subcompact_root
        // The root sets the monospace family and the 1.3 line height; every child inherits them.
        subcompact::apply_text_style(div(), &t)
            .id("mkit-subcompact-console")
            .size_full()
            .flex()
            .flex_col()
            .bg(t.colors.background)
            // ANCHOR_END: subcompact_root
            .child(header)
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .flex()
                    .gap(px(t.spacing.xsmall))
                    .p(px(t.spacing.xsmall))
                    .child(hosts)
                    .child(processes)
                    .child(tuning),
            )
            .child(div().px(px(t.spacing.xsmall)).child(log))
            .child(div().py(px(t.spacing.xsmall)).child(keys))
            .child(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre as gpui;
    use gpui_pre::TestAppContext;

    #[test]
    fn every_palette_key_maps_to_a_distinct_subcompact_palette() {
        let names: Vec<_> = PALETTE_KEYS.iter().map(|key| palette(key).unwrap().name).collect();
        assert_eq!(names, ["subcompact-phosphor", "subcompact-amber", "subcompact-paper"]);
        assert!(palette("solarized").is_none());
    }

    #[gpui_pre::test]
    fn console_renders_in_every_palette_and_switches_at_runtime(cx: &mut TestAppContext) {
        cx.update(|cx| {
            subcompact::set_phosphor_theme(cx);
            bind_keys(cx);
        });
        let window = cx.add_window(|_, cx| SubcompactConsole::new(cx));
        cx.run_until_parked();
        for key in PALETTE_KEYS.into_iter().rev() {
            window
                .update(cx, |console, _, cx| console.select_palette(key, cx))
                .expect("console window");
            cx.run_until_parked();
            assert_eq!(cx.read(|app| *app.global::<Theme>()), palette(key).unwrap());
        }
        // An unknown key leaves the installed palette alone.
        window.update(cx, |console, _, cx| console.select_palette("nope", cx)).unwrap();
        assert_eq!(cx.read(|app| app.global::<Theme>().name), PHOSPHOR.name);
    }
}
