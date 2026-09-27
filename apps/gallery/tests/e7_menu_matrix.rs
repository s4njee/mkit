use gpui_pre::{
    App, Context, Entity, FocusHandle, Focusable, InputEvent, IntoElement, Keystroke, MouseButton,
    MouseDownEvent, Render, Window, div, point, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    context_menu::{self, ContextMenu, MenuItem as ContextItem},
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    dropdown_menu::{self, DropdownMenu, MenuItem as DropdownItem},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf};

const SIZE: (f32, f32) = (480.0, 320.0);
const ANCHOR: (f32, f32) = (136.0, 88.0);
const OUTSIDE: (f32, f32) = (456.0, 292.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Component {
    Dropdown,
    Context,
}

impl Component {
    fn id(self) -> &'static str {
        match self {
            Self::Dropdown => "dropdown-menu",
            Self::Context => "context-menu",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Closed,
    Open,
    OutsideDismissed,
    Submenu,
}

impl State {
    fn id(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
            Self::OutsideDismissed => "outside_dismissed",
            Self::Submenu => "submenu",
        }
    }

    fn from_manifest(value: &str) -> Self {
        match value {
            "closed" => Self::Closed,
            "open" => Self::Open,
            "outside_dismissed" => Self::OutsideDismissed,
            "submenu" => Self::Submenu,
            other => panic!("unmapped menu screenshot state: {other}"),
        }
    }
}

fn dropdown_items() -> Vec<DropdownItem> {
    vec![
        DropdownItem::new("more-actions", "More actions").submenu(vec![
            DropdownItem::new("duplicate-view", "Duplicate view").shortcut("⌘D"),
            DropdownItem::new("move-to-folder", "Move to folder…").shortcut("⌘M"),
        ]),
        DropdownItem::new("new-file", "New file").shortcut("⌘N"),
        DropdownItem::new("favorite", "Favorite").checked(false),
        DropdownItem::new("archive", "Archive").disabled(true),
    ]
}

fn context_items() -> Vec<ContextItem> {
    vec![
        ContextItem::new("arrange", "Arrange").submenu(vec![
            ContextItem::new("bring-forward", "Bring forward").shortcut("⌘]"),
            ContextItem::new("send-backward", "Send backward").shortcut("⌘["),
        ]),
        ContextItem::new("rename", "Rename").shortcut("F2"),
        ContextItem::new("pin", "Pin to top").checked(true),
        ContextItem::new("remove", "Remove").disabled(true),
    ]
}

struct MenuFixture {
    component: Component,
    dropdown: Option<Entity<DropdownMenu>>,
    context: Option<Entity<ContextMenu>>,
    host_focus: Option<FocusHandle>,
}

impl MenuFixture {
    fn new(component: Component) -> Self {
        Self { component, dropdown: None, context: None, host_focus: None }
    }
}

impl Render for MenuFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let host_focus =
            self.host_focus.get_or_insert_with(|| cx.focus_handle().tab_stop(true)).clone();
        let menu = match self.component {
            Component::Dropdown => {
                let host_focus = host_focus.clone();
                self.dropdown
                    .get_or_insert_with(|| {
                        cx.new(move |_| {
                            DropdownMenu::new(dropdown_items())
                                .anchor_at(point(px(ANCHOR.0), px(ANCHOR.1)))
                                .return_focus_to(&host_focus)
                        })
                    })
                    .clone()
                    .into_any_element()
            }
            Component::Context => {
                let host_focus = host_focus.clone();
                self.context
                    .get_or_insert_with(|| {
                        cx.new(move |_| {
                            ContextMenu::new(context_items())
                                .anchor_at(point(px(ANCHOR.0), px(ANCHOR.1)))
                                .return_focus_to(&host_focus)
                        })
                    })
                    .clone()
                    .into_any_element()
            }
        };

        div()
            .size_full()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.heading_small))
                    .child(match self.component {
                        Component::Dropdown => "Dropdown Menu",
                        Component::Context => "Context Menu",
                    }),
            )
            .child(
                div()
                    .id("menu-invocation-target")
                    .debug_selector(|| "menu-invocation-target".into())
                    .track_focus(&host_focus)
                    .tab_stop(true)
                    .w(px(190.0))
                    .h(px(36.0))
                    .px(px(theme.spacing.small))
                    .flex()
                    .items_center()
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.hairline))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.surface)
                    .text_color(theme.colors.text)
                    .child("Invocation target"),
            )
            .child(menu)
    }
}

