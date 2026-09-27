use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    shortcut_editor::{self, ShortcutAction, ShortcutEditor},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::PathBuf,
};

const SIZE: (f32, f32) = (720.0, 420.0);
const SAVED_PATH: &str = "target/shortcut-editor-matrix/keymap.json";
const FAILED_PATH: &str = "target/shortcut-editor-matrix-missing/keymap.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Viewing,
    Capturing,
    Conflict,
    Dirty,
    Saved,
    SaveError,
}

impl State {
    fn from_manifest(value: &str) -> Self {
        match value {
            "viewing" => Self::Viewing,
            "capturing" => Self::Capturing,
            "conflict" => Self::Conflict,
            "dirty" => Self::Dirty,
            "saved" => Self::Saved,
            "save_error" => Self::SaveError,
            other => panic!("unmapped ShortcutEditor screenshot state: {other}"),
        }
    }
}

fn actions() -> Vec<ShortcutAction> {
    vec![
        ShortcutAction::new("open", "Open file").category("File"),
        ShortcutAction::new("save", "Save file").category("File"),
        ShortcutAction::new("palette", "Command palette").category("View"),
        ShortcutAction::new("sidebar", "Toggle sidebar").category("View"),
    ]
}

fn bindings() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("open".into(), "cmd-o".into()),
        ("save".into(), "cmd-s".into()),
        ("palette".into(), "cmd-k".into()),
        ("sidebar".into(), "cmd-b".into()),
    ])
}

struct Fixture {
    state: State,
    editor: Option<Entity<ShortcutEditor>>,
}
impl Fixture {
    fn new(state: State) -> Self {
        Self { state, editor: None }
    }
}

impl Render for Fixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let editor = self
            .editor
            .get_or_insert_with(|| cx.new(|_| ShortcutEditor::new(actions(), bindings())))
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading_small)).child("Keyboard shortcuts"))
            .child(div().w(px(640.0)).child(editor))
            .when(state == State::SaveError, |el| {
                el.child(
                    div()
                        .text_color(theme.colors.text_muted)
                        .child("The draft remains available for retry."),
                )
            })
    }
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        Fixture::new(state),
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(shortcut_editor::default_key_bindings());
        },
    )?;
    if state != State::Viewing {
        session.update(|root, window, cx| {
            let editor = root.read(cx).editor.as_ref().expect("editor initialized").clone();
            editor.focus_handle(cx).focus(window, cx);
        })?;
        match state {
            State::Viewing => unreachable!(),
            State::Capturing => session.simulate_keystrokes("enter")?,
            State::Conflict => session.simulate_keystrokes("down enter cmd-o")?,
            State::Dirty | State::Saved | State::SaveError => {
                session.simulate_keystrokes("enter cmd-p")?;
            }
        }
        if state == State::Saved || state == State::SaveError {
            if state == State::Saved {
                let parent = PathBuf::from(SAVED_PATH).parent().unwrap().to_path_buf();
                fs::create_dir_all(parent).expect("create stable screenshot output directory");
            } else {
                let missing = PathBuf::from(FAILED_PATH).parent().unwrap().to_path_buf();
                let _ = fs::remove_dir_all(missing);
            }
            session.update(|root, _, cx| {
                let editor = root.read(cx).editor.as_ref().expect("editor initialized").clone();
                editor.update(cx, |editor, cx| {
                    let path = if state == State::Saved { SAVED_PATH } else { FAILED_PATH };
                    let result = editor.save_to_path(path, cx);
                    assert_eq!(result.is_ok(), state == State::Saved);
                });
            })?;
        }
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/shortcut-editor/tests/baselines")
        .join(baseline);
    if let Some(candidate_root) = std::env::var_os("E8_SHORTCUT_EDITOR_CANDIDATE_DIR") {
        let candidate = PathBuf::from(candidate_root).join(baseline);
        fs::create_dir_all(candidate.parent().expect("candidate parent"))
            .expect("create candidate dir");
        actual.save(&candidate).expect("write screenshot candidate");
        println!("Captured ShortcutEditor candidate: {}", candidate.display());
        return;
    }
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated ShortcutEditor screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| {
            panic!("missing ShortcutEditor baseline {}: {error}", path.display())
        })
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 32 };
    if !tolerance.matches(&expected, actual) {
        panic!("ShortcutEditor screenshot differs: {}", path.display());
    }
    println!("Matched ShortcutEditor screenshot: {}", path.display());
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../registry/shortcut-editor/tests/conformance.json"
    ))
    .expect("ShortcutEditor manifest");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 6 * 3 * 2, "six states × three themes × two scales");
    let mut seen = HashSet::new();
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        let state = State::from_manifest(state_name);
        let theme_name = case["theme"].as_str().expect("theme");
        let theme_value = match theme_name {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped ShortcutEditor screenshot theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        assert!(matches!(scale, 1 | 2));
        assert_eq!(case["load_fixture"], format!("{state_name}_fixture"));
        assert_eq!(case["platform"], "macos");
        assert!(seen.insert((state_name, theme_name, scale)), "duplicate screenshot case");
        let actual = capture(state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, case["baseline"].as_str().expect("baseline"));
    }
    let _ = fs::remove_file(SAVED_PATH);
    let _ = fs::remove_dir(PathBuf::from(SAVED_PATH).parent().unwrap());
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping ShortcutEditor matrix: GPUI headless capture requires macOS Metal.")
        }
        Err(error) => panic!("ShortcutEditor screenshot matrix failed: {error}"),
    }
}
