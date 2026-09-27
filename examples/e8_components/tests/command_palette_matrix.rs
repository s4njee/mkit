use gpui_pre::{
    App, Context, Entity, FocusHandle, IntoElement, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    command_palette::{self, CommandAction, CommandPalette},
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf};

const SIZE: (f32, f32) = (720.0, 480.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Closed,
    OpenEmpty,
    OpenFiltered,
    OpenNoMatches,
    Disabled,
}

impl State {
    fn from_manifest(value: &str) -> Self {
        match value {
            "closed" => Self::Closed,
            "open_empty" => Self::OpenEmpty,
            "open_filtered" => Self::OpenFiltered,
            "open_no_matches" => Self::OpenNoMatches,
            "disabled" => Self::Disabled,
            other => panic!("unmapped CommandPalette screenshot state: {other}"),
        }
    }
}

fn actions() -> Vec<CommandAction> {
    vec![
        CommandAction::new("open-file", "Open File").group("File").keybinding("⌘O"),
        CommandAction::new("save-file", "Save File")
            .group("File")
            .keywords(["write", "document"])
            .keybinding("⌘S"),
        CommandAction::new("save-project", "Save Project")
            .group("Project")
            .keywords(["workspace"])
            .keybinding("⌘⇧S"),
        CommandAction::new("toggle-sidebar", "Toggle Sidebar").group("View").keybinding("⌘B"),
        CommandAction::new("disabled-command", "Disabled Command").disabled(true),
    ]
}

struct PaletteFixture {
    state: State,
    palette: Option<Entity<CommandPalette>>,
    opener: Option<FocusHandle>,
    after: Option<FocusHandle>,
}

impl PaletteFixture {
    fn new(state: State) -> Self {
        Self { state, palette: None, opener: None, after: None }
    }
}

impl Render for PaletteFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let opener = self.opener.get_or_insert_with(|| cx.focus_handle().tab_stop(true)).clone();
        let after = self.after.get_or_insert_with(|| cx.focus_handle().tab_stop(true)).clone();
        let palette = self.palette.get_or_insert_with(|| {
            cx.new(|_| CommandPalette::new(actions(), false).disabled(state == State::Disabled))
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
                    .child("Command palette host"),
            )
            .child(
                div()
                    .track_focus(&opener)
                    .tab_stop(true)
                    .text_color(theme.colors.text_muted)
                    .child("Open commands"),
            )
            .when(state == State::Disabled, |element| {
                element.child(
                    div().text_color(theme.colors.text_muted).child("Command palette disabled"),
                )
            })
            .child(palette.clone())
            .child(
                div()
                    .track_focus(&after)
                    .tab_stop(true)
                    .text_color(theme.colors.text_muted)
                    .child("Next host control"),
            )
    }
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        PaletteFixture::new(state),
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(command_palette::default_key_bindings());
        },
    )?;

    if matches!(state, State::OpenEmpty | State::OpenFiltered | State::OpenNoMatches) {
        session.update(|root, window, cx| {
            let fixture = root.read(cx);
            let palette = fixture.palette.as_ref().expect("palette initialized").clone();
            let opener = fixture.opener.as_ref().expect("opener initialized").clone();
            opener.focus(window, cx);
            palette.update(cx, |palette, cx| palette.set_open(true, cx));
        })?;

        if matches!(state, State::OpenFiltered | State::OpenNoMatches) {
            session.simulate_input(if state == State::OpenFiltered {
                "save"
            } else {
                "zz-no-match"
            })?;
            session.update(|root, _, cx| {
                let palette = root.read(cx).palette.as_ref().expect("palette initialized").clone();
                let palette = palette.read(cx);
                if state == State::OpenFiltered {
                    assert_eq!(palette.query(), "save");
                    assert_eq!(palette.active_action(), Some("save-file"));
                } else {
                    assert_eq!(palette.query(), "zz-no-match");
                    assert_eq!(palette.active_action(), None);
                }
                assert!(palette.is_open());
            })?;
        } else {
            session.update(|root, _, cx| {
                let palette = root.read(cx).palette.as_ref().expect("palette initialized").clone();
                let palette = palette.read(cx);
                assert!(palette.is_open());
                assert_eq!(palette.query(), "");
                assert_eq!(palette.active_action(), Some("open-file"));
            })?;
        }
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/command-palette/tests/baselines")
        .join(baseline.strip_prefix("command-palette/").expect("palette baseline prefix"));
    if let Some(candidate_root) = std::env::var_os("E8_COMMAND_PALETTE_CANDIDATE_DIR") {
        let candidate = PathBuf::from(candidate_root)
            .join(baseline.strip_prefix("command-palette/").expect("palette baseline prefix"));
        fs::create_dir_all(candidate.parent().expect("candidate parent"))
            .expect("create candidate dir");
        actual.save(&candidate).expect("write screenshot candidate");
        println!("Captured CommandPalette candidate: {}", candidate.display());
        return;
    }
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated CommandPalette screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| {
            panic!("missing CommandPalette baseline {}: {error}", path.display())
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
        panic!("CommandPalette screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched CommandPalette screenshot: {}", path.display());
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../registry/command-palette/tests/conformance.json"
    ))
    .expect("CommandPalette manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 5 * 3 * 2, "all five states × three themes × two scales");
    let mut seen = HashSet::new();
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        let state = State::from_manifest(state_name);
        let theme_name = case["theme"].as_str().expect("theme");
        let theme_value = match theme_name {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped CommandPalette screenshot theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        assert!(matches!(scale, 1 | 2));
        assert_eq!(case["load_fixture"], format!("{state_name}_fixture"));
        assert_eq!(case["platform"], "macos");
        assert!(seen.insert((state_name, theme_name, scale)), "duplicate screenshot case");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, baseline);
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!(
                "Skipping CommandPalette screenshot matrix: GPUI headless capture requires macOS Metal."
            );
        }
        Err(error) => panic!("CommandPalette screenshot matrix failed: {error}"),
    }
}
