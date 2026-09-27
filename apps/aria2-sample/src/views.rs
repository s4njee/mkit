use super::*;

impl Aria2Sample {
    pub(super) fn detail_view(&self, t: Theme, cx: &mut Context<Self>) -> gpui_pre::AnyElement {
        let owner = cx.entity();
        let row = self
            .selected
            .and_then(|id| self.downloads.iter().find(|row| row.id == id))
            .or_else(|| self.downloads.first());
        let Some(row) = row else {
            return empty_panel("No download selected", t).into_any_element();
        };
        let percent = row.percent;
        let mut tabs = div().flex().gap(px(8.0));
        for (i, (label, mode)) in [
            ("Connection lanes", DetailMode::Lanes),
            ("Block map", DetailMode::Blocks),
            ("Ribbon", DetailMode::Ribbon),
        ]
        .into_iter()
        .enumerate()
        {
            let selected = self.detail_mode == mode;
            tabs = tabs.child(Self::mkit_button(
                label,
                70 + i,
                if selected { ButtonVariant::Default } else { ButtonVariant::Outline },
                true,
                owner.clone(),
                move |s, cx| {
                    s.detail_mode = mode;
                    cx.notify();
                },
            ));
        }
        let mut card = panel(t)
            .w(px(570.0))
            .child(div().text_size(px(13.5)).text_color(t.colors.text).child(row.name.clone()))
            .child(
                div()
                    .flex()
                    .gap(px(14.0))
                    .font_family("IBM Plex Mono")
                    .text_size(px(11.0))
                    .text_color(t.colors.text_muted)
                    .child(row.size.clone())
                    .child(format!("{}%", row.percent))
                    .child(row.speed.clone()),
            )
            .child(div().h(px(1.0)).w_full().bg(t.colors.border))
            .child(tabs);
        card = match self.detail_mode {
            DetailMode::Lanes => {
                let mut lanes = div()
                    .flex()
                    .flex_col()
                    .gap(px(5.0))
                    .child(section_label("CONNECTION LANES", t));
                for i in 0..row.connections {
                    let stalled = i == 4 || i == 11;
                    let start = (i as f32 / row.connections as f32 * 100.0).round();
                    let fill = (percent as f32 / row.connections as f32).max(2.0);
                    lanes = lanes.child(
                        div()
                            .h(px(16.0))
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .w(px(22.0))
                                    .font_family("IBM Plex Mono")
                                    .text_size(px(9.5))
                                    .text_color(theme::meta(t))
                                    .child(format!("{:02}", i + 1)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .h(px(9.0))
                                    .rounded(px(2.0))
                                    .bg(theme::rule(t))
                                    .flex()
                                    .child(div().h_full().w(px(start * 3.8)))
                                    .child(div().h_full().w(px(fill * 3.8)).bg(if stalled {
                                        t.colors.warning
                                    } else {
                                        t.colors.accent
                                    })),
                            )
                            .child(
                                div()
                                    .w(px(62.0))
                                    .font_family("IBM Plex Mono")
                                    .text_size(px(9.5))
                                    .text_color(if stalled {
                                        t.colors.warning
                                    } else {
                                        t.colors.text_muted
                                    })
                                    .child(if stalled {
                                        "stalled".to_owned()
                                    } else {
                                        format!("{:.1} MB/s", 0.4 + i as f32 * 0.06)
                                    }),
                            ),
                    );
                }
                card.child(lanes).child(summary_strip(
                    t,
                    &["FASTEST LANE", "STALLED", "RETRIES"],
                    &["1.4 MB/s", "2 lanes", "3"],
                ))
            }
            DetailMode::Blocks => {
                let mut blocks = div().flex().flex_col().gap(px(2.0));
                for y in 0..16 {
                    let mut line = div().flex().gap(px(2.0));
                    for x in 0..32 {
                        let index = y * 32 + x;
                        let fill = if index < (percent as usize * 512 / 100) {
                            t.colors.accent
                        } else if index < (percent as usize * 512 / 100) + 16 {
                            t.colors.accent.opacity(0.48)
                        } else {
                            theme::track(t)
                        };
                        line = line.child(div().flex_1().h(px(10.0)).rounded(px(1.0)).bg(fill));
                    }
                    blocks = blocks.child(line);
                }
                card.child(section_label("BLOCK MAP · 512 AGGREGATED CELLS", t))
                    .child(
                        div()
                            .p(px(10.0))
                            .rounded(px(6.0))
                            .bg(t.colors.elevated_surface)
                            .child(blocks),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .font_family("IBM Plex Mono")
                            .text_size(px(10.5))
                            .text_color(t.colors.text_muted)
                            .child(format!("{}% complete", row.percent))
                            .child(format!("{} / 1024 blocks", row.percent as usize * 1024 / 100)),
                    )
                    .child(Progress::new("Overall download progress", row.percent as f32))
            }
            DetailMode::Ribbon => {
                let mut ribbon = div().h(px(34.0)).flex().rounded(px(5.0)).bg(theme::rule(t));
                for i in 0..row.connections {
                    let segment = ((i as usize * 37 + 30) % 85) as f32;
                    ribbon = ribbon.child(
                        div()
                            .flex_1()
                            .h_full()
                            .border_r(px(1.0))
                            .border_color(t.colors.surface)
                            .child(div().h_full().w(px(segment * 0.24)).bg(if i == 4 || i == 11 {
                                t.colors.warning
                            } else {
                                t.colors.accent
                            })),
                    );
                }
                card.child(section_label("FILE RANGE 0 → 6.10 GB", t))
                    .child(ribbon)
                    .child(
                        div()
                            .font_family("IBM Plex Mono")
                            .text_size(px(10.0))
                            .text_color(theme::meta(t))
                            .child(format!("{} segments · each segment ≈ 390 MB", row.connections)),
                    )
                    .child(section_label("THROUGHPUT · LAST 60 S", t))
                    .child(
                        div()
                            .h(px(88.0))
                            .p(px(8.0))
                            .flex()
                            .items_end()
                            .gap(px(4.0))
                            .bg(t.colors.elevated_surface)
                            .children(
                                [
                                    24., 36., 30., 48., 40., 61., 56., 70., 59., 76., 62., 68.,
                                    81., 74., 88.,
                                ]
                                .into_iter()
                                .map(|height| {
                                    div().flex_1().h(px(height)).bg(t.colors.accent.opacity(0.28))
                                }),
                            ),
                    )
                    .child(Progress::new("Overall download progress", row.percent as f32))
            }
        };
        div()
            .id("detail-scroll")
            .flex_1()
            .min_h(px(0.0))
            .overflow_y_scroll()
            .p(px(22.0))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.0))
            .bg(t.colors.background)
            .child(page_heading(
                "Download detail",
                "Per-connection progress in the sample fixture",
                t,
            ))
            .child(card)
            .child(Self::mkit_button(
                "Back to downloads",
                74,
                ButtonVariant::Outline,
                true,
                owner,
                |s, cx| {
                    s.scene = Scene::Downloads;
                    cx.notify();
                },
            ))
            .into_any_element()
    }

    pub(super) fn queues_view(&self, t: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let mut cards = div().flex().flex_col().gap(px(10.0));
        for (name, detail, window, percent, color, state) in [
            (
                "Main queue",
                "3 running · 2 waiting · no limit",
                "Always on",
                47.,
                t.colors.accent,
                "RUNNING",
            ),
            (
                "Overnight",
                "4 waiting · 5 MB/s cap",
                "01:00 – 07:00",
                0.,
                t.colors.warning,
                "WAITING",
            ),
            (
                "Datasets",
                "1 running · 12 waiting · 2 at a time",
                "Weekends",
                22.,
                t.colors.danger,
                "PAUSED",
            ),
        ] {
            cards = cards.child(
                div()
                    .p(px(12.0))
                    .flex()
                    .items_center()
                    .gap(px(14.0))
                    .rounded(px(8.0))
                    .border(px(1.0))
                    .border_color(t.colors.border)
                    .bg(t.colors.surface)
                    .child(div().size(px(9.0)).rounded(px(3.0)).bg(color))
                    .child(
                        div()
                            .w(px(232.0))
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(div().text_size(px(12.5)).child(name))
                            .child(
                                div()
                                    .font_family("IBM Plex Mono")
                                    .text_size(px(10.5))
                                    .text_color(theme::meta(t))
                                    .child(detail),
                            ),
                    )
                    .child(
                        div()
                            .w(px(92.0))
                            .font_family("IBM Plex Mono")
                            .text_size(px(11.0))
                            .child(window),
                    )
                    .child(
                        div()
                            .w(px(120.0))
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .h(px(5.0))
                                    .w_full()
                                    .rounded(px(3.0))
                                    .bg(theme::track(t))
                                    .child(div().h_full().w(px(percent * 1.2)).bg(color)),
                            )
                            .child(
                                div()
                                    .font_family("IBM Plex Mono")
                                    .text_size(px(10.0))
                                    .text_color(theme::meta(t))
                                    .child(format!("{percent:.0}% of queue")),
                            ),
                    )
                    .child(
                        div()
                            .font_family("IBM Plex Mono")
                            .text_size(px(10.0))
                            .text_color(color)
                            .child(state),
                    ),
            );
        }
        let mut grid = div().flex().flex_col().gap(px(2.0));
        let mut labels = div().flex().gap(px(2.0)).child(div().w(px(34.0)));
        for hour in 0..24 {
            labels = labels.child(
                div()
                    .w(px(19.0))
                    .font_family("IBM Plex Mono")
                    .text_size(px(8.0))
                    .text_color(theme::meta(t))
                    .child(if hour % 3 == 0 { hour.to_string() } else { String::new() }),
            );
        }
        grid = grid.child(labels);
        for (day, label) in
            ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"].into_iter().enumerate()
        {
            let mut line = div().flex().items_center().gap(px(2.0)).child(
                div()
                    .w(px(34.0))
                    .font_family("IBM Plex Mono")
                    .text_size(px(9.5))
                    .text_color(theme::meta(t))
                    .child(label),
            );
            for hour in 0..24 {
                let level = self.schedule[day][hour];
                line = line.child(
                    div()
                        .id(format!("schedule-{day}-{hour}"))
                        .debug_selector(move || format!("schedule-{day}-{hour}"))
                        .w(px(19.0))
                        .h(px(14.0))
                        .rounded(px(2.0))
                        .bg(match level {
                            2 => t.colors.accent,
                            1 => t.colors.accent.opacity(0.4),
                            _ => theme::track(t),
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.schedule[day][hour] = (this.schedule[day][hour] + 1) % 3;
                            cx.notify();
                        })),
                );
            }
            grid = grid.child(line);
        }
        div()
            .id("queues-scroll")
            .flex_1()
            .min_h(px(0.0))
            .overflow_y_scroll()
            .p(px(22.0))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.0))
            .bg(t.colors.background)
            .child(page_heading(
                "Queues & scheduler",
                "Queues run within their weekly windows; this sample changes local state only.",
                t,
            ))
            .child(
                panel(t)
                    .w(px(660.0))
                    .child(cards)
                    .child(section_label("WEEKLY WINDOW · OVERNIGHT", t))
                    .child(grid)
                    .child(
                        div()
                            .font_family("IBM Plex Mono")
                            .text_size(px(10.0))
                            .text_color(theme::meta(t))
                            .child("Click a cell to cycle paused → throttled → full speed"),
                    ),
            )
    }

    pub(super) fn settings_view(&self, t: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        let mut options = div().flex().flex_col();
        for (i, (label, flag)) in [
            ("Max concurrent downloads", "--max-concurrent-downloads"),
            ("Connections per server", "--max-connection-per-server"),
            ("Minimum split size", "--min-split-size"),
            ("Global download limit", "--max-overall-download-limit"),
            ("Disk cache", "--disk-cache"),
            ("File allocation", "--file-allocation"),
        ]
        .into_iter()
        .enumerate()
        {
            options = options.child(
                div()
                    .h(px(49.0))
                    .flex()
                    .items_center()
                    .border_b(px(1.0))
                    .border_color(theme::rule(t))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(3.0))
                            .child(div().text_size(px(11.5)).child(label))
                            .child(
                                div()
                                    .font_family("IBM Plex Mono")
                                    .text_size(px(10.0))
                                    .text_color(theme::meta(t))
                                    .child(flag),
                            ),
                    )
                    .child(settings_field(self.settings_fields[i + 3].clone()).w(px(92.0))),
            );
        }
        let mut theme_actions = div().flex().gap(px(8.0));
        for (i, (label, theme_value)) in
            [("Light", theme::LIGHT), ("Dark", theme::DARK), ("Paper", theme::PAPER)]
                .into_iter()
                .enumerate()
        {
            theme_actions = theme_actions.child(
                Button::new(label)
                    .id(90 + i)
                    .variant(if t.name == theme_value.name {
                        ButtonVariant::Default
                    } else {
                        ButtonVariant::Outline
                    })
                    .on_activate(move |_, cx| theme::install(cx, theme_value)),
            );
        }
        div()
            .id("settings-scroll")
            .flex_1()
            .min_h(px(0.0))
            .overflow_y_scroll()
            .p(px(22.0))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.0))
            .bg(t.colors.background)
            .child(page_heading(
                "Engine settings",
                "Connection and global limits for the mock aria2 engine",
                t,
            ))
            .child(
                panel(t)
                    .w(px(660.0))
                    .child(
                        div()
                            .p(px(11.0))
                            .rounded(px(8.0))
                            .bg(theme::tint(t))
                            .flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(t.colors.success)
                                    .child("●  Mock aria2 1.37.0 session"),
                            )
                            .child(
                                div()
                                    .font_family("IBM Plex Mono")
                                    .text_size(px(10.5))
                                    .text_color(t.colors.text_muted)
                                    .child("local · pid 4821"),
                            ),
                    )
                    .child(section_label("RPC ENDPOINT", t))
                    .child(
                        div()
                            .flex()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(5.0))
                                    .child(section_label("HOST", t))
                                    .child(settings_field(self.settings_fields[0].clone())),
                            )
                            .child(
                                div()
                                    .w(px(108.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(5.0))
                                    .child(section_label("PORT", t))
                                    .child(settings_field(self.settings_fields[1].clone())),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(5.0))
                            .child(section_label("SECRET TOKEN", t))
                            .child(settings_field(self.settings_fields[2].clone())),
                    )
                    .child(section_label("LIMITS", t))
                    .child(options)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(Self::mkit_button(
                                "Apply",
                                96,
                                ButtonVariant::Default,
                                true,
                                owner.clone(),
                                |s, cx| {
                                    let host =
                                        s.settings_fields[0].read(cx).text().trim().to_owned();
                                    let port =
                                        s.settings_fields[1].read(cx).text().trim().to_owned();
                                    s.notice =
                                        Some(if host.is_empty() || port.parse::<u16>().is_err() {
                                            "Enter a host and a valid TCP port.".into()
                                        } else {
                                            "Settings saved in sample state only.".into()
                                        });
                                    cx.notify();
                                },
                            ))
                            .child(Self::mkit_button(
                                "Restart engine",
                                97,
                                ButtonVariant::Outline,
                                true,
                                owner,
                                |s, cx| {
                                    s.notice =
                                        Some("No aria2 process is launched by this sample.".into());
                                    cx.notify();
                                },
                            ))
                            .child(div().flex_1())
                            .child(
                                div()
                                    .font_family("IBM Plex Mono")
                                    .text_size(px(10.5))
                                    .text_color(theme::meta(t))
                                    .child("aria2.changeGlobalOption"),
                            ),
                    )
                    .child(section_label("APPEARANCE", t))
                    .child(theme_actions),
            )
    }

    pub(super) fn capture_view(&self, t: Theme, cx: &mut Context<Self>) -> gpui_pre::Div {
        let owner = cx.entity();
        let mut links = div().flex().flex_col();
        for (i, (name, meta, size)) in [
            ("ubuntu-24.04.2-desktop-amd64.iso", "application/octet-stream · ranges ok", "6.10 GB"),
            ("ubuntu-24.04.2-desktop-amd64.iso.torrent", "application/x-bittorrent", "312 KB"),
            ("SHA256SUMS", "text/plain", "1.4 KB"),
            ("SHA256SUMS.gpg", "application/pgp-signature", "833 B"),
        ]
        .into_iter()
        .enumerate()
        {
            let checked = self.captured[i];
            links = links.child(
                div()
                    .id(format!("captured-link-{i}"))
                    .debug_selector(move || format!("captured-link-{i}"))
                    .h(px(51.0))
                    .px(px(14.0))
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .border_b(px(1.0))
                    .border_color(theme::rule(t))
                    .child(
                        div()
                            .size(px(15.0))
                            .rounded(px(4.0))
                            .border(px(1.0))
                            .border_color(if checked { t.colors.accent } else { t.colors.border })
                            .bg(if checked { t.colors.accent } else { t.colors.surface })
                            .text_color(t.colors.accent_text)
                            .text_center()
                            .child(if checked { "✓" } else { "" }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(div().text_size(px(11.5)).child(name))
                            .child(
                                div()
                                    .font_family("IBM Plex Mono")
                                    .text_size(px(10.0))
                                    .text_color(theme::meta(t))
                                    .child(meta),
                            ),
                    )
                    .child(
                        div()
                            .font_family("IBM Plex Mono")
                            .text_size(px(10.5))
                            .text_color(t.colors.text_muted)
                            .child(size),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.captured[i] = !this.captured[i];
                        cx.notify();
                    })),
            );
        }
        div()
            .flex_1()
            .p(px(22.0))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.0))
            .bg(t.colors.background)
            .child(page_heading(
                "Browser capture",
                "A mock of the extension popup; browser integration is outside this sample.",
                t,
            ))
            .child(
                panel(t)
                    .w(px(360.0))
                    .child(div().text_size(px(12.5)).child("4 links captured"))
                    .child(links)
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .font_family("IBM Plex Mono")
                                    .text_size(px(10.5))
                                    .text_color(theme::meta(t))
                                    .child("→ Main queue"),
                            )
                            .child(
                                Button::new(format!(
                                    "Download {}",
                                    self.captured.iter().filter(|selected| **selected).count()
                                ))
                                .id(100)
                                .disabled(!self.captured.iter().any(|selected| *selected))
                                .on_activate(move |_, cx| {
                                    owner.update(cx, |s, cx| {
                                        s.notice = Some(
                                        "Captured links are mock data; no download was started."
                                            .into(),
                                    );
                                        cx.notify();
                                    });
                                }),
                            ),
                    ),
            )
    }
}

