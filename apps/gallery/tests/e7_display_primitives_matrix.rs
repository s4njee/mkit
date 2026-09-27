use gpui_pre::{
    App, Context, IntoElement, Render, RenderImage, Window, div, point, prelude::*, px, size,
};
use image::{Frame, Rgba, RgbaImage};
use mkit::{
    avatar::{Avatar, Size as AvatarSize},
    badge::{Badge, Status},
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    key_hint::{KeyChord, KeyHint},
    link::{Link, default_key_bindings as link_key_bindings},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{cell::Cell, fs, path::PathBuf, rc::Rc, sync::Arc};

const SIZE: (f32, f32) = (520.0, 230.0);
const BADGE_STATES: [&str; 6] =
    ["count", "count_overflow", "status_success", "status_warning", "status_danger", "disabled"];
const AVATAR_STATES: [&str; 4] = ["image", "initials", "fallback", "image_alt"];
const KEY_HINT_STATES: [&str; 3] = ["single_key", "modifier_chord", "named_key"];
const LINK_STATES: [&str; 5] = ["enabled", "hover", "focused", "visited", "disabled"];

#[derive(Clone, Copy)]
enum Component {
    Badge,
    Avatar,
    KeyHint,
    Link,
}

struct DisplayFixture {
    component: Component,
    state: &'static str,
    activated: Rc<Cell<bool>>,
}

impl Render for DisplayFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let (title, content): (&str, gpui_pre::AnyElement) = match self.component {
            Component::Badge => {
                let badge = match self.state {
                    "count" => Badge::count(7),
                    "count_overflow" => Badge::count(128).max_count(99),
                    "status_success" => Badge::status(Status::Success, "Connected"),
                    "status_warning" => Badge::status(Status::Warning, "Pending"),
                    "status_danger" => Badge::status(Status::Danger, "Failed"),
                    "disabled" => Badge::status(Status::Neutral, "Unavailable").disabled(true),
                    other => panic!("unmapped Badge state {other}"),
                };
                ("Badge", badge.into_any_element())
            }
            Component::Avatar => {
                let avatar = match self.state {
                    "image" => Avatar::new().image(portrait_image()),
                    "initials" => Avatar::new().initials("JD"),
                    "fallback" => Avatar::new(),
                    "image_alt" => Avatar::new()
                        .image(portrait_image())
                        .aria_label("Jordan Davis")
                        .size(AvatarSize::Large),
                    other => panic!("unmapped Avatar state {other}"),
                };
                ("Avatar", avatar.into_any_element())
            }
            Component::KeyHint => {
                let hint = match self.state {
                    "single_key" => KeyHint::new(KeyChord::new("K")),
                    "modifier_chord" => KeyHint::new(KeyChord::new("P").control().shift()),
                    "named_key" => KeyHint::new(KeyChord::new("ArrowRight").alt()),
                    other => panic!("unmapped KeyHint state {other}"),
                };
                ("KeyHint", hint.into_any_element())
            }
            Component::Link => {
                let activated = self.activated.clone();
                let mut link = Link::new("Open documentation")
                    .id(701)
                    .on_activate(move |_, _| activated.set(true));
                if self.state == "visited" {
                    link = link.visited(true);
                } else if self.state == "disabled" {
                    link = Link::new("Open documentation").id(701).disabled(true);
                }
                ("Link", link.into_any_element())
            }
        };
        div()
            .size_full()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.large))
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.heading_small))
                    .child(format!("{title} · {}", self.state)),
            )
            .child(content)
    }
}

fn portrait_image() -> Arc<RenderImage> {
    let mut image = RgbaImage::from_fn(64, 64, |_, _| Rgba([104, 153, 204, 255]));
    for y in 0..64 {
        for x in 0..64 {
            let xi = x as i32;
            let yi = y as i32;
            let head = (xi - 32).pow(2) + (yi - 23).pow(2) <= 12_i32.pow(2);
            let shoulders = yi >= 39 && (xi - 32).abs() < (yi - 27) * 2;
            if head {
                image.put_pixel(x, y, Rgba([228, 179, 137, 255]));
            } else if shoulders {
                image.put_pixel(x, y, Rgba([45, 65, 117, 255]));
            }
        }
    }
    // GPUI RenderImage pixels are BGRA.
    for pixel in image.pixels_mut() {
        pixel.0.swap(0, 2);
    }
    Arc::new(RenderImage::new(vec![Frame::new(image)]))
}

fn theme(name: &str) -> Theme {
    match name {
        "light" => SHADCN_LIGHT,
        "dark" => SHADCN_DARK,
        "high-contrast" => HIGH_CONTRAST,
        other => panic!("unmapped display primitive theme {other}"),
    }
}

