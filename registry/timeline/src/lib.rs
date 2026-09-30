//! App-supplied timeline tracks and clips with a zoomable ruler, playhead, and selection.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Bounds, Context, DispatchPhase, EventEmitter, FocusHandle, Focusable, FontWeight,
    IntoElement, KeyBinding, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PathBuilder, Pixels, Render, Rgba, ScrollDelta, ScrollHandle, ScrollWheelEvent, Window,
    actions, canvas, div, point, prelude::*, px, relative,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

pub const KEY_CONTEXT: &str = "MkitTimeline";
pub const TRACK_LABEL_WIDTH_MULTIPLIER: f32 = 3.0;
pub const VISIBLE_TRACK_ROWS: f32 = 8.0;
pub const TIME_PADDING_FRACTION: f64 = 0.05;
const ZOOM_FACTOR: f64 = 1.25;
const PAN_FRACTION: f64 = 0.1;
const MIN_TICK_SPACING: f64 = 72.0;
const MAX_TICKS: usize = 512;
const KEYFRAME_STACK_ROWS: usize = 3;
pub const SNAP_TOLERANCE_PX: f64 = 8.0;
pub const PLAYHEAD_STEP_SECONDS: f64 = 0.1;

/// Colours derived from theme tokens; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    high_contrast: bool,
    /// shadcn "muted": ruler fill and hover fill.
    muted: Rgba,
    /// Half-strength muted fill behind a track's keyframe marker band.
    band: Rgba,
    /// Frame border, dividers, ticks, and the resting clip border.
    divider: Rgba,
    /// Opaque outline-button border.
    control_border: Rgba,
    selected_track_bg: Rgba,
    selected_track_text: Rgba,
    /// Clip fill, label, and outline: shadcn's secondary button look.
    clip_bg: Rgba,
    clip_text: Rgba,
    clip_border: Rgba,
    /// Outline of the selected clip, drawn inside its focus ring.
    clip_selected_border: Rgba,
    ring: Rgba,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            high_contrast: true,
            muted: c.background,
            band: c.background,
            divider: c.border,
            control_border: c.border,
            selected_track_bg: c.accent,
            selected_track_text: c.accent_text,
            clip_bg: c.accent,
            clip_text: c.accent_text,
            clip_border: c.border,
            clip_selected_border: c.focus,
            ring: c.focus,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    Look {
        high_contrast: false,
        muted,
        band: mix(c.text, c.background, if dark { 0.06 } else { 0.02 }),
        divider: if dark { c.text.opacity(0.1) } else { c.border },
        control_border: if dark { mix(c.text, c.background, 0.1) } else { c.border },
        selected_track_bg: muted,
        selected_track_text: c.text,
        clip_bg: muted,
        clip_text: c.text,
        clip_border: if dark { mix(c.text, c.background, 0.1) } else { c.border },
        clip_selected_border: c.text,
        ring: c.focus.opacity(0.5),
    }
}
fn box_shadow(shadow: ShadowToken) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: shadow.color.into(),
        offset: point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}
/// shadcn/ui focus ring width, drawn outside the focused or selected element.
const FOCUS_RING_WIDTH: f32 = 3.0;
fn focus_ring(color: Rgba) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: color.into(),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(FOCUS_RING_WIDTH),
        inset: false,
    }
}
/// Decorative Lucide icon drawn as a vector stroke so it stays crisp at every scale. Each
/// polyline is a list of points on a 24-unit grid; the stroke is 2 units, Lucide's default.
fn icon(size: f32, lines: &'static [&'static [(f32, f32)]], color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let origin = bounds.origin;
            let mut path = PathBuilder::stroke(unit * 2.0);
            for line in lines {
                for (i, (x, y)) in line.iter().enumerate() {
                    let at = origin + point(unit * *x, unit * *y);
                    if i == 0 { path.move_to(at) } else { path.line_to(at) }
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
}
/// Lucide `plus`.
const PLUS: &[&[(f32, f32)]] = &[&[(5., 12.), (19., 12.)], &[(12., 5.), (12., 19.)]];
/// Lucide `minus`.
const MINUS: &[&[(f32, f32)]] = &[&[(5., 12.), (19., 12.)]];
/// Lucide `check`.
const CHECK: &[&[(f32, f32)]] = &[&[(20., 6.), (9., 17.), (4., 12.)]];

/// A diamond through the four points `half` away from `centre`.
fn diamond(
    mut path: PathBuilder,
    centre: gpui_pre::Point<Pixels>,
    half: Pixels,
) -> Option<gpui_pre::Path<Pixels>> {
    path.move_to(centre - point(px(0.), half));
    path.line_to(centre + point(half, px(0.)));
    path.line_to(centre + point(px(0.), half));
    path.line_to(centre - point(half, px(0.)));
    path.close();
    path.build().ok()
}
/// A keyframe marker drawn as a vector diamond in the restyled Slider thumb look: an opaque
/// `background` fill, a hairline `accent` border, and the small shadow. A selected keyframe is
/// filled with `accent` and gains the focus ring in place of the shadow.
fn keyframe_diamond(theme: Theme, look: Look, selected: bool) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let c = theme.colors;
            let centre = bounds.center();
            let half = px(theme.spacing.large / 2.0);
            if selected {
                // The ring's perpendicular width is `FOCUS_RING_WIDTH`; a diamond's half-diagonal
                // grows by that width times the square root of two.
                let ring = half + px(FOCUS_RING_WIDTH * std::f32::consts::SQRT_2);
                if let Some(path) = diamond(PathBuilder::fill(), centre, ring) {
                    window.paint_path(path, look.ring);
                }
            } else {
                let shadow = theme.shadows.small;
                let offset = centre + point(px(shadow.x), px(shadow.y));
                if let Some(path) =
                    diamond(PathBuilder::fill(), offset, half + px(shadow.blur / 2.0))
                {
                    window.paint_path(path, shadow.color);
                }
            }
            if let Some(path) = diamond(PathBuilder::fill(), centre, half) {
                window.paint_path(path, if selected { c.accent } else { c.background });
            }
            let border = PathBuilder::stroke(px(theme.borders.hairline));
            if let Some(path) = diamond(border, centre, half) {
                window.paint_path(path, c.accent);
            }
        },
    )
    .absolute()
    .inset_0()
}

actions!(
    timeline,
    [
        ZoomIn,
        ZoomOut,
        PanEarlier,
        PanLater,
        FitRange,
        StepEarlier,
        StepLater,
        PreviousObject,
        NextObject,
        PickUpClip,
        MoveEarlier,
        MoveLater,
        MoveTrackUp,
        MoveTrackDown,
        CommitEdit,
        CancelEdit,
        ToggleSnapBypass,
        PreviousKeyframe,
        NextKeyframe,
        PickUpKeyframe
    ]
);

