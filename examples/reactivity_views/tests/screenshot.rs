use gpui_pre::{px, size};
use mkit_example_reactivity_views::ReactivityDemo;
#[cfg(target_os = "macos")]
use mkit_harness::PixelTolerance;
#[cfg(not(target_os = "macos"))]
use mkit_harness::ScreenshotError;
use mkit_harness::screenshot;
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf};

#[cfg(target_os = "macos")]
fn baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../book/src/images/reactivity-views.png")
}

#[cfg(target_os = "macos")]
fn capture_and_compare() {
    let actual =
        screenshot(ReactivityDemo::default(), size(px(640.), px(400.)), 2.0, gpui_kit::init)
            .expect("macOS Metal headless renderer should be available");
    assert_eq!(actual.dimensions(), (1280, 800));

    let baseline = baseline_path();
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        actual.save(&baseline).unwrap();
        return;
    }

    let expected = image::open(&baseline)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-example-reactivity-views --test screenshot", baseline.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 8 };
    if !tolerance.matches(&expected, &actual) {
        let diff = baseline.with_file_name("reactivity-views-diff.png");
        write_diff(&expected, &actual, &diff);
        panic!(
            "reactivity screenshot differs from {}; diff written to {}",
            baseline.display(),
            diff.display()
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn report_unsupported_platform() {
    let error =
        screenshot(ReactivityDemo::default(), size(px(640.), px(400.)), 1.0, gpui_kit::init)
            .expect_err("headless screenshots are unavailable off macOS for this GPUI version");
    assert!(matches!(error, ScreenshotError::UnsupportedPlatform));
    eprintln!("Skipping reactivity screenshot comparison: {error}");
}

fn main() {
    #[cfg(target_os = "macos")]
    capture_and_compare();
    #[cfg(not(target_os = "macos"))]
    report_unsupported_platform();
}

#[cfg(target_os = "macos")]
fn write_diff(expected: &image::RgbaImage, actual: &image::RgbaImage, path: &std::path::Path) {
    let (width, height) = actual.dimensions();
    let mut diff = image::RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let actual_pixel = actual.get_pixel(x, y);
            let expected_pixel =
                expected.get_pixel_checked(x, y).copied().unwrap_or(image::Rgba([0; 4]));
            diff.put_pixel(
                x,
                y,
                image::Rgba([
                    actual_pixel[0].abs_diff(expected_pixel[0]).saturating_mul(4),
                    actual_pixel[1].abs_diff(expected_pixel[1]).saturating_mul(4),
                    actual_pixel[2].abs_diff(expected_pixel[2]).saturating_mul(4),
                    255,
                ]),
            );
        }
    }
    diff.save(path).unwrap();
}
