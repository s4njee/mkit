use gpui_pre::{
    App, Context, Entity, FocusHandle, InputEvent, IntoElement, MouseButton, MouseDownEvent,
    MouseUpEvent, Render, Window, div, point, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    sidebar::{Item as SidebarItem, Sidebar},
    tooltip::Tooltip,
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (480.0, 220.0);
const SIDEBAR_PANE_WIDTH: f32 = 192.0;

struct TooltipFixture {
    state: &'static str,
    focus: Option<FocusHandle>,
}

impl Render for TooltipFixture {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_stop(true)).clone();
        let child = div()
            .id("tooltip-demo-child")
            .debug_selector(|| "tooltip-demo-child".into())
            .track_focus(&focus)
            .tab_stop(true)
            .w(px(176.0))
            .h(px(36.0))
            .px(px(theme.spacing.small))
            .flex()
            .items_center()
            .rounded(px(theme.radii.small))
            .border(px(theme.borders.hairline))
            .border_color(if focus.is_focused(window) {
                theme.colors.focus
            } else {
                theme.colors.border
            })
            .bg(theme.colors.surface)
            .text_color(theme.colors.text)
            .text_size(px(theme.typography.body))
            .child("Save changes");
        let target = if self.state == "disabled" {
            Tooltip::new("Saves the current changes", child).disabled(true)
        } else {
            Tooltip::new("Saves the current changes", child)
        };
        div()
            .size_full()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .items_start()
            .gap(px(theme.spacing.medium))
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.heading_small))
                    .child("Tooltip"),
            )
            .child(div().text_color(theme.colors.text_muted).child(self.state.to_owned()))
            .child(target)
    }
}

struct SidebarFixture {
    state: &'static str,
    sidebar: Option<Entity<Sidebar>>,
}

impl Render for SidebarFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let sidebar = self.sidebar.get_or_insert_with(|| {
            cx.new(|_| {
                let sidebar = Sidebar::new(
                    "Primary navigation",
                    vec![
                        SidebarItem::new("home", "Home"),
                        SidebarItem::new("projects", "Projects"),
                        SidebarItem::new("settings", "Settings"),
                    ],
                    Some("home".into()),
                );
                if state == "disabled" { sidebar.disabled(true) } else { sidebar }
            })
        });
        div()
            .size_full()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .items_start()
            .gap(px(theme.spacing.medium))
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.heading_small))
                    .child("Sidebar"),
            )
            .child(div().text_color(theme.colors.text_muted).child(state.to_owned()))
            // The owner sizes a sidebar pane; this matches the docs-site web
            // preview's 190px column rounded up to the 8px spacing grid.
            .child(div().flex().flex_col().w(px(SIDEBAR_PANE_WIDTH)).child(sidebar.clone()))
    }
}

fn theme_for(name: &str) -> Theme {
    match name {
        "light" => SHADCN_LIGHT,
        "dark" => SHADCN_DARK,
        "high-contrast" => HIGH_CONTRAST,
        other => panic!("unmapped screenshot theme: {other}"),
    }
}

fn capture_tooltip(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        TooltipFixture { state, focus: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
        },
    )?;
    match state {
        "visible" => panic!("Tooltip popup screenshot is unsupported by the headless renderer"),
        "keyboard_focused_child" => session.update(|root, window, cx| {
            let focus = root
                .update(cx, |fixture, _| fixture.focus.clone().expect("Tooltip child rendered"));
            focus.focus(window, cx);
            assert!(focus.is_focused(window), "Tooltip child must receive keyboard focus");
        })?,
        "disabled" => session.update(|root, _, cx| {
            assert!(root.read(cx).focus.is_some(), "Tooltip child must be rendered");
        })?,
        other => panic!("unmapped Tooltip state: {other}"),
    }
    session.capture()
}

