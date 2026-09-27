use gpui_pre::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb, size};
#[cfg(not(target_os = "macos"))]
use mkit_harness::ScreenshotError;
use mkit_harness::{PixelTolerance, screenshot};
use std::{fs, path::PathBuf};

struct Sample;

impl Render for Sample {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x223344))
            .child(
                div()
                    .h(px(30.))
                    .text_color(rgb(0xffffff))
                    .text_size(px(24.))
                    .child("GPUI text probe"),
            )
            .child(div().w(px(80.)).h(px(16.)).bg(rgb(0x66ccaa)))
    }
}

#[cfg(target_os = "macos")]
fn committed_baseline_matches_headless_capture() {
    let image = screenshot(Sample, size(px(180.), px(60.)), 1.0, gpui_kit::base::init)
        .expect("macOS Metal headless renderer should be available");
    assert_eq!(image.dimensions(), (180, 60));
    let repeated = screenshot(Sample, size(px(180.), px(60.)), 1.0, gpui_kit::base::init)
        .expect("second macOS Metal capture should succeed");
    assert_eq!(image, repeated, "same-process repeated renders should be pixel-identical");
    if let Some(path) = std::env::var_os("MKIT_SCREENSHOT_PROBE_OUTPUT") {
        image.save(path).expect("write screenshot stability probe");
    }
    assert_visible_pixels(&image);

    let double_scale = screenshot(Sample, size(px(180.), px(60.)), 2.0, gpui_kit::base::init)
        .expect("2x macOS Metal capture should succeed");
    assert_eq!(double_scale.dimensions(), (360, 120));
    assert_visible_pixels(&double_scale);

    let baseline = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("snapshots/sample.png");
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        image.save(&baseline).unwrap();
        return;
    }

    let expected = image::open(&baseline)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-harness --test screenshot", baseline.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 8 };
    if !tolerance.matches(&expected, &image) {
        let diff_path = baseline.with_file_name("sample-diff.png");
        write_diff(&expected, &image, &diff_path);
        panic!(
            "screenshot differs from {}; diff written to {}. Refresh with UPDATE_SNAPSHOTS=1 cargo test -p mkit-harness --test screenshot",
            baseline.display(),
            diff_path.display()
        );
    }
}

fn assert_visible_pixels(image: &image::RgbaImage) {
    assert!(
        image.pixels().filter(|pixel| *pixel == &image::Rgba([0x66, 0xcc, 0xaa, 0xff])).count() > 8,
        "the contrasting child view should be visible"
    );
    let text_pixels = image
        .enumerate_pixels()
        .filter(|(x, y, pixel)| {
            *x > 80 && *y < 28 && pixel[0] > 180 && pixel[1] > 180 && pixel[2] > 180
        })
        .count();
    assert!(text_pixels > 8, "white text pixels found: {text_pixels}");
}

fn perceptual_tolerance_ignores_rounding_but_rejects_visible_changes() {
    let expected = image::RgbaImage::from_pixel(10, 10, image::Rgba([20, 30, 40, 255]));
    let mut rounded = expected.clone();
    rounded.put_pixel(0, 0, image::Rgba([22, 29, 41, 255]));
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 1 };
    assert!(tolerance.matches(&expected, &rounded));

    let mut visibly_changed = expected.clone();
    for y in 0..3 {
        for x in 0..3 {
            visibly_changed.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
        }
    }
    assert!(!tolerance.matches(&expected, &visibly_changed));
}

#[cfg(not(target_os = "macos"))]
fn screenshots_report_unsupported_platform() {
    let error = screenshot(Sample, size(px(180.), px(60.)), 1.0, |_| {})
        .expect_err("this GPUI pin has no renderer off macOS");
    assert!(matches!(error, ScreenshotError::UnsupportedPlatform));
}

fn main() {
    perceptual_tolerance_ignores_rounding_but_rejects_visible_changes();
    #[cfg(target_os = "macos")]
    committed_baseline_matches_headless_capture();
    #[cfg(not(target_os = "macos"))]
    screenshots_report_unsupported_platform();
}

fn write_diff(expected: &image::RgbaImage, actual: &image::RgbaImage, path: &std::path::Path) {
    let (width, height) = actual.dimensions();
    let mut diff = image::RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let a = actual.get_pixel(x, y);
            let b = expected.get_pixel_checked(x, y).copied().unwrap_or(image::Rgba([0; 4]));
            diff.put_pixel(
                x,
                y,
                image::Rgba([
                    a[0].abs_diff(b[0]).saturating_mul(4),
                    a[1].abs_diff(b[1]).saturating_mul(4),
                    a[2].abs_diff(b[2]).saturating_mul(4),
                    255,
                ]),
            );
        }
    }
    diff.save(path).unwrap();
}
