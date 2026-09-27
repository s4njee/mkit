use gpui_pre::{px, size};
use mkit_example_cookbook::{
    Cookbook, CounterDemo, ModalFocusTrap, ReorderRecipeView, ScrollRecipeView,
};
#[cfg(target_os = "macos")]
use mkit_harness::{HeadlessSession, PixelTolerance};
#[cfg(not(target_os = "macos"))]
use mkit_harness::{ScreenshotError, screenshot};
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf};

#[cfg(target_os = "macos")]
fn compare(relative_path: &str, actual: image::RgbaImage) {
    let baseline = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    assert_eq!(actual.dimensions(), (1920, 1280));
    let background = actual.get_pixel(0, 0);
    let foreground_pixels = actual
        .pixels()
        .filter(|pixel| {
            pixel.0[..3]
                .iter()
                .zip(&background.0[..3])
                .any(|(channel, background)| channel.abs_diff(*background) > 24)
        })
        .count();
    assert!(
        foreground_pixels > 100,
        "{} must contain visible foreground content, got {foreground_pixels} differing pixels",
        relative_path
    );
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        actual.save(&baseline).unwrap();
        return;
    }
    let expected = image::open(&baseline).unwrap().to_rgba8();
    let matches =
        (PixelTolerance { channel_delta: 2, max_different_pixels: 8 }).matches(&expected, &actual);
    if !matches {
        let mut diff = image::RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in actual.enumerate_pixels() {
            let changed = expected.get_pixel(x, y) != pixel;
            diff.put_pixel(
                x,
                y,
                if changed { image::Rgba([255, 0, 0, 255]) } else { image::Rgba([0, 0, 0, 0]) },
            );
        }
        let diff_path = baseline.with_file_name("cookbook-diff.png");
        diff.save(&diff_path).expect("save cookbook screenshot diff");
        eprintln!("cookbook screenshot diff saved to {}", diff_path.display());
    }
    assert!(matches, "cookbook screenshot differs from {}", baseline.display());
}

#[cfg(target_os = "macos")]
fn capture() {
    let mut session =
        HeadlessSession::new(Cookbook::default(), size(px(960.), px(640.)), 2., gpui_kit::init)
            .expect("macOS Metal headless renderer should be available");
    compare(
        "../../book/src/images/cookbook.png",
        session.capture().expect("capture cookbook fixture"),
    );
    session
        .update(|root, _, cx| {
            root.update(cx, |cookbook, cx| cookbook.set_alternate_palette(true, cx))
        })
        .expect("switch to the token-based alternate palette");
    compare(
        "../../book/src/images/cookbook-dark.png",
        session.capture().expect("capture alternate palette"),
    );
    let mut scrolled = HeadlessSession::new(
        ScrollRecipeView::default(),
        size(px(960.), px(640.)),
        2.,
        gpui_kit::init,
    )
    .expect("create scroll recipe screenshot session");
    scrolled
        .update(|root, _, cx| root.update(cx, |view, _| view.set_demo_offset(px(-120.))))
        .expect("prepare scrolled content state");
    compare(
        "../../book/src/images/cookbook-scrolled.png",
        scrolled.capture().expect("capture scrolled recipe content"),
    );
    let mut reordered = HeadlessSession::new(
        ReorderRecipeView::default(),
        size(px(960.), px(640.)),
        2.,
        gpui_kit::init,
    )
    .expect("create drag reorder screenshot session");
    reordered
        .update(|root, _, cx| {
            root.update(cx, |view, _| {
                view.set_preview_order(vec!["Second recipe", "Third recipe", "First recipe"])
            })
        })
        .expect("prepare reordered list state");
    compare(
        "../../book/src/images/cookbook-dragged.png",
        reordered.capture().expect("capture reordered recipe list"),
    );

    let mut counter =
        HeadlessSession::new(CounterDemo::default(), size(px(960.), px(640.)), 2., gpui_kit::init)
            .expect("create counter recipe screenshot session");
    compare(
        "../../book/src/images/cookbook-counter.png",
        counter.capture().expect("capture counter recipe"),
    );

    let mut modal = HeadlessSession::new(
        ModalFocusTrap::default(),
        size(px(960.), px(640.)),
        2.,
        gpui_kit::init,
    )
    .expect("create modal recipe screenshot session");
    compare(
        "../../book/src/images/cookbook-modal.png",
        modal.capture().expect("capture modal recipe"),
    );
}

#[cfg(not(target_os = "macos"))]
fn report_unsupported_platform() {
    let error = screenshot(Cookbook::default(), size(px(960.), px(640.)), 1., gpui_kit::init)
        .expect_err("this GPUI version only supports headless screenshots on macOS");
    assert!(matches!(error, ScreenshotError::UnsupportedPlatform));
    eprintln!("Skipping cookbook screenshot comparison: {error}");
}

fn main() {
    #[cfg(target_os = "macos")]
    capture();
    #[cfg(not(target_os = "macos"))]
    report_unsupported_platform();
}
