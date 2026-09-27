use gpui_pre::{px, size};
use mkit_example_settings_screen::SettingsScreen;
#[cfg(not(target_os = "macos"))]
use mkit_harness::ScreenshotError;
#[cfg(not(target_os = "macos"))]
use mkit_harness::screenshot;
#[cfg(target_os = "macos")]
use mkit_harness::{HeadlessSession, PixelTolerance};
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf};

#[cfg(target_os = "macos")]
fn baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../book/src/images/settings-screen.png")
}

#[cfg(target_os = "macos")]
fn capture_and_compare() {
    let mut session = HeadlessSession::new(
        SettingsScreen::default(),
        size(px(960.), px(640.)),
        2.0,
        gpui_kit::init,
    )
    .expect("macOS Metal headless renderer should be available");
    let actual = session.capture().unwrap();
    compare_or_update(actual, baseline_path());

    session
        .update(|root, _, cx| {
            root.update(cx, |screen, _| screen.scroll_to_bottom());
        })
        .expect("settings view should scroll to the bottom");
    let scrolled = session.capture().unwrap();
    compare_or_update(scrolled, scrolled_baseline_path());
}

#[cfg(target_os = "macos")]
fn scrolled_baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../book/src/images/settings-screen-scrolled.png")
}

#[cfg(target_os = "macos")]
fn compare_or_update(actual: image::RgbaImage, baseline: PathBuf) {
    assert_eq!(actual.dimensions(), (1920, 1280));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        actual.save(&baseline).unwrap();
        return;
    }
    let expected = image::open(&baseline)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-example-settings-screen --test screenshot", baseline.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 8 };
    if !tolerance.matches(&expected, &actual) {
        let diff = baseline.with_file_name(format!(
            "{}-diff.png",
            baseline.file_stem().unwrap().to_string_lossy()
        ));
        write_diff(&expected, &actual, &diff);
        panic!(
            "settings-screen screenshot differs from {}; diff written to {}",
            baseline.display(),
            diff.display()
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn report_unsupported_platform() {
    let error =
        screenshot(SettingsScreen::default(), size(px(960.), px(640.)), 1.0, gpui_kit::init)
            .expect_err("headless screenshots are unavailable off macOS for this GPUI version");
    assert!(matches!(error, ScreenshotError::UnsupportedPlatform));
    eprintln!("Skipping settings-screen screenshot comparison: {error}");
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
