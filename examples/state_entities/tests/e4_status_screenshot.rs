use gpui_pre::{px, size};
use mkit_example_state_entities::e4_status::StatusAnnouncer;
#[cfg(target_os = "macos")]
use mkit_harness::{HeadlessSession, PixelTolerance};
#[cfg(not(target_os = "macos"))]
use mkit_harness::{ScreenshotError, screenshot};
#[cfg(target_os = "macos")]
use std::{fs, path::Path};

const WINDOW_SIZE: (u32, u32) = (640, 400);
const SCALE: f32 = 2.0;

fn initialize_theme(cx: &mut gpui_pre::App) {
    mkit_core::theme::set_light_theme(cx);
}

#[cfg(target_os = "macos")]
fn capture_states() -> (image::RgbaImage, image::RgbaImage) {
    let mut session = HeadlessSession::new(
        StatusAnnouncer::new("Waiting for upload"),
        size(px(WINDOW_SIZE.0 as f32), px(WINDOW_SIZE.1 as f32)),
        SCALE,
        initialize_theme,
    )
    .expect("macOS Metal headless renderer should be available");
    let initial = session.capture().expect("initial status frame should render");

    session
        .update(|root, _window, cx| {
            root.update(cx, |status, cx| {
                assert!(status.update_status("Upload complete", cx));
            });
        })
        .expect("status update should redraw the same headless window");
    let updated = session.capture().expect("updated status frame should render");
    (initial, updated)
}

#[cfg(target_os = "macos")]
fn save_or_compare(baseline: &Path, actual: &image::RgbaImage) {
    assert_eq!(
        actual.dimensions(),
        ((WINDOW_SIZE.0 as f32 * SCALE) as u32, (WINDOW_SIZE.1 as f32 * SCALE) as u32)
    );
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        actual.save(&baseline).unwrap();
        return;
    }

    let expected = image::open(baseline)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-example-state-entities --test e4_status_screenshot", baseline.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 8 };
    if !tolerance.matches(&expected, actual) {
        let diff = baseline.with_file_name(format!(
            "{}-diff.png",
            baseline.file_stem().unwrap().to_string_lossy()
        ));
        write_diff(&expected, actual, &diff);
        panic!(
            "status screenshot differs from {}; diff written to {}",
            baseline.display(),
            diff.display()
        );
    }
}

#[cfg(target_os = "macos")]
fn capture_and_compare() {
    let (initial, updated) = capture_states();
    let images = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../book/src/images");
    let initial_baseline = images.join("e4-status-initial.png");
    let updated_baseline = images.join("e4-status-updated.png");
    save_or_compare(&initial_baseline, &initial);
    save_or_compare(&updated_baseline, &updated);
    assert!(!PixelTolerance::default().matches(&initial, &updated));
}

#[cfg(not(target_os = "macos"))]
fn report_unsupported_platform() {
    let error = screenshot(
        StatusAnnouncer::new("Waiting for upload"),
        size(px(WINDOW_SIZE.0 as f32), px(WINDOW_SIZE.1 as f32)),
        SCALE,
        initialize_theme,
    )
    .expect_err("headless screenshots are unavailable off macOS for this GPUI version");
    assert!(matches!(error, ScreenshotError::UnsupportedPlatform));
    eprintln!("Skipping E4.5 status screenshot comparison: {error}");
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
