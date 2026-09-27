use gpui_pre::{px, size};
use mkit_example_interaction::e4_overlay::OverlayDismissalFixture;
#[cfg(target_os = "macos")]
use mkit_harness::PixelTolerance;
#[cfg(not(target_os = "macos"))]
use mkit_harness::ScreenshotError;
use mkit_harness::screenshot;
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf};

#[cfg(target_os = "macos")]
fn baseline_path() -> PathBuf {
    let relative = if std::env::var_os("MKIT_E5_ADAPTER").is_some() {
        "../../book/src/images/overlay-dismissal-smoke/nested_open/light-1x.png"
    } else {
        "../../book/src/images/e4-overlay-nested.png"
    };
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[cfg(target_os = "macos")]
fn capture(theme: mkit_core::theme::Theme, scale: f32) -> image::RgbaImage {
    screenshot(
        OverlayDismissalFixture::nested_open(),
        size(px(640.0), px(480.0)),
        scale,
        move |cx| {
            mkit_core::theme::set_theme(cx, theme);
            mkit_example_interaction::e4_overlay::bind_overlay_dismiss_key(cx, "escape")
                .expect("Escape is a valid dismissal binding");
        },
    )
    .expect("macOS Metal headless renderer should be available")
}

#[cfg(target_os = "macos")]
fn capture_and_compare() {
    use mkit_core::theme::{DARK, HIGH_CONTRAST, LIGHT};

    let actual = capture(LIGHT, 2.0);
    assert_eq!(actual.dimensions(), (1280, 960));

    let baseline = baseline_path();
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        actual.save(&baseline).unwrap();
    } else {
        let expected = image::open(&baseline)
            .unwrap_or_else(|error| panic!("missing baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-example-interaction --test e4_overlay_screenshot", baseline.display()))
            .to_rgba8();
        let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 8 };
        if !tolerance.matches(&expected, &actual) {
            let diff = baseline.with_file_name("e4-overlay-nested-diff.png");
            write_diff(&expected, &actual, &diff);
            panic!(
                "E4.3 overlay screenshot differs from {}; diff written to {}",
                baseline.display(),
                diff.display()
            );
        }
    }

    let variants = [
        ("dark", DARK, 2.0, (1280, 960)),
        ("high-contrast", HIGH_CONTRAST, 2.0, (1280, 960)),
        ("light-scale-2", LIGHT, 2.0, (1280, 960)),
    ];
    for (name, theme, scale, expected_size) in variants {
        let image = capture(theme, scale);
        assert_eq!(image.dimensions(), expected_size, "{name} capture dimensions");
        if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
            let path = std::env::temp_dir().join(format!("mkit-e4-overlay-{name}.png"));
            image.save(&path).unwrap();
            eprintln!("Captured {name} overlay preview at {}", path.display());
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn report_unsupported_platform() {
    let error =
        screenshot(OverlayDismissalFixture::nested_open(), size(px(640.0), px(480.0)), 1.0, |cx| {
            mkit_core::theme::set_light_theme(cx);
            mkit_example_interaction::e4_overlay::bind_overlay_dismiss_key(cx, "escape")
                .expect("Escape is a valid dismissal binding");
        })
        .expect_err("headless screenshots are unavailable off macOS for this GPUI version");
    assert!(matches!(error, ScreenshotError::UnsupportedPlatform));
    eprintln!("Skipping E4.3 overlay screenshot comparison: {error}");
}

fn main() {
    #[cfg(target_os = "macos")]
    {
        capture_and_compare();
        if std::env::var_os("MKIT_E5_ADAPTER").is_some() {
            println!(
                "MKIT_E5_RESULT={{\"passed\":true,\"actual\":{{\"baseline\":\"overlay-dismissal-smoke/nested_open/light-1x.png\",\"matched\":true}}}}"
            );
        }
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
            let actual = actual.get_pixel(x, y);
            let expected = expected.get_pixel_checked(x, y).copied().unwrap_or(image::Rgba([0; 4]));
            diff.put_pixel(
                x,
                y,
                image::Rgba([
                    actual[0].abs_diff(expected[0]).saturating_mul(4),
                    actual[1].abs_diff(expected[1]).saturating_mul(4),
                    actual[2].abs_diff(expected[2]).saturating_mul(4),
                    255,
                ]),
            );
        }
    }
    diff.save(path).unwrap();
}