fn capture(
    component: Component,
    state: State,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        MenuFixture::new(component),
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(dropdown_menu::default_key_bindings());
            cx.bind_keys(context_menu::default_key_bindings());
        },
    )?;

    if state != State::Closed {
        session.update(|root, window, cx| {
            let (component, host_focus, dropdown, context) = {
                let fixture = root.read(cx);
                (
                    fixture.component,
                    fixture.host_focus.as_ref().expect("host focus initialized").clone(),
                    fixture.dropdown.clone(),
                    fixture.context.clone(),
                )
            };
            assert_eq!(component, component_for(&dropdown, &context));
            host_focus.focus(window, cx);
            match component {
                Component::Dropdown => dropdown
                    .as_ref()
                    .expect("DropdownMenu initialized")
                    .update(cx, |menu, cx| menu.set_open(true, cx)),
                Component::Context => context
                    .as_ref()
                    .expect("ContextMenu initialized")
                    .update(cx, |menu, cx| menu.set_open(true, cx)),
            }
        })?;

        match state {
            State::Submenu => {
                session.update(|root, window, cx| {
                    let (component, dropdown, context) = {
                        let fixture = root.read(cx);
                        (fixture.component, fixture.dropdown.clone(), fixture.context.clone())
                    };
                    match component {
                        Component::Dropdown => {
                            let menu = dropdown.as_ref().unwrap().clone();
                            menu.update(cx, |menu, cx| menu.focus_handle(cx).focus(window, cx));
                        }
                        Component::Context => {
                            let menu = context.as_ref().unwrap().clone();
                            menu.update(cx, |menu, cx| menu.focus_handle(cx).focus(window, cx));
                        }
                    }
                    window.dispatch_keystroke(
                        Keystroke::parse("right").expect("Right action key"),
                        cx,
                    );
                    let (open, focused) = match component {
                        Component::Dropdown => {
                            let menu = dropdown.as_ref().unwrap();
                            (
                                menu.read(cx).is_open(),
                                menu.update(cx, |m, cx| m.focus_handle(cx).is_focused(window)),
                            )
                        }
                        Component::Context => {
                            let menu = context.as_ref().unwrap();
                            (
                                menu.read(cx).is_open(),
                                menu.update(cx, |m, cx| m.focus_handle(cx).is_focused(window)),
                            )
                        }
                    };
                    assert!(open, "submenu action must keep menu open");
                    assert!(focused, "submenu action should keep menu focus in the active pane");
                })?;
            }
            State::OutsideDismissed => {
                session.update(|root, window, cx| {
                    let (component, dropdown, context) = {
                        let fixture = root.read(cx);
                        (fixture.component, fixture.dropdown.clone(), fixture.context.clone())
                    };
                    match component {
                        Component::Dropdown => {
                            let menu = dropdown.as_ref().unwrap().clone();
                            menu.update(cx, |menu, cx| menu.focus_handle(cx).focus(window, cx));
                        }
                        Component::Context => {
                            let menu = context.as_ref().unwrap().clone();
                            menu.update(cx, |menu, cx| menu.focus_handle(cx).focus(window, cx));
                        }
                    }
                    let position = point(px(OUTSIDE.0), px(OUTSIDE.1));
                    window.dispatch_event(
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
                })?;
                session.update(|root, window, cx| {
                    let fixture = root.read(cx);
                    assert_menu_closed(fixture, cx, "outside mouse-down must dismiss menu");
                    let host = fixture.host_focus.as_ref().unwrap();
                    assert!(host.is_focused(window), "outside dismissal must restore host focus");
                })?;
            }
            State::Open => {
                session.update(|root, window, cx| {
                    let fixture = root.read(cx);
                    assert_menu_open(fixture, cx, "open fixture must show its menu");
                    assert!(
                        fixture.host_focus.as_ref().unwrap().is_focused(window),
                        "open fixture should retain invocation-target focus"
                    );
                })?;
            }
            State::Closed => unreachable!(),
        }
    } else {
        session.update(|root, _, cx| {
            let fixture = root.read(cx);
            assert_menu_closed(fixture, cx, "closed fixture must not show a menu");
        })?;
    }
    session.capture()
}

