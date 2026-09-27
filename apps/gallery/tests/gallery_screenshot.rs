use gpui_pre::{App, px, size};
use mkit::core::theme::{self, LIGHT};
use mkit_gallery::ComponentGallery;
use mkit_harness::{PixelTolerance, ScreenshotError, screenshot};
use std::{fs, path::PathBuf};

const SIZE: (f32, f32) = (900.0, 680.0);

fn baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("snapshots/gallery-light-1x.png")
}

fn capture() -> Result<image::RgbaImage, ScreenshotError> {
    screenshot(ComponentGallery::new(), size(px(SIZE.0), px(SIZE.1)), 1.0, |cx: &mut App| {
        gpui_kit::base::init(cx);
        theme::set_theme(cx, LIGHT);
    })
}

/// Ensure the component rows contain their own visible ink. A baseline with a
/// blank combobox or value field must fail before any snapshot comparison.
fn require_control_glyphs(image: &image::RgbaImage) {
    assert_eq!(image.dimensions(), (SIZE.0 as u32, SIZE.1 as u32));
    let ink_count = |x0: u32, y0: u32, x1: u32, y1: u32| {
        image
            .enumerate_pixels()
            .filter(|(x, y, pixel)| {
                *x >= x0
                    && *x < x1
                    && *y >= y0
                    && *y < y1
                    && pixel[3] > 240
                    && pixel[0] < 100
                    && pixel[1] < 105
                    && pixel[2] < 120
            })
            .count()
    };
    assert!(ink_count(32, 185, 868, 300) > 20, "combobox preview has no readable glyphs");
    assert!(ink_count(32, 390, 868, 520) > 20, "number field preview has no readable glyphs");
}

fn write_diff(expected: &image::RgbaImage, actual: &image::RgbaImage, path: &std::path::Path) {
    let mut diff = image::RgbaImage::new(actual.width(), actual.height());
    for (x, y, pixel) in diff.enumerate_pixels_mut() {
        let left = expected.get_pixel(x, y);
        let right = actual.get_pixel(x, y);
        *pixel = image::Rgba([
            left[0].abs_diff(right[0]).saturating_mul(4),
            left[1].abs_diff(right[1]).saturating_mul(4),
            left[2].abs_diff(right[2]).saturating_mul(4),
            255,
        ]);
    }
    diff.save(path).unwrap();
}

fn run() -> Result<(), ScreenshotError> {
    let actual = capture()?;
    require_control_glyphs(&actual);
    let baseline = baseline_path();
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(baseline.parent().expect("baseline parent")).unwrap();
        actual.save(&baseline).unwrap();
        println!("Updated gallery light screenshot: {}", baseline.display());
        return Ok(());
    }

    let expected = image::open(&baseline)
        .unwrap_or_else(|error| panic!("missing baseline {}: {error}; run UPDATE_SNAPSHOTS=1 cargo test -p mkit-gallery --test gallery_screenshot", baseline.display()))
        .to_rgba8();
    require_control_glyphs(&expected);
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 32 };
    if !tolerance.matches(&expected, &actual) {
        let diff = baseline.with_file_name("gallery-light-1x-diff.png");
        write_diff(&expected, &actual, &diff);
        panic!("gallery light screenshot differs; pixel diff written to {}", diff.display());
    }
    println!("Gallery light screenshot matches {}", baseline.display());
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!(
                "Skipping gallery screenshot smoke test: GPUI headless capture requires macOS Metal."
            );
        }
        Err(error) => panic!("gallery screenshot capture failed: {error}"),
    }
}