fn capture(
    component: Component,
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        DisplayFixture { component, state, activated: Rc::new(Cell::new(false)) },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(link_key_bindings());
        },
    )?;
    if matches!(component, Component::Link) {
        if state == "focused" {
            session.update(|_, window, cx| {
                window.dispatch_keystroke(gpui_pre::Keystroke::parse("tab").expect("Tab key"), cx);
                window.focus_next(cx);
            })?;
            session.simulate_keystrokes("enter")?;
            assert!(
                session.update(|root, _, cx| root.read(cx).activated.get())?,
                "Enter dispatches the link activation callback after Tab focus"
            );
        } else if state == "hover" {
            let position = point(px(82.0), px(65.0));
            session.update(|_, window, cx| window.simulate_mouse_move(position, cx))?;
        }
    }
    session.capture()
}

fn baseline_path(component: &str, baseline: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../registry/{component}/tests/baselines"))
        .join(baseline)
}

fn compare_or_update(component: &str, actual: &RgbaImage, baseline: &str) {
    let path = std::env::var_os("SNAPSHOT_CANDIDATE_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(baseline))
        .unwrap_or_else(|| baseline_path(component, baseline));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write screenshot");
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
            let before = expected.get_pixel(x, y);
            let after = actual.get_pixel(x, y);
            *pixel = Rgba([
                before[0].abs_diff(after[0]).saturating_mul(4),
                before[1].abs_diff(after[1]).saturating_mul(4),
                before[2].abs_diff(after[2]).saturating_mul(4),
                255,
            ]);
        }
        diff_image.save(&diff).expect("write screenshot diff");
        panic!("{component} screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched {component} screenshot: {baseline}");
}

fn run_matrix(
    component: Component,
    component_name: &str,
    manifest_text: &str,
    states: &'static [&'static str],
) -> Result<(), ScreenshotError> {
    let manifest: Value = serde_json::from_str(manifest_text).expect("manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), states.len() * 6, "every state × three themes × two scales");
    let selected_state = std::env::var("UPDATE_SNAPSHOT_STATE").ok();
    let updating =
        std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1"));
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        let state = states
            .iter()
            .copied()
            .find(|state| *state == state_name)
            .unwrap_or_else(|| panic!("unmapped {component_name} state {state_name}"));
        if updating && selected_state.as_deref().is_some_and(|selected| selected != state) {
            continue;
        }
        let theme_name = case["theme"].as_str().expect("theme");
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let baseline = case["baseline"].as_str().expect("baseline");
        let expected_fixture = match (component_name, state) {
            ("badge", "count") => "count_fixture",
            ("badge", "count_overflow") => "overflow_fixture",
            ("badge", "status_success") => "success_fixture",
            ("badge", "status_warning") => "warning_fixture",
            ("badge", "status_danger") => "danger_fixture",
            ("badge", "disabled") => "disabled_fixture",
            ("avatar", "image") => "image_fixture",
            ("avatar", "initials") => "initials_fixture",
            ("avatar", "fallback") => "fallback_fixture",
            ("avatar", "image_alt") => "image_alt_fixture",
            ("key-hint", "single_key") => "single_fixture",
            ("key-hint", "modifier_chord") => "chord_fixture",
            ("key-hint", "named_key") => "named_fixture",
            ("link", "enabled") => "enabled_fixture",
            ("link", "hover") => "hover_fixture",
            ("link", "focused") => "focused_fixture",
            ("link", "visited") => "visited_fixture",
            ("link", "disabled") => "disabled_fixture",
            _ => unreachable!("state is checked against its component list"),
        };
        assert_eq!(case["load_fixture"].as_str(), Some(expected_fixture));
        let actual = capture(component, state, theme(theme_name), scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(component_name, &actual, baseline);
    }
    Ok(())
}

fn run() -> Result<(), ScreenshotError> {
    run_matrix(
        Component::Badge,
        "badge",
        include_str!("../../../registry/badge/tests/conformance.json"),
        &BADGE_STATES,
    )?;
    run_matrix(
        Component::Avatar,
        "avatar",
        include_str!("../../../registry/avatar/tests/conformance.json"),
        &AVATAR_STATES,
    )?;
    run_matrix(
        Component::KeyHint,
        "key-hint",
        include_str!("../../../registry/key-hint/tests/conformance.json"),
        &KEY_HINT_STATES,
    )?;
    run_matrix(
        Component::Link,
        "link",
        include_str!("../../../registry/link/tests/conformance.json"),
        &LINK_STATES,
    )?;
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping E7.15 display primitive screenshots: macOS Metal is required.")
        }
        Err(error) => panic!("E7.15 display primitive matrix failed: {error}"),
    }
}
