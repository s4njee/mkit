//! Shared helpers for benchmark acceptance tests.
//!
//! The evaluation runner copies this file into a generated grading crate as
//! `tests/support/mod.rs`, next to one benchmark's `acceptance.rs` and
//! `screenshots.rs`. It is never copied into an evaluation agent's workspace.
//!
//! Every helper talks to the candidate only through the contract in
//! `evals/benchmarks/CONTRACT.md`: `bench_app::init`, `bench_app::root`,
//! `bench_app::Root::snapshot`, and `debug_selector` names.

#![allow(dead_code)]

use gpui_pre::{
    AnyWindowHandle, Bounds, Entity, Modifiers, MouseButton, Pixels, Point, ScrollDelta,
    ScrollWheelEvent, TestAppContext, TouchPhase, VisualTestContext, point, px,
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

pub use bench_app::Root;

static FIXTURE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// The benchmark's pristine fixture directory. The grading crate sets
/// `MKIT_EVAL_FIXTURE`; an empty directory is used when a benchmark has none.
pub fn pristine_fixture() -> Option<PathBuf> {
    std::env::var_os("MKIT_EVAL_FIXTURE").map(PathBuf::from).filter(|path| path.is_dir())
}

/// Copy the fixture into a fresh directory so tests can mutate files freely.
pub fn fresh_fixture() -> PathBuf {
    let n = FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "mkit-eval-fixture-{}-{n}-{}",
        std::process::id(),
        unique_suffix()
    ));
    std::fs::create_dir_all(&dir).expect("create fixture directory");
    if let Some(source) = pristine_fixture() {
        copy_dir(&source, &dir).expect("copy benchmark fixture");
    }
    dir
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default()
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            std::fs::create_dir_all(&target)?;
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Directory for screenshots, skip markers, and other review artefacts.
pub fn artifacts_dir() -> PathBuf {
    let dir = std::env::var_os("MKIT_EVAL_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("mkit-eval-artifacts"));
    std::fs::create_dir_all(&dir).expect("create artifacts directory");
    dir
}

/// Record that a criterion could not be checked on this host. The runner reads
/// these markers and reports the criterion as `skipped`, never as `passed`.
pub fn skip(test: &str, reason: &str) {
    let dir = artifacts_dir().join("skipped");
    std::fs::create_dir_all(&dir).expect("create skip directory");
    std::fs::write(dir.join(format!("{test}.txt")), reason).expect("write skip marker");
    eprintln!("MKIT_EVAL_SKIPPED {test}: {reason}");
}

/// An open candidate window driven through GPUI's visual test context.
pub struct Bench<'a> {
    pub view: Entity<Root>,
    pub cx: &'a mut VisualTestContext,
    pub fixture: PathBuf,
}

/// Initialize the candidate, open its root view, and draw the first frame.
pub fn open(cx: &mut TestAppContext) -> Bench<'_> {
    open_with_fixture(cx, fresh_fixture())
}

/// Like [`open`], with a caller-prepared fixture directory.
pub fn open_with_fixture(cx: &mut TestAppContext, fixture: PathBuf) -> Bench<'_> {
    cx.update(bench_app::init);
    let root_fixture = fixture.clone();
    let (view, visual) = cx.add_window_view(move |_, _| bench_app::root(&root_fixture));
    let mut bench = Bench { view, cx: visual, fixture };
    bench.draw();
    bench.draw();
    bench
}

