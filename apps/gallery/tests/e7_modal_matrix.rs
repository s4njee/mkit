use gpui_pre::{
    App, Context, Entity, FocusHandle, IntoElement, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    dialog::Dialog,
    sheet::Sheet,
};
use mkit_harness::{PixelTolerance, ScreenshotError, screenshot};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (520.0, 360.0);

#[derive(Clone, Copy)]
enum State {
    Closed,
    Open,
    Busy,
    Interactive,
}

#[derive(Clone, Copy)]
enum Component {
    Dialog,
    Sheet,
}

struct ModalFixture {
    component: Component,
    state: State,
    dialog: Option<Entity<Dialog>>,
    sheet: Option<Entity<Sheet>>,
    first_control: Option<FocusHandle>,
    second_control: Option<FocusHandle>,
}

impl Render for ModalFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let interactive = matches!(self.state, State::Interactive);
        if interactive && self.first_control.is_none() {
            self.first_control = Some(cx.focus_handle().tab_stop(true));
            self.second_control = Some(cx.focus_handle().tab_stop(true));
        }
        match self.component {
            Component::Dialog if self.dialog.is_none() => {
                let state = self.state;
                let first = self.first_control.clone();
                let second = self.second_control.clone();
                self.dialog = Some(cx.new(move |_| {
                    match state {
                        State::Interactive => {
                            let first_for_content =
                                first.clone().expect("first interactive control");
                            let second_for_content =
                                second.clone().expect("second interactive control");
                            Dialog::with_content("Project settings", move || {
                                interactive_content(
                                    first_for_content.clone(),
                                    second_for_content.clone(),
                                    theme,
                                )
                            })
                            .focus_stops(vec![
                                first.expect("first focus stop"),
                                second.expect("second focus stop"),
                            ])
                        }
                        _ => Dialog::controlled(
                            "Project settings",
                            "Review the project settings before continuing.",
                            !matches!(state, State::Closed),
                        )
                        .busy(matches!(state, State::Busy)),
                    }
                }));
            }
            Component::Sheet if self.sheet.is_none() => {
                let state = self.state;
                let first = self.first_control.clone();
                let second = self.second_control.clone();
                self.sheet = Some(cx.new(move |_| {
                    match state {
                        State::Interactive => {
                            let first_for_content =
                                first.clone().expect("first interactive control");
                            let second_for_content =
                                second.clone().expect("second interactive control");
                            Sheet::with_content("Project settings", move || {
                                interactive_content(
                                    first_for_content.clone(),
                                    second_for_content.clone(),
                                    theme,
                                )
                            })
                            .focus_stops(vec![
                                first.expect("first focus stop"),
                                second.expect("second focus stop"),
                            ])
                        }
                        _ => Sheet::controlled(
                            "Project settings",
                            "Review the project settings before continuing.",
                            !matches!(state, State::Closed),
                        )
                        .busy(matches!(state, State::Busy)),
                    }
                }));
            }
            _ => {}
        }

        let mut page = div()
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
                    .child("Workspace settings"),
            )
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.body))
                    .child("Manage the project details and access."),
            )
            .child(
                div()
                    .p(px(theme.spacing.medium))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.elevated_surface)
                    .text_color(theme.colors.text)
                    .child("Project visibility · Team members · Notifications"),
            );
        page = match self.component {
            Component::Dialog => {
                page.child(self.dialog.as_ref().expect("dialog initialized").clone())
            }
            Component::Sheet => page.child(self.sheet.as_ref().expect("sheet initialized").clone()),
        };
        page
    }
}

fn interactive_content(first: FocusHandle, second: FocusHandle, theme: Theme) -> impl IntoElement {
    let control = |handle: &FocusHandle, label: &'static str| {
        div()
            .track_focus(handle)
            .tab_stop(true)
            .p(px(theme.spacing.medium))
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.regular))
            .border_color(theme.colors.border)
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .child(label)
    };
    div()
        .flex()
        .flex_col()
        .gap(px(theme.spacing.medium))
        .child(control(&first, "Workspace name · Design system"))
        .child(control(&second, "Access · Team only"))
}

fn capture(
    component: Component,
    state: State,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    screenshot(
        ModalFixture {
            component,
            state,
            dialog: None,
            sheet: None,
            first_control: None,
            second_control: None,
        },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
        },
    )
}

fn compare_or_update(actual: &RgbaImage, component: Component, baseline: &str) {
    let root = match component {
        Component::Dialog => "../../registry/dialog/tests/baselines",
        Component::Sheet => "../../registry/sheet/tests/baselines",
    };
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(root).join(baseline);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated modal screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing modal baseline {}: {error}", path.display()))
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
        diff_image.save(&diff).expect("write modal diff");
        panic!("modal screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched modal screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let selected_state = std::env::var("E7_MODAL_STATE").ok();
    for (component, manifest_text, expected_cases) in [
        (Component::Dialog, include_str!("../../../registry/dialog/tests/conformance.json"), 24),
        (Component::Sheet, include_str!("../../../registry/sheet/tests/conformance.json"), 24),
    ] {
        let manifest: Value = serde_json::from_str(manifest_text).expect("modal manifest JSON");
        let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
        assert_eq!(cases.len(), expected_cases, "four states × three themes × two scales");
        for case in cases {
            if selected_state
                .as_deref()
                .is_some_and(|selected| selected != case["state"].as_str().expect("state"))
            {
                continue;
            }
            let state = match case["state"].as_str().expect("state") {
                "closed" => State::Closed,
                "open" => State::Open,
                "busy" => State::Busy,
                "interactive" => State::Interactive,
                other => panic!("unmapped modal state: {other}"),
            };
            let theme_value = match case["theme"].as_str().expect("theme") {
                "light" => SHADCN_LIGHT,
                "dark" => SHADCN_DARK,
                "high-contrast" => HIGH_CONTRAST,
                other => panic!("unmapped modal theme: {other}"),
            };
            let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
            let baseline = case["baseline"].as_str().expect("baseline");
            let actual = capture(component, state, theme_value, scale)?;
            assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
            compare_or_update(&actual, component, baseline);
        }
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping Dialog and Sheet screenshots: macOS Metal is required.");
        }
        Err(error) => panic!("Dialog and Sheet screenshot matrix failed: {error}"),
    }
}
