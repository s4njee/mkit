use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Keystroke, Render, Window, div, prelude::*, px,
    size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    multi_select::{self, MultiSelect, OptionItem as MultiOption},
    select::{self, OptionItem as SingleOption, Select},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (480.0, 300.0);

#[derive(Clone, Copy)]
enum Component {
    Select,
    MultiSelect,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Closed,
    Open,
    Disabled,
    AllOptionsDisabled,
}

struct SelectionFixture {
    component: Component,
    state: State,
    select: Option<Entity<Select>>,
    multi_select: Option<Entity<MultiSelect>>,
}

impl SelectionFixture {
    fn new(component: Component, state: State) -> Self {
        Self { component, state, select: None, multi_select: None }
    }
}

impl Render for SelectionFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let control = match self.component {
            Component::Select => {
                if self.select.is_none() {
                    self.select = Some(cx.new(|_| {
                        Select::new(
                            "Project status",
                            vec![
                                SingleOption::new("planned", "Planned"),
                                SingleOption::new("active", "Active"),
                                SingleOption::new("review", "In review"),
                                SingleOption::new("archived", "Archived").disabled(true),
                            ],
                            Some("active".into()),
                        )
                        .disabled(self.state == State::Disabled)
                    }));
                }
                self.select.as_ref().expect("Select initialized").clone().into_any_element()
            }
            Component::MultiSelect => {
                if self.multi_select.is_none() {
                    let all_disabled = self.state == State::AllOptionsDisabled;
                    let disabled = self.state == State::Disabled;
                    self.multi_select = Some(cx.new(move |_| {
                        MultiSelect::new(
                            "Team members",
                            vec![
                                MultiOption::new("alex", "Alex").disabled(all_disabled),
                                MultiOption::new("blair", "Blair").disabled(all_disabled),
                                MultiOption::new("casey", "Casey").disabled(all_disabled),
                                MultiOption::new("devon", "Devon").disabled(all_disabled),
                            ],
                            if all_disabled {
                                Vec::new()
                            } else {
                                vec!["alex".into(), "casey".into()]
                            },
                        )
                        .disabled(disabled)
                    }));
                }
                self.multi_select
                    .as_ref()
                    .expect("MultiSelect initialized")
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
            .gap(px(theme.spacing.large))
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.heading_small))
                    .child(match self.component {
                        Component::Select => "Select",
                        Component::MultiSelect => "Multi-select",
                    }),
            )
            .child(div().w(px(260.0)).child(control))
    }
}

fn capture(
    component: Component,
    state: State,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        SelectionFixture::new(component, state),
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(select::default_key_bindings());
            cx.bind_keys(multi_select::default_key_bindings());
        },
    )?;
    if state == State::Open {
        session.update(|root, window, cx| {
            let fixture = root.read(cx);
            match component {
                Component::Select => {
                    let control = fixture.select.as_ref().expect("Select initialized").clone();
                    control.update(cx, |select, cx| select.focus_handle(cx).focus(window, cx));
                }
                Component::MultiSelect => {
                    let control =
                        fixture.multi_select.as_ref().expect("MultiSelect initialized").clone();
                    control.update(cx, |select, cx| select.focus_handle(cx).focus(window, cx));
                }
            }
            window.dispatch_keystroke(Keystroke::parse("enter").expect("Enter key"), cx);
            let fixture = root.read(cx);
            let open = match component {
                Component::Select => fixture.select.as_ref().unwrap().read(cx).is_open(),
                Component::MultiSelect => fixture.multi_select.as_ref().unwrap().read(cx).is_open(),
            };
            assert!(open, "the open-state screenshot must render the real popup");
        })?;
    } else if state == State::AllOptionsDisabled {
        session.update(|root, window, cx| {
            let control = root
                .read(cx)
                .multi_select
                .as_ref()
                .expect("all-disabled MultiSelect initialized")
                .clone();
            control.update(cx, |select, cx| select.focus_handle(cx).focus(window, cx));
            window.dispatch_keystroke(Keystroke::parse("down").expect("Down key"), cx);
            assert!(
                !control.read(cx).is_open(),
                "all-disabled fixture must remain closed after navigation"
            );
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, component: &str, baseline: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry")
        .join(component)
        .join("tests/baselines")
        .join(baseline);
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
        diff_image.save(&diff).expect("write diff");
        panic!("{component} screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched {component} screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    for (component, kind, manifest) in [
        (
            "select",
            Component::Select,
            include_str!("../../../registry/select/tests/conformance.json"),
        ),
        (
            "multi-select",
            Component::MultiSelect,
            include_str!("../../../registry/multi-select/tests/conformance.json"),
        ),
    ] {
        let manifest: Value = serde_json::from_str(manifest).expect("selection manifest JSON");
        let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
        assert_eq!(cases.len(), if component == "select" { 18 } else { 24 });
        for case in cases {
            let state = match case["state"].as_str().expect("state") {
                "closed" => State::Closed,
                "open" => State::Open,
                "disabled" => State::Disabled,
                "all_options_disabled" => State::AllOptionsDisabled,
                other => panic!("unmapped selection state: {other}"),
            };
            let theme_value = match case["theme"].as_str().expect("theme") {
                "light" => SHADCN_LIGHT,
                "dark" => SHADCN_DARK,
                "high-contrast" => HIGH_CONTRAST,
                other => panic!("unmapped selection theme: {other}"),
            };
            let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
            let baseline = case["baseline"].as_str().expect("baseline");
            let actual = capture(kind, state, theme_value, scale)?;
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
            println!("Skipping Select/MultiSelect screenshot matrices: macOS Metal is required.");
        }
        Err(error) => panic!("selection screenshot matrix failed: {error}"),
    }
}
