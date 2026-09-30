use gpui_pre::{
    App, Context, Entity, Focusable, InputEvent, IntoElement, Keystroke, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render, Window, div, point, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    tree::{self, Tree, TreeNode},
    virtual_list::{self, ListItem, VirtualList},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf};

const SIZE: (f32, f32) = (480.0, 260.0);

struct ListFixture {
    state: &'static str,
    list: Option<Entity<VirtualList>>,
}

impl Render for ListFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let list = self.list.get_or_insert_with(|| {
            let items = (1..=8)
                .map(|index| ListItem::new(format!("item-{index}"), format!("Item {index:02}")))
                .collect();
            cx.new(|_| match state {
                "empty" => VirtualList::new("Files", Vec::new()).viewport_height(220.0),
                "idle" | "selected" => {
                    VirtualList::new("Files", items).row_height(34.0).viewport_height(220.0)
                }
                other => panic!("unmapped VirtualList state: {other}"),
            })
        });
        div().size_full().bg(theme.colors.background).child(list.clone())
    }
}

fn tree_roots(state: &str) -> Vec<TreeNode> {
    let branch = match state {
        "collapsed" | "expanded" => TreeNode::branch(
            "folder",
            "Documents",
            vec![TreeNode::leaf("readme", "Read me"), TreeNode::leaf("notes", "Notes")],
        ),
        "loading" => TreeNode::lazy("folder", "Documents"),
        other => panic!("unmapped Tree state: {other}"),
    };
    vec![branch, TreeNode::leaf("archive", "Archive")]
}

struct TreeFixture {
    state: &'static str,
    tree: Option<Entity<Tree>>,
}

impl Render for TreeFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let tree =
            self.tree.get_or_insert_with(|| cx.new(|_| Tree::new("Documents", tree_roots(state))));
        div().size_full().bg(theme.colors.background).child(tree.clone())
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

fn capture_list(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
    check_pointer: bool,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        ListFixture { state, list: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(virtual_list::default_key_bindings());
        },
    )?;

    if state == "selected" {
        session.update(|root, window, cx| {
            let list = root.read(cx).list.as_ref().expect("VirtualList initialized").clone();
            list.update(cx, |list, cx| list.focus_handle(cx).focus(window, cx));
            window.dispatch_keystroke(Keystroke::parse("space").expect("Space key"), cx);
            window.dispatch_keystroke(Keystroke::parse("down").expect("Down key"), cx);
            let view = list.read(cx);
            assert_eq!(view.active(), Some("item-2"));
            assert_eq!(view.selected(), &["item-1".to_owned()]);
        })?;
    } else {
        session.update(|root, _, cx| {
            let list = root.read(cx).list.as_ref().expect("VirtualList initialized").read(cx);
            assert_eq!(list.active(), (state != "empty").then_some("item-1"));
            assert!(list.selected().is_empty());
        })?;
    }

    let image = session.capture()?;
    if check_pointer {
        session.update(|root, window, cx| {
            let list = root.read(cx).list.as_ref().expect("VirtualList initialized").clone();
            // The 34 px second row starts immediately below the one-pixel border.
            let position = point(px(70.0), px(50.0));
            window.dispatch_event(
                MouseMoveEvent { position, pressed_button: None, modifiers: Default::default() }
                    .to_platform_input(),
                cx,
            );
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
            window.dispatch_event(
                MouseUpEvent {
                    position,
                    button: MouseButton::Left,
                    modifiers: Default::default(),
                    click_count: 1,
                }
                .to_platform_input(),
                cx,
            );
            let view = list.read(cx);
            assert_eq!(view.active(), Some("item-2"), "row click activates the second item");
            assert_eq!(view.selected(), &["item-2".to_owned()], "row click selects the item");
        })?;
    }
    Ok(image)
}

