//! Harness screenshots of Stepper hosted in Dialog and Sheet at its middle step.
//!
//! `UPDATE_SNAPSHOTS=1` rewrites baselines; `SNAPSHOT_CANDIDATE_DIR=<dir>` writes candidates
//! there for review without touching baselines. `SNAPSHOT_CASE=<name>` limits the run.

fn main() {
    #[cfg(target_os = "macos")]
    run();
}

#[cfg(target_os = "macos")]
fn run() {
    use gpui_pre::{App, px, size};
    use image::RgbaImage;
    use mkit::core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme};
    use mkit_example_e7_compositions::{ModalKind, StepperModalScene, bind_keys};
    use mkit_harness::{HeadlessSession, PixelTolerance};
    use std::path::{Path, PathBuf};

    fn scene_screenshot(kind: ModalKind, palette: Theme, scale: f32) -> RgbaImage {
        let mut session = HeadlessSession::new(
            StepperModalScene::new(kind),
            size(px(720.0), px(480.0)),
            scale,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
                bind_keys(cx);
            },
        )
        .expect("open stepper modal scene");
        // Dialog focuses its surface first; Sheet focuses the step control. Move to Next and
        // advance to the middle step through the keyboard path under test.
        let keys = match kind {
            ModalKind::Dialog => "tab tab enter",
            ModalKind::Sheet => "tab enter",
        };
        session.simulate_keystrokes(keys).expect("advance to middle step");
        session.capture().expect("capture stepper modal scene")
    }

    fn compare(actual: &RgbaImage, path: &Path) {
        let name = path.file_name().expect("baseline file name");
        if let Some(dir) = std::env::var_os("SNAPSHOT_CANDIDATE_DIR") {
            let candidate = PathBuf::from(dir).join(name);
            std::fs::create_dir_all(candidate.parent().unwrap()).expect("create candidate dir");
            actual.save(&candidate).expect("save candidate screenshot");
            return;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
            std::fs::create_dir_all(path.parent().unwrap()).expect("create baseline dir");
            actual.save(path).expect("save stepper modal baseline");
            println!("Updated stepper modal screenshot: {}", path.display());
            return;
        }
        let expected = image::open(path)
            .unwrap_or_else(|e| panic!("missing stepper modal baseline {}: {e}", path.display()))
            .to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, actual),
            "stepper modal screenshot differs: {}",
            path.display()
        );
        println!("Matched stepper modal screenshot: {}", name.to_string_lossy());
    }

    // The dark 2x captures double as the book images.
    let cases = [
        (ModalKind::Dialog, SHADCN_LIGHT, 1.0, "tests/baselines/stepper-dialog-light-1x.png"),
        (ModalKind::Dialog, SHADCN_LIGHT, 2.0, "tests/baselines/stepper-dialog-light-2x.png"),
        (ModalKind::Dialog, SHADCN_DARK, 1.0, "tests/baselines/stepper-dialog-dark-1x.png"),
        (ModalKind::Dialog, SHADCN_DARK, 2.0, "../../book/src/images/e7-stepper-in-dialog.png"),
        (
            ModalKind::Dialog,
            HIGH_CONTRAST,
            1.0,
            "tests/baselines/stepper-dialog-high-contrast-1x.png",
        ),
        (
            ModalKind::Dialog,
            HIGH_CONTRAST,
            2.0,
            "tests/baselines/stepper-dialog-high-contrast-2x.png",
        ),
        (ModalKind::Sheet, SHADCN_LIGHT, 1.0, "tests/baselines/stepper-sheet-light-1x.png"),
        (ModalKind::Sheet, SHADCN_LIGHT, 2.0, "tests/baselines/stepper-sheet-light-2x.png"),
        (ModalKind::Sheet, SHADCN_DARK, 1.0, "tests/baselines/stepper-sheet-dark-1x.png"),
        (ModalKind::Sheet, SHADCN_DARK, 2.0, "../../book/src/images/e7-stepper-in-sheet.png"),
        (
            ModalKind::Sheet,
            HIGH_CONTRAST,
            1.0,
            "tests/baselines/stepper-sheet-high-contrast-1x.png",
        ),
        (
            ModalKind::Sheet,
            HIGH_CONTRAST,
            2.0,
            "tests/baselines/stepper-sheet-high-contrast-2x.png",
        ),
    ];
    let only = std::env::var("SNAPSHOT_CASE").ok();
    for (kind, palette, scale, relative) in cases {
        if only.as_deref().is_some_and(|name| !relative.contains(name)) {
            continue;
        }
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        compare(&scene_screenshot(kind, palette, scale), &path);
    }
}