impl Bench<'_> {
    /// Draw a frame so element bounds and first-render state exist.
    pub fn draw(&mut self) {
        self.cx.update(|window, cx| window.draw(cx).clear(cx));
        self.cx.run_until_parked();
    }

    /// The candidate's documented state snapshot.
    pub fn snapshot(&mut self) -> Value {
        self.cx.run_until_parked();
        self.view.read_with(self.cx, |root, cx| root.snapshot(cx))
    }

    /// Dispatch space-separated GPUI keystrokes, e.g. `"down down enter"`.
    pub fn press(&mut self, keys: &str) {
        self.cx.simulate_keystrokes(keys);
        self.draw();
    }

    /// Type text one character at a time through GPUI's keyboard dispatch.
    pub fn type_text(&mut self, text: &str) {
        for ch in text.chars() {
            if ch == ' ' {
                self.cx.simulate_keystrokes("space");
            } else {
                self.cx.simulate_input(&ch.to_string());
            }
        }
        self.draw();
    }

    /// Bounds of an element tagged with `.debug_selector(|| name.into())`.
    pub fn bounds(&mut self, selector: &str) -> Option<Bounds<Pixels>> {
        self.draw();
        let selector: &'static str = Box::leak(selector.to_owned().into_boxed_str());
        self.cx.debug_bounds(selector)
    }

    /// Bounds of a required target; fails with a contract message when absent.
    pub fn target(&mut self, selector: &str) -> Bounds<Pixels> {
        self.bounds(selector).unwrap_or_else(|| {
            panic!(
                "debug_selector `{selector}` is not rendered; the benchmark spec requires it \
                 (see the Targets section of the task)"
            )
        })
    }

    pub fn click(&mut self, selector: &str) {
        let center = self.target(selector).center();
        self.click_at(center);
    }

    pub fn click_at(&mut self, position: Point<Pixels>) {
        self.cx.simulate_mouse_move(position, None, Modifiers::default());
        self.cx.simulate_click(position, Modifiers::default());
        self.draw();
    }

    /// Press at `from`, move through `steps` intermediate points, release at `to`.
    pub fn drag(&mut self, from: Point<Pixels>, to: Point<Pixels>, steps: usize) {
        let steps = steps.max(1);
        self.cx.simulate_mouse_move(from, None, Modifiers::default());
        self.cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let p = point(from.x + (to.x - from.x) * t, from.y + (to.y - from.y) * t);
            self.cx.simulate_mouse_move(p, MouseButton::Left, Modifiers::default());
        }
        self.cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
        self.draw();
    }

    /// Drag through an explicit list of points (at least two).
    pub fn drag_path(&mut self, points: &[Point<Pixels>]) {
        assert!(points.len() >= 2, "drag_path needs at least two points");
        self.cx.simulate_mouse_move(points[0], None, Modifiers::default());
        self.cx.simulate_mouse_down(points[0], MouseButton::Left, Modifiers::default());
        for p in &points[1..] {
            self.cx.simulate_mouse_move(*p, MouseButton::Left, Modifiers::default());
        }
        self.cx.simulate_mouse_up(*points.last().unwrap(), MouseButton::Left, Modifiers::default());
        self.draw();
    }

    /// Scroll-wheel event at `position`, optionally with modifiers held.
    pub fn scroll(&mut self, position: Point<Pixels>, dy: f32, modifiers: Modifiers) {
        self.cx.simulate_mouse_move(position, None, modifiers);
        self.cx.simulate_event(ScrollWheelEvent {
            position,
            delta: ScrollDelta::Pixels(point(px(0.), px(dy))),
            modifiers,
            touch_phase: TouchPhase::Moved,
        });
        self.draw();
    }

    /// Run an `mkit-harness` input script. `@name` targets resolve through
    /// `debug_selector` bounds at the moment each command runs.
    pub fn run_script(&mut self, source: &str) {
        let script = mkit_harness::parse_script(source).expect("valid harness input script");
        for action in script.actions() {
            match action {
                mkit_harness::ScriptAction::Press(keys) => self.press(keys),
                mkit_harness::ScriptAction::Type(text) => self.type_text(text),
                mkit_harness::ScriptAction::Click(target) => self.click(target),
                mkit_harness::ScriptAction::Drag { from, to } => {
                    let from = self.target(from).center();
                    let to = self.target(to).center();
                    self.drag(from, to, 4);
                }
            }
        }
    }

    /// Handle of the window that shows the candidate root.
    pub fn main_window(&mut self) -> AnyWindowHandle {
        self.cx.update(|window, _| window.window_handle())
    }

    /// Every open window other than the root window, in GPUI's order.
    pub fn other_windows(&mut self) -> Vec<AnyWindowHandle> {
        self.cx.run_until_parked();
        let main = self.main_window();
        self.cx.windows().into_iter().filter(|handle| *handle != main).collect()
    }

    /// A visual test context for another window, drawn once so it can take focus.
    pub fn window_cx(&mut self, handle: AnyWindowHandle) -> VisualTestContext {
        let mut other = VisualTestContext::from_window(handle, &self.cx.cx);
        other.update(|window, cx| window.draw(cx).clear(cx));
        other.run_until_parked();
        other.update(|window, cx| window.draw(cx).clear(cx));
        other
    }

    /// The live accessibility tree, or why it could not be read.
    pub fn a11y(&mut self) -> Result<String, String> {
        self.draw();
        self.cx.update(|window, _| {
            mkit_harness::AccessibilitySnapshot::capture(window)
                .map(|snapshot| snapshot.as_text().to_owned())
                .map_err(|error| error.to_string())
        })
    }

    /// Assert on the accessibility tree when GPUI exposes one; otherwise
    /// record the criterion as skipped. Each needle must appear in the tree.
    pub fn check_a11y(&mut self, test: &str, needles: &[&str]) {
        match self.a11y() {
            Ok(tree) => {
                for needle in needles {
                    assert!(
                        tree.contains(needle),
                        "accessibility tree is missing `{needle}`:\n{tree}"
                    );
                }
            }
            Err(reason) => skip(test, &format!("accessibility tree unavailable: {reason}")),
        }
    }
}