pub fn default_key_bindings() -> [KeyBinding; 20] {
    [
        KeyBinding::new("=", ZoomIn, Some(KEY_CONTEXT)),
        KeyBinding::new("-", ZoomOut, Some(KEY_CONTEXT)),
        KeyBinding::new("left", PanEarlier, Some(KEY_CONTEXT)),
        KeyBinding::new("right", PanLater, Some(KEY_CONTEXT)),
        KeyBinding::new("f", FitRange, Some(KEY_CONTEXT)),
        KeyBinding::new(",", StepEarlier, Some(KEY_CONTEXT)),
        KeyBinding::new(".", StepLater, Some(KEY_CONTEXT)),
        KeyBinding::new("up", PreviousObject, Some(KEY_CONTEXT)),
        KeyBinding::new("down", NextObject, Some(KEY_CONTEXT)),
        KeyBinding::new("space", PickUpClip, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", CommitEdit, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", CancelEdit, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-s", ToggleSnapBypass, Some(KEY_CONTEXT)),
        KeyBinding::new("n", PickUpKeyframe, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-left", MoveEarlier, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-right", MoveLater, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-up", MoveTrackUp, Some(KEY_CONTEXT)),
        KeyBinding::new("alt-down", MoveTrackDown, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-shift-left", PreviousKeyframe, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-shift-right", NextKeyframe, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyframeId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe {
    pub id: KeyframeId,
    pub owner: TimelineObject,
    pub time: f64,
    pub label: String,
}

impl Keyframe {
    pub fn new(id: KeyframeId, owner: TimelineObject, time: f64, label: impl Into<String>) -> Self {
        Self { id, owner, time, label: label.into() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipMoveRequested {
    pub clip_id: ClipId,
    pub track_id: TrackId,
    pub start: f64,
}
impl EventEmitter<ClipMoveRequested> for Timeline {}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyframeMoveRequested {
    pub keyframe_id: KeyframeId,
    pub time: f64,
}
impl EventEmitter<KeyframeMoveRequested> for Timeline {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyframeSelectionChanged {
    pub selected: Option<KeyframeId>,
}
impl EventEmitter<KeyframeSelectionChanged> for Timeline {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SnapCandidate {
    pub time: f64,
    pub priority: u8,
}

/// Snap to the nearest candidate within a screen-space tolerance. Ties use priority, then time.
pub fn snap_time(
    time: f64,
    range: TimeRange,
    width: f64,
    candidates: &[SnapCandidate],
    bypass: bool,
) -> f64 {
    if bypass || !time.is_finite() || !width.is_finite() || width <= 0.0 {
        return time;
    }
    let px_per_second = width / range.duration();
    candidates
        .iter()
        .filter(|c| c.time.is_finite())
        .filter_map(|c| {
            let distance = (c.time - time).abs() * px_per_second;
            (distance <= SNAP_TOLERANCE_PX).then_some((distance, c.priority, c.time))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.total_cmp(&b.2)))
        .map_or(time, |candidate| candidate.2)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeRange {
    start: f64,
    end: f64,
}

impl TimeRange {
    pub fn new(start: f64, end: f64) -> Option<Self> {
        (start.is_finite() && end.is_finite() && start < end && (end - start).is_finite())
            .then_some(Self { start, end })
    }

    pub fn start(self) -> f64 {
        self.start
    }
    pub fn end(self) -> f64 {
        self.end
    }

    pub fn duration(self) -> f64 {
        self.end - self.start
    }
}

pub type TrackId = u64;
pub type ClipId = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimelineObject {
    Track(TrackId),
    Clip(ClipId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Clip {
    pub id: ClipId,
    pub start: f64,
    pub end: f64,
    pub label: String,
}

impl Clip {
    pub fn new(id: ClipId, start: f64, end: f64, label: impl Into<String>) -> Self {
        Self { id, start, end, label: label.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub id: TrackId,
    pub label: String,
    pub clips: Vec<Clip>,
}

impl Track {
    pub fn new(id: TrackId, label: impl Into<String>, clips: Vec<Clip>) -> Self {
        Self { id, label: label.into(), clips: normalize_clips(clips) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisibleRangeChanged {
    pub start: f64,
    pub end: f64,
}

impl EventEmitter<VisibleRangeChanged> for Timeline {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayheadChanged {
    pub time: f64,
}

impl EventEmitter<PlayheadChanged> for Timeline {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionChanged {
    pub selected: Option<TimelineObject>,
}

impl EventEmitter<SelectionChanged> for Timeline {}

/// A read-only presentation of app-owned tracks and clips.
pub struct Timeline {
    label: String,
    tracks: Vec<Track>,
    visible_range: TimeRange,
    controlled: bool,
    playhead: f64,
    selected: Option<TimelineObject>,
    keyframes: Vec<Keyframe>,
    selected_keyframe: Option<KeyframeId>,
    edit: Option<EditPreview>,
    snap_bypass: bool,
    scrubbing: bool,
    focus: Option<FocusHandle>,
    scroll: ScrollHandle,
    time_surface_bounds: std::rc::Rc<Cell<Option<Bounds<Pixels>>>>,
    lane_bounds: std::rc::Rc<RefCell<HashMap<TrackId, Bounds<Pixels>>>>,
    dragging: Option<(DragTarget, f64)>,
    playhead_bounds: std::rc::Rc<Cell<Option<Bounds<Pixels>>>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum EditPreview {
    Clip { id: ClipId, track_id: TrackId, start: f64 },
    Keyframe { id: KeyframeId, time: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragTarget {
    Clip(ClipId),
    Keyframe(KeyframeId),
}

impl Timeline {
    pub fn new(label: impl Into<String>, tracks: Vec<Track>, visible_range: TimeRange) -> Self {
        Self::base(label, tracks, visible_range, false)
    }

    pub fn controlled(
        label: impl Into<String>,
        tracks: Vec<Track>,
        visible_range: TimeRange,
    ) -> Self {
        Self::base(label, tracks, visible_range, true)
    }

    fn base(
        label: impl Into<String>,
        tracks: Vec<Track>,
        visible_range: TimeRange,
        controlled: bool,
    ) -> Self {
        Self {
            label: label.into(),
            tracks: normalize_tracks(tracks),
            visible_range,
            controlled,
            playhead: visible_range.start,
            selected: None,
            keyframes: Vec::new(),
            selected_keyframe: None,
            edit: None,
            snap_bypass: false,
            scrubbing: false,
            focus: None,
            scroll: ScrollHandle::new(),
            time_surface_bounds: Default::default(),
            lane_bounds: Default::default(),
            dragging: None,
            playhead_bounds: Default::default(),
        }
    }

    pub fn visible_range(&self) -> TimeRange {
        self.visible_range
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }
    pub fn selected_keyframe(&self) -> Option<KeyframeId> {
        self.selected_keyframe
    }
    pub fn set_keyframes(&mut self, keyframes: Vec<Keyframe>, cx: &mut Context<Self>) {
        self.keyframes = keyframes
            .into_iter()
            .filter(|k| k.time.is_finite() && self.contains_object(k.owner))
            .collect();
        self.keyframes.sort_by(|a, b| a.time.total_cmp(&b.time).then(a.id.cmp(&b.id)));
        if self.selected_keyframe.is_some_and(|id| !self.keyframes.iter().any(|k| k.id == id)) {
            self.selected_keyframe = None;
        }
        if matches!(self.edit, Some(EditPreview::Keyframe { id, .. }) if !self.keyframes.iter().any(|k| k.id == id))
        {
            self.edit = None;
        }
        cx.notify();
    }

    pub fn request_clip_move(
        &mut self,
        clip_id: ClipId,
        track_id: TrackId,
        start: f64,
        cx: &mut Context<Self>,
    ) {
        if !start.is_finite() || !self.tracks.iter().any(|t| t.id == track_id) {
            return;
        }
        let Some(clip) = self.tracks.iter().flat_map(|t| &t.clips).find(|c| c.id == clip_id) else {
            return;
        };
        if !(start + (clip.end - clip.start)).is_finite() {
            return;
        }
        let _ = clip;
        let width = self
            .time_surface_bounds
            .get()
            .map(|b| f64::from(b.size.width.as_f32()))
            .unwrap_or(700.0);
        let candidates = self.snap_candidates();
        let start = snap_time(start, self.visible_range, width, &candidates, self.snap_bypass);
        self.edit = Some(EditPreview::Clip { id: clip_id, track_id, start });
        cx.notify();
    }
    pub fn request_keyframe_move(&mut self, id: KeyframeId, time: f64, cx: &mut Context<Self>) {
        if !time.is_finite() || !self.keyframes.iter().any(|k| k.id == id) {
            return;
        }
        let width = self
            .time_surface_bounds
            .get()
            .map(|b| f64::from(b.size.width.as_f32()))
            .unwrap_or(700.0);
        let candidates = self.snap_candidates();
        let time = snap_time(time, self.visible_range, width, &candidates, self.snap_bypass);
        self.edit = Some(EditPreview::Keyframe { id, time });
        cx.notify();
    }
    fn snap_candidates(&self) -> Vec<SnapCandidate> {
        let width = self
            .time_surface_bounds
            .get()
            .map(|b| f64::from(b.size.width.as_f32()))
            .unwrap_or(700.0);
        let mut candidates: Vec<_> = ruler_ticks(self.visible_range, width, MIN_TICK_SPACING)
            .into_iter()
            .map(|time| SnapCandidate { time, priority: 0 })
            .collect();
        candidates.push(SnapCandidate { time: self.playhead, priority: 1 });
        for clip in self.tracks.iter().flat_map(|t| &t.clips) {
            candidates.push(SnapCandidate { time: clip.start, priority: 2 });
            candidates.push(SnapCandidate { time: clip.end, priority: 2 });
        }
        candidates
    }
    pub fn cancel_edit(&mut self, cx: &mut Context<Self>) {
        self.dragging = None;
        if self.edit.take().is_some() {
            cx.notify();
        }
    }
    pub fn commit_edit(&mut self, cx: &mut Context<Self>) {
        let Some(edit) = self.edit.take() else {
            return;
        };
        match edit {
            EditPreview::Clip { id, track_id, start } => {
                if let Some((original_track_id, clip)) = self.tracks.iter().find_map(|track| {
                    track.clips.iter().find(|c| c.id == id).map(|c| (track.id, c.clone()))
                }) && (clip.start != start || original_track_id != track_id)
                {
                    if !self.controlled {
                        if let Some(track) =
                            self.tracks.iter_mut().find(|track| track.id == original_track_id)
                        {
                            track.clips.retain(|candidate| candidate.id != id);
                        }
                        if let Some(track) =
                            self.tracks.iter_mut().find(|track| track.id == track_id)
                        {
                            let mut moved = clip;
                            let duration = moved.end - moved.start;
                            moved.start = start;
                            moved.end = start + duration;
                            track.clips.push(moved);
                            track.clips = normalize_clips(std::mem::take(&mut track.clips));
                        }
                    }
                    cx.emit(ClipMoveRequested { clip_id: id, track_id, start });
                }
            }
            EditPreview::Keyframe { id, time } => {
                if self.keyframes.iter().any(|k| k.id == id && k.time != time) {
                    if !self.controlled
                        && let Some(keyframe) = self.keyframes.iter_mut().find(|k| k.id == id)
                    {
                        keyframe.time = time;
                    }
                    cx.emit(KeyframeMoveRequested { keyframe_id: id, time });
                }
            }
        }
        cx.notify();
    }
    pub fn toggle_snap_bypass(&mut self) {
        self.snap_bypass = !self.snap_bypass;
    }

    pub fn playhead(&self) -> f64 {
        self.playhead
    }

    pub fn selection(&self) -> Option<TimelineObject> {
        self.selected
    }

    /// Apply an owner playback-clock value without emitting a user event.
    pub fn set_playhead(&mut self, time: f64, cx: &mut Context<Self>) {
        if time.is_finite() && self.playhead != time {
            self.playhead = time;
            cx.notify();
        }
    }

    /// Apply an owner selection if the object exists. This setter does not emit.
    pub fn set_selection(&mut self, selected: Option<TimelineObject>, cx: &mut Context<Self>) {
        if selected.is_some_and(|object| !self.contains_object(object)) {
            return;
        }
        if self.selected != selected {
            self.selected = selected;
            cx.notify();
        }
    }

    fn contains_object(&self, object: TimelineObject) -> bool {
        match object {
            TimelineObject::Track(id) => self.tracks.iter().any(|track| track.id == id),
            TimelineObject::Clip(id) => {
                self.tracks.iter().any(|track| track.clips.iter().any(|clip| clip.id == id))
            }
        }
    }

    fn objects(&self) -> Vec<TimelineObject> {
        self.tracks
            .iter()
            .flat_map(|track| {
                std::iter::once(TimelineObject::Track(track.id))
                    .chain(track.clips.iter().map(|clip| TimelineObject::Clip(clip.id)))
            })
            .collect()
    }

    fn request_selection(&mut self, selected: Option<TimelineObject>, cx: &mut Context<Self>) {
        let selected = selected.filter(|object| self.contains_object(*object));
        if self.selected == selected {
            return;
        }
        if !self.controlled {
            self.selected = selected;
            cx.notify();
        }
        cx.emit(SelectionChanged { selected });
    }

    fn request_playhead(&mut self, time: f64, cx: &mut Context<Self>) {
        if !time.is_finite() {
            return;
        }
        let time = time.clamp(self.visible_range.start, self.visible_range.end);
        if self.playhead == time {
            return;
        }
        if !self.controlled {
            self.playhead = time;
            cx.notify();
        }
        cx.emit(PlayheadChanged { time });
    }

    fn seek_from_pointer(&mut self, x: f32, bounds: Bounds<Pixels>, cx: &mut Context<Self>) {
        let width = f32::from(bounds.size.width);
        if width <= 0.0 {
            return;
        }
        let local_x = x - f32::from(bounds.origin.x);
        if let Some(time) = pixel_to_time(local_x, self.visible_range, width) {
            self.request_playhead(time, cx);
        }
    }

    fn pointer_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != MouseButton::Left {
            return;
        }
        let Some(bounds) = self.playhead_bounds.get() else { return };
        self.scrubbing = true;
        if let Some(focus) = &self.focus {
            window.focus(focus, cx);
        }
        self.seek_from_pointer(f32::from(event.position.x), bounds, cx);
    }

    fn time_at_pointer(&self, x: f32, bounds: Bounds<Pixels>) -> Option<f64> {
        let width = f32::from(bounds.size.width);
        if width <= 0.0 {
            return None;
        }
        pixel_to_time(x - f32::from(bounds.origin.x), self.visible_range, width)
    }

    fn begin_clip_drag(&mut self, id: ClipId, track_id: TrackId, event: &MouseDownEvent) {
        let Some(bounds) = self.lane_bounds.borrow().get(&track_id).copied() else {
            return;
        };
        let Some(pointer_time) = self.time_at_pointer(f32::from(event.position.x), bounds) else {
            return;
        };
        let Some(clip) = self.tracks.iter().flat_map(|t| &t.clips).find(|c| c.id == id) else {
            return;
        };
        self.dragging = Some((DragTarget::Clip(id), pointer_time - clip.start));
    }

    fn begin_keyframe_drag(&mut self, id: KeyframeId, event: &MouseDownEvent) {
        let Some(keyframe) = self.keyframes.iter().find(|k| k.id == id) else {
            return;
        };
        let owner = keyframe.owner;
        let track_id = match owner {
            TimelineObject::Track(id) => id,
            TimelineObject::Clip(id) => self
                .tracks
                .iter()
                .find(|t| t.clips.iter().any(|c| c.id == id))
                .map(|t| t.id)
                .unwrap_or(0),
        };
        let Some(bounds) = self.lane_bounds.borrow().get(&track_id).copied() else {
            return;
        };
        let Some(pointer_time) = self.time_at_pointer(f32::from(event.position.x), bounds) else {
            return;
        };
        self.dragging = Some((DragTarget::Keyframe(id), pointer_time - keyframe.time));
    }

    fn pointer_drag_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if !event.dragging() {
            return;
        }
        self.preview_drag_at(event.position, cx);
    }

    fn preview_drag_at(
        &mut self,
        position: gpui_pre::Point<Pixels>,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some((drag, offset)) = self.dragging else {
            return false;
        };
        let bounds_by_track = self.lane_bounds.borrow();
        let Some((track_id, bounds)) =
            bounds_by_track.iter().find(|(_, bounds)| bounds.contains(&position))
        else {
            return false;
        };
        let track_id = *track_id;
        let Some(time) =
            self.time_at_pointer(f32::from(position.x), *bounds).map(|time| time - offset)
        else {
            return false;
        };
        drop(bounds_by_track);
        match drag {
            DragTarget::Clip(id) => self.request_clip_move(id, track_id, time, cx),
            DragTarget::Keyframe(id) => self.request_keyframe_move(id, time, cx),
        }
        true
    }

    fn pointer_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        self.pointer_drag_move(event, cx);
        if self.scrubbing
            && event.dragging()
            && let Some(bounds) = self.playhead_bounds.get()
        {
            self.seek_from_pointer(f32::from(event.position.x), bounds, cx);
        }
    }

    fn pointer_up(&mut self, event: &MouseUpEvent, cx: &mut Context<Self>) {
        if event.button == MouseButton::Left {
            self.scrubbing = false;
            if self.dragging.is_some() {
                let valid_destination = self.preview_drag_at(event.position, cx);
                self.dragging = None;
                if valid_destination {
                    self.commit_edit(cx);
                } else {
                    self.cancel_edit(cx);
                }
            }
        }
    }

    fn step_playhead(&mut self, direction: f64, cx: &mut Context<Self>) {
        self.request_playhead(self.playhead + PLAYHEAD_STEP_SECONDS * direction, cx);
    }

    fn navigate_selection(&mut self, direction: isize, cx: &mut Context<Self>) {
        let objects = self.objects();
        if objects.is_empty() {
            return;
        }
        let current = self
            .selected
            .and_then(|selected| objects.iter().position(|object| *object == selected));
        let index = match (current, direction.signum()) {
            (Some(index), -1) => index.saturating_sub(1),
            (Some(index), 1) => (index + 1).min(objects.len() - 1),
            (None, -1) => objects.len() - 1,
            _ => 0,
        };
        self.request_selection(Some(objects[index]), cx);
    }

    /// Apply an owner-approved range. This setter does not emit a user event.
    pub fn set_visible_range(&mut self, range: TimeRange, cx: &mut Context<Self>) {
        if self.visible_range != range {
            self.visible_range = range;
            cx.notify();
        }
    }

    /// Replace app-owned track data without changing or emitting the view range.
    pub fn set_tracks(&mut self, tracks: Vec<Track>, cx: &mut Context<Self>) {
        self.tracks = normalize_tracks(tracks);
        let tracks = &self.tracks;
        self.keyframes.retain(|keyframe| contains_in_tracks(tracks, keyframe.owner));
        if self
            .selected_keyframe
            .is_some_and(|id| !self.keyframes.iter().any(|keyframe| keyframe.id == id))
        {
            self.selected_keyframe = None;
        }
        if matches!(self.edit, Some(EditPreview::Keyframe { id, .. }) if !self.keyframes.iter().any(|keyframe| keyframe.id == id))
        {
            self.edit = None;
            self.dragging = None;
        }
        if matches!(self.edit, Some(EditPreview::Clip { id, .. }) if !self.tracks.iter().any(|t| t.clips.iter().any(|c| c.id == id)))
        {
            self.edit = None;
        }
        if self.selected.is_some_and(|selected| !self.contains_object(selected)) {
            self.selected = None;
        }
        cx.notify();
    }

    pub fn is_controlled(&self) -> bool {
        self.controlled
    }

    pub fn fit_range(&mut self, cx: &mut Context<Self>) {
        if let Some((start, end)) = content_extent(&self.tracks) {
            let span = end - start;
            let padding = span * TIME_PADDING_FRACTION;
            if let Some(range) = TimeRange::new(start - padding, end + padding) {
                self.propose_range(range, cx);
            }
        }
    }

    pub fn zoom_at_fraction(&mut self, factor: f64, anchor: f64, cx: &mut Context<Self>) {
        if !factor.is_finite() || factor <= 0.0 || !anchor.is_finite() {
            return;
        }
        let range = self.visible_range;
        let factor = factor.clamp(0.05, 20.0);
        let start = anchor + (range.start - anchor) / factor;
        let end = anchor + (range.end - anchor) / factor;
        if let Some(next) = TimeRange::new(start, end) {
            self.propose_range(next, cx);
        }
    }

    pub fn pan_by(&mut self, seconds: f64, cx: &mut Context<Self>) {
        if !seconds.is_finite() {
            return;
        }
        let range = self.visible_range;
        if let Some(next) = TimeRange::new(range.start + seconds, range.end + seconds) {
            self.propose_range(next, cx);
        }
    }

    fn zoom_center(&mut self, factor: f64, cx: &mut Context<Self>) {
        let anchor = (self.visible_range.start + self.visible_range.end) / 2.0;
        self.zoom_at_fraction(factor, anchor, cx);
    }

    fn pan_fraction(&mut self, direction: f64, cx: &mut Context<Self>) {
        if let Some(EditPreview::Keyframe { id, time }) = self.edit {
            self.nudge_keyframe(id, time, -direction, cx);
            return;
        }
        if let Some(EditPreview::Clip { id, track_id, start }) = self.edit {
            let step = tick_step(self.visible_range, 800.0, MIN_TICK_SPACING).unwrap_or(0.1);
            self.request_clip_move(id, track_id, start + step * direction, cx);
            return;
        }
        self.pan_by(self.visible_range.duration() * PAN_FRACTION * direction, cx);
    }

    fn nudge_keyframe(
        &mut self,
        id: KeyframeId,
        time: f64,
        direction: f64,
        cx: &mut Context<Self>,
    ) {
        let step = tick_step(self.visible_range, 800.0, MIN_TICK_SPACING).unwrap_or(0.1);
        self.request_keyframe_move(id, time + step * direction, cx);
    }
    fn on_pick_up_clip(&mut self, _: &PickUpClip, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(TimelineObject::Clip(id)) = self.selected
            && let Some((track_id, clip)) = self
                .tracks
                .iter()
                .find_map(|t| t.clips.iter().find(|c| c.id == id).map(|c| (t.id, c)))
        {
            self.request_clip_move(id, track_id, clip.start, cx);
        }
    }
    fn on_pick_up_keyframe(&mut self, _: &PickUpKeyframe, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_keyframe
            && let Some(k) = self.keyframes.iter().find(|k| k.id == id)
        {
            self.request_keyframe_move(id, k.time, cx);
        }
    }
    fn on_commit_edit(&mut self, _: &CommitEdit, _: &mut Window, cx: &mut Context<Self>) {
        self.commit_edit(cx);
    }
    fn on_cancel_edit(&mut self, _: &CancelEdit, _: &mut Window, cx: &mut Context<Self>) {
        self.cancel_edit(cx);
    }
    fn on_toggle_snap(&mut self, _: &ToggleSnapBypass, _: &mut Window, _: &mut Context<Self>) {
        self.toggle_snap_bypass();
    }
    fn on_previous_keyframe(
        &mut self,
        _: &PreviousKeyframe,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate_keyframe(-1, cx);
    }
    fn on_next_keyframe(&mut self, _: &NextKeyframe, _: &mut Window, cx: &mut Context<Self>) {
        self.navigate_keyframe(1, cx);
    }
    fn navigate_keyframe(&mut self, direction: isize, cx: &mut Context<Self>) {
        if self.keyframes.is_empty() {
            return;
        }
        let current =
            self.selected_keyframe.and_then(|id| self.keyframes.iter().position(|k| k.id == id));
        let index = match (current, direction.signum()) {
            (Some(i), -1) => i.saturating_sub(1),
            (Some(i), 1) => (i + 1).min(self.keyframes.len() - 1),
            _ => 0,
        };
        let selected = Some(self.keyframes[index].id);
        if selected != self.selected_keyframe {
            self.selected_keyframe = selected;
            cx.emit(KeyframeSelectionChanged { selected });
            cx.notify();
        }
    }

    fn propose_range(&mut self, range: TimeRange, cx: &mut Context<Self>) {
        if self.visible_range == range {
            return;
        }
        if !self.controlled {
            self.visible_range = range;
            cx.notify();
        }
        cx.emit(VisibleRangeChanged { start: range.start, end: range.end });
    }

    fn on_zoom_in(&mut self, _: &ZoomIn, _: &mut Window, cx: &mut Context<Self>) {
        self.zoom_center(ZOOM_FACTOR, cx);
    }
    fn on_zoom_out(&mut self, _: &ZoomOut, _: &mut Window, cx: &mut Context<Self>) {
        self.zoom_center(1.0 / ZOOM_FACTOR, cx);
    }
    fn on_pan_earlier(&mut self, _: &PanEarlier, _: &mut Window, cx: &mut Context<Self>) {
        self.pan_fraction(-1.0, cx);
    }
    fn on_pan_later(&mut self, _: &PanLater, _: &mut Window, cx: &mut Context<Self>) {
        self.pan_fraction(1.0, cx);
    }
    fn on_move_earlier(&mut self, _: &MoveEarlier, _: &mut Window, cx: &mut Context<Self>) {
        self.pan_fraction(-1.0, cx);
    }
    fn on_move_later(&mut self, _: &MoveLater, _: &mut Window, cx: &mut Context<Self>) {
        self.pan_fraction(1.0, cx);
    }
    fn move_clip_track(&mut self, direction: isize, cx: &mut Context<Self>) {
        let Some(EditPreview::Clip { id, track_id, start }) = self.edit else {
            return;
        };
        let Some(index) = self.tracks.iter().position(|t| t.id == track_id) else {
            return;
        };
        let Some(target) =
            index.checked_add_signed(direction).and_then(|i| self.tracks.get(i)).map(|t| t.id)
        else {
            return;
        };
        self.edit = Some(EditPreview::Clip { id, track_id: target, start });
        cx.notify();
    }
    fn on_move_track_up(&mut self, _: &MoveTrackUp, _: &mut Window, cx: &mut Context<Self>) {
        self.move_clip_track(-1, cx);
    }
    fn on_move_track_down(&mut self, _: &MoveTrackDown, _: &mut Window, cx: &mut Context<Self>) {
        self.move_clip_track(1, cx);
    }
    fn on_fit(&mut self, _: &FitRange, _: &mut Window, cx: &mut Context<Self>) {
        self.fit_range(cx);
    }
    fn on_step_earlier(&mut self, _: &StepEarlier, _: &mut Window, cx: &mut Context<Self>) {
        self.step_playhead(-1.0, cx);
    }
    fn on_step_later(&mut self, _: &StepLater, _: &mut Window, cx: &mut Context<Self>) {
        self.step_playhead(1.0, cx);
    }
    fn on_previous_object(&mut self, _: &PreviousObject, _: &mut Window, cx: &mut Context<Self>) {
        self.navigate_selection(-1, cx);
    }
    fn on_next_object(&mut self, _: &NextObject, _: &mut Window, cx: &mut Context<Self>) {
        self.navigate_selection(1, cx);
    }

    fn on_wheel(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let bounds = self.time_surface_bounds.get();
        let anchor = bounds.and_then(|bounds| {
            let width = f32::from(bounds.size.width);
            (width > 0.0).then(|| {
                let fraction = ((f32::from(event.position.x) - f32::from(bounds.origin.x)) / width)
                    .clamp(0.0, 1.0) as f64;
                self.visible_range.start + self.visible_range.duration() * fraction
            })
        });
        let zoom = event.modifiers.control || event.modifiers.platform;
        match event.delta {
            ScrollDelta::Lines(delta) if zoom => {
                if let Some(anchor) = anchor {
                    self.zoom_at_fraction(1.12_f64.powf(delta.y as f64), anchor, cx);
                }
            }
            ScrollDelta::Pixels(delta) if zoom => {
                if let Some(anchor) = anchor {
                    self.zoom_at_fraction((f64::from(delta.y.as_f32()) / 600.0).exp(), anchor, cx);
                }
            }
            ScrollDelta::Lines(delta) => {
                let horizontal = if delta.x != 0.0 {
                    Some(delta.x)
                } else if event.modifiers.shift {
                    Some(delta.y)
                } else {
                    None
                };
                if let Some(direction) = horizontal {
                    self.pan_by(self.visible_range.duration() * 0.08 * direction as f64, cx);
                }
            }
            ScrollDelta::Pixels(delta) => {
                let horizontal = if delta.x.as_f32().abs() > 0.0 {
                    Some(delta.x.as_f32())
                } else if event.modifiers.shift {
                    Some(delta.y.as_f32())
                } else {
                    None
                };
                let Some(delta) = horizontal else { return };
                let Some(bounds) = bounds else { return };
                let width = f32::from(bounds.size.width);
                if width > 0.0 {
                    self.pan_by(self.visible_range.duration() * delta as f64 / width as f64, cx);
                }
            }
        }
    }
}

impl Focusable for Timeline {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone().expect("timeline focus initialized during render")
    }
}

impl Render for Timeline {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_stop(true)).clone();
        let theme = *cx.global::<Theme>();
        let look = look(&theme);
        let transparent = theme.colors.background.opacity(0.);
        let range = self.visible_range;
        let playhead = self.playhead;
        let playhead_fraction =
            ((playhead - range.start) / range.duration()).clamp(0.0, 1.0) as f32;
        let label_width = theme.controls.large * TRACK_LABEL_WIDTH_MULTIPLIER;
        let ruler_height = theme.controls.small;
        let row_height = theme.controls.large;
        let show_empty = content_extent(&self.tracks).is_none();
        let marker_size = theme.controls.small;
        let marker_pitch = marker_size + theme.spacing.xsmall;
        let tick_width = self
            .time_surface_bounds
            .get()
            .map(|bounds| f64::from(bounds.size.width.as_f32()))
            .unwrap_or_else(|| (700.0 - f64::from(label_width)).max(240.0));
        let ticks = ruler_ticks(range, tick_width, MIN_TICK_SPACING);

        let fit_entity = cx.entity();
        let zoom_in_entity = cx.entity();
        let zoom_out_entity = cx.entity();
        let icon_size = theme.spacing.large;
        let fit_button =
            tool_button("timeline-fit", "Fit", "Fit all clips", theme, move |_, _, cx| {
                fit_entity.update(cx, |this, cx| this.fit_range(cx));
            });
        let zoom_in = tool_button(
            "timeline-zoom-in",
            icon(icon_size, PLUS, theme.colors.text),
            "Zoom in",
            theme,
            move |_, _, cx| {
                zoom_in_entity.update(cx, |this, cx| this.zoom_center(ZOOM_FACTOR, cx));
            },
        );
        let zoom_out = tool_button(
            "timeline-zoom-out",
            icon(icon_size, MINUS, theme.colors.text),
            "Zoom out",
            theme,
            move |_, _, cx| {
                zoom_out_entity.update(cx, |this, cx| this.zoom_center(1.0 / ZOOM_FACTOR, cx));
            },
        );
        let toolbar = div()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.body))
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.label.clone()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(theme.spacing.xsmall))
                    .child(zoom_out)
                    .child(zoom_in)
                    .child(fit_button),
            );

        let mut ruler = div()
            .id("timeline-ruler")
            .debug_selector(|| "timeline-ruler".into())
            .relative()
            .flex()
            .w_full()
            .h(px(ruler_height))
            .bg(look.muted)
            .border_b(px(theme.borders.hairline))
            .border_color(look.divider)
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label("Time ruler");
        ruler = ruler.child(
            div()
                .w(px(label_width))
                .h_full()
                .flex()
                .items_center()
                .px(px(theme.spacing.medium))
                .border_r(px(theme.borders.hairline))
                .border_color(look.divider)
                .text_color(theme.colors.text_muted)
                .text_size(px(theme.typography.caption))
                .font_weight(FontWeight::MEDIUM)
                .child("Track"),
        );
        let mut ruler_lane = div()
            .id("timeline-time-surface")
            .debug_selector(|| "timeline-time-surface".into())
            .relative()
            .flex_1()
            .h_full()
            .role(gpui_pre::accesskit::Role::Slider)
            .aria_label("Playhead")
            .aria_value(format_time(playhead))
            .aria_min_numeric_value(range.start)
            .aria_max_numeric_value(range.end)
            .aria_numeric_value(playhead)
            .on_scroll_wheel(cx.listener(Self::on_wheel));
        for tick in ticks.iter().copied() {
            let fraction = ((tick - range.start) / range.duration()) as f32;
            ruler_lane = ruler_lane.child(
                div()
                    .absolute()
                    .left(relative(fraction))
                    .top_0()
                    .bottom_0()
                    .w(px(theme.borders.hairline))
                    .bg(look.divider),
            );
            ruler_lane = ruler_lane.child(
                div()
                    .absolute()
                    .when(fraction >= 0.95, |el| {
                        el.right_0()
                            .w(px(theme.controls.large * 2.0))
                            .justify_end()
                            .pr(px(theme.spacing.xsmall))
                    })
                    .when(fraction < 0.95, |el| {
                        el.left(relative(fraction)).pl(px(theme.spacing.xsmall))
                    })
                    .top_0()
                    .bottom_0()
                    .flex()
                    .items_end()
                    .child(
                        div()
                            .text_color(theme.colors.text_muted)
                            .text_size(px(theme.typography.caption))
                            .child(format_time(tick)),
                    ),
            );
        }
        let playhead_head = theme.spacing.medium;
        ruler_lane = ruler_lane
            .child(
                div()
                    .id("timeline-playhead")
                    .absolute()
                    .left(relative(playhead_fraction))
                    .top_0()
                    .bottom_0()
                    .w(px(theme.borders.hairline.max(1.0)))
                    .bg(theme.colors.accent),
            )
            .child(
                // The playhead head: the restyled Slider thumb look at `spacing.medium`.
                div()
                    .absolute()
                    .left(relative(playhead_fraction))
                    .ml(px((theme.borders.hairline.max(1.0) - playhead_head) / 2.0))
                    .top_0()
                    .size(px(playhead_head))
                    .rounded(px(theme.radii.pill))
                    .border(px(theme.borders.hairline))
                    .border_color(theme.colors.accent)
                    .bg(theme.colors.background)
                    .shadow(vec![box_shadow(theme.shadows.small)]),
            );
        let pointer_entity = cx.entity();
        let pointer_bounds = self.playhead_bounds.clone();
        ruler_lane = ruler_lane.child(
            canvas(
                move |bounds, _, _| bounds,
                move |bounds, _, window, _cx| {
                    pointer_bounds.set(Some(bounds));
                    let down = pointer_entity.clone();
                    window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                        if phase == DispatchPhase::Capture && bounds.contains(&event.position) {
                            down.update(cx, |this, cx| this.pointer_down(event, window, cx));
                        }
                    });
                    let moved = pointer_entity.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture {
                            moved.update(cx, |this, cx| this.pointer_move(event, cx));
                        }
                    });
                    let up = pointer_entity.clone();
                    window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture {
                            up.update(cx, |this, cx| this.pointer_up(event, cx));
                        }
                    });
                },
            )
            .absolute()
            .inset_0(),
        );
        ruler = ruler.child(ruler_lane);

        let mut rows = Vec::with_capacity(self.tracks.len());
        for track in &self.tracks {
            let track_keyframes: Vec<&Keyframe> = self
                .keyframes
                .iter()
                .filter(|keyframe| {
                    keyframe.time >= range.start
                        && keyframe.time <= range.end
                        && match keyframe.owner {
                            TimelineObject::Track(id) => id == track.id,
                            TimelineObject::Clip(id) => {
                                track.clips.iter().any(|clip| clip.id == id)
                            }
                        }
                })
                .collect();
            let marker_band = if track_keyframes.is_empty() {
                0.0
            } else {
                marker_pitch * KEYFRAME_STACK_ROWS as f32
            };
            let track_row_height = row_height + marker_band;
            let marker_lane_width = self
                .time_surface_bounds
                .get()
                .map(|bounds| f64::from(bounds.size.width.as_f32()))
                .unwrap_or_else(|| (700.0 - f64::from(label_width)).max(240.0));
            let mut clusters: Vec<Vec<&Keyframe>> = Vec::new();
            for keyframe in track_keyframes {
                let marker_x = (keyframe.time - range.start) / range.duration() * marker_lane_width;
                let same_cluster = clusters.last().and_then(|cluster| cluster.first()).is_some_and(
                    |first: &&Keyframe| {
                        let first_x =
                            (first.time - range.start) / range.duration() * marker_lane_width;
                        marker_x - first_x <= f64::from(marker_size)
                    },
                );
                if same_cluster {
                    clusters.last_mut().expect("cluster exists").push(keyframe);
                } else {
                    clusters.push(vec![keyframe]);
                }
            }
            let mut marker_layout = HashMap::new();
            for cluster in clusters {
                let first = cluster.first().expect("nonempty marker cluster");
                let last = cluster.last().expect("nonempty marker cluster");
                let first_x = (first.time - range.start) / range.duration() * marker_lane_width;
                let last_x = (last.time - range.start) / range.duration() * marker_lane_width;
                let cluster_span = (last_x - first_x).max(0.0) as f32;
                let columns = cluster.len().div_ceil(KEYFRAME_STACK_ROWS);
                let column_pitch = marker_size + theme.spacing.xsmall + cluster_span;
                for (index, keyframe) in cluster.into_iter().enumerate() {
                    let column = index / KEYFRAME_STACK_ROWS;
                    let row = index % KEYFRAME_STACK_ROWS;
                    let centered_column = column as f32 - (columns - 1) as f32 / 2.0;
                    marker_layout.insert(keyframe.id, (row, centered_column * column_pitch));
                }
            }
            let track_label = track.label.clone();
            let track_id = track.id;
            let lane_id = format!("timeline-lane-{}", track.id);
            let lane_selector = lane_id.clone();
            let selected_track = self.selected == Some(TimelineObject::Track(track.id));
            let mut lane = div()
                .id(lane_id)
                .debug_selector(move || lane_selector.clone())
                .relative()
                .flex_1()
                .h_full()
                .border_l(px(theme.borders.hairline))
                .border_b(px(theme.borders.hairline))
                .border_color(look.divider)
                .bg(theme.colors.background)
                .role(gpui_pre::accesskit::Role::Group)
                .aria_label(format!("{} track", track.label))
                .aria_description(format!("Track {}", track.id));
            lane = lane.on_scroll_wheel(cx.listener(Self::on_wheel));
            if marker_band > 0.0 {
                lane = lane.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .h(px(marker_band))
                        .bg(look.band)
                        .border_b(px(theme.borders.hairline))
                        .border_color(look.divider),
                );
            }
            for tick in ticks.iter().copied() {
                let fraction = ((tick - range.start) / range.duration()) as f32;
                lane = lane.child(
                    div()
                        .absolute()
                        .left(relative(fraction))
                        .top_0()
                        .bottom_0()
                        .w(px(theme.borders.hairline))
                        .bg(look.divider),
                );
            }
            lane = lane.child(
                div()
                    .absolute()
                    .left(relative(playhead_fraction))
                    .top_0()
                    .bottom_0()
                    .w(px(theme.borders.hairline.max(1.0)))
                    .bg(theme.colors.accent),
            );
            for clip in &track.clips {
                let preview = match self.edit {
                    Some(EditPreview::Clip { id, track_id, start }) if id == clip.id => {
                        Some((track_id, start))
                    }
                    _ => None,
                };
                if preview.is_some_and(|(target, _)| target != track.id) {
                    continue;
                }
                if clip.end <= range.start || clip.start >= range.end {
                    continue;
                }
                let actual_start = preview.map_or(clip.start, |(_, start)| start);
                let duration = clip.end - clip.start;
                let start = actual_start.max(range.start);
                let end = (actual_start + duration).min(range.end);
                let left = ((start - range.start) / range.duration()) as f32;
                let width = ((end - start) / range.duration()).max(0.0) as f32;
                let clip_id = clip.id;
                let selected_clip = self.selected == Some(TimelineObject::Clip(clip.id));
                lane = lane.child(
                    div()
                        .id(format!("timeline-clip-{}", clip.id))
                        .debug_selector(move || format!("timeline-clip-{clip_id}"))
                        .absolute()
                        .left(relative(left))
                        .w(relative(width))
                        .top(px(marker_band + theme.spacing.xsmall))
                        .bottom(px(theme.spacing.xsmall))
                        .min_w(px(theme.controls.xsmall * 0.5))
                        .px(px(theme.spacing.xsmall))
                        .flex()
                        .items_center()
                        .gap(px(theme.spacing.xsmall))
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .rounded(px(theme.radii.medium))
                        .border(px(theme.borders.hairline))
                        .border_color(if selected_clip {
                            look.clip_selected_border
                        } else {
                            look.clip_border
                        })
                        .bg(look.clip_bg)
                        .shadow(if selected_clip {
                            vec![focus_ring(look.ring)]
                        } else {
                            vec![box_shadow(theme.shadows.small)]
                        })
                        .text_color(look.clip_text)
                        .text_size(px(theme.typography.caption))
                        .font_weight(FontWeight::MEDIUM)
                        .role(gpui_pre::accesskit::Role::Group)
                        .aria_label(format!("{} clip", clip.label))
                        .aria_description(format!(
                            "On track {}. From {} to {} seconds. {}.",
                            track_label,
                            format_time(actual_start),
                            format_time(actual_start + duration),
                            if selected_clip { "Selected" } else { "Not selected" }
                        ))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, event, window, cx| {
                                this.request_selection(Some(TimelineObject::Clip(clip_id)), cx);
                                this.begin_clip_drag(clip_id, track_id, event);
                                if let Some(focus) = &this.focus {
                                    window.focus(focus, cx);
                                }
                            }),
                        )
                        .when(selected_clip, |el| el.child(icon(icon_size, CHECK, look.clip_text)))
                        .child(clip.label.clone()),
                );
            }
            if let Some(EditPreview::Clip { id, track_id, start }) = self.edit
                && track_id == track.id
                && !track.clips.iter().any(|clip| clip.id == id)
                && let Some(clip) = self.tracks.iter().flat_map(|t| &t.clips).find(|c| c.id == id)
            {
                let left = ((start.max(range.start) - range.start) / range.duration()) as f32;
                let width = ((clip.end - clip.start) / range.duration()) as f32;
                lane = lane.child(
                    div()
                        .id("timeline-clip-move-preview")
                        .absolute()
                        .left(relative(left))
                        .w(relative(width))
                        .top(px(marker_band + theme.spacing.xsmall))
                        .bottom(px(theme.spacing.xsmall))
                        .px(px(theme.spacing.xsmall))
                        .flex()
                        .items_center()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .rounded(px(theme.radii.medium))
                        .bg(look.clip_bg)
                        .border(px(theme.borders.hairline))
                        .border_color(look.clip_selected_border)
                        .shadow(vec![focus_ring(look.ring)])
                        .text_color(look.clip_text)
                        .text_size(px(theme.typography.caption))
                        .font_weight(FontWeight::MEDIUM)
                        .aria_label(format!("{} clip move preview", clip.label))
                        .aria_description(format!(
                            "Proposed on track {} at {} seconds",
                            track.label,
                            format_time(start)
                        ))
                        .child(clip.label.clone()),
                );
            }
            for keyframe in &self.keyframes {
                let Some(&(marker_row, horizontal_offset)) = marker_layout.get(&keyframe.id) else {
                    continue;
                };
                let keyframe_id = keyframe.id;
                let selected = self.selected_keyframe == Some(keyframe_id);
                let preview_time = match self.edit {
                    Some(EditPreview::Keyframe { id, time }) if id == keyframe_id => time,
                    _ => keyframe.time,
                };
                let px_at = ((preview_time - range.start) / range.duration()) as f32;
                lane = lane.child(
                    div()
                        .id(format!("timeline-keyframe-{}", keyframe.id.0))
                        .debug_selector(move || format!("timeline-keyframe-{}", keyframe_id.0))
                        .absolute()
                        .left(relative(px_at))
                        .ml(px(horizontal_offset))
                        .top(px(marker_row as f32 * marker_pitch))
                        .w(px(marker_size))
                        .h(px(marker_size))
                        .rounded(px(theme.radii.small))
                        .border(px(theme.borders.hairline))
                        .border_color(transparent)
                        .hover(move |s| {
                            if look.high_contrast {
                                s.border_color(theme.colors.border)
                            } else {
                                s.bg(look.muted)
                            }
                        })
                        .role(gpui_pre::accesskit::Role::Group)
                        .aria_label(format!("{} keyframe", keyframe.label))
                        .aria_description(format!(
                            "On {} at {}. {}.",
                            match keyframe.owner {
                                TimelineObject::Track(id) => format!("track {id}"),
                                TimelineObject::Clip(id) => format!("clip {id}"),
                            },
                            format_time(preview_time),
                            if selected { "Selected" } else { "Not selected" }
                        ))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, event, window, cx| {
                                this.begin_keyframe_drag(keyframe_id, event);
                                if this.selected_keyframe != Some(keyframe_id) {
                                    this.selected_keyframe = Some(keyframe_id);
                                    cx.emit(KeyframeSelectionChanged {
                                        selected: Some(keyframe_id),
                                    });
                                }
                                cx.stop_propagation();
                                if let Some(f) = &this.focus {
                                    window.focus(f, cx);
                                }
                            }),
                        )
                        .child(keyframe_diamond(theme, look, selected)),
                );
            }
            let lane_bounds = self.lane_bounds.clone();
            let lane_track_id = track.id;
            let time_surface_bounds = self.time_surface_bounds.clone();
            lane = lane.child(
                canvas(
                    move |bounds, _, _| bounds,
                    move |bounds, _, _, _| {
                        lane_bounds.borrow_mut().insert(lane_track_id, bounds);
                        time_surface_bounds.set(Some(bounds));
                    },
                )
                .absolute()
                .inset_0(),
            );
            rows.push(
                div()
                    .id(format!("timeline-track-{}", track.id))
                    .h(px(track_row_height))
                    .w_full()
                    .flex()
                    .items_center()
                    .role(gpui_pre::accesskit::Role::Group)
                    .aria_label(format!("{} track", track.label))
                    .child(
                        div()
                            .id(format!("timeline-track-label-{}", track.id))
                            .role(gpui_pre::accesskit::Role::Group)
                            .w(px(label_width))
                            .h_full()
                            .p(px(theme.spacing.xsmall))
                            .border_b(px(theme.borders.hairline))
                            .border_color(look.divider)
                            .aria_label(format!("{} track", track.label))
                            .aria_description(if selected_track {
                                "Selected"
                            } else {
                                "Not selected"
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, window, cx| {
                                    this.request_selection(
                                        Some(TimelineObject::Track(track_id)),
                                        cx,
                                    );
                                    if let Some(focus) = &this.focus {
                                        window.focus(focus, cx);
                                    }
                                }),
                            )
                            .child(
                                // The restyled Sidebar/Tree row: radius-small, accent fill when
                                // selected, a muted fill on hover, and a reserved hairline border.
                                div()
                                    .size_full()
                                    .flex()
                                    .items_center()
                                    .px(px(theme.spacing.small))
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .rounded(px(theme.radii.small))
                                    .border(px(theme.borders.hairline))
                                    .border_color(transparent)
                                    .text_size(px(theme.typography.body))
                                    .when(selected_track, |el| {
                                        el.bg(look.selected_track_bg)
                                            .text_color(look.selected_track_text)
                                            .font_weight(FontWeight::MEDIUM)
                                    })
                                    .when(!selected_track, |el| {
                                        el.text_color(theme.colors.text).hover(move |s| {
                                            if look.high_contrast {
                                                s.border_color(theme.colors.border)
                                            } else {
                                                s.bg(look.muted)
                                            }
                                        })
                                    })
                                    .child(track.label.clone()),
                            ),
                    )
                    .child(lane),
            );
        }

        let scroll = div()
            .id("timeline-tracks-scroll")
            .debug_selector(|| "timeline-tracks-scroll".into())
            .h(px(row_height * VISIBLE_TRACK_ROWS))
            .w_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .role(gpui_pre::accesskit::Role::ScrollView)
            .aria_label("Timeline tracks")
            .children(rows);
        let scroll = if show_empty {
            scroll.child(
                div()
                    .id("timeline-empty-state")
                    .role(gpui_pre::accesskit::Role::Status)
                    .aria_label("No clips to display")
                    .text_color(theme.colors.text_muted)
                    .px(px(theme.spacing.medium))
                    .child("No clips to display"),
            )
        } else {
            scroll
        };

        div()
            .id("mkit-timeline")
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.xsmall))
            .p(px(theme.spacing.medium))
            .rounded(px(theme.radii.large))
            .border(px(theme.borders.hairline))
            .border_color(look.divider)
            .bg(theme.colors.surface)
            .text_color(theme.colors.text)
            .focus_visible(move |el| {
                el.border_color(theme.colors.focus).shadow(vec![focus_ring(look.ring)])
            })
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.label.clone())
            .aria_description("Ordered tracks and clips with time spans. Use left and right arrows to pan, plus and minus to zoom, and F to fit all clips.")
            .on_action(cx.listener(Self::on_zoom_in))
            .on_action(cx.listener(Self::on_zoom_out))
            .on_action(cx.listener(Self::on_pan_earlier))
            .on_action(cx.listener(Self::on_pan_later))
            .on_action(cx.listener(Self::on_move_earlier))
            .on_action(cx.listener(Self::on_move_later))
            .on_action(cx.listener(Self::on_move_track_up))
            .on_action(cx.listener(Self::on_move_track_down))
            .on_action(cx.listener(Self::on_fit))
            .on_action(cx.listener(Self::on_step_earlier))
            .on_action(cx.listener(Self::on_step_later))
            .on_action(cx.listener(Self::on_previous_object))
            .on_action(cx.listener(Self::on_next_object))
            .on_action(cx.listener(Self::on_pick_up_clip))
            .on_action(cx.listener(Self::on_pick_up_keyframe))
            .on_action(cx.listener(Self::on_commit_edit))
            .on_action(cx.listener(Self::on_cancel_edit))
            .on_action(cx.listener(Self::on_toggle_snap))
            .on_action(cx.listener(Self::on_previous_keyframe))
            .on_action(cx.listener(Self::on_next_keyframe))
            .child(toolbar)
            .child(ruler)
            .child(scroll)
    }
}

