use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Keystroke, Render, Window, actions, div,
    prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    menu_bar::{MenuBar, MenuBarModel, MenuDefinition, MenuEntry},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

actions!(menu_bar_matrix, [NewFile, SaveFile, UndoEdit, CopyText]);
const SIZE: (f32, f32) = (560.0, 320.0);

fn model() -> MenuBarModel {
    MenuBarModel::new([
        MenuDefinition::new("file", "File").mnemonic('f').items([
            MenuEntry::command("new", "New", NewFile),
            MenuEntry::command("save", "Save", SaveFile).shortcut("⌘S").disabled(true),
            MenuEntry::separator(),
            MenuEntry::submenu(
                "recent",
                "Recent",
                [MenuEntry::command("recent-one", "Document.txt", NewFile)],
            ),
        ]),
        MenuDefinition::new("edit", "Edit").mnemonic('e').items([
            MenuEntry::command("undo", "Undo", UndoEdit).checked(true),
            MenuEntry::command("copy", "Copy", CopyText).shortcut("⌘C"),
        ]),
    ])
}

struct Fixture {
    state: &'static str,
    bar: Option<Entity<MenuBar>>,
}
impl Render for Fixture {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let bar = self.bar.get_or_insert_with(|| {
            cx.new(|cx| {
                let mut bar = MenuBar::new(model(), cx);
                if state == "open" {
                    bar.set_open_menu(Some("file"), cx);
                }
                if state == "focused" {
                    let focus = bar.focus_handle(cx);
                    window.focus(&focus, cx);
                }
                if state == "disabled" {
                    bar = bar.disabled(true);
                }
                bar
            })
        });
        div()
            .size_full()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(
                div()
                    .text_size(px(theme.typography.heading_small))
                    .text_color(theme.colors.text)
                    .child("MenuBar"),
            )
            .child(bar.clone())
    }
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        Fixture { state, bar: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::menu_bar::default_key_bindings());
        },
    )?;
    if state == "focused" {
        // A key press switches GPUI to keyboard modality, as for a user who reached the bar
        // with the keyboard; Escape with no popup open changes nothing else.
        session.update(|root, window, cx| {
            let bar = root.read(cx).bar.clone().expect("menu bar rendered");
            window.dispatch_keystroke(Keystroke::parse("escape").expect("Escape key"), cx);
            assert!(bar.read(cx).focus_handle(cx).is_focused(window), "bar keeps focus");
            assert!(!bar.read(cx).is_open(), "focused fixture stays closed");
        })?;
    }
    session.capture()
}
fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let relative = baseline.strip_prefix("menu-bar/").expect("component prefix");
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/menu-bar/tests/baselines")
        .join(relative);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("parent")).expect("create baseline directory");
        actual.save(&path).expect("write baseline");
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}", path.display()))
        .to_rgba8();
    assert!(
        PixelTolerance { channel_delta: 2, max_different_pixels: 32 }.matches(&expected, actual),
        "screenshot differs: {}",
        path.display()
    );
}
fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/menu-bar/tests/conformance.json"))
            .expect("manifest");
    let cases = manifest["screenshot_cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 4 * 3 * 2);
    for case in cases {
        let state = match case["state"].as_str().unwrap() {
            "idle" => "idle",
            "focused" => "focused",
            "open" => "open",
            "disabled" => "disabled",
            other => panic!("unmapped state {other}"),
        };
        let theme = match case["theme"].as_str().unwrap() {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped theme {other}"),
        };
        let scale = case["scale"].as_u64().unwrap() as u32;
        let image = capture(state, theme, scale)?;
        assert_eq!(image.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&image, case["baseline"].as_str().unwrap());
    }
    Ok(())
}
fn main() {
    match run() {
        Ok(()) => println!("MenuBar screenshot matrix passed."),
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping MenuBar screenshots: macOS Metal is required.")
        }
        Err(error) => panic!("MenuBar screenshot matrix failed: {error}"),
    }
}
