//! A worked GPUI settings screen covering layout, text, media, and conditional children.

use gpui_pre::{
    App, AppContext, Context, Entity, FontWeight, Hsla, Render, RenderImage, ScrollHandle,
    StyledText, Subscription, Window, div, img, prelude::*, px, svg,
};
use image::Frame;
use std::sync::Arc;

/// Preferences changed by controls in the screen.
pub struct SettingsState {
    pub notifications_enabled: bool,
    pub compact_mode: bool,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self { notifications_enabled: true, compact_mode: false }
    }
}

// ANCHOR: settings_screen_raster
fn landscape_image() -> Arc<RenderImage> {
    // Local generated illustration; regenerate with assets/generate-landscape.py (no external source).
    let bytes = include_bytes!("../assets/workspace-landscape.png");
    let mut pixels = image::load_from_memory(bytes)
        .expect("checked-in workspace landscape is a PNG")
        .into_rgba8();
    // RenderImage stores BGRA pixels. The checked-in PNG is ordinary RGBA.
    for pixel in pixels.pixels_mut() {
        pixel.0.swap(0, 2);
    }
    Arc::new(RenderImage::new(vec![Frame::new(pixels)]))
}
// ANCHOR_END: settings_screen_raster

/// Stateful settings view. The raster is embedded and decoded once per view.
pub struct SettingsScreen {
    state: Option<Entity<SettingsState>>,
    landscape: Arc<RenderImage>,
    scroll_handle: ScrollHandle,
    _observation: Option<Subscription>,
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self {
            state: None,
            landscape: landscape_image(),
            scroll_handle: ScrollHandle::new(),
            _observation: None,
        }
    }
}

impl SettingsScreen {
    fn observe_settings(&mut self, cx: &mut Context<Self>) {
        if self._observation.is_none() {
            let state = self.state.as_ref().expect("state initialized").clone();
            self._observation = Some(cx.observe(&state, |_, _, cx| cx.notify()));
        }
    }
}

// ANCHOR: settings_screen_layout
impl Render for SettingsScreen {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_pre::IntoElement {
        if self.state.is_none() {
            self.state = Some(cx.new(|_| SettingsState::default()));
        }
        self.observe_settings(cx);
        let colors = &gpui_kit::component::ActiveTheme::theme(&**cx).colors;
        let state = self.state.as_ref().expect("state initialized").clone();
        let notifications_enabled = state.read(cx).notifications_enabled;
        let compact_mode = state.read(cx).compact_mode;
        let landscape = self.landscape.clone();
        let notification_state = if notifications_enabled { "On" } else { "Off" };
        let compact_state = if compact_mode { "Compact" } else { "Comfortable" };

        div()
            .size_full()
            .flex()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(
                div()
                    .w(px(224.))
                    .flex_shrink_0()
                    .h_full()
                    .flex()
                    .flex_col()
                    .gap_7()
                    .p_5()
                    .bg(colors.sidebar)
                    .child(brand_mark(colors))
                    .child(navigation(colors))
                    .child(account_card(colors, landscape)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(page_header(colors))
                    .child(content_scroll_area(
                        colors,
                        compact_state,
                        state,
                        notifications_enabled,
                        compact_mode,
                        notification_state,
                        &self.scroll_handle,
                        window,
                    )),
            )
    }
}
// ANCHOR_END: settings_screen_layout

fn brand_mark(colors: &gpui_kit::component::ThemeColor) -> impl gpui_pre::IntoElement {
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .size(px(34.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_lg()
                .bg(colors.primary)
                .child(sync_glyph(colors.primary_foreground, 18.)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().font_weight(FontWeight::BOLD).child("Northstar"))
                .child(div().text_xs().text_color(colors.muted_foreground).child("WORKSPACE")),
        )
}

fn navigation(colors: &gpui_kit::component::ThemeColor) -> impl gpui_pre::IntoElement {
    let items = [
        ("overview", "Overview", "◫"),
        ("activity", "Activity", "◷"),
        ("members", "Members", "♙"),
        ("settings", "Settings", "⚙"),
    ];
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().pb_2().text_xs().text_color(colors.muted_foreground).child("WORKSPACE"))
        .children(items.into_iter().map(|(id, label, icon)| {
            let selected = id == "settings";
            let background = if selected { colors.accent } else { colors.sidebar };
            let foreground =
                if selected { colors.accent_foreground } else { colors.sidebar_foreground };
            div()
                .id(format!("nav-{id}"))
                .flex()
                .items_center()
                .gap_3()
                .rounded_md()
                .px_3()
                .py_2()
                .bg(background)
                .text_color(foreground)
                .child(div().w(px(18.)).text_center().child(icon))
                .child(label)
        }))
}

// ANCHOR: settings_screen_text_truncate
fn account_card(
    colors: &gpui_kit::component::ThemeColor,
    landscape: Arc<RenderImage>,
) -> impl gpui_pre::IntoElement {
    div()
        .mt_auto()
        .flex()
        .flex_col()
        .gap_3()
        .rounded_lg()
        .border_1()
        .border_color(colors.border)
        .bg(colors.popover)
        .p_3()
        .child(landscape_photo(landscape))
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .size(px(30.))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(colors.secondary)
                        .text_color(colors.secondary_foreground)
                        .child("MR"),
                )
                .child(
                    div()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(div().font_weight(FontWeight::MEDIUM).child("Morgan Reed"))
                        .child(
                            div()
                                .truncate()
                                .text_xs()
                                .text_color(colors.muted_foreground)
                                .child("morgan.reed@northstar.example"),
                        ),
                ),
        )
}
// ANCHOR_END: settings_screen_text_truncate