/// A small outline button (the restyled Button's `outline` variant) at the dense
/// `controls.xsmall` toolbar height.
fn tool_button(
    id: &'static str,
    content: impl IntoElement,
    label: &'static str,
    theme: Theme,
    on_click: impl Fn(&gpui_pre::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let look = look(&theme);
    let c = theme.colors;
    div()
        .id(id)
        .role(gpui_pre::accesskit::Role::Button)
        .aria_label(label)
        .tab_index(0)
        .h(px(theme.controls.xsmall))
        .min_w(px(theme.controls.small))
        .px(px(theme.spacing.small))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme.radii.medium))
        .border(px(theme.borders.regular))
        .border_color(look.control_border)
        .bg(c.background)
        .shadow(vec![box_shadow(theme.shadows.small)])
        .text_color(c.text)
        .text_size(px(theme.typography.body))
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap()
        .hover(
            move |s| if look.high_contrast { s.border_color(c.accent) } else { s.bg(look.muted) },
        )
        .focus_visible(move |s| {
            s.border_color(c.focus).bg(c.background).shadow(vec![focus_ring(look.ring)])
        })
        .on_click(on_click)
        .child(content)
}

fn normalize_clips(mut clips: Vec<Clip>) -> Vec<Clip> {
    clips.retain(|clip| {
        clip.start.is_finite()
            && clip.end.is_finite()
            && clip.start < clip.end
            && (clip.end - clip.start).is_finite()
    });
    clips.sort_by(|a, b| a.start.total_cmp(&b.start).then(a.id.cmp(&b.id)));
    clips
}