fn component_for(
    dropdown: &Option<Entity<DropdownMenu>>,
    context: &Option<Entity<ContextMenu>>,
) -> Component {
    if dropdown.is_some() {
        Component::Dropdown
    } else if context.is_some() {
        Component::Context
    } else {
        panic!("menu entity not initialized")
    }
}

fn assert_menu_open(fixture: &MenuFixture, cx: &App, message: &str) {
    let open = match fixture.component {
        Component::Dropdown => fixture.dropdown.as_ref().unwrap().read(cx).is_open(),
        Component::Context => fixture.context.as_ref().unwrap().read(cx).is_open(),
    };
    assert!(open, "{message}");
}

fn assert_menu_closed(fixture: &MenuFixture, cx: &App, message: &str) {
    let open = match fixture.component {
        Component::Dropdown => fixture.dropdown.as_ref().unwrap().read(cx).is_open(),
        Component::Context => fixture.context.as_ref().unwrap().read(cx).is_open(),
    };
    assert!(!open, "{message}");
}

fn compare_or_update(actual: &RgbaImage, component: Component, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry")
        .join(component.id())
        .join("tests/baselines")
        .join(baseline.strip_prefix(&format!("{}/", component.id())).expect("baseline prefix"));
    let update_filter = std::env::var("UPDATE_SNAPSHOT_FILTER").ok();
    let selected_by_filter = update_filter
        .as_deref()
        .is_none_or(|filter| filter.split(',').any(|selected| baseline.contains(selected.trim())));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1"))
        && selected_by_filter
    {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated {} screenshot: {}", component.id(), path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| {
            panic!("missing {} baseline {}: {error}", component.id(), path.display())
        })
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 32 };
    if !tolerance.matches(&expected, actual) {
        let diff = path.with_file_name(format!(
            "{}-diff.png",
            path.file_stem().expect("baseline stem").to_string_lossy()
        ));
        let mut diff_image = RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in diff_image.enumerate_pixels_mut() {
            let before = expected.get_pixel(x, y);
            let after = actual.get_pixel(x, y);
            *pixel = image::Rgba([
                before[0].abs_diff(after[0]).saturating_mul(4),
                before[1].abs_diff(after[1]).saturating_mul(4),
                before[2].abs_diff(after[2]).saturating_mul(4),
                255,
            ]);
        }
        diff_image.save(&diff).expect("write screenshot diff");
        panic!(
            "{} screenshot differs: {}; diff: {}",
            component.id(),
            path.display(),
            diff.display()
        );
    }
    println!("Matched {} screenshot: {}", component.id(), baseline);
}

fn run_component(component: Component, manifest_text: &str) -> Result<(), ScreenshotError> {
    let manifest: Value = serde_json::from_str(manifest_text).expect("menu manifest JSON");
    assert_eq!(manifest["component"], component.id());
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 4 * 3 * 2, "all state/theme/scale combinations must be declared");
    let mut seen = HashSet::new();
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        let state = State::from_manifest(state_name);
        let theme_name = case["theme"].as_str().expect("theme");
        let theme_value = match theme_name {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped menu screenshot theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        assert!(matches!(scale, 1 | 2));
        assert_eq!(case["load_fixture"], format!("{}_{}", component.id(), state.id()));
        assert_eq!(case["platform"], "macos");
        assert!(seen.insert((state_name, theme_name, scale)), "duplicate screenshot case");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(component, state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, component, baseline);
    }
    assert_eq!(seen.len(), 24);
    Ok(())
}

fn run() -> Result<(), ScreenshotError> {
    let dropdown = include_str!("../../../registry/dropdown-menu/tests/conformance.json");
    let context = include_str!("../../../registry/context-menu/tests/conformance.json");
    run_component(Component::Dropdown, dropdown)?;
    run_component(Component::Context, context)
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping menu screenshot matrices: macOS Metal is required.");
        }
        Err(error) => panic!("menu screenshot matrix failed: {error}"),
    }
}