// ANCHOR: settings_screen_media
/// The screenshot uses this checked-in, generated landscape image, decoded to GPUI's image type.
fn landscape_photo(image: Arc<RenderImage>) -> impl gpui_pre::IntoElement {
    img(image).w_full().h(px(74.)).rounded_md().object_fit(gpui_pre::ObjectFit::Cover)
}

/// Include the small SVG asset as bytes so headless previews need no filesystem asset loader.
fn sync_glyph(color: Hsla, size: f32) -> impl gpui_pre::IntoElement {
    svg().data(include_bytes!("../assets/sync-mark.svg")).size(px(size)).text_color(color)
}
// ANCHOR_END: settings_screen_media

fn page_header(colors: &gpui_kit::component::ThemeColor) -> impl gpui_pre::IntoElement {
    div()
        .h(px(62.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_between()
        .px_7()
        .border_b_1()
        .border_color(colors.border)
        .child(
            div().text_sm().text_color(colors.muted_foreground).child("Workspace  /  Preferences"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .text_xs()
                .text_color(colors.muted_foreground)
                .child(sync_glyph(colors.primary, 15.))
                .child("All changes saved"),
        )
}

// ANCHOR: settings_screen_text
fn intro(colors: &gpui_kit::component::ThemeColor) -> impl gpui_pre::IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_2xl().font_weight(FontWeight::BOLD).child("Preferences"))
        .child(
            div()
                .max_w(px(590.))
                .line_clamp(2)
                .text_color(colors.muted_foreground)
                .child("Make Northstar feel like yours. Your preferences follow you across every device in this workspace."),
        )
}
// ANCHOR_END: settings_screen_text

// ANCHOR: settings_screen_scroll
#[allow(clippy::too_many_arguments)]
fn content_scroll_area(
    colors: &gpui_kit::component::ThemeColor,
    compact_state: &'static str,
    state: Entity<SettingsState>,
    notifications_enabled: bool,
    compact_mode: bool,
    notification_state: &'static str,
    scroll_handle: &ScrollHandle,
    window: &Window,
) -> impl gpui_pre::IntoElement {
    div()
        .id("settings-scroll-region")
        .debug_selector(|| "settings-scroll-region".into())
        .track_scroll(scroll_handle)
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p_7()
        .child(
            div()
                .max_w(px(760.))
                .flex()
                .flex_col()
                .gap_6()
                .child(intro(colors))
                .child(workspace_cards(colors, compact_state))
                .child(notification_section(colors, state.clone(), notifications_enabled))
                .child(display_section(colors, state, compact_mode))
                .child(sync_section(colors, window))
                .child(saved_places(colors, notification_state)),
        )
}

fn workspace_cards(
    colors: &gpui_kit::component::ThemeColor,
    compact_state: &'static str,
) -> impl gpui_pre::IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(section_heading(colors, "Workspace defaults", "Used when you join a new project"))
        .child(
            div()
                .grid()
                .grid_cols(2)
                .gap_3()
                .child(choice_card(colors, "Language", "English (US)", "⌄"))
                .child(choice_card(colors, "Appearance", compact_state, "◐")),
        )
}
// ANCHOR_END: settings_screen_scroll

