use gpui_pre::{App, px, size};
use image::RgbaImage;
use mkit_aria2_sample::{Aria2Sample, install_fonts, model::Scene, theme};
use mkit_harness::{PixelTolerance, ScreenshotError, screenshot};
use std::{fs, path::PathBuf};

fn view(name: &str) -> Aria2Sample {
    match name {
        "main" => Aria2Sample::new(),
        "add" => Aria2Sample::for_add_dialog(),
        "detail" => Aria2Sample::for_scene(Scene::Detail),
        "queues" => Aria2Sample::for_scene(Scene::Queues),
        "settings" => Aria2Sample::for_scene(Scene::Settings),
        "capture" => Aria2Sample::for_scene(Scene::Capture),
        _ => panic!("unknown sample scene"),
    }
}

fn save_or_compare(image: RgbaImage, name: &str, theme_name: &str, scale: u32) {
    let height = match name {
        "detail" => 850,
        "settings" => 960,
        _ => 640,
    };
    assert_eq!(image.dimensions(), (1080 * scale, height * scale));
    let filename = format!("{name}-{theme_name}-{scale}x.png");
    if let Some(dir) = std::env::var_os("ARIA2_CAPTURE_DIR") {
        let output = PathBuf::from(dir);
        fs::create_dir_all(&output).expect("create capture directory");
        image.save(output.join(filename)).expect("save candidate");
        return;
    }
    let baseline = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("snapshots").join(filename);
    let expected = image::open(&baseline)
        .unwrap_or_else(|error| panic!("missing {}: {error}", baseline.display()))
        .to_rgba8();
    if !(PixelTolerance { channel_delta: 2, max_different_pixels: 32 }).matches(&expected, &image) {
        let diff = baseline.with_extension("diff.png");
        let mut pixels = RgbaImage::new(image.width(), image.height());
        for (x, y, out) in pixels.enumerate_pixels_mut() {
            let left = expected.get_pixel(x, y);
            let right = image.get_pixel(x, y);
            *out = image::Rgba([
                left[0].abs_diff(right[0]).saturating_mul(4),
                left[1].abs_diff(right[1]).saturating_mul(4),
                left[2].abs_diff(right[2]).saturating_mul(4),
                255,
            ]);
        }
        pixels.save(&diff).expect("save visual diff");
        panic!("sample screenshot differs; inspect {}", diff.display());
    }
}

fn run() -> Result<(), ScreenshotError> {
    let scenes = ["main", "add", "detail", "queues", "settings", "capture"];
    let themes = [("light", theme::LIGHT), ("dark", theme::DARK), ("paper", theme::PAPER)];
    for name in scenes {
        if std::env::var("ARIA2_SCENE").is_ok_and(|selected| selected != name) {
            continue;
        }
        for (theme_name, value) in themes {
            if std::env::var("ARIA2_THEME").is_ok_and(|selected| selected != theme_name) {
                continue;
            }
            for scale in [1, 2] {
                if std::env::var("ARIA2_SCALE").is_ok_and(|selected| selected != scale.to_string())
                {
                    continue;
                }
                let image = screenshot(
                    view(name),
                    size(
                        px(1080.0),
                        px(match name {
                            "detail" => 850.0,
                            "settings" => 960.0,
                            _ => 640.0,
                        }),
                    ),
                    scale as f32,
                    |cx: &mut App| {
                        gpui_kit::base::init(cx);
                        install_fonts(cx);
                        theme::install(cx, value);
                        cx.bind_keys(mkit::button::default_key_bindings());
                        cx.bind_keys(mkit::text_field::default_key_bindings());
                        cx.bind_keys(mkit::dialog::default_key_bindings());
                        cx.bind_keys(mkit::data_table::default_key_bindings());
                        cx.bind_keys(mkit::select::default_key_bindings());
                        cx.bind_keys(mkit::slider::default_key_bindings());
                        cx.bind_keys(mkit_aria2_sample::default_key_bindings());
                    },
                )?;
                save_or_compare(image, name, theme_name, scale);
            }
        }
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => println!("Screenshots require macOS Metal."),
        Err(error) => panic!("sample screenshot failed: {error}"),
    }
}
