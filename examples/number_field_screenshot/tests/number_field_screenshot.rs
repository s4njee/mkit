use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
    size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, SHADCN_LIGHT, Theme},
    scrubbable_number_field::ScrubbableNumberField,
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

const CASE_ID: &str = "screen-idle-light-1x";
const BASELINE: &str = "../../registry/scrubbable-number-field/tests/baselines/scrubbable-number-field/idle/light-1x.png";
const SIZE: (f32, f32) = (420.0, 160.0);

struct NumberFieldScreenshot {
    field: Option<Entity<ScrubbableNumberField>>,
}

impl Render for NumberFieldScreenshot {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.field.is_none() {
            self.field = Some(cx.new(|_| {
                ScrubbableNumberField::new(12.5)
                    .bounds(Some(0.0), Some(100.0))
                    .step(1.0)
                    .label("Opacity")
                    .unit("%")
            }));
        }
        let colors = cx.global::<Theme>().colors;
        let root = div().size_full().flex().items_center().justify_center().bg(colors.background);
        root.child(div().w(px(220.0)).child(self.field.as_ref().expect("initialized").clone()))
    }
}

fn initialize(cx: &mut App) {
    gpui_kit::base::init(cx);
    theme::set_theme(cx, SHADCN_LIGHT);
}

fn baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(BASELINE)
}

fn capture() -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        NumberFieldScreenshot { field: None },
        size(px(SIZE.0), px(SIZE.1)),
        1.0,
        initialize,
    )?;
    session.capture()
}

fn reviewability_issue(image: &RgbaImage) -> Option<&'static str> {
    if image.dimensions() != (SIZE.0 as u32, SIZE.1 as u32) {
        return Some("captured screenshot dimensions are unexpected");
    }
    let background = [255_u8, 255, 255];
    let border = [232_u8, 232, 232];
    let mut background_pixels = 0;
    let mut border_pixels = 0;
    let mut ink_pixels = 0;
    for pixel in image.pixels() {
        let [red, green, blue, alpha] = pixel.0;
        if alpha > 240
            && red.abs_diff(background[0]) < 3
            && green.abs_diff(background[1]) < 3
            && blue.abs_diff(background[2]) < 3
        {
            background_pixels += 1;
        }
        if alpha > 240
            && red.abs_diff(border[0]) < 10
            && green.abs_diff(border[1]) < 10
            && blue.abs_diff(border[2]) < 10
        {
            border_pixels += 1;
        }
        if alpha > 240 && red < 100 && green < 100 && blue < 110 {
            ink_pixels += 1;
        }
    }
    if background_pixels <= 10_000 {
        Some("the themed window background was not captured")
    } else if border_pixels <= 20 {
        Some("the number field border was not visible in the captured pixels")
    } else if ink_pixels <= 20 {
        Some("captured number field has no readable value glyphs")
    } else {
        None
    }
}

fn compare_or_update(actual: RgbaImage) -> Value {
    if let Some(reason) = reviewability_issue(&actual) {
        return json!({"status":"unsupported","reason":reason});
    }
    let baseline = baseline_path();
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().expect("baseline parent")).unwrap();
        actual.save(&baseline).unwrap();
        return json!({"baseline":"scrubbable-number-field/idle/light-1x.png","updated":true});
    }

    let expected = image::open(&baseline)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-example-number-field-screenshot --test number_field_screenshot", baseline.display()))
        .to_rgba8();
    if let Some(reason) = reviewability_issue(&expected) {
        panic!("committed baseline is not reviewable: {reason}");
    }
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 16 };
    if !tolerance.matches(&expected, &actual) {
        let diff_path = baseline.with_file_name("light-1x-diff.png");
        write_diff(&expected, &actual, &diff_path);
        panic!("number-field screenshot differs; pixel diff written to {}", diff_path.display());
    }
    json!({"baseline":"scrubbable-number-field/idle/light-1x.png","matched":true})
}

fn write_diff(expected: &RgbaImage, actual: &RgbaImage, path: &std::path::Path) {
    let mut diff = RgbaImage::new(actual.width(), actual.height());
    for (x, y, pixel) in diff.enumerate_pixels_mut() {
        let left = expected.get_pixel(x, y);
        let right = actual.get_pixel(x, y);
        *pixel = image::Rgba([
            left[0].abs_diff(right[0]).saturating_mul(4),
            left[1].abs_diff(right[1]).saturating_mul(4),
            left[2].abs_diff(right[2]).saturating_mul(4),
            255,
        ]);
    }
    diff.save(path).unwrap();
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case JSON");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"screenshot_cases","id":CASE_ID})
    } else {
        serde_json::from_str(&input).expect("parse generated case JSON")
    };
    assert_eq!(case.get("kind").and_then(Value::as_str), Some("screenshot_cases"));
    assert_eq!(case.get("id").and_then(Value::as_str), Some(CASE_ID));

    match capture() {
        Ok(image) => {
            if let Some(path) = std::env::var_os("MKIT_DUMP_SCREENSHOT") {
                image.save(path).expect("write diagnostic screenshot");
            }
            let report = compare_or_update(image);
            if report.get("status").is_some() {
                println!("{report}");
            } else {
                println!("{}", json!({"passed":true,"actual":report}));
            }
        }
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!(
                "{}",
                json!({"status":"unsupported","reason":"GPUI headless screenshots require the macOS Metal renderer"})
            );
        }
        Err(error) => panic!("failed to capture number-field screenshot: {error}"),
    }
}