fn choice_card(
    colors: &gpui_kit::component::ThemeColor,
    label: &'static str,
    value: &'static str,
    icon: &'static str,
) -> impl gpui_pre::IntoElement {
    div()
        .id(format!("choice-{label}"))
        .min_w_0()
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .rounded_lg()
        .border_1()
        .border_color(colors.border)
        .bg(colors.popover)
        .p_4()
        .child(
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().text_xs().text_color(colors.muted_foreground).child(label))
                .child(div().truncate().font_weight(FontWeight::MEDIUM).child(value)),
        )
        .child(div().text_color(colors.muted_foreground).child(icon))
}

fn section_heading(
    colors: &gpui_kit::component::ThemeColor,
    title: &'static str,
    detail: &'static str,
) -> impl gpui_pre::IntoElement {
    div()
        .flex()
        .items_end()
        .justify_between()
        .gap_4()
        .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
        .child(div().text_xs().text_color(colors.muted_foreground).child(detail))
}

// ANCHOR: settings_screen_conditionals
fn notification_section(
    colors: &gpui_kit::component::ThemeColor,
    state: Entity<SettingsState>,
    enabled: bool,
) -> impl gpui_pre::IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(section_heading(colors, "Notifications", "Choose what reaches you"))
        .child(setting_row(
            colors,
            "desktop-alerts",
            "Desktop alerts",
            "A short message when someone mentions you or a project needs attention.",
            enabled,
            state,
        ))
        .child(if enabled {
            div()
                .id("alerts-enabled-note")
                .debug_selector(|| "alerts-enabled-note".into())
                .rounded_md()
                .bg(colors.muted)
                .px_3()
                .py_2()
                .text_xs()
                .child("Quiet hours are respected automatically.")
        } else {
            div()
                .id("alerts-disabled-note")
                .debug_selector(|| "alerts-disabled-note".into())
                .rounded_md()
                .bg(colors.muted)
                .px_3()
                .py_2()
                .text_xs()
                .child("Desktop alerts are paused. You can still review activity in your inbox.")
        })
}

fn setting_row(
    colors: &gpui_kit::component::ThemeColor,
    id: &'static str,
    title: &'static str,
    detail: &'static str,
    enabled: bool,
    state: Entity<SettingsState>,
) -> impl gpui_pre::IntoElement {
    let switch_background = if enabled { colors.primary } else { colors.secondary };
    let thumb = div().size(px(18.)).rounded_full().bg(colors.primary_foreground);
    let switch = div()
        .id(format!("toggle-{id}"))
        .debug_selector(|| format!("toggle-{id}"))
        .w(px(48.))
        .h(px(28.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .rounded_full()
        .bg(switch_background)
        .px_1();
    let switch = if enabled { switch.justify_end() } else { switch.justify_start() };
    div()
        .id(format!("setting-row-{id}"))
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .rounded_lg()
        .border_1()
        .border_color(colors.border)
        .bg(colors.popover)
        .p_4()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().font_weight(FontWeight::MEDIUM).child(title))
                .child(
                    div().line_clamp(2).text_xs().text_color(colors.muted_foreground).child(detail),
                ),
        )
        .child(switch.child(thumb).on_click(move |_, _, cx| {
            state.update(cx, |state, cx| {
                state.notifications_enabled = !state.notifications_enabled;
                cx.notify();
            });
        }))
}
// ANCHOR_END: settings_screen_conditionals

fn display_section(
    colors: &gpui_kit::component::ThemeColor,
    state: Entity<SettingsState>,
    compact: bool,
) -> impl gpui_pre::IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(section_heading(colors, "Display", "Adjust your reading space"))
        .child(
            div()
                .id("compact-mode-row")
                .flex()
                .items_center()
                .justify_between()
                .rounded_lg()
                .border_1()
                .border_color(colors.border)
                .bg(colors.popover)
                .p_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().font_weight(FontWeight::MEDIUM).child("Compact spacing"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.muted_foreground)
                                .child("Reduce the space between project rows."),
                        ),
                )
                .child(
                    div()
                        .id("toggle-compact-mode")
                        .debug_selector(|| "toggle-compact-mode".into())
                        .rounded_md()
                        .bg(colors.secondary)
                        .px_3()
                        .py_2()
                        .text_xs()
                        .child(if compact { "Compact" } else { "Comfortable" })
                        .on_click(move |_, _, cx| {
                            state.update(cx, |state, cx| {
                                state.compact_mode = !state.compact_mode;
                                cx.notify();
                            });
                        }),
                ),
        )
}