fn capture_sidebar(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        SidebarFixture { state, sidebar: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::sidebar::default_key_bindings());
        },
    )?;
    // Click the centre of Projects (the second 32px row below the 8px pane
    // padding) in the fixed vertical fixture. The disabled fixture proves
    // disabled links ignore the same pointer activation.
    session.update(|_, window, cx| {
        let position = point(px(58.0), px(152.0));
        let _ = window.dispatch_event(
            MouseDownEvent {
                position,
                button: MouseButton::Left,
                modifiers: Default::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        let _ = window.dispatch_event(
            MouseUpEvent {
                position,
                button: MouseButton::Left,
                modifiers: Default::default(),
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
    })?;
    session.update(|root, _, cx| {
        let sidebar =
            root.update(cx, |fixture, _| fixture.sidebar.clone().expect("Sidebar rendered"));
        let expected = if state == "selection" { Some("projects") } else { Some("home") };
        assert_eq!(sidebar.read(cx).value(), expected, "Sidebar activation behavior: {state}");
    })?;
    session.capture()
}

fn compare_or_update(component: &str, actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../registry/{component}/tests/baselines"))
        .join(baseline);
    let update_filter = std::env::var("UPDATE_SNAPSHOTS").unwrap_or_default();
    let update_selected =
        update_filter.split(',').any(|prefix| !prefix.is_empty() && baseline.starts_with(prefix));
    if update_filter == "1" || update_selected {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated {component} screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing {component} baseline {}: {error}", path.display()))
        .to_rgba8();
    if !(PixelTolerance { channel_delta: 2, max_different_pixels: 32 }).matches(&expected, actual) {
        let diff = path
            .with_file_name(format!("{}-diff.png", path.file_stem().unwrap().to_string_lossy()));
        let mut image = RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let left = expected.get_pixel(x, y);
            let right = actual.get_pixel(x, y);
            *pixel = image::Rgba([
                left[0].abs_diff(right[0]).saturating_mul(4),
                left[1].abs_diff(right[1]).saturating_mul(4),
                left[2].abs_diff(right[2]).saturating_mul(4),
                255,
            ]);
        }
        image.save(&diff).expect("write diff");
        panic!("{component} screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched {component} screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    for (component, expected_cases) in [("tooltip", 18), ("sidebar", 12)] {
        let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../registry/{component}/tests/conformance.json"));
        let manifest: Value =
            serde_json::from_slice(&fs::read(&manifest_path).expect("read screenshot manifest"))
                .expect("manifest JSON");
        let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
        assert_eq!(cases.len(), expected_cases, "{component} declared state/theme/scale coverage");
        let mut tooltip_focused: std::collections::HashMap<(String, u32), RgbaImage> =
            std::collections::HashMap::new();
        for case in cases {
            let state = case["state"].as_str().expect("state");
            let state: &'static str = match (component, state) {
                ("tooltip", "visible") => "visible",
                ("tooltip", "keyboard_focused_child") => "keyboard_focused_child",
                ("tooltip", "disabled") => "disabled",
                ("sidebar", "selection") => "selection",
                ("sidebar", "disabled") => "disabled",
                (_, other) => panic!("unmapped {component} state: {other}"),
            };
            let theme = theme_for(case["theme"].as_str().expect("theme"));
            let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
            let baseline = case["baseline"].as_str().expect("baseline");
            if case["status"].as_str() == Some("unsupported_headless_capture") {
                assert_eq!(component, "tooltip");
                assert_eq!(state, "visible");
                println!("Skipped unsupported Tooltip popup capture: {baseline}");
                continue;
            }
            let actual = if component == "tooltip" {
                capture_tooltip(state, theme, scale)?
            } else {
                capture_sidebar(state, theme, scale)?
            };
            assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
            if component == "tooltip" {
                let key = (case["theme"].as_str().unwrap().to_owned(), scale);
                match state {
                    "keyboard_focused_child" => {
                        tooltip_focused.insert(key, actual.clone());
                    }
                    "disabled" => {
                        let focused = tooltip_focused
                            .get(&key)
                            .expect("focused Tooltip rendered before disabled comparison");
                        let scale = scale as usize;
                        let changed_in_trigger = (10 * scale..205 * scale)
                            .flat_map(|x| (85 * scale..140 * scale).map(move |y| (x, y)))
                            .filter(|&(x, y)| {
                                (0..3).any(|channel| {
                                    actual.get_pixel(x as u32, y as u32)[channel]
                                        .abs_diff(focused.get_pixel(x as u32, y as u32)[channel])
                                        > 2
                                })
                            })
                            .count();
                        assert!(
                            changed_in_trigger > 32,
                            "focused Tooltip child must show a focus-token cue in the trigger area; changed pixels={changed_in_trigger}"
                        );
                    }
                    _ => {}
                }
            }
            compare_or_update(component, &actual, baseline);
        }
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping Tooltip and Sidebar screenshot matrices: macOS Metal is required.");
        }
        Err(error) => panic!("Tooltip and Sidebar screenshot matrices failed: {error}"),
    }
}