fn capture_tree(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
    check_pointer: bool,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        TreeFixture { state, tree: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(tree::default_key_bindings());
        },
    )?;

    session.update(|root, window, cx| {
        let tree = root.read(cx).tree.as_ref().expect("Tree initialized").clone();
        if state == "expanded" || state == "loading" {
            // Clicking the active label gives the tab-indexed tree keyboard focus.
            let position = point(px(90.0), px(10.0));
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
            window.dispatch_event(
                MouseUpEvent {
                    position,
                    button: MouseButton::Left,
                    modifiers: Default::default(),
                    click_count: 1,
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_keystroke(Keystroke::parse("right").expect("Right key"), cx);
        }
        let view = tree.read(cx);
        assert_eq!(view.active(), Some("folder"), "active tree node: {state}");
        assert_eq!(view.is_expanded("folder"), state != "collapsed");
    })?;

    let image = session.capture()?;
    if check_pointer {
        session.update(|root, window, cx| {
            let tree = root.read(cx).tree.as_ref().expect("Tree initialized").clone();
            // The collapsed tree has two 32 px rows below the border and 4 px padding; clicking
            // the second label activates Archive.
            let row = point(px(70.0), px(48.0));
            window.dispatch_event(
                MouseMoveEvent {
                    position: row,
                    pressed_button: None,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseDownEvent {
                    position: row,
                    button: MouseButton::Left,
                    modifiers: Default::default(),
                    click_count: 1,
                    first_mouse: false,
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseUpEvent {
                    position: row,
                    button: MouseButton::Left,
                    modifiers: Default::default(),
                    click_count: 1,
                }
                .to_platform_input(),
                cx,
            );
            assert_eq!(tree.read(cx).active(), Some("archive"), "row click activates Archive");

            let disclosure = point(px(24.0), px(10.0));
            window.dispatch_event(
                MouseMoveEvent {
                    position: disclosure,
                    pressed_button: None,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseDownEvent {
                    position: disclosure,
                    button: MouseButton::Left,
                    modifiers: Default::default(),
                    click_count: 1,
                    first_mouse: false,
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseUpEvent {
                    position: disclosure,
                    button: MouseButton::Left,
                    modifiers: Default::default(),
                    click_count: 1,
                }
                .to_platform_input(),
                cx,
            );
            assert!(tree.read(cx).is_expanded("folder"), "disclosure click expands the branch");
        })?;
    }
    Ok(image)
}

fn compare_or_update(component: &str, actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry")
        .join(component)
        .join("tests/baselines")
        .join(baseline.strip_prefix(&format!("{component}/")).expect("baseline prefix"));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated {component} screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing {component} baseline {}: {error}", path.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 32 };
    if !tolerance.matches(&expected, actual) {
        let diff = path.with_file_name(format!(
            "{}-diff.png",
            path.file_stem().expect("baseline stem").to_string_lossy()
        ));
        let mut diff_image = RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in diff_image.enumerate_pixels_mut() {
            let left = expected.get_pixel(x, y);
            let right = actual.get_pixel(x, y);
            *pixel = image::Rgba([
                left[0].abs_diff(right[0]).saturating_mul(4),
                left[1].abs_diff(right[1]).saturating_mul(4),
                left[2].abs_diff(right[2]).saturating_mul(4),
                255,
            ]);
        }
        diff_image.save(&diff).expect("write screenshot diff");
        panic!("{component} screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched {component} screenshot: {baseline}");
}

fn run_component(component: &str) -> Result<(), ScreenshotError> {
    let source = match component {
        "virtual-list" => include_str!("../../../registry/virtual-list/tests/conformance.json"),
        "tree" => include_str!("../../../registry/tree/tests/conformance.json"),
        _ => unreachable!(),
    };
    let manifest: Value = serde_json::from_str(source).expect("conformance manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 18, "three states × three themes × two scales");
    let mut seen = HashSet::new();
    for case in cases {
        let state = case["state"].as_str().expect("state");
        let state: &'static str = match (component, state) {
            ("virtual-list", "empty") => "empty",
            ("virtual-list", "idle") => "idle",
            ("virtual-list", "selected") => "selected",
            ("tree", "collapsed") => "collapsed",
            ("tree", "expanded") => "expanded",
            ("tree", "loading") => "loading",
            (_, other) => panic!("unmapped {component} screenshot state: {other}"),
        };
        let theme_name = case["theme"].as_str().expect("theme");
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let baseline = case["baseline"].as_str().expect("baseline");
        assert!(seen.insert((state, theme_name, scale)), "duplicate screenshot tuple");
        let check_pointer = state == if component == "virtual-list" { "idle" } else { "collapsed" }
            && theme_name == "light"
            && scale == 1;
        let actual = if component == "virtual-list" {
            capture_list(state, theme_for(theme_name), scale, check_pointer)?
        } else {
            capture_tree(state, theme_for(theme_name), scale, check_pointer)?
        };
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(component, &actual, baseline);
    }
    Ok(())
}

fn run() -> Result<(), ScreenshotError> {
    run_component("virtual-list")?;
    run_component("tree")
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping collection screenshot matrices: macOS Metal is required.");
        }
        Err(error) => panic!("collection screenshot matrix failed: {error}"),
    }
}
