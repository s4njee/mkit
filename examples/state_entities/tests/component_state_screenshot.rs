use gpui_pre::{px, size};
use mkit_example_state_entities::component_state::ComponentStateDemo;
#[cfg(target_os = "macos")]
use mkit_harness::PixelTolerance;
#[cfg(not(target_os = "macos"))]
use mkit_harness::ScreenshotError;
use mkit_harness::screenshot;
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf};

#[cfg(target_os = "macos")]
use mkit_core::theme::{DARK, HIGH_CONTRAST, LIGHT, Theme, set_theme};

#[cfg(target_os = "macos")]
fn baseline_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

#[cfg(target_os = "macos")]
fn capture_and_compare(theme: Theme, scale: f32, relative_path: &str) {
    let actual = screenshot(ComponentStateDemo::default(), size(px(640.), px(400.)), scale, |cx| {
        gpui_kit::init(cx);
        set_theme(cx, theme);
    })
    .expect("macOS Metal headless renderer should be available");
    assert_eq!(actual.dimensions(), ((640.0 * scale) as u32, (400.0 * scale) as u32));

    let baseline = baseline_path(relative_path);
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        actual.save(&baseline).unwrap();
        return;
    }

    let expected = image::open(&baseline)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-example-state-entities --test component_state_screenshot", baseline.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 8 };
    if !tolerance.matches(&expected, &actual) {
        let diff_name =
            baseline.file_name().unwrap().to_string_lossy().replace(".png", "-diff.png");
        let diff = baseline.with_file_name(diff_name);
        write_diff(&expected, &actual, &diff);
        panic!(
            "component state screenshot differs from {}; diff written to {}",
            baseline.display(),
            diff.display()
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn report_unsupported_platform() {
    let error = screenshot(ComponentStateDemo::default(), size(px(640.), px(400.)), 1.0, |cx| {
        gpui_kit::init(cx);
        mkit_core::theme::set_light_theme(cx);
    })
    .expect_err("headless screenshots are unavailable off macOS for this GPUI version");
    assert!(matches!(error, ScreenshotError::UnsupportedPlatform));
    eprintln!("Skipping component state screenshot comparison: {error}");
}

fn main() {
    #[cfg(target_os = "macos")]
    {
        capture_and_compare(LIGHT, 2.0, "../../book/src/images/component-state-demo.png");
        capture_and_compare(DARK, 2.0, "../../book/src/images/component-state-demo-dark.png");
        capture_and_compare(
            HIGH_CONTRAST,
            2.0,
            "../../book/src/images/component-state-demo-high-contrast.png",
        );
        capture_and_compare(LIGHT, 2.0, "../../book/src/images/component-state-demo-scale-2.png");
    }
    #[cfg(not(target_os = "macos"))]
    report_unsupported_platform();
}

#[cfg(target_os = "macos")]
fn write_diff(expected: &image::RgbaImage, actual: &image::RgbaImage, path: &std::path::Path) {
    let (width, height) = actual.dimensions();
    let mut diff = image::RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let current = actual.get_pixel(x, y);
            let expected = expected.get_pixel_checked(x, y).copied().unwrap_or(image::Rgba([0; 4]));
            diff.put_pixel(
                x,
                y,
                image::Rgba([
                    current[0].abs_diff(expected[0]).saturating_mul(4),
                    current[1].abs_diff(expected[1]).saturating_mul(4),
                    current[2].abs_diff(expected[2]).saturating_mul(4),
                    255,
                ]),
            );
        }
    }
    diff.save(path).unwrap();
}
