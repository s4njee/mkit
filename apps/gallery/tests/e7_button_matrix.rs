use gpui_pre::{App, Context, IntoElement, Render, Window, div, prelude::*, px, size};
use image::RgbaImage;
use mkit::{
    button::{self, Button},
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    icon_button::{self, IconButton},
};
use mkit_harness::{PixelTolerance, ScreenshotError, screenshot};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (760.0, 430.0);

#[derive(Clone, Copy)]
enum Component {
    Button,
    IconButton,
}

#[derive(Clone, Copy)]
enum State {
    Idle,
    Disabled,
    Loading,
}

struct ButtonFixture {
    component: Component,
    state: State,
}

impl Render for ButtonFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let button_variants = [
            ("Default", button::Variant::Default),
            ("Secondary", button::Variant::Secondary),
            ("Outline", button::Variant::Outline),
            ("Ghost", button::Variant::Ghost),
            ("Destructive", button::Variant::Destructive),
            ("Link", button::Variant::Link),
        ];
        let icon_variants = [
            ("Default", icon_button::Variant::Default),
            ("Secondary", icon_button::Variant::Secondary),
            ("Outline", icon_button::Variant::Outline),
            ("Ghost", icon_button::Variant::Ghost),
            ("Destructive", icon_button::Variant::Destructive),
        ];
        let sizes = ["Small", "Default", "Large"];
        let mut content = div().flex().flex_col().gap(px(t.spacing.small));
        let column_header = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.small))
            .child(div().w(px(126.0)).text_color(t.colors.text_muted).child("Variant"))
            .children(sizes.iter().map(|name| {
                div().w(px(174.0)).text_color(t.colors.text_muted).text_center().child(*name)
            }));
        content = content.child(column_header);

        match self.component {
            Component::Button => {
                for (variant_index, (name, variant)) in button_variants.into_iter().enumerate() {
                    let row = div()
                        .flex()
                        .items_center()
                        .gap(px(t.spacing.small))
                        .child(div().w(px(126.0)).text_color(t.colors.text).child(name))
                        .children(sizes.iter().enumerate().map(|(size_index, size_name)| {
                            let size = match *size_name {
                                "Small" => button::Size::Small,
                                "Large" => button::Size::Large,
                                _ => button::Size::Default,
                            };
                            let mut button = Button::new("Continue")
                                .variant(variant)
                                .size(size)
                                .id(variant_index * 3 + size_index + 1);
                            if matches!(self.state, State::Disabled) {
                                button = button.disabled(true);
                            }
                            if matches!(self.state, State::Loading) {
                                button = button.loading(true);
                            }
                            div().w(px(174.0)).flex().justify_center().child(button)
                        }));
                    content = content.child(row);
                }
            }
            Component::IconButton => {
                for (variant_index, (name, variant)) in icon_variants.into_iter().enumerate() {
                    let row = div()
                        .flex()
                        .items_center()
                        .gap(px(t.spacing.small))
                        .child(div().w(px(126.0)).text_color(t.colors.text).child(name))
                        .children(sizes.iter().enumerate().map(|(size_index, size_name)| {
                            let size = match *size_name {
                                "Small" => icon_button::Size::Small,
                                "Large" => icon_button::Size::Large,
                                _ => icon_button::Size::Default,
                            };
                            let mut button = IconButton::new("More actions", div().child("•••"))
                                .variant(variant)
                                .size(size)
                                .id(variant_index * 3 + size_index + 1);
                            if matches!(self.state, State::Disabled) {
                                button = button.disabled(true);
                            }
                            div().w(px(174.0)).flex().justify_center().child(button)
                        }));
                    content = content.child(row);
                }
            }
        }

        let state_label = match self.state {
            State::Idle => "Enabled",
            State::Disabled => "Disabled",
            State::Loading => "Loading",
        };
        div()
            .size_full()
            .bg(t.colors.background)
            .p(px(t.spacing.large))
            .flex()
            .flex_col()
            .gap(px(t.spacing.medium))
            .child(div().text_color(t.colors.text).text_size(px(t.typography.heading_small)).child(
                match self.component {
                    Component::Button => "Button",
                    Component::IconButton => "Icon button",
                },
            ))
            .child(div().text_color(t.colors.text_muted).child(state_label))
            .child(content)
    }
}

fn capture(
    component: Component,
    state: State,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    screenshot(
        ButtonFixture { component, state },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(button::default_key_bindings());
            cx.bind_keys(icon_button::default_key_bindings());
        },
    )
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
        diff_image.save(&diff).expect("write screenshot diff");
        panic!("{component} screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched {component} screenshot: {baseline}");
}

fn run_matrix(component: &str, manifest_text: &str) -> Result<(), ScreenshotError> {
    let manifest: Value = serde_json::from_str(manifest_text).expect("button manifest JSON");
    assert_eq!(manifest["component"], component);
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    for case in cases {
        let state = match case["state"].as_str().expect("state") {
            "idle" => State::Idle,
            "disabled" => State::Disabled,
            "loading" if component == "button" => State::Loading,
            other => panic!("unmapped {component} state: {other}"),
        };
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped screenshot theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(
            if component == "button" { Component::Button } else { Component::IconButton },
            state,
            theme_value,
            scale,
        )?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, component, baseline);
    }
    Ok(())
}

fn run() -> Result<(), ScreenshotError> {
    run_matrix("button", include_str!("../../../registry/button/tests/conformance.json"))?;
    run_matrix("icon-button", include_str!("../../../registry/icon-button/tests/conformance.json"))
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping Button and IconButton screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("Button and IconButton screenshot matrix failed: {error}"),
    }
}