fn normalize_tracks(tracks: Vec<Track>) -> Vec<Track> {
    tracks.into_iter().map(|track| Track::new(track.id, track.label, track.clips)).collect()
}

fn contains_in_tracks(tracks: &[Track], object: TimelineObject) -> bool {
    match object {
        TimelineObject::Track(id) => tracks.iter().any(|track| track.id == id),
        TimelineObject::Clip(id) => {
            tracks.iter().any(|track| track.clips.iter().any(|clip| clip.id == id))
        }
    }
}

fn content_extent(tracks: &[Track]) -> Option<(f64, f64)> {
    let min = tracks
        .iter()
        .flat_map(|track| &track.clips)
        .map(|clip| clip.start)
        .min_by(f64::total_cmp)?;
    let max =
        tracks.iter().flat_map(|track| &track.clips).map(|clip| clip.end).max_by(f64::total_cmp)?;
    (min < max && (max - min).is_finite()).then_some((min, max))
}

/// Convert a time in seconds to an x coordinate in logical pixels.
pub fn time_to_pixel(time: f64, range: TimeRange, width: f32) -> Option<f32> {
    if !time.is_finite() || !width.is_finite() || width <= 0.0 || !range.duration().is_finite() {
        return None;
    }
    let pixel = (time - range.start) / range.duration() * f64::from(width);
    (pixel.is_finite() && pixel.abs() <= f32::MAX as f64).then_some(pixel as f32)
}