// ANCHOR: settings_screen_text_runs
fn sync_section(
    colors: &gpui_kit::component::ThemeColor,
    window: &Window,
) -> impl gpui_pre::IntoElement {
    let message = "Workspace sync is ready. Your choices are available on each device.";
    let emphasized = "Workspace sync";
    let mut emphasis_run = window.text_style().to_run(emphasized.len());
    emphasis_run.color = colors.primary;
    emphasis_run.font.weight = FontWeight::BOLD;
    let body_run = window.text_style().to_run(message.len() - emphasized.len());
    div()
        .flex()
        .items_start()
        .gap_3()
        .rounded_lg()
        .border_1()
        .border_color(colors.border)
        .bg(colors.popover)
        .p_4()
        .child(sync_glyph(colors.primary, 20.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().font_weight(FontWeight::MEDIUM).child("Synced across devices"))
                .child(StyledText::new(message).with_runs(vec![emphasis_run, body_run])),
        )
}
// ANCHOR_END: settings_screen_text_runs

// ANCHOR: settings_screen_list
fn saved_places(
    colors: &gpui_kit::component::ThemeColor,
    notification_state: &'static str,
) -> impl gpui_pre::IntoElement {
    let places = [
        ("research", "Research notes", "Last opened today · 12 items"),
        ("design", "Design library", "Shared with 8 teammates"),
        ("release", "Release checklist", "Updated yesterday"),
    ];
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(section_heading(colors, "Pinned workspaces", "Quick access"))
        .children(places.into_iter().map(|(id, name, detail)| {
            div()
                .id(format!("pinned-workspace-{id}"))
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .rounded_md()
                .border_1()
                .border_color(colors.border)
                .bg(colors.popover)
                .p_3()
                .child(div().font_weight(FontWeight::MEDIUM).child(name))
                .child(div().truncate().text_xs().text_color(colors.muted_foreground).child(detail))
        }))
        .child(
            div()
                .text_xs()
                .text_color(colors.muted_foreground)
                .child(format!("Desktop alerts: {notification_state}")),
        )
}
// ANCHOR_END: settings_screen_list

impl SettingsScreen {
    /// Return the current controls for the interaction test and inspector.
    pub fn settings_snapshot(&self, cx: &App) -> Option<(bool, bool)> {
        self.state.as_ref().map(|state| {
            let state = state.read(cx);
            (state.notifications_enabled, state.compact_mode)
        })
    }

    /// Request the bottom of the scrollable settings body be brought into view.
    pub fn scroll_to_bottom(&self) {
        self.scroll_handle.scroll_to_bottom();
    }
}

#[cfg(test)]
mod tests {
    use super::SettingsScreen;
    use gpui_pre::{
        Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase, point, px,
    };

    #[gpui_pre::test]
    fn notification_control_changes_state_and_conditional_copy(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (view, visual) = cx.add_window_view(|_, _| SettingsScreen::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("toggle-desktop-alerts").is_some());
        assert!(visual.debug_bounds("alerts-enabled-note").is_some());

        let toggle = visual.debug_bounds("toggle-desktop-alerts").expect("toggle is rendered");
        visual.simulate_click(toggle.center(), Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));

        assert!(visual.debug_bounds("alerts-enabled-note").is_none());
        assert!(visual.debug_bounds("alerts-disabled-note").is_some());
        assert_eq!(
            view.read_with(visual, |screen, cx| screen.settings_snapshot(cx)),
            Some((false, false))
        );
    }

    #[gpui_pre::test]
    fn settings_content_has_scroll_extent_and_moves_when_scrolled(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (view, visual) = cx.add_window_view(|_, _| SettingsScreen::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));

        let (max_offset, initial_offset) = view.read_with(visual, |screen, _| {
            (screen.scroll_handle.max_offset(), screen.scroll_handle.offset())
        });
        assert!(max_offset.y > px(0.), "settings body should overflow vertically");
        assert_eq!(initial_offset.y, px(0.));

        let scroll_bounds =
            visual.debug_bounds("settings-scroll-region").expect("scroll region rendered");
        visual.simulate_mouse_move(scroll_bounds.center(), None, Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_event(ScrollWheelEvent {
            position: scroll_bounds.center(),
            delta: ScrollDelta::Pixels(point(px(0.), px(-320.))),
            modifiers: Modifiers::default(),
            touch_phase: TouchPhase::Moved,
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));

        let scrolled_offset = view.read_with(visual, |screen, _| screen.scroll_handle.offset());
        assert!(scrolled_offset.y < px(0.), "scroll input should move the settings body");
    }
}