fn panel(t: Theme) -> gpui_pre::Div {
    div()
        .p(px(18.0))
        .flex()
        .flex_col()
        .gap(px(15.0))
        .rounded(px(10.0))
        .border(px(1.0))
        .border_color(t.colors.border)
        .bg(t.colors.surface)
}

fn settings_field(field: Entity<TextField>) -> gpui_pre::Div {
    div().font_family("IBM Plex Mono").text_size(px(11.5)).child(field)
}

fn page_heading(title: &'static str, subtitle: &'static str, t: Theme) -> gpui_pre::Div {
    div()
        .w(px(660.0))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(div().text_size(px(15.0)).text_color(t.colors.text).child(title))
        .child(div().text_size(px(11.5)).text_color(t.colors.text_muted).child(subtitle))
}

fn summary_strip(
    t: Theme,
    labels: &[&'static str; 3],
    values: &[&'static str; 3],
) -> gpui_pre::Div {
    div()
        .pt(px(14.0))
        .border_t(px(1.0))
        .border_color(t.colors.border)
        .flex()
        .gap(px(18.0))
        .children(labels.iter().zip(values).map(|(label, value)| {
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(section_label(label, t))
                .child(div().font_family("IBM Plex Mono").text_size(px(12.0)).child(*value))
        }))
}

fn empty_panel(text: &'static str, t: Theme) -> gpui_pre::Div {
    div().flex_1().flex().items_center().justify_center().bg(t.colors.background).child(text)
}