/// Convert an x coordinate in logical pixels into seconds.
pub fn pixel_to_time(pixel: f32, range: TimeRange, width: f32) -> Option<f64> {
    if !pixel.is_finite() || !width.is_finite() || width <= 0.0 || !range.duration().is_finite() {
        return None;
    }
    let time = range.start + f64::from(pixel / width) * range.duration();
    time.is_finite().then_some(time)
}

/// Choose a stable 1/2/5 × 10ⁿ time tick step near the requested pixel spacing.
pub fn tick_step(range: TimeRange, width: f64, min_spacing: f64) -> Option<f64> {
    if !range.duration().is_finite()
        || range.duration() <= 0.0
        || !width.is_finite()
        || width <= 0.0
        || !min_spacing.is_finite()
        || min_spacing <= 0.0
    {
        return None;
    }
    let target = range.duration() / (width / min_spacing).max(1.0);
    if !target.is_finite() || target <= 0.0 {
        return None;
    }
    let magnitude = 10_f64.powf(target.log10().floor());
    if !magnitude.is_finite() || magnitude <= 0.0 {
        return None;
    }
    let factor = target / magnitude;
    Some(
        magnitude
            * if factor <= 1.0 {
                1.0
            } else if factor <= 2.0 {
                2.0
            } else if factor <= 5.0 {
                5.0
            } else {
                10.0
            },
    )
}

