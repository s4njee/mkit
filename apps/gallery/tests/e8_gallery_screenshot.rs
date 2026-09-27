use gpui_pre::{App, px, size};
use mkit::core::theme::{self, SHADCN_DARK, SHADCN_LIGHT, Theme};
use mkit_gallery::e8::ProGallery;
use mkit_harness::{PixelTolerance, ScreenshotError, screenshot};
use std::{fs, path::PathBuf};

const CANVAS_SIZE: (f32, f32) = (1080.0, 880.0);
const ADJUST_SIZE: (f32, f32) = (960.0, 2080.0);
const INSPECT_SIZE: (f32, f32) = (960.0, 1360.0);
const COMMAND_SIZE: (f32, f32) = (960.0, 1240.0);
const TIME_SIZE: (f32, f32) = (960.0, 1440.0);
const GRAPH_SIZE: (f32, f32) = (960.0, 1560.0);

fn baseline_path(scene: &str, name: &str, scale: u32) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("snapshots/e8-{scene}-{name}-{scale}x.png"))
}

fn capture(
    view: ProGallery,
    dimensions: (f32, f32),
    theme_value: Theme,
    scale: u32,
) -> Result<image::RgbaImage, ScreenshotError> {
    screenshot(view, size(px(dimensions.0), px(dimensions.1)), scale as f32, |cx: &mut App| {
        gpui_kit::base::init(cx);
        theme::set_theme(cx, theme_value);
    })
}

fn assert_visible_content(image: &image::RgbaImage, scale: u32, dimensions: (f32, f32)) {
    assert_eq!(image.dimensions(), (dimensions.0 as u32 * scale, dimensions.1 as u32 * scale));
    let nonbackground =
        image.pixels().filter(|pixel| pixel[0] < 180 && pixel[1] < 180 && pixel[2] < 180).count();
    assert!(nonbackground > 1000 * scale as usize, "E8 gallery rendered no visible controls");
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
    let selected_scene = std::env::var("E8_SCENE").ok();
    let scenes = ["canvas", "adjust", "inspect", "command", "time", "graph"];
    for scene in scenes {
        if selected_scene.as_deref().is_some_and(|selected| selected != scene) {
            continue;
        }
        for (name, theme_value) in [("shadcn-light", SHADCN_LIGHT), ("shadcn-dark", SHADCN_DARK)] {
            for scale in [1, 2] {
                let view = match scene {
                    "adjust" => ProGallery::adjust_preview(),
                    "inspect" => ProGallery::inspect_preview(),
                    "command" => ProGallery::command_preview(),
                    "time" => ProGallery::time_preview(),
                    "graph" => ProGallery::graph_preview(),
                    _ => ProGallery::canvas_preview(),
                };
                let dimensions = match scene {
                    "adjust" => ADJUST_SIZE,
                    "inspect" => INSPECT_SIZE,
                    "command" => COMMAND_SIZE,
                    "time" => TIME_SIZE,
                    "graph" => GRAPH_SIZE,
                    _ => CANVAS_SIZE,
                };
                let actual = capture(view, dimensions, theme_value, scale)?;
                assert_visible_content(&actual, scale, dimensions);
                let baseline = baseline_path(scene, name, scale);
                if std::env::var_os("UPDATE_SNAPSHOTS").as_deref()
                    == Some(std::ffi::OsStr::new("1"))
                {
                    fs::create_dir_all(baseline.parent().expect("baseline parent")).unwrap();
                    actual.save(&baseline).unwrap();
                    println!("Updated E8 gallery screenshot: {}", baseline.display());
                    continue;
                }
                if !baseline.exists() {
                    panic!(
                        "missing baseline {}; inspect and update with UPDATE_SNAPSHOTS=1",
                        baseline.display()
                    );
                }
                let expected = image::open(&baseline)
                    .unwrap_or_else(|error| {
                        panic!("missing baseline {}: {error}", baseline.display())
                    })
                    .to_rgba8();
                assert_visible_content(&expected, scale, dimensions);
                let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 32 };
                if !tolerance.matches(&expected, &actual) {
                    let diff =
                        baseline.with_file_name(format!("e8-{scene}-{name}-{scale}x-diff.png"));
                    write_diff(&expected, &actual, &diff);
                    panic!(
                        "E8 {scene} screenshot differs; pixel diff written to {}",
                        diff.display()
                    );
                }
            }
        }
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping E8 screenshot matrix: GPUI headless capture requires macOS Metal.");
        }
        Err(error) => panic!("E8 screenshot capture failed: {error}"),
    }
}
