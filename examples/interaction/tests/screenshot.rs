use gpui_pre::{px, size};
use mkit_example_interaction::InteractionList;
#[cfg(target_os = "macos")]
use mkit_harness::PixelTolerance;
#[cfg(not(target_os = "macos"))]
use mkit_harness::ScreenshotError;
use mkit_harness::screenshot;
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf};

#[cfg(target_os = "macos")]
fn baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../book/src/images/interaction.png")
}

#[cfg(target_os = "macos")]
fn keyboard_baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../book/src/images/interaction-keyboard.png")
}

#[cfg(target_os = "macos")]
fn capture_and_compare() {
    let actual = screenshot(InteractionList::default(), size(px(640.), px(520.)), 2.0, |cx| {
        gpui_kit::init(cx);
        mkit_example_interaction::bind_interaction_keys(cx, "alt-j")
            .expect("default alternate key is valid");
        mkit_example_interaction::set_interaction_menus(cx);
    })
    .expect("macOS Metal headless renderer should be available");
    assert_eq!(actual.dimensions(), (1280, 1040));

    let baseline = baseline_path();
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        actual.save(&baseline).unwrap();
        return;
    }

    let expected = image::open(&baseline)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-example-interaction --test screenshot", baseline.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 8 };
    if !tolerance.matches(&expected, &actual) {
        let diff = baseline.with_file_name("interaction-diff.png");
        write_diff(&expected, &actual, &diff);
        panic!(
            "interaction screenshot differs from {}; diff written to {}",
            baseline.display(),
            diff.display()
        );
    }
}

#[cfg(target_os = "macos")]
fn capture_keyboard_and_compare() {
    let mut session = mkit_harness::HeadlessSession::new(
        InteractionList::default(),
        size(px(640.), px(520.)),
        2.0,
        |cx| {
            gpui_kit::init(cx);
            mkit_example_interaction::bind_interaction_keys(cx, "alt-j")
                .expect("default alternate key is valid");
            mkit_example_interaction::set_interaction_menus(cx);
        },
    )
    .expect("macOS Metal headless renderer should be available");
    session
        .update(|root, window, cx| {
            window.dispatch_keystroke(
                gpui_pre::Keystroke::parse("down").expect("valid Down keystroke"),
                cx,
            );
            assert_eq!(root.read(cx).selected_index(), 1, "Down moves selection before capture");
        })
        .expect("dispatch keyboard state in headless window");
    let actual = session.capture().expect("capture keyboard interaction state");
    assert_eq!(actual.dimensions(), (1280, 1040));

    let baseline = keyboard_baseline_path();
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        actual.save(&baseline).unwrap();
        return;
    }

    let expected = image::open(&baseline)
        .unwrap_or_else(|error| panic!("missing keyboard baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-example-interaction --test screenshot", baseline.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 8 };
    if !tolerance.matches(&expected, &actual) {
        let diff = baseline.with_file_name("interaction-keyboard-diff.png");
        write_diff(&expected, &actual, &diff);
        panic!(
            "interaction keyboard screenshot differs from {}; diff written to {}",
            baseline.display(),
            diff.display()
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn report_unsupported_platform() {
    let error = screenshot(InteractionList::default(), size(px(640.), px(520.)), 1.0, |cx| {
        gpui_kit::init(cx);
    })
    .expect_err("headless screenshots are unavailable off macOS for this GPUI version");
    assert!(matches!(error, ScreenshotError::UnsupportedPlatform));
    eprintln!("Skipping interaction screenshot comparison: {error}");
}

fn main() {
    #[cfg(target_os = "macos")]
    capture_and_compare();
    #[cfg(target_os = "macos")]
    capture_keyboard_and_compare();
    #[cfg(not(target_os = "macos"))]
    report_unsupported_platform();
}

#[cfg(target_os = "macos")]
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
