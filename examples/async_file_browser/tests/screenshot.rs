use gpui_pre::{px, size};
#[cfg(not(target_os = "macos"))]
use mkit_example_async_file_browser::{BrowserEntry, EntryKind, sample_directory};
use mkit_example_async_file_browser::{FileBrowser, read_directory};
#[cfg(not(target_os = "macos"))]
use mkit_harness::ScreenshotError;
#[cfg(not(target_os = "macos"))]
use mkit_harness::screenshot;
#[cfg(target_os = "macos")]
use mkit_harness::{HeadlessSession, PixelTolerance};
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf};

#[cfg(target_os = "macos")]
fn baseline_path(state: &str) -> PathBuf {
    let image = match state {
        "loading" => "../../book/src/images/async-file-browser-loading.png",
        "loaded" => "../../book/src/images/async-file-browser-loaded.png",
        "error" => "../../book/src/images/async-file-browser-error.png",
        _ => unreachable!("only fixed screenshot states are supported"),
    };
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(image)
}

#[cfg(target_os = "macos")]
fn capture_and_compare() {
    let fixture = tempfile::tempdir().unwrap();
    fs::create_dir_all(fixture.path().join("Reference")).unwrap();
    fs::write(fixture.path().join("Getting-started.md"), "safe screenshot fixture").unwrap();
    fs::write(fixture.path().join("notes.txt"), "safe screenshot fixture").unwrap();
    fs::write(fixture.path().join("Reference/README.md"), "safe screenshot fixture").unwrap();
    let entries = read_directory(fixture.path(), false).expect("fixture directory is readable");
    assert_eq!(
        entries.iter().map(|entry| entry.name.as_str()).collect::<Vec<_>>(),
        ["Getting-started.md", "notes.txt", "Reference"]
    );

    let mut session = HeadlessSession::new(
        FileBrowser::new(fixture.path().to_path_buf()),
        size(px(900.), px(600.)),
        2.0,
        gpui_kit::init,
    )
    .expect("macOS Metal headless renderer should be available");
    compare_or_update(session.capture().unwrap(), baseline_path("loading"));

    session
        .update(|root, _, cx| {
            root.update(cx, |browser, _| {
                *browser = FileBrowser::preview(fixture.path().to_path_buf(), Ok(entries.clone()));
            });
        })
        .expect("install loaded preview state");
    compare_or_update(session.capture().unwrap(), baseline_path("loaded"));

    let missing_path = fixture.path().join("missing");
    let missing_error = read_directory(&missing_path, false).unwrap_err();
    session
        .update(move |root, _, cx| {
            root.update(cx, |browser, _| {
                *browser = FileBrowser::preview(fixture.path().to_path_buf(), Err(missing_error));
            });
        })
        .expect("install error preview state");
    compare_or_update(session.capture().unwrap(), baseline_path("error"));
}

#[cfg(target_os = "macos")]
fn compare_or_update(actual: image::RgbaImage, baseline: PathBuf) {
    assert_eq!(actual.dimensions(), (1800, 1200));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        actual.save(&baseline).unwrap();
        return;
    }
    let expected = image::open(&baseline)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-example-async-file-browser --test screenshot", baseline.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 8 };
    if !tolerance.matches(&expected, &actual) {
        let diff = baseline.with_file_name(format!(
            "{}-diff.png",
            baseline.file_stem().unwrap().to_string_lossy()
        ));
        write_diff(&expected, &actual, &diff);
        panic!(
            "file-browser screenshot differs from {}; diff written to {}",
            baseline.display(),
            diff.display()
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn report_unsupported_platform() {
    let error = screenshot(
        FileBrowser::preview(
            sample_directory(),
            Ok(vec![BrowserEntry { name: "Example.txt".into(), kind: EntryKind::File }]),
        ),
        size(px(900.), px(600.)),
        1.0,
        gpui_kit::init,
    )
    .expect_err("headless screenshots are unavailable off macOS for this GPUI version");
    assert!(matches!(error, ScreenshotError::UnsupportedPlatform));
    eprintln!("Skipping file-browser screenshot comparison: {error}");
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