pub fn ruler_ticks(range: TimeRange, width: f64, min_spacing: f64) -> Vec<f64> {
    let Some(step) = tick_step(range, width, min_spacing) else { return Vec::new() };
    let mut tick = (range.start / step).ceil() * step;
    let mut ticks = Vec::new();
    while tick <= range.end && ticks.len() < MAX_TICKS {
        ticks.push(tick);
        let next = tick + step;
        if !next.is_finite() || next <= tick {
            break;
        }
        tick = next;
    }
    ticks
}

fn format_time(seconds: f64) -> String {
    if seconds.abs() < 10.0 {
        format!("{seconds:.2}s")
    } else if seconds.abs() < 100.0 {
        format!("{seconds:.1}s")
    } else {
        format!("{seconds:.0}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{
        Focusable, Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext, point, px,
    };
    use std::{cell::RefCell, rc::Rc};

    fn fixture() -> Vec<Track> {
        vec![Track::new(
            7,
            "Video",
            vec![Clip::new(4, 2.0, 6.0, "Opening"), Clip::new(5, 8.0, 10.0, "Closing")],
        )]
    }

    #[test]
    fn time_conversion_round_trips_and_rejects_invalid_inputs() {
        let range = TimeRange::new(-5.0, 15.0).unwrap();
        assert_eq!(time_to_pixel(5.0, range, 400.0), Some(200.0));
        assert_eq!(pixel_to_time(200.0, range, 400.0), Some(5.0));
        assert!(time_to_pixel(f64::NAN, range, 400.0).is_none());
        assert!(pixel_to_time(10.0, range, 0.0).is_none());
        assert!(TimeRange::new(1.0, 1.0).is_none());
    }

    #[test]
    fn snapping_uses_screen_tolerance_priority_and_bypass() {
        let range = TimeRange::new(0.0, 10.0).unwrap();
        let candidates =
            [SnapCandidate { time: 5.05, priority: 2 }, SnapCandidate { time: 5.04, priority: 0 }];
        assert_eq!(snap_time(5.045, range, 100.0, &candidates, false), 5.04);
        assert_eq!(snap_time(5.045, range, 100.0, &candidates, true), 5.045);
        assert_eq!(snap_time(9.0, range, 100.0, &candidates, false), 9.0);
    }

    #[gpui_pre::test]
    fn controlled_clip_and_keyframe_edits_emit_only_on_commit(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let range = TimeRange::new(0.0, 10.0).unwrap();
        let (timeline, visual) =
            cx.add_window_view(|_, _| Timeline::controlled("Edit timeline", fixture(), range));
        timeline.update(visual, |view, cx| {
            view.set_keyframes(
                vec![Keyframe::new(KeyframeId(3), TimelineObject::Clip(4), 3.0, "Beat")],
                cx,
            )
        });
        let clips = Rc::new(RefCell::new(Vec::new()));
        let clip_log = clips.clone();
        let keys = Rc::new(RefCell::new(Vec::new()));
        let key_log = keys.clone();
        let _clip_sub = visual.update(|_, app| {
            app.subscribe(&timeline, move |_, e: &ClipMoveRequested, _| {
                clip_log.borrow_mut().push(*e)
            })
        });
        let _key_sub = visual.update(|_, app| {
            app.subscribe(&timeline, move |_, e: &KeyframeMoveRequested, _| {
                key_log.borrow_mut().push(*e)
            })
        });
        timeline.update(visual, |view, cx| {
            view.request_clip_move(4, 7, 2.0, cx);
        });
        assert!(clips.borrow().is_empty());
        timeline.update(visual, |view, cx| view.commit_edit(cx));
        assert!(clips.borrow().is_empty());
        timeline.update(visual, |view, cx| {
            view.request_clip_move(4, 7, 5.0, cx);
            view.commit_edit(cx);
        });
        timeline.update(visual, |view, cx| {
            view.request_keyframe_move(KeyframeId(3), 6.0, cx);
            view.cancel_edit(cx);
        });
        assert_eq!(
            clips.borrow().as_slice(),
            &[ClipMoveRequested { clip_id: 4, track_id: 7, start: 5.0 }]
        );
        assert!(keys.borrow().is_empty());
        timeline.update(visual, |view, cx| {
            view.request_keyframe_move(KeyframeId(3), 7.0, cx);
            view.commit_edit(cx);
        });
        assert_eq!(
            keys.borrow().as_slice(),
            &[KeyframeMoveRequested { keyframe_id: KeyframeId(3), time: 7.0 }]
        );
    }

    #[gpui_pre::test]
    fn uncontrolled_committed_edits_apply_once(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let range = TimeRange::new(0.0, 10.0).unwrap();
        let (timeline, visual) =
            cx.add_window_view(|_, _| Timeline::new("Edit timeline", fixture(), range));
        timeline.update(visual, |view, cx| {
            view.set_keyframes(
                vec![Keyframe::new(KeyframeId(3), TimelineObject::Clip(4), 3.0, "Beat")],
                cx,
            )
        });
        timeline.update(visual, |view, cx| {
            view.request_clip_move(4, 7, 4.0, cx);
            view.commit_edit(cx);
        });
        assert_eq!(timeline.read_with(visual, |view, _| view.tracks()[0].clips[0].start), 4.0);
        timeline.update(visual, |view, cx| {
            view.request_keyframe_move(KeyframeId(3), 6.0, cx);
            view.commit_edit(cx);
        });
        assert_eq!(timeline.read_with(visual, |view, _| view.keyframes()[0].time), 6.0);
    }

    #[test]
    fn tracks_sort_clips_and_ticks_use_readable_125_steps() {
        let track = Track::new(
            1,
            "A",
            vec![
                Clip::new(2, 8.0, 9.0, "B"),
                Clip::new(1, 1.0, 2.0, "A"),
                Clip::new(3, 2.0, 2.0, "invalid"),
            ],
        );
        assert_eq!(track.clips.iter().map(|clip| clip.id).collect::<Vec<_>>(), [1, 2]);
        let range = TimeRange::new(0.0, 20.0).unwrap();
        assert_eq!(tick_step(range, 800.0, 80.0), Some(2.0));
        assert_eq!(
            ruler_ticks(range, 800.0, 80.0),
            vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0]
        );
    }

    #[gpui_pre::test]
    fn gpui_keyboard_actions_emit_and_controlled_range_waits_for_owner(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = TimeRange::new(0.0, 10.0).unwrap();
        let (timeline, visual) =
            cx.add_window_view(|_, _| Timeline::controlled("Edit timeline", fixture(), initial));
        let changes = Rc::new(RefCell::new(Vec::new()));
        let log = changes.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&timeline, move |_, event: &VisibleRangeChanged, _| {
                log.borrow_mut().push(*event)
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| timeline.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("right");
        assert_eq!(timeline.read_with(visual, |view, _| view.visible_range()), initial);
        assert_eq!(changes.borrow().len(), 1);
        let requested = changes.borrow()[0];
        assert!(requested.start > initial.start);
        assert_eq!(requested.end - requested.start, initial.duration());
        visual.simulate_keystrokes("=");
        visual.simulate_keystrokes("f");
        visual.simulate_keystrokes("left");
        visual.simulate_keystrokes("-");
        assert_eq!(changes.borrow().len(), 5);
        assert_eq!(timeline.read_with(visual, |view, _| view.visible_range()), initial);
    }

    #[gpui_pre::test]
    fn playhead_step_and_selection_navigation_obey_controlled_owner(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let initial = TimeRange::new(0.0, 10.0).unwrap();
        let (timeline, visual) =
            cx.add_window_view(|_, _| Timeline::controlled("Edit timeline", fixture(), initial));
        let playheads = Rc::new(RefCell::new(Vec::new()));
        let selections = Rc::new(RefCell::new(Vec::new()));
        let playhead_log = playheads.clone();
        let selection_log = selections.clone();
        let _playhead_subscription = visual.update(|_, app| {
            app.subscribe(&timeline, move |_, event: &PlayheadChanged, _| {
                playhead_log.borrow_mut().push(event.time)
            })
        });
        let _selection_subscription = visual.update(|_, app| {
            app.subscribe(&timeline, move |_, event: &SelectionChanged, _| {
                selection_log.borrow_mut().push(event.selected)
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| timeline.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes(".");
        visual.simulate_keystrokes("down");
        assert_eq!(timeline.read_with(visual, |view, _| view.selection()), None);
        timeline.update(visual, |view, cx| view.set_selection(Some(TimelineObject::Track(7)), cx));
        visual.simulate_keystrokes("down");
        assert_eq!(timeline.read_with(visual, |view, _| view.playhead()), 0.0);
        assert_eq!(
            timeline.read_with(visual, |view, _| view.selection()),
            Some(TimelineObject::Track(7))
        );
        assert_eq!(playheads.borrow().as_slice(), &[PLAYHEAD_STEP_SECONDS]);
        assert_eq!(
            selections.borrow().as_slice(),
            &[Some(TimelineObject::Track(7)), Some(TimelineObject::Clip(4))]
        );
    }

    #[gpui_pre::test]
    fn uncontrolled_playhead_steps_clamp_and_track_selection_updates(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (timeline, visual) = cx.add_window_view(|_, _| {
            Timeline::new("Edit timeline", fixture(), TimeRange::new(0.0, 0.2).unwrap())
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| timeline.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes(",");
        assert_eq!(timeline.read_with(visual, |view, _| view.playhead()), 0.0);
        visual.simulate_keystrokes(". . .");
        assert!((timeline.read_with(visual, |view, _| view.playhead()) - 0.2).abs() < 1e-9);
        visual.simulate_keystrokes("down");
        assert_eq!(
            timeline.read_with(visual, |view, _| view.selection()),
            Some(TimelineObject::Track(7))
        );
    }

    #[gpui_pre::test]
    fn ruler_pointer_drag_scrubs_without_starting_playback(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let (timeline, visual) = cx.add_window_view(|_, _| {
            Timeline::new("Edit timeline", fixture(), TimeRange::new(0.0, 10.0).unwrap())
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let bounds = visual.debug_bounds("timeline-time-surface").unwrap();
        let x_at = |fraction: f32| bounds.origin.x + px(f32::from(bounds.size.width) * fraction);
        let y = bounds.center().y;
        visual.simulate_mouse_down(point(x_at(0.25), y), MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(
            point(x_at(0.75), y),
            Some(MouseButton::Left),
            Modifiers::default(),
        );
        visual.simulate_mouse_up(point(x_at(0.75), y), MouseButton::Left, Modifiers::default());
        assert!((timeline.read_with(visual, |view, _| view.playhead()) - 7.5).abs() < 1e-6);
    }

    #[gpui_pre::test]
    fn clip_pointer_drag_commits_snapped_move(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let (timeline, visual) = cx.add_window_view(|_, _| {
            Timeline::new("Edit timeline", fixture(), TimeRange::new(0.0, 10.0).unwrap())
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let clip = visual.debug_bounds("timeline-clip-4").unwrap();
        let start = clip.center();
        let end = point(start.x + px(80.0), start.y);
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        let moved = timeline.read_with(visual, |view, _| view.tracks()[0].clips[0].start);
        assert!(moved > 2.0, "clip start was not moved: {moved}");
    }

    #[gpui_pre::test]
    fn pointer_release_outside_lanes_cancels_clip_and_keyframe_moves(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let (timeline, visual) = cx.add_window_view(|_, _| {
            Timeline::new("Edit timeline", fixture(), TimeRange::new(0.0, 10.0).unwrap())
        });
        timeline.update(visual, |view, cx| {
            view.set_keyframes(
                vec![Keyframe::new(KeyframeId(9), TimelineObject::Clip(4), 3.0, "Beat")],
                cx,
            )
        });
        let clip_requests = Rc::new(RefCell::new(Vec::new()));
        let key_requests = Rc::new(RefCell::new(Vec::new()));
        let clip_log = clip_requests.clone();
        let key_log = key_requests.clone();
        let _subscriptions = visual.update(|_, app| {
            (
                app.subscribe(&timeline, move |_, event: &ClipMoveRequested, _| {
                    clip_log.borrow_mut().push(*event);
                }),
                app.subscribe(&timeline, move |_, event: &KeyframeMoveRequested, _| {
                    key_log.borrow_mut().push(*event);
                }),
            )
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let clip_start = visual.debug_bounds("timeline-clip-4").unwrap().center();
        let clip_valid = point(clip_start.x + px(80.0), clip_start.y);
        let outside = point(clip_valid.x, px(0.0));
        visual.simulate_mouse_down(clip_start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(clip_valid, Some(MouseButton::Left), Modifiers::default());
        assert!(timeline.read_with(visual, |view, _| view.edit.is_some()));
        visual.simulate_mouse_move(outside, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(outside, MouseButton::Left, Modifiers::default());
        assert!(timeline.read_with(visual, |view, _| view.edit.is_none()));
        assert_eq!(timeline.read_with(visual, |view, _| view.tracks()[0].clips[0].start), 2.0);
        assert!(clip_requests.borrow().is_empty());

        visual.update(|window, cx| window.draw(cx).clear(cx));
        let key_start = visual.debug_bounds("timeline-keyframe-9").unwrap().center();
        let key_valid = point(key_start.x + px(80.0), key_start.y);
        let outside = point(key_valid.x, px(0.0));
        visual.simulate_mouse_down(key_start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(key_valid, Some(MouseButton::Left), Modifiers::default());
        assert!(timeline.read_with(visual, |view, _| view.edit.is_some()));
        visual.simulate_mouse_move(outside, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(outside, MouseButton::Left, Modifiers::default());
        assert!(timeline.read_with(visual, |view, _| view.edit.is_none()));
        assert_eq!(timeline.read_with(visual, |view, _| view.keyframes()[0].time), 3.0);
        assert!(key_requests.borrow().is_empty());
    }

    #[gpui_pre::test]
    fn pointer_release_uses_final_valid_position_without_move_event(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let (timeline, visual) = cx.add_window_view(|_, _| {
            Timeline::new("Edit timeline", fixture(), TimeRange::new(0.0, 10.0).unwrap())
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let start = visual.debug_bounds("timeline-clip-4").unwrap().center();
        let end = point(start.x + px(80.0), start.y);
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        let moved = timeline.read_with(visual, |view, _| view.tracks()[0].clips[0].start);
        assert!(moved > 2.0, "release coordinate was ignored: {moved}");
    }

    #[gpui_pre::test]
    fn equal_time_keyframes_remain_keyboard_navigable_by_stable_id(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (timeline, visual) = cx.add_window_view(|_, _| {
            Timeline::new("Edit timeline", fixture(), TimeRange::new(0.0, 10.0).unwrap())
        });
        timeline.update(visual, |view, cx| {
            view.set_keyframes(
                vec![
                    Keyframe::new(KeyframeId(8), TimelineObject::Clip(4), 3.0, "Second"),
                    Keyframe::new(KeyframeId(3), TimelineObject::Clip(4), 3.0, "First"),
                ],
                cx,
            )
        });
        let selected = Rc::new(RefCell::new(Vec::new()));
        let log = selected.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&timeline, move |_, event: &KeyframeSelectionChanged, _| {
                log.borrow_mut().push(event.selected);
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("timeline-keyframe-3").is_some());
        assert!(visual.debug_bounds("timeline-keyframe-8").is_some());
        visual.update(|window, cx| timeline.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("ctrl-shift-right ctrl-shift-right");
        assert_eq!(selected.borrow().as_slice(), &[Some(KeyframeId(3)), Some(KeyframeId(8))]);
    }

    #[gpui_pre::test]
    fn dense_keyframes_have_distinct_pointer_targets_and_stable_keyboard_order(
        cx: &mut TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (timeline, visual) = cx.add_window_view(|_, _| {
            Timeline::new("Edit timeline", fixture(), TimeRange::new(0.0, 10.0).unwrap())
        });
        let keyframes = vec![
            Keyframe::new(KeyframeId(19), TimelineObject::Clip(4), 3.04, "Fifth"),
            Keyframe::new(KeyframeId(17), TimelineObject::Clip(4), 3.0, "Third"),
            Keyframe::new(KeyframeId(15), TimelineObject::Clip(4), 3.0, "First"),
            Keyframe::new(KeyframeId(18), TimelineObject::Clip(4), 3.02, "Fourth"),
            Keyframe::new(KeyframeId(16), TimelineObject::Clip(4), 3.0, "Second"),
        ];
        timeline.update(visual, |view, cx| view.set_keyframes(keyframes, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));

        let bounds = [
            (15, visual.debug_bounds("timeline-keyframe-15").unwrap()),
            (16, visual.debug_bounds("timeline-keyframe-16").unwrap()),
            (17, visual.debug_bounds("timeline-keyframe-17").unwrap()),
            (18, visual.debug_bounds("timeline-keyframe-18").unwrap()),
            (19, visual.debug_bounds("timeline-keyframe-19").unwrap()),
        ];
        for (index, (left_id, left)) in bounds.iter().enumerate() {
            for (right_id, right) in bounds.iter().skip(index + 1) {
                let separated = left.right() <= right.left()
                    || right.right() <= left.left()
                    || left.bottom() <= right.top()
                    || right.bottom() <= left.top();
                assert!(
                    separated,
                    "dense keyframe hit targets overlap: {left_id} {left:?}; {right_id} {right:?}"
                );
            }
        }

        for (id, bounds) in bounds {
            let center = bounds.center();
            visual.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
            visual.simulate_mouse_up(center, MouseButton::Left, Modifiers::default());
            assert_eq!(
                timeline.read_with(visual, |view, _| view.selected_keyframe()),
                Some(KeyframeId(id)),
                "pointer should select the marker with this distinct hit target"
            );
        }

        visual.update(|window, cx| timeline.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("ctrl-shift-left");
        assert_eq!(
            timeline.read_with(visual, |view, _| view.selected_keyframe()),
            Some(KeyframeId(18)),
            "navigation follows time then stable ID after selecting the final marker"
        );
    }

    #[gpui_pre::test]
    fn keyframe_pointer_drag_commits_time_move(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let (timeline, visual) = cx.add_window_view(|_, _| {
            Timeline::new("Edit timeline", fixture(), TimeRange::new(0.0, 10.0).unwrap())
        });
        timeline.update(visual, |view, cx| {
            view.set_keyframes(
                vec![Keyframe::new(KeyframeId(9), TimelineObject::Clip(4), 3.0, "Beat")],
                cx,
            )
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let marker = visual.debug_bounds("timeline-keyframe-9").unwrap();
        let start = marker.center();
        let end = point(start.x + px(100.0), start.y);
        visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        let time = timeline.read_with(visual, |view, _| view.keyframes()[0].time);
        assert!(time > 3.0, "keyframe did not move, time={time}");
    }

    #[gpui_pre::test]
    fn uncontrolled_timeline_applies_fit_and_pan(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (timeline, visual) = cx.add_window_view(|_, _| {
            Timeline::new("Edit timeline", fixture(), TimeRange::new(0.0, 4.0).unwrap())
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| timeline.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("f");
        let fitted = timeline.read_with(visual, |view, _| view.visible_range());
        assert!((fitted.start - 1.6).abs() < f64::EPSILON);
        assert!((fitted.end - 10.4).abs() < f64::EPSILON);
        visual.simulate_keystrokes("left");
        assert!(timeline.read_with(visual, |view, _| view.visible_range().start) < fitted.start);
    }

    #[gpui_pre::test]
    fn wheel_modifier_zoom_anchors_and_horizontal_scroll_pans(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let (timeline, visual) = cx.add_window_view(|_, _| {
            Timeline::new("Edit timeline", fixture(), TimeRange::new(0.0, 10.0).unwrap())
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let bounds = visual.debug_bounds("timeline-lane-7").expect("track lane is rendered");
        let pointer = bounds.origin + point(px(0.75 * f32::from(bounds.size.width)), px(20.0));
        let initial = timeline.read_with(visual, |view, _| view.visible_range());
        visual.simulate_event(ScrollWheelEvent {
            position: pointer,
            delta: ScrollDelta::Lines(point(0.0, 1.0)),
            modifiers: Modifiers { control: true, ..Modifiers::default() },
            ..Default::default()
        });
        let zoomed = timeline.read_with(visual, |view, _| view.visible_range());
        let anchor_before = initial.start + initial.duration() * 0.75;
        let anchor_after = zoomed.start + zoomed.duration() * 0.75;
        assert!(
            (anchor_before - anchor_after).abs() < 0.001,
            "pointer anchor moved: before={anchor_before}, after={anchor_after}, ranges={initial:?}->{zoomed:?}"
        );
        visual.simulate_event(ScrollWheelEvent {
            position: pointer,
            delta: ScrollDelta::Pixels(point(px(30.0), px(0.0))),
            ..Default::default()
        });
        assert!(timeline.read_with(visual, |view, _| view.visible_range().start) > zoomed.start);
    }
}