/// Read a string field, failing with the snapshot in the message.
pub fn str_field<'a>(snapshot: &'a Value, key: &str) -> &'a str {
    snapshot
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("snapshot field `{key}` must be a string: {snapshot}"))
}

pub fn u64_field(snapshot: &Value, key: &str) -> u64 {
    snapshot
        .get(key)
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("snapshot field `{key}` must be an unsigned integer: {snapshot}"))
}

pub fn f64_field(snapshot: &Value, key: &str) -> f64 {
    snapshot
        .get(key)
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("snapshot field `{key}` must be a number: {snapshot}"))
}

pub fn bool_field(snapshot: &Value, key: &str) -> bool {
    snapshot
        .get(key)
        .and_then(Value::as_bool)
        .unwrap_or_else(|| panic!("snapshot field `{key}` must be a boolean: {snapshot}"))
}

pub fn array_field<'a>(snapshot: &'a Value, key: &str) -> &'a Vec<Value> {
    snapshot
        .get(key)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("snapshot field `{key}` must be an array: {snapshot}"))
}

pub fn approx(actual: f64, expected: f64, tolerance: f64) -> bool {
    (actual - expected).abs() <= tolerance
}

/// Headless screenshot checks. They run in a `harness = false` test binary
/// because the macOS renderer must be driven from the process main thread.
pub mod shots {
    use super::*;
    use gpui_pre::size;
    use image::RgbaImage;
    use mkit_harness::{HeadlessSession, ScreenshotError};

    pub const WIDTH: f32 = 800.;
    pub const HEIGHT: f32 = 600.;

    pub enum Outcome {
        Passed,
        Skipped(String),
    }

    pub type Check = fn() -> Result<Outcome, String>;

    /// Open the candidate root in an 800 x 600 headless window at scale 1.
    pub fn session(fixture: &Path) -> Result<HeadlessSession<Root>, Outcome> {
        HeadlessSession::new(
            bench_app::root(fixture),
            size(px(WIDTH), px(HEIGHT)),
            1.0,
            bench_app::init,
        )
        .map_err(|error| match error {
            ScreenshotError::UnsupportedPlatform => Outcome::Skipped(error.to_string()),
            other => Outcome::Skipped(format!("headless session failed: {other}")),
        })
    }

    pub fn capture(session: &mut HeadlessSession<Root>, name: &str) -> Result<RgbaImage, String> {
        let image = session.capture().map_err(|error| error.to_string())?;
        let path = artifacts_dir().join("screenshots").join(format!("{name}.png"));
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        image.save(&path).map_err(|e| e.to_string())?;
        Ok(image)
    }

