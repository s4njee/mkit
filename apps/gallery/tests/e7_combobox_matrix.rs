use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Keystroke, Render, Window, div, prelude::*, px,
    size,
};
use image::RgbaImage;
use mkit::{
    combobox::{self, Combobox, OptionItem},
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf};

const SIZE: (f32, f32) = (480.0, 300.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    ClosedEmpty,
    OpenEmpty,
    OpenUnfilteredActive,
    OpenFiltered,
    OpenNoMatches,
    ClosedCommitted,
    ClosedDraft,
    Disabled,
}

impl State {
    fn from_manifest(value: &str) -> Self {
        match value {
            "closed_empty" => Self::ClosedEmpty,
            "open_empty" => Self::OpenEmpty,
            "open_unfiltered_active" => Self::OpenUnfilteredActive,
            "open_filtered" => Self::OpenFiltered,
            "open_no_matches" => Self::OpenNoMatches,
            "closed_committed" => Self::ClosedCommitted,
            "closed_draft" => Self::ClosedDraft,
            "disabled" => Self::Disabled,
            other => panic!("unmapped Combobox screenshot state: {other}"),
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::ClosedEmpty => "closed_empty",
            Self::OpenEmpty => "open_empty",
            Self::OpenUnfilteredActive => "open_unfiltered_active",
            Self::OpenFiltered => "open_filtered",
            Self::OpenNoMatches => "open_no_matches",
            Self::ClosedCommitted => "closed_committed",
            Self::ClosedDraft => "closed_draft",
            Self::Disabled => "disabled",
        }
    }
}

fn options() -> Vec<OptionItem> {
    vec![
        OptionItem::new("us", "United States"),
        OptionItem::new("ca", "Canada"),
        OptionItem::new("cm", "Cameroon"),
        OptionItem::new("kh", "Cambodia"),
        OptionItem::new("jp", "Japan"),
        OptionItem::new("xx", "Example disabled").disabled(true),
    ]
}

struct ComboboxFixture {
    state: State,
    control: Option<Entity<Combobox>>,
}

impl ComboboxFixture {
    fn new(state: State) -> Self {
        Self { state, control: None }
    }
}

impl Render for ComboboxFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let control = self.control.get_or_insert_with(|| {
            cx.new(|_| {
                Combobox::new(
                    "Country",
                    options(),
                    (state == State::ClosedCommitted || state == State::ClosedDraft)
                        .then(|| "us".to_owned()),
                )
                .disabled(state == State::Disabled)
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
                    .child("Combobox"),
            )
            .child(div().w(px(320.0)).child(control.clone()))
    }
}

fn press(window: &mut Window, key: &str, cx: &mut App) {
    window.dispatch_keystroke(Keystroke::parse(key).unwrap_or_else(|e| panic!("{key}: {e}")), cx);
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        ComboboxFixture::new(state),
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(combobox::default_key_bindings());
        },
    )?;

    if matches!(
        state,
        State::OpenEmpty
            | State::OpenUnfilteredActive
            | State::OpenFiltered
            | State::OpenNoMatches
            | State::ClosedDraft
    ) {
        session.update(|root, window, cx| {
            let control = root.read(cx).control.as_ref().expect("Combobox initialized").clone();
            control.update(cx, |combobox, cx| combobox.focus_handle(cx).focus(window, cx));

            match state {
                State::OpenEmpty => press(window, "alt-down", cx),
                State::OpenUnfilteredActive => press(window, "down", cx),
                State::OpenFiltered => {
                    control.update(cx, |combobox, cx| combobox.set_query("Cam", cx));
                    press(window, "down", cx);
                }
                State::OpenNoMatches => {
                    control.update(cx, |combobox, cx| combobox.set_query("Atlantis", cx));
                }
                State::ClosedDraft => {
                    control.update(cx, |combobox, cx| combobox.set_query("Cam", cx));
                    press(window, "tab", cx);
                }
                _ => unreachable!(),
            }

            let control = control.read(cx);
            assert_eq!(
                control.is_open(),
                matches!(
                    state,
                    State::OpenEmpty
                        | State::OpenUnfilteredActive
                        | State::OpenFiltered
                        | State::OpenNoMatches
                ),
                "{} popup visibility",
                state.id()
            );
            match state {
                State::OpenEmpty => {
                    assert_eq!(control.query(), "");
                    assert_eq!(control.active_option(), None);
                }
                State::OpenUnfilteredActive => {
                    assert_eq!(control.active_option(), Some("us"));
                }
                State::OpenFiltered => {
                    assert_eq!(control.query(), "Cam");
                    assert_eq!(control.active_option(), Some("cm"));
                }
                State::OpenNoMatches => {
                    assert_eq!(control.query(), "Atlantis");
                    assert_eq!(control.active_option(), None);
                }
                State::ClosedDraft => {
                    assert_eq!(control.query(), "Cam");
                    assert_eq!(control.value(), Some("us"));
                    assert_eq!(control.active_option(), None);
                }
                _ => unreachable!(),
            }
        })?;
    } else if state == State::ClosedCommitted {
        session.update(|root, _, cx| {
            let control = root.read(cx).control.as_ref().expect("Combobox initialized").read(cx);
            assert_eq!(control.value(), Some("us"));
            assert_eq!(control.query(), "United States");
            assert!(!control.is_open());
        })?;
    } else if state == State::Disabled {
        session.update(|root, _, cx| {
            let control = root.read(cx).control.as_ref().expect("Combobox initialized").read(cx);
            assert!(!control.is_open());
            assert_eq!(control.query(), "");
            assert_eq!(control.value(), None);
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/combobox/tests/baselines")
        .join(baseline.strip_prefix("combobox/").expect("combobox baseline prefix"));
    let update_filter = std::env::var("UPDATE_SNAPSHOT_STATES").ok();
    let state = baseline.split('/').nth(1).expect("baseline state path");
    let selected_by_filter = update_filter
        .as_deref()
        .is_none_or(|filter| filter.split(',').any(|selected| selected.trim() == state));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1"))
        && selected_by_filter
    {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated Combobox screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing Combobox baseline {}: {error}", path.display()))
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
        panic!("Combobox screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched Combobox screenshot: {}", path.display());
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/combobox/tests/conformance.json"))
            .expect("Combobox manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 8 * 3 * 2, "all state/theme/scale combinations must be declared");
    let mut seen = HashSet::new();
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        let state = State::from_manifest(state_name);
        let theme_name = case["theme"].as_str().expect("theme");
        let theme_value = match theme_name {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped Combobox screenshot theme: {other}"),
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
    assert_eq!(seen.len(), 48);
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping Combobox screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("Combobox screenshot matrix failed: {error}"),
    }
}
