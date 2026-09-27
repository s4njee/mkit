use gpui_pre::{px, size};
use mkit_example_platform_shipping::ShippingDemo;
#[cfg(target_os = "macos")]
use mkit_harness::{HeadlessSession, PixelTolerance};
#[cfg(not(target_os = "macos"))]
use mkit_harness::{ScreenshotError, screenshot};
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf};

#[cfg(target_os = "macos")]
fn compare(actual: image::RgbaImage) {
    let baseline = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../book/src/images/platform-shipping.png");
    assert_eq!(actual.dimensions(), (1520, 1080));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        actual.save(&baseline).unwrap();
        return;
    }
    let expected = image::open(&baseline).unwrap().to_rgba8();
    if !(PixelTolerance { channel_delta: 2, max_different_pixels: 8 }).matches(&expected, &actual) {
        let diff = baseline.with_file_name("platform-shipping-diff.png");
        write_diff(&expected, &actual, &diff);
        panic!("platform-shipping screenshot differs; diff written to {}", diff.display());
    }
}

#[cfg(target_os = "macos")]
fn capture() {
    let mut session =
        HeadlessSession::new(ShippingDemo::default(), size(px(760.), px(540.)), 2., gpui_kit::init)
            .expect("macOS Metal headless renderer should be available");
    compare(session.capture().expect("capture the platform example"));
}

#[cfg(not(target_os = "macos"))]
fn report_unsupported_platform() {
    let error = screenshot(ShippingDemo::default(), size(px(760.), px(540.)), 1., gpui_kit::init)
        .expect_err("this GPUI version only supports headless screenshots on macOS");
    assert!(matches!(error, ScreenshotError::UnsupportedPlatform));
    eprintln!("Skipping platform-shipping screenshot comparison: {error}");
}

fn main() {
    #[cfg(target_os = "macos")]
    capture();
    #[cfg(not(target_os = "macos"))]
    report_unsupported_platform();
}

#[cfg(target_os = "macos")]
fn write_diff(expected: &image::RgbaImage, actual: &image::RgbaImage, path: &std::path::Path) {
    let mut diff = image::RgbaImage::new(actual.width(), actual.height());
    for y in 0..actual.height() {
        for x in 0..actual.width() {
            let left = expected.get_pixel(x, y);
            let right = actual.get_pixel(x, y);
            diff.put_pixel(
                x,
                y,
                image::Rgba([
                    left[0].abs_diff(right[0]).saturating_mul(4),
                    left[1].abs_diff(right[1]).saturating_mul(4),
                    left[2].abs_diff(right[2]).saturating_mul(4),
                    255,
                ]),
            );
        }
    }
    diff.save(path).unwrap();
}