    pub fn press(session: &mut HeadlessSession<Root>, keys: &str) -> Result<(), String> {
        session.simulate_keystrokes(keys).map_err(|error| error.to_string())
    }

    pub fn type_text(session: &mut HeadlessSession<Root>, text: &str) -> Result<(), String> {
        session.simulate_input(text).map_err(|error| error.to_string())
    }

    /// Number of distinct RGB colours, capped for speed.
    pub fn distinct_colors(image: &RgbaImage) -> usize {
        let mut seen = std::collections::HashSet::new();
        for pixel in image.pixels() {
            seen.insert([pixel[0], pixel[1], pixel[2]]);
            if seen.len() > 4096 {
                break;
            }
        }
        seen.len()
    }

    /// Mean relative luminance in 0.0..=1.0.
    pub fn mean_luminance(image: &RgbaImage) -> f64 {
        let total: f64 = image
            .pixels()
            .map(|p| 0.2126 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.0722 * p[2] as f64)
            .sum();
        total / (image.width() as f64 * image.height() as f64 * 255.0)
    }

    /// Fraction of pixels whose RGB differs by more than 8 in any channel.
    pub fn diff_fraction(a: &RgbaImage, b: &RgbaImage) -> f64 {
        if a.dimensions() != b.dimensions() {
            return 1.0;
        }
        let differing = a
            .pixels()
            .zip(b.pixels())
            .filter(|(x, y)| (0..3).any(|c| x[c].abs_diff(y[c]) > 8))
            .count();
        differing as f64 / (a.width() as f64 * a.height() as f64)
    }

    /// A rendered frame must show more than a flat background.
    pub fn require_content(image: &RgbaImage, name: &str) -> Result<(), String> {
        let expected = ((WIDTH as u32), (HEIGHT as u32));
        if image.dimensions() != expected {
            return Err(format!(
                "{name}: expected {expected:?} pixels, got {:?}",
                image.dimensions()
            ));
        }
        let colors = distinct_colors(image);
        if colors < 3 {
            return Err(format!("{name}: frame looks blank ({colors} distinct colours)"));
        }
        Ok(())
    }

    pub fn require_change(before: &RgbaImage, after: &RgbaImage, what: &str) -> Result<(), String> {
        let fraction = diff_fraction(before, after);
        if fraction <= 0.0 {
            return Err(format!("{what}: rendered frame did not change"));
        }
        Ok(())
    }

    /// Run checks, print one `MKIT_EVAL_RESULT` JSON line each, and exit
    /// non-zero when any check fails. Panics count as failures.
    pub fn run(checks: &[(&str, Check)]) {
        let mut failed = false;
        for (name, check) in checks {
            let outcome = std::panic::catch_unwind(check)
                .unwrap_or_else(|panic| Err(panic_message(panic.as_ref())));
            let (status, detail) = match outcome {
                Ok(Outcome::Passed) => ("passed", String::new()),
                Ok(Outcome::Skipped(reason)) => {
                    skip(name, &reason);
                    ("skipped", reason)
                }
                Err(message) => {
                    failed = true;
                    ("failed", message)
                }
            };
            println!(
                "MKIT_EVAL_RESULT {}",
                serde_json::json!({"test": name, "status": status, "detail": detail})
            );
        }
        if failed {
            std::process::exit(1);
        }
    }

    fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
        panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|s| (*s).to_owned()))
            .unwrap_or_else(|| "check panicked".into())
    }

    /// Convert a session-construction result into an early return.
    #[macro_export]
    macro_rules! session_or_skip {
        ($fixture:expr) => {
            match $crate::support::shots::session($fixture) {
                Ok(session) => session,
                Err($crate::support::shots::Outcome::Skipped(reason)) => {
                    return Ok($crate::support::shots::Outcome::Skipped(reason));
                }
                Err($crate::support::shots::Outcome::Passed) => unreachable!(),
            }
        };
    }
}
