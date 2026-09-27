use gpui_pre::{App, Focusable, Keystroke, px, size};
use mkit::core::theme::{self, SHADCN_DARK, SHADCN_LIGHT, Theme};
use mkit_gallery::e7::EverydayGallery;
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError, screenshot};
use std::{fs, path::PathBuf};

const GALLERY_SIZE: (f32, f32) = (960.0, 2040.0);
const INPUTS_SIZE: (f32, f32) = (960.0, 1780.0);
const MENU_SIZE: (f32, f32) = (960.0, 620.0);
const OVERLAY_SIZE: (f32, f32) = (960.0, 1240.0);
const NAV_SIZE: (f32, f32) = (960.0, 760.0);
const COLLECTION_SIZE: (f32, f32) = (960.0, 820.0);
const LAYOUT_SIZE: (f32, f32) = (960.0, 900.0);

fn baseline_path(scene: &str, name: &str, scale: u32) -> PathBuf {
    let prefix = if scene == "inputs" { "e7-inputs" } else { "e7" };
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("snapshots/{prefix}-{name}-{scale}x.png"))
}

fn capture(
    view: EverydayGallery,
    dimensions: (f32, f32),
    theme_value: Theme,
    scale: u32,
    settle_deferred: bool,
    open_selection_previews: bool,
) -> Result<image::RgbaImage, ScreenshotError> {
    if settle_deferred || open_selection_previews {
        let mut session = HeadlessSession::new(
            view,
            size(px(dimensions.0), px(dimensions.1)),
            scale as f32,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, theme_value);
                cx.bind_keys(mkit::select::default_key_bindings());
                cx.bind_keys(mkit::multi_select::default_key_bindings());
            },
        )?;
        if open_selection_previews {
            session.update(|root, window, cx| {
                let (select, multi_select) = root.read(cx).selection_preview_entities();
                select.update(cx, |select, cx| select.focus_handle(cx).focus(window, cx));
                window.dispatch_keystroke(Keystroke::parse("enter").expect("Enter key"), cx);
                multi_select.update(cx, |select, cx| select.focus_handle(cx).focus(window, cx));
                window.dispatch_keystroke(Keystroke::parse("enter").expect("Enter key"), cx);
                assert!(select.read(cx).is_open());
                assert!(multi_select.read(cx).is_open());
            })?;
        }
        if settle_deferred {
            session.update(|_, _, _| {})?;
        }
        return session.capture();
    }
    screenshot(view, size(px(dimensions.0), px(dimensions.1)), scale as f32, |cx: &mut App| {
        gpui_kit::base::init(cx);
        theme::set_theme(cx, theme_value);
    })
}

fn assert_visible_content(image: &image::RgbaImage, scale: u32, dimensions: (f32, f32)) {
    assert_eq!(image.dimensions(), (dimensions.0 as u32 * scale, dimensions.1 as u32 * scale));
    let nonbackground =
        image.pixels().filter(|pixel| pixel[0] < 180 && pixel[1] < 180 && pixel[2] < 180).count();
    assert!(nonbackground > 1000 * scale as usize, "E7 gallery rendered no visible controls");
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
    let capture_dir = std::env::var_os("E7_CAPTURE_DIR").map(PathBuf::from);
    let selected_scene = std::env::var("E7_SCENE").ok();
    let scenes = ["gallery", "inputs", "menus", "overlays", "navigation", "collections", "layout"];
    for scene in scenes {
        if selected_scene.as_deref().is_some_and(|selected| selected != scene) {
            continue;
        }
        for (name, theme_value) in [("shadcn-light", SHADCN_LIGHT), ("shadcn-dark", SHADCN_DARK)] {
            for scale in [1, 2] {
                let view = match scene {
                    "inputs" => EverydayGallery::inputs_preview(),
                    "menus" => EverydayGallery::menus_preview(),
                    "overlays" => EverydayGallery::overlays_preview(),
                    "navigation" => EverydayGallery::navigation_preview(),
                    "collections" => EverydayGallery::collections_preview(),
                    "layout" => EverydayGallery::layout_preview(),
                    _ => EverydayGallery::new(),
                };
                let dimensions = match scene {
                    "inputs" => INPUTS_SIZE,
                    "menus" => MENU_SIZE,
                    "overlays" => OVERLAY_SIZE,
                    "navigation" => NAV_SIZE,
                    "collections" => COLLECTION_SIZE,
                    "layout" => LAYOUT_SIZE,
                    _ => GALLERY_SIZE,
                };
                let actual = capture(
                    view,
                    dimensions,
                    theme_value,
                    scale,
                    scene == "overlays",
                    scene == "inputs",
                )?;
                assert_visible_content(&actual, scale, dimensions);
                let baseline = if scene == "inputs" {
                    baseline_path(scene, name, scale)
                } else if scene == "gallery" {
                    baseline_path("gallery", name, scale)
                } else {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join(format!("snapshots/e7-{scene}-{name}-{scale}x.png"))
                };
                if let Some(dir) = &capture_dir {
                    fs::create_dir_all(dir).unwrap();
                    let capture_path = dir.join(baseline.file_name().unwrap());
                    actual.save(&capture_path).unwrap();
                    println!("Captured E7 screenshot: {}", capture_path.display());
                    continue;
                }
                if std::env::var_os("UPDATE_SNAPSHOTS").as_deref()
                    == Some(std::ffi::OsStr::new("1"))
                {
                    fs::create_dir_all(baseline.parent().expect("baseline parent")).unwrap();
                    actual.save(&baseline).unwrap();
                    println!("Updated E7 gallery screenshot: {}", baseline.display());
                    continue;
                }
                if scene != "gallery" && !baseline.exists() {
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
                        baseline.with_file_name(format!("e7-{scene}-{name}-{scale}x-diff.png"));
                    write_diff(&expected, &actual, &diff);
                    panic!(
                        "E7 {scene} screenshot differs; pixel diff written to {}",
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
            println!("Skipping E7 screenshot matrix: GPUI headless capture requires macOS Metal.");
        }
        Err(error) => panic!("E7 screenshot capture failed: {error}"),
    }
}
