use gpui_pre::{Bounds, EntityInputHandler, Pixels, Window, point, px, size};
use mkit_example_text_input::TextField;
#[cfg(target_os = "macos")]
use mkit_harness::{HeadlessSession, PixelTolerance};
#[cfg(not(target_os = "macos"))]
use mkit_harness::{ScreenshotError, screenshot};
#[cfg(target_os = "macos")]
use std::{fs, path::PathBuf};

#[cfg(target_os = "macos")]
fn compare(actual: image::RgbaImage, literal_path: &str) {
    let baseline = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(literal_path);
    assert_eq!(actual.dimensions(), (1280, 440));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().unwrap()).unwrap();
        actual.save(&baseline).unwrap();
        return;
    }
    let expected = image::open(&baseline).unwrap().to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 8 };
    if !tolerance.matches(&expected, &actual) {
        let diff = baseline.with_file_name(format!(
            "{}-diff.png",
            baseline.file_stem().unwrap().to_string_lossy()
        ));
        write_diff(&expected, &actual, &diff);
        panic!(
            "text-field screenshot differs from {}; diff written to {}",
            baseline.display(),
            diff.display()
        );
    }
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

#[cfg(target_os = "macos")]
fn capture_states() {
    assert_shaped_candidate_bounds();
    let mut session = HeadlessSession::new(
        TextField::new_for_demo(),
        size(px(640.), px(220.)),
        2.,
        gpui_kit::init,
    )
    .unwrap();
    compare(session.capture().unwrap(), "../../book/src/images/text-field-idle.png");
    session
        .update(|root, window, cx| {
            root.update(cx, |field, _| field.set_fixture_state("River", 5..5, None));
            let focus =
                root.read(cx).focus_handle_for_demo().expect("field focus handle initialized");
            window.focus(&focus, cx);
            window.refresh();
        })
        .unwrap();
    compare(session.capture().unwrap(), "../../book/src/images/text-field-focused.png");
    session
        .update(|root, window, cx| {
            root.update(cx, |field, _| field.set_fixture_state("River", 0..5, None));
            let focus =
                root.read(cx).focus_handle_for_demo().expect("field focus handle initialized");
            window.focus(&focus, cx);
            window.refresh();
        })
        .unwrap();
    compare(session.capture().unwrap(), "../../book/src/images/text-field-selected.png");
    session
        .update(|root, window, cx| {
            root.update(cx, |field, _| field.set_fixture_state("かな", 2..2, Some(0..2)));
            let focus =
                root.read(cx).focus_handle_for_demo().expect("field focus handle initialized");
            window.focus(&focus, cx);
            window.refresh();
        })
        .unwrap();
    compare(session.capture().unwrap(), "../../book/src/images/text-field-marked.png");
}

#[cfg(target_os = "macos")]
fn assert_shaped_candidate_bounds() {
    let mut session = HeadlessSession::new(
        TextField::new_for_demo(),
        size(px(640.), px(220.)),
        1.,
        gpui_kit::init,
    )
    .expect("macOS text system should be available");
    let element_bounds = Bounds::new(point(px(47.), px(83.)), size(px(260.), px(48.)));
    let (narrow, wide, emoji) = session
        .update(|root, window, cx| {
            root.update(cx, |field, cx| {
                fn measure(
                    field: &mut TextField,
                    text: &str,
                    range: std::ops::Range<usize>,
                    element_bounds: Bounds<Pixels>,
                    window: &mut Window,
                    cx: &mut gpui_pre::Context<TextField>,
                ) -> (gpui_pre::ShapedLine, Bounds<Pixels>, Pixels) {
                    field.set_fixture_state(text, range.clone(), None);
                    let style = window.text_style();
                    let font_size = style.font_size.to_pixels(window.rem_size());
                    let run = style.to_run(text.len());
                    let line = window.text_system().shape_line(
                        text.to_owned().into(),
                        font_size,
                        &[run],
                        None,
                    );
                    let bounds = field
                        .bounds_for_range(range, element_bounds, window, cx)
                        .expect("candidate range geometry");
                    (line, bounds, window.line_height())
                }

                let narrow = measure(field, "iiii", 0..4, element_bounds, window, cx);
                let wide = measure(field, "WWWW", 0..4, element_bounds, window, cx);
                let emoji = measure(field, "iW😀X", 2..4, element_bounds, window, cx);
                (narrow, wide, emoji)
            })
        })
        .expect("measure candidate bounds");

    assert!(wide.0.width() > narrow.0.width(), "real font shaping should distinguish W from i");
    for (line, candidate, line_height, start, end) in [
        (&narrow.0, narrow.1, narrow.2, 0, 4),
        (&wide.0, wide.1, wide.2, 0, 4),
        (&emoji.0, emoji.1, emoji.2, 2, 6),
    ] {
        let expected_x = element_bounds.left() + px(12.) + line.x_for_index(start);
        let expected_width = (line.x_for_index(end) - line.x_for_index(start)).max(px(1.));
        let expected_y =
            element_bounds.top() + ((element_bounds.size.height - line_height).max(px(0.)) / 2.);
        assert_eq!(candidate.origin.x, expected_x);
        assert_eq!(candidate.origin.y, expected_y);
        assert_eq!(candidate.size.width, expected_width);
    }
    assert_eq!(emoji.1.origin.x, element_bounds.left() + px(12.) + emoji.0.x_for_index(2));
}

#[cfg(not(target_os = "macos"))]
fn report_unsupported_platform() {
    let error = screenshot(TextField::new_for_demo(), size(px(640.), px(220.)), 1., gpui_kit::init)
        .expect_err("pinned GPUI screenshots are macOS-only");
    assert!(matches!(error, ScreenshotError::UnsupportedPlatform));
    eprintln!("Skipping text-field screenshot comparison: {error}");
}

fn main() {
    #[cfg(target_os = "macos")]
    capture_states();
    #[cfg(not(target_os = "macos"))]
    report_unsupported_platform();
}
