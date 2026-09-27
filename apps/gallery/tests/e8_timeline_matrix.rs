use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Render, Window, div, prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    timeline::{
        Clip, ClipMoveRequested, Keyframe, KeyframeId, KeyframeMoveRequested, TimeRange, Timeline,
        TimelineObject, Track,
    },
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};

const SIZE: (f32, f32) = (960.0, 520.0);
const STATES: [&str; 8] = [
    "clip_move",
    "clip_move_committed",
    "clip_move_cancelled",
    "keyframe_move",
    "keyframe_move_committed",
    "keyframe_move_cancelled",
    "keyframe_selection",
    "keyframe_density",
];

struct TimelineFixture {
    state: &'static str,
    timeline: Option<Entity<Timeline>>,
}

impl Render for TimelineFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let timeline = self.timeline.get_or_insert_with(|| {
            cx.new(|cx| {
                let mut timeline = Timeline::controlled(
                    "Animation timeline",
                    vec![
                        Track::new(
                            1,
                            "Video",
                            vec![
                                Clip::new(11, 0.0, 4.5, "Opening"),
                                Clip::new(12, 5.0, 9.5, "City view"),
                            ],
                        ),
                        Track::new(2, "Music", vec![Clip::new(21, 0.0, 12.0, "Score")]),
                    ],
                    TimeRange::new(0.0, 12.0).expect("valid range"),
                );
                timeline.set_playhead(5.6, cx);
                timeline.set_selection(Some(TimelineObject::Clip(12)), cx);
                let keyframes = if self.state == "keyframe_density" {
                    (0..8)
                        .map(|index| {
                            Keyframe::new(
                                KeyframeId(201 + index),
                                TimelineObject::Clip(12),
                                5.6 + (index % 3) as f64 * 0.01,
                                format!("Cluster {}", index + 1),
                            )
                        })
                        .collect()
                } else {
                    vec![
                        Keyframe::new(KeyframeId(101), TimelineObject::Clip(12), 5.6, "Opacity"),
                        Keyframe::new(KeyframeId(102), TimelineObject::Clip(12), 7.25, "Opacity"),
                    ]
                };
                timeline.set_keyframes(keyframes, cx);
                timeline
            })
        });
        div()
            .id("timeline-matrix-fixture")
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .child(
                div()
                    .text_size(px(theme.typography.heading))
                    .child(format!("Timeline T3/T4 · {}", self.state.replace('_', " "))),
            )
            .child(timeline.clone())
    }
}

fn baseline_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/timeline/tests/baselines")
        .join(name)
}

fn theme(name: &str) -> Theme {
    match name {
        "light" => SHADCN_LIGHT,
        "dark" => SHADCN_DARK,
        "high-contrast" => HIGH_CONTRAST,
        other => panic!("unmapped Timeline theme: {other}"),
    }
}

fn accept_clip_move(session: &mut HeadlessSession<TimelineFixture>, request: ClipMoveRequested) {
    session
        .update(|root, _, cx| {
            let timeline = root.read(cx).timeline.as_ref().expect("timeline initialized").clone();
            timeline.update(cx, |view, cx| {
                let mut tracks = view.tracks().to_vec();
                for track in &mut tracks {
                    if let Some(clip) =
                        track.clips.iter_mut().find(|clip| clip.id == request.clip_id)
                    {
                        let duration = clip.end - clip.start;
                        clip.start = request.start;
                        clip.end = request.start + duration;
                    }
                }
                view.set_tracks(tracks, cx);
            });
        })
        .expect("apply clip move");
}

fn accept_keyframe_move(
    session: &mut HeadlessSession<TimelineFixture>,
    request: KeyframeMoveRequested,
) {
    session
        .update(|root, _, cx| {
            let timeline = root.read(cx).timeline.as_ref().expect("timeline initialized").clone();
            timeline.update(cx, |view, cx| {
                let mut keyframes = view.keyframes().to_vec();
                if let Some(keyframe) =
                    keyframes.iter_mut().find(|keyframe| keyframe.id == request.keyframe_id)
                {
                    keyframe.time = request.time;
                }
                view.set_keyframes(keyframes, cx);
            });
        })
        .expect("apply keyframe move");
}

