use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    search_field::{self, SearchField},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (500.0, 130.0);

struct SearchFieldFixture {
    state: &'static str,
    search: Option<Entity<SearchField>>,
}

impl Render for SearchFieldFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let search = self.search.get_or_insert_with(|| {
            cx.new(|_| match state {
                "empty" => SearchField::new("Search files").placeholder("Search files…"),
                "query" => SearchField::new("Search files")
                    .placeholder("Search files…")
                    .default_query("calendar"),
                "results" => SearchField::new("Search files")
                    .placeholder("Search files…")
                    .default_query("reports")
                    .result_count(Some(12)),
                "disabled" => SearchField::new("Search files")
                    .placeholder("Search files…")
                    .default_query("archived")
                    .disabled(true),
                "focused" => SearchField::new("Search files").placeholder("Search files…"),
                other => panic!("unmapped SearchField state: {other}"),
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
                    .child("SearchField"),
            )
            .child(search.clone())
    }
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        SearchFieldFixture { state, search: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            theme::set_theme(cx, theme_value);
            cx.bind_keys(search_field::default_key_bindings());
        },
    )?;
    if state == "focused" {
        session.update(|root, window, cx| {
            let search = root.read(cx).search.as_ref().expect("SearchField initialized").clone();
            let focus = search.read(cx).focus_handle(cx);
            focus.focus(window, cx);
            assert!(focus.is_focused(window), "the focused screenshot must focus the input");
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let relative = baseline.strip_prefix("search-field/").expect("baseline component prefix");
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/search-field/tests/baselines")
        .join(relative);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent"))
            .expect("create baseline directory");
        actual.save(&path).expect("write baseline");
        println!("Updated SearchField screenshot: {}", path.display());
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
        serde_json::from_str(include_str!("../../../registry/search-field/tests/conformance.json"))
            .expect("SearchField manifest");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    let states = ["empty", "query", "results", "disabled", "focused"];
    assert_eq!(cases.len(), states.len() * 3 * 2, "state/theme/scale matrix is complete");
    for case in cases {
        let state = case["state"].as_str().expect("state");
        assert!(states.contains(&state), "unexpected SearchField state: {state}");
        let state_id = match state {
            "empty" => "empty",
            "query" => "query",
            "results" => "results",
            "disabled" => "disabled",
            "focused" => "focused",
            other => panic!("unmapped state: {other}"),
        };
        let theme_value = match case["theme"].as_str().expect("theme") {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let actual = capture(state_id, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, case["baseline"].as_str().expect("baseline"));
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => println!("SearchField screenshot matrix passed."),
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping SearchField screenshots: macOS Metal is required.")
        }
        Err(error) => panic!("SearchField screenshot matrix failed: {error}"),
    }
}
