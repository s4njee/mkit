use gpui_pre::{InputEvent, ScrollDelta, ScrollWheelEvent, point, px, size};
#[cfg(target_os = "macos")]
use mkit_example_custom_rendering::SurfaceDemo;
use mkit_example_custom_rendering::{AnimationDemo, OverlayDemo, PanZoomView, VirtualizedLists};
#[cfg(target_os = "macos")]
use mkit_harness::PixelTolerance;
#[cfg(not(target_os = "macos"))]
use mkit_harness::ScreenshotError;
#[cfg(not(target_os = "macos"))]
use mkit_harness::screenshot;
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf};

#[cfg(target_os = "macos")]
fn baseline(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[cfg(target_os = "macos")]
fn compare(relative_path: &str, actual: &image::RgbaImage) {
    let path = baseline(relative_path);
    let name = path.file_stem().unwrap().to_string_lossy();
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        actual.save(&path).unwrap();
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-example-custom-rendering --test screenshots", path.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 8 };
    if !tolerance.matches(&expected, actual) {
        let diff = path.with_file_name(format!("{name}-diff.png"));
        let mut image = image::RgbaImage::new(actual.width(), actual.height());
        for y in 0..actual.height() {
            for x in 0..actual.width() {
                let actual_pixel = actual.get_pixel(x, y);
                let expected_pixel =
                    expected.get_pixel_checked(x, y).copied().unwrap_or(image::Rgba([0; 4]));
                image.put_pixel(
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
        image.save(&diff).unwrap();
        panic!(
            "custom-rendering screenshot differs from {}; diff written to {}",
            path.display(),
            diff.display()
        );
    }
}

#[cfg(target_os = "macos")]
fn assert_surface_pixels(actual: &image::RgbaImage) {
    let gray_surface_pixels = actual
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _] = pixel.0;
            red > 30 && red < 240 && red.abs_diff(green) < 4 && green.abs_diff(blue) < 4
        })
        .count();
    assert!(
        gray_surface_pixels > 1_000,
        "expected the YUV surface's grayscale image to appear in the capture; found {gray_surface_pixels} matching pixels"
    );
}

#[cfg(target_os = "macos")]
fn captures() -> [image::RgbaImage; 7] {
    let mut initial = mkit_harness::HeadlessSession::new(
        PanZoomView::default(),
        size(px(640.), px(480.)),
        2.0,
        |_| {},
    )
    .expect("macOS Metal headless renderer should be available");
    let initial_pixels = initial.capture().expect("capture initial canvas");

    let mut panned = mkit_harness::HeadlessSession::new(
        PanZoomView::default(),
        size(px(640.), px(480.)),
        2.0,
        |_| {},
    )
    .expect("macOS Metal headless renderer should be available");
    panned
        .update(|_, window, cx| {
            window.dispatch_event(
                gpui_pre::MouseDownEvent {
                    position: point(px(200.), px(200.)),
                    button: gpui_pre::MouseButton::Left,
                    modifiers: Default::default(),
                    click_count: 1,
                    first_mouse: false,
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                gpui_pre::MouseMoveEvent {
                    position: point(px(236.), px(224.)),
                    pressed_button: Some(gpui_pre::MouseButton::Left),
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                gpui_pre::MouseUpEvent {
                    position: point(px(236.), px(224.)),
                    button: gpui_pre::MouseButton::Left,
                    modifiers: Default::default(),
                    click_count: 1,
                }
                .to_platform_input(),
                cx,
            );
        })
        .expect("pan canvas by drag");
    let panned_pixels = panned.capture().expect("capture panned canvas");

    let mut zoomed = mkit_harness::HeadlessSession::new(
        PanZoomView::default(),
        size(px(640.), px(480.)),
        2.0,
        |_| {},
    )
    .expect("macOS Metal headless renderer should be available");
    zoomed
        .update(|_, window, cx| {
            window.dispatch_event(
                ScrollWheelEvent {
                    position: point(px(280.), px(210.)),
                    delta: ScrollDelta::Lines(point(0., 2.)),
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
        })
        .expect("zoom canvas around pointer");
    let zoomed_pixels = zoomed.capture().expect("capture zoomed canvas");

    let mut overlay = mkit_harness::HeadlessSession::new(
        OverlayDemo::default(),
        size(px(640.), px(480.)),
        2.0,
        |_| {},
    )
    .expect("create overlay harness");
    let overlay_pixels = overlay.capture().expect("capture anchored/deferred overlays");

    let mut lists = mkit_harness::HeadlessSession::new(
        VirtualizedLists::default(),
        size(px(640.), px(480.)),
        2.0,
        |_| {},
    )
    .expect("create virtualized-list harness");
    let list_pixels = lists.capture().expect("capture uniform and variable lists");

    let mut animation =
        mkit_harness::HeadlessSession::new(AnimationDemo, size(px(640.), px(480.)), 2.0, |cx| {
            cx.set_reduce_motion(true)
        })
        .expect("create reduced-motion animation harness");
    let animation_pixels = animation.capture().expect("capture stable reduced-motion endpoint");

    let mut surface =
        mkit_harness::HeadlessSession::new(SurfaceDemo, size(px(640.), px(480.)), 2.0, |_| {})
            .expect("create macOS IOSurface-backed CoreVideo harness");
    let surface_pixels = surface.capture().expect("capture Metal-compatible CVPixelBuffer");
    assert_surface_pixels(&surface_pixels);
    [
        initial_pixels,
        panned_pixels,
        zoomed_pixels,
        overlay_pixels,
        list_pixels,
        animation_pixels,
        surface_pixels,
    ]
}

fn main() {
    #[cfg(target_os = "macos")]
    {
        let images = captures();
        compare("../../book/src/images/custom-rendering-initial.png", &images[0]);
        compare("../../book/src/images/custom-rendering-panned.png", &images[1]);
        compare("../../book/src/images/custom-rendering-zoomed.png", &images[2]);
        compare("../../book/src/images/custom-rendering-overlay.png", &images[3]);
        compare("../../book/src/images/custom-rendering-lists.png", &images[4]);
        compare("../../book/src/images/custom-rendering-animation.png", &images[5]);
        compare("../../book/src/images/custom-rendering-surface-macos.png", &images[6]);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let result = screenshot(PanZoomView::default(), size(px(640.), px(480.)), 1.0, |_| {});
        assert!(matches!(result, Err(ScreenshotError::UnsupportedPlatform)));
        eprintln!("Skipping custom-rendering screenshots: {result:?}");
    }
}