fn capture(
    state: &'static str,
    theme_value: Theme,
    scale: u32,
) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        TimelineFixture { state, timeline: None },
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(mkit::timeline::default_key_bindings());
        },
    )?;

    let clip_requests = Rc::new(RefCell::new(Vec::new()));
    let clip_log = clip_requests.clone();
    let keyframe_requests = Rc::new(RefCell::new(Vec::new()));
    let keyframe_log = keyframe_requests.clone();
    let _clip_subscription = session.update(|root, _, cx| {
        let timeline = root.read(cx).timeline.as_ref().expect("timeline initialized").clone();
        cx.subscribe(&timeline, move |_, event: &ClipMoveRequested, _| {
            clip_log.borrow_mut().push(*event)
        })
    })?;
    let _keyframe_subscription = session.update(|root, _, cx| {
        let timeline = root.read(cx).timeline.as_ref().expect("timeline initialized").clone();
        cx.subscribe(&timeline, move |_, event: &KeyframeMoveRequested, _| {
            keyframe_log.borrow_mut().push(*event)
        })
    })?;

    session.update(|root, window, cx| {
        let timeline = root.read(cx).timeline.as_ref().expect("timeline initialized").clone();
        timeline.update(cx, |timeline, cx| timeline.focus_handle(cx).focus(window, cx));
    })?;
    match state {
        "clip_move" => session.simulate_keystrokes("space shift-right")?,
        "clip_move_committed" => {
            session.simulate_keystrokes("space shift-right enter")?;
            assert_eq!(clip_requests.borrow().len(), 1, "Enter commits the clip move");
            accept_clip_move(&mut session, clip_requests.borrow_mut().pop().unwrap());
        }
        "clip_move_cancelled" => {
            session.simulate_keystrokes("space shift-right escape")?;
            assert!(clip_requests.borrow().is_empty(), "Escape must not commit the clip move");
            session.simulate_keystrokes("enter")?;
            assert!(clip_requests.borrow().is_empty(), "cancelled preview stays cleared");
        }
        "keyframe_move" => session.simulate_keystrokes("ctrl-shift-right n shift-right")?,
        "keyframe_move_committed" => {
            session.simulate_keystrokes("ctrl-shift-right n shift-right enter")?;
            assert_eq!(keyframe_requests.borrow().len(), 1, "Enter commits the keyframe move");
            accept_keyframe_move(&mut session, keyframe_requests.borrow_mut().pop().unwrap());
        }
        "keyframe_move_cancelled" => {
            session.simulate_keystrokes("ctrl-shift-right n shift-right escape")?;
            assert!(
                keyframe_requests.borrow().is_empty(),
                "Escape must not commit the keyframe move"
            );
            session.simulate_keystrokes("enter")?;
            assert!(keyframe_requests.borrow().is_empty(), "cancelled preview stays cleared");
        }
        "keyframe_selection" => {
            session.simulate_keystrokes("ctrl-shift-right")?;
            session.update(|root, _, cx| {
                let timeline = root.read(cx).timeline.as_ref().expect("timeline initialized");
                assert_eq!(timeline.read(cx).selected_keyframe(), Some(KeyframeId(101)));
            })?;
        }
        "keyframe_density" => {
            session.simulate_keystrokes("ctrl-shift-right")?;
            session.update(|root, _, cx| {
                let timeline = root.read(cx).timeline.as_ref().expect("timeline initialized");
                assert_eq!(timeline.read(cx).selected_keyframe(), Some(KeyframeId(201)));
            })?;
        }
        other => panic!("unmapped Timeline state: {other}"),
    }

    if state == "clip_move" {
        let image = session.capture()?;
        session.simulate_keystrokes("enter")?;
        assert_eq!(clip_requests.borrow().len(), 1, "the preview is a real keyboard edit");
        return Ok(image);
    }
    if state == "keyframe_move" {
        let image = session.capture()?;
        session.simulate_keystrokes("enter")?;
        assert_eq!(keyframe_requests.borrow().len(), 1, "the preview is a real keyboard edit");
        return Ok(image);
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    let path = std::env::var_os("SNAPSHOT_CANDIDATE_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(baseline))
        .unwrap_or_else(|| baseline_path(baseline));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write Timeline baseline");
        println!("Updated Timeline screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing Timeline baseline {}: {error}", path.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 32 };
    if !tolerance.matches(&expected, actual) {
        let diff = path.with_file_name(format!(
            "{}-diff.png",
            path.file_stem().expect("baseline stem").to_string_lossy()
        ));
        let mut image = RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let left = expected.get_pixel(x, y);
            let right = actual.get_pixel(x, y);
            *pixel = image::Rgba([
                left[0].abs_diff(right[0]).saturating_mul(4),
                left[1].abs_diff(right[1]).saturating_mul(4),
                left[2].abs_diff(right[2]).saturating_mul(4),
                255,
            ]);
        }
        image.save(&diff).expect("write Timeline diff");
        panic!("Timeline screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched Timeline screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/timeline/tests/conformance.json"))
            .expect("Timeline manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    let selected_state = std::env::var("UPDATE_SNAPSHOT_STATE").ok();
    let updating =
        std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1"));
    let focused_cases: Vec<_> = cases
        .iter()
        .filter(|case| STATES.contains(&case["state"].as_str().expect("state")))
        .collect();
    assert_eq!(
        focused_cases.len(),
        STATES.len() * 6,
        "eight T3/T4 states × three themes × two scales"
    );
    for case in focused_cases {
        let state_value = case["state"].as_str().expect("state");
        let state =
            *STATES.iter().find(|candidate| **candidate == state_value).expect("known state");
        let fixture = format!("{state}_fixture");
        assert_eq!(case["load_fixture"].as_str(), Some(fixture.as_str()));
        if updating && selected_state.as_deref().is_some_and(|selected| selected != state) {
            continue;
        }
        let theme_name = case["theme"].as_str().expect("theme");
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(state, theme(theme_name), scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, baseline);
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping Timeline screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("Timeline screenshot matrix failed: {error}"),
    }
}
