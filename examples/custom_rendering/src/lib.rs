//! Custom scene painting and a pointer-driven pan-and-zoom canvas.

use gpui_pre::prelude::*;
use gpui_pre::{AnimationExt, Refineable};
use gpui_pre::{
    App, Bounds, Context, Element, ElementId, IntoElement, LayoutId, Pixels, Point, Render,
    ScrollDelta, ScrollWheelEvent, Style, StyleRefinement, Styled, Window, canvas, div, fill, hsla,
    point, px, quad, size,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewTransform {
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for ViewTransform {
    fn default() -> Self {
        Self { scale: 1.0, offset_x: 180.0, offset_y: 120.0 }
    }
}

impl ViewTransform {
    pub fn world_to_screen(self, world: Point<f32>) -> Point<f32> {
        point(self.offset_x + world.x * self.scale, self.offset_y + world.y * self.scale)
    }

    pub fn screen_to_world(self, screen: Point<f32>) -> Point<f32> {
        point((screen.x - self.offset_x) / self.scale, (screen.y - self.offset_y) / self.scale)
    }

    pub fn pan_by(&mut self, dx: f32, dy: f32) {
        self.offset_x += dx;
        self.offset_y += dy;
    }

    pub fn zoom_at(&mut self, screen: Point<f32>, factor: f32) {
        let fixed_world_point = self.screen_to_world(screen);
        self.scale = (self.scale * factor).clamp(0.35, 3.0);
        let moved_screen_point = self.world_to_screen(fixed_world_point);
        self.offset_x += screen.x - moved_screen_point.x;
        self.offset_y += screen.y - moved_screen_point.y;
    }
}

// ANCHOR: pan_zoom_view
#[derive(Default)]
pub struct PanZoomView {
    pub transform: ViewTransform,
    pointer_down: bool,
    last_pointer: Option<Point<f32>>,
}

impl Render for PanZoomView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let transform = self.transform;
        let down = cx.listener(|this, event: &gpui_pre::MouseDownEvent, _, cx| {
            this.pointer_down = true;
            this.last_pointer = Some(point(event.position.x.as_f32(), event.position.y.as_f32()));
            cx.notify();
        });
        let moved = cx.listener(|this, event: &gpui_pre::MouseMoveEvent, _, cx| {
            if !this.pointer_down || !event.dragging() {
                return;
            }
            let current = point(event.position.x.as_f32(), event.position.y.as_f32());
            if let Some(previous) = this.last_pointer {
                this.transform.pan_by(current.x - previous.x, current.y - previous.y);
            }
            this.last_pointer = Some(current);
            cx.notify();
        });
        let up = cx.listener(|this, _: &gpui_pre::MouseUpEvent, _, cx| {
            this.pointer_down = false;
            this.last_pointer = None;
            cx.notify();
        });
        let wheel = cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
            let delta = match event.delta {
                ScrollDelta::Lines(delta) => delta.y,
                ScrollDelta::Pixels(delta) => delta.y.as_f32() / 40.0,
            };
            let pointer = point(event.position.x.as_f32(), event.position.y.as_f32());
            this.transform.zoom_at(pointer, 1.12_f32.powf(delta));
            cx.notify();
        });
        div()
            .id("pan-zoom-viewport")
            .debug_selector(|| "pan-zoom-viewport".into())
            .size_full()
            .on_mouse_down(gpui_pre::MouseButton::Left, down)
            .on_mouse_move(moved)
            .on_mouse_up(gpui_pre::MouseButton::Left, up)
            .on_scroll_wheel(wheel)
            .child(
                canvas(
                    move |bounds, _, cx| {
                        let image = gpui_pre::Image::from_bytes(
                            gpui_pre::ImageFormat::Png,
                            include_bytes!("../assets/mark.png").to_vec(),
                        )
                        .to_image_data(cx.svg_renderer())
                        .ok();
                        (bounds, transform, image)
                    },
                    |bounds, (_, transform, image), window, _| {
                        paint_scene(bounds, transform, window);
                        if let Some(image) = image {
                            let target = Bounds::new(
                                point(bounds.origin.x + px(188.), bounds.origin.y + px(94.)),
                                size(px(36.), px(36.)),
                            );
                            let _ = window.paint_image(target, target, 0.0.into(), image, 0, false);
                        }
                    },
                )
                .size_full(),
            )
            .child(
                div()
                    .absolute()
                    .top_4()
                    .left_4()
                    .px_3()
                    .py_2()
                    .bg(hsla(0.0, 0.0, 1.0, 0.92))
                    .child(format!(
                        "Pan-and-zoom canvas · scale {:.2} · drag to pan, wheel to zoom",
                        self.transform.scale
                    )),
            )
    }
}

fn paint_scene(bounds: Bounds<Pixels>, transform: ViewTransform, window: &mut Window) {
    window.paint_quad(fill(bounds, hsla(0.60, 0.18, 0.10, 1.0)));
    let width = bounds.size.width.as_f32();
    let height = bounds.size.height.as_f32();
    let left = bounds.origin.x.as_f32();
    let top = bounds.origin.y.as_f32();
    let grid_color = hsla(0.58, 0.16, 0.28, 1.0);
    for world_x in -12..=18 {
        let screen_x = left + transform.offset_x + world_x as f32 * 32.0 * transform.scale;
        if screen_x >= left && screen_x <= left + width {
            window.paint_quad(fill(
                Bounds::new(point(px(screen_x), px(top)), size(px(1.), px(height))),
                grid_color,
            ));
        }
    }
    for world_y in -8..=14 {
        let screen_y = top + transform.offset_y + world_y as f32 * 32.0 * transform.scale;
        if screen_y >= top && screen_y <= top + height {
            window.paint_quad(fill(
                Bounds::new(point(px(left), px(screen_y)), size(px(width), px(1.))),
                grid_color,
            ));
        }
    }
    paint_primitives(bounds, window);
    let origin = transform.world_to_screen(point(0.0, 0.0));
    let marker = Bounds::new(
        point(px(left + origin.x - 12.0), px(top + origin.y - 12.0)),
        size(px(24.0 * transform.scale), px(24.0 * transform.scale)),
    );
    window.paint_quad(fill(marker, hsla(0.08, 0.85, 0.62, 1.0)).corner_radii(px(5.0)));
}
// ANCHOR_END: pan_zoom_view

// ANCHOR: custom_element
pub struct PhaseElement {
    phases: Rc<RefCell<Vec<&'static str>>>,
    bounds: Rc<RefCell<Option<Bounds<Pixels>>>>,
    style: StyleRefinement,
}

impl PhaseElement {
    pub fn new(
        phases: Rc<RefCell<Vec<&'static str>>>,
        bounds: Rc<RefCell<Option<Bounds<Pixels>>>>,
    ) -> Self {
        Self { phases, bounds, style: StyleRefinement::default() }
    }
}

impl IntoElement for PhaseElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Styled for PhaseElement {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Element for PhaseElement {
    type RequestLayoutState = Style;
    type PrepaintState = Bounds<Pixels>;

    fn id(&self) -> Option<ElementId> {
        Some("phase-element".into())
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&gpui_pre::GlobalElementId>,
        _: Option<&gpui_pre::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.phases.borrow_mut().push("request_layout");
        let mut style = Style::default();
        style.refine(&self.style);
        let layout = window.request_layout(style.clone(), [], cx);
        (layout, style)
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui_pre::GlobalElementId>,
        _: Option<&gpui_pre::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Window,
        _: &mut App,
    ) -> Self::PrepaintState {
        self.phases.borrow_mut().push("prepaint");
        *self.bounds.borrow_mut() = Some(bounds);
        bounds
    }

    fn paint(
        &mut self,
        _: Option<&gpui_pre::GlobalElementId>,
        _: Option<&gpui_pre::InspectorElementId>,
        bounds: Bounds<Pixels>,
        style: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.phases.borrow_mut().push("paint");
        style.paint(bounds, window, cx, |window, _| {
            window.paint_quad(fill(bounds, hsla(0.38, 0.62, 0.48, 1.0)));
        });
    }
}
// ANCHOR_END: custom_element

// ANCHOR: canvas_primitives
pub fn paint_primitives(bounds: Bounds<Pixels>, window: &mut Window) {
    let box_bounds = Bounds::new(
        point(bounds.origin.x + px(28.), bounds.origin.y + px(30.)),
        size(px(140.), px(88.)),
    );
    window.paint_drop_shadows(
        box_bounds,
        px(10.).into(),
        &[gpui_pre::BoxShadow::new(px(3.), px(5.), hsla(0.0, 0.0, 0.0, 0.55)).blur_radius(px(8.))],
    );
    window.paint_quad(quad(
        box_bounds,
        px(10.),
        hsla(0.53, 0.72, 0.52, 1.0),
        px(2.),
        hsla(0.0, 0.0, 0.9, 1.0),
        gpui_pre::BorderStyle::Solid,
    ));
    let mut path = gpui_pre::PathBuilder::stroke(px(4.));
    path.move_to(point(
        box_bounds.origin.x,
        box_bounds.origin.y + box_bounds.size.height + px(16.),
    ));
    path.line_to(point(
        box_bounds.origin.x + px(45.),
        box_bounds.origin.y + box_bounds.size.height + px(4.),
    ));
    path.line_to(point(
        box_bounds.origin.x + px(90.),
        box_bounds.origin.y + box_bounds.size.height + px(16.),
    ));
    if let Ok(path) = path.build() {
        window.paint_path(path, hsla(0.12, 0.9, 0.65, 1.0));
    }
}
// ANCHOR_END: canvas_primitives

// ANCHOR: overlay_layers
pub fn overlay_layers(hit_layer: Rc<std::cell::Cell<u8>>) -> impl IntoElement {
    let low_hit = hit_layer.clone();
    let high_hit = hit_layer;
    div()
        .relative()
        .size_full()
        .child(
            gpui_pre::deferred(
                div()
                    .absolute()
                    .id("overlay-high")
                    .debug_selector(|| "overlay-high".into())
                    .top_8()
                    .left_8()
                    .w(px(100.))
                    .h(px(50.))
                    .bg(hsla(0.6, 0.8, 0.4, 1.0))
                    .child("higher layer")
                    .on_mouse_down(gpui_pre::MouseButton::Left, move |_, _, _| high_hit.set(2)),
            )
            .with_priority(2),
        )
        .child(
            gpui_pre::deferred(
                div()
                    .absolute()
                    .id("overlay-low")
                    .debug_selector(|| "overlay-low".into())
                    .top_8()
                    .left_8()
                    .w(px(140.))
                    .h(px(80.))
                    .bg(hsla(0.1, 0.8, 0.4, 1.0))
                    .child("lower layer")
                    .on_mouse_down(gpui_pre::MouseButton::Left, move |_, _, _| low_hit.set(1)),
            )
            .with_priority(1),
        )
        .child(
            gpui_pre::anchored()
                .position(point(px(620.), px(460.)))
                .anchor(gpui_pre::Anchor::BottomRight)
                .snap_to_window_with_margin(px(8.))
                .child(
                    div()
                        .id("anchored-popover")
                        .debug_selector(|| "anchored-popover".into())
                        .p_2()
                        .bg(hsla(0.0, 0.0, 0.2, 1.0))
                        .child("anchored popover"),
                ),
        )
}
// ANCHOR_END: overlay_layers

// ANCHOR: virtualized_lists
pub struct VirtualizedLists {
    /// Retain state in the view so scrolling and measured item heights survive renders.
    state: gpui_pre::ListState,
    uniform_rows: Rc<std::cell::Cell<usize>>,
    variable_rows: Rc<std::cell::Cell<usize>>,
}

impl Default for VirtualizedLists {
    fn default() -> Self {
        Self {
            state: gpui_pre::ListState::new(1_000, gpui_pre::ListAlignment::Top, px(50.)),
            uniform_rows: Rc::new(std::cell::Cell::new(0)),
            variable_rows: Rc::new(std::cell::Cell::new(0)),
        }
    }
}

impl Render for VirtualizedLists {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let uniform_rows = self.uniform_rows.clone();
        let variable_rows = self.variable_rows.clone();
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(hsla(0.60, 0.18, 0.10, 1.0))
            .child(
                gpui_pre::uniform_list("uniform-rows", 1_000, move |range, _window, _cx| {
                    uniform_rows.set(uniform_rows.get() + range.len());
                    range
                        .map(|ix| {
                            div()
                                .h(px(24.))
                                .w_full()
                                .px_3()
                                .bg(hsla(0.53, 0.35, 0.24, 1.0))
                                .text_color(hsla(0.0, 0.0, 0.95, 1.0))
                                .child(format!("Uniform row {ix}"))
                        })
                        .collect()
                })
                .h(px(140.)),
            )
            .child(
                gpui_pre::list(self.state.clone(), move |ix, _window, _cx| {
                    variable_rows.set(variable_rows.get() + 1);
                    div()
                        .h(px(if ix % 2 == 0 { 22. } else { 38. }))
                        .w_full()
                        .px_3()
                        .bg(hsla(0.08, 0.42, 0.28, 1.0))
                        .text_color(hsla(0.0, 0.0, 0.95, 1.0))
                        .child(format!("Variable row {ix}"))
                        .into_any_element()
                })
                .h(px(260.)),
            )
    }
}

#[derive(Default)]
pub struct OverlayDemo {
    hit_layer: Rc<std::cell::Cell<u8>>,
}

impl Render for OverlayDemo {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        overlay_layers(self.hit_layer.clone())
    }
}
// ANCHOR_END: virtualized_lists

// ANCHOR: animation_motion
pub fn animation_motion() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(hsla(0.58, 0.38, 0.22, 1.0))
        .with_animation(
            "fade-in",
            gpui_pre::Animation::new(std::time::Duration::from_millis(240)).with_easing(|t| t * t),
            |div, progress| {
                div.opacity(progress)
                    .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                    .child("Animation follows app reduced-motion setting")
            },
        )
}

pub struct AnimationDemo;

impl Render for AnimationDemo {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        animation_motion()
    }
}
// ANCHOR_END: animation_motion

// ANCHOR: surface_macos
#[cfg(target_os = "macos")]
pub fn surface_macos() -> impl IntoElement {
    use core_foundation::{
        base::{CFType, TCFType},
        boolean::CFBoolean,
        dictionary::CFDictionary,
        string::CFString,
    };
    use core_video::pixel_buffer::{
        CVPixelBuffer, CVPixelBufferKeys, kCVPixelFormatType_420YpCbCr8BiPlanarFullRange,
    };
    let iosurface_properties = CFDictionary::<CFString, CFType>::from_CFType_pairs(&[]);
    let options = CFDictionary::from_CFType_pairs(&[
        (CFString::from(CVPixelBufferKeys::IOSurfaceProperties), iosurface_properties.as_CFType()),
        (
            CFString::from(CVPixelBufferKeys::MetalCompatibility),
            CFBoolean::true_value().as_CFType(),
        ),
    ]);
    let buffer =
        CVPixelBuffer::new(kCVPixelFormatType_420YpCbCr8BiPlanarFullRange, 32, 32, Some(&options))
            .expect("CoreVideo should allocate a bi-planar YUV buffer");
    assert_eq!(buffer.lock_base_address(0), core_video::r#return::kCVReturnSuccess);
    let y_row_bytes = buffer.get_bytes_per_row_of_plane(0);
    let uv_row_bytes = buffer.get_bytes_per_row_of_plane(1);
    assert!(buffer.get_width_of_plane(0) >= 32 && buffer.get_height_of_plane(0) >= 32);
    assert!(buffer.get_width_of_plane(1) >= 16 && buffer.get_height_of_plane(1) >= 16);
    assert!(y_row_bytes >= 32 && uv_row_bytes >= 32);
    let y_base = unsafe { buffer.get_base_address_of_plane(0).cast::<u8>() };
    let uv_base = unsafe { buffer.get_base_address_of_plane(1).cast::<u8>() };
    assert!(!y_base.is_null() && !uv_base.is_null());
    for y in 0..32 {
        for x in 0..32 {
            // The Metal renderer in this GPUI pin asserts full-range bi-planar YUV.
            // The buffer is locked, plane dimensions/strides are checked above, and these
            // offsets stay within the reported Y plane allocation.
            unsafe {
                *y_base.add(y * y_row_bytes + x) =
                    if (8..24).contains(&x) && (8..24).contains(&y) { 220 } else { 48 };
            }
        }
    }
    for y in 0..16 {
        for x in 0..16 {
            let offset = y * uv_row_bytes + x * 2;
            // Each UV sample is two bytes; the checked plane has 16 columns and 16 rows.
            unsafe {
                *uv_base.add(offset) = 128;
                *uv_base.add(offset + 1) = 128;
            }
        }
    }
    assert_eq!(buffer.unlock_base_address(0), core_video::r#return::kCVReturnSuccess);
    gpui_pre::surface(buffer).size(px(128.))
}

#[cfg(target_os = "macos")]
pub struct SurfaceDemo;

#[cfg(target_os = "macos")]
impl Render for SurfaceDemo {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .bg(hsla(0.58, 0.36, 0.18, 1.0))
            .child(
                div()
                    .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                    .child("CoreVideo surface · IOSurface + Metal-compatible YUV"),
            )
            .child(
                div()
                    .id("surface-frame")
                    .debug_selector(|| "surface-frame".into())
                    .child(surface_macos()),
            )
    }
}

#[cfg(not(target_os = "macos"))]
pub fn surface_macos() -> impl IntoElement {
    // This pin exposes CVPixelBuffer-backed surface() only on macOS.
    div().p_3().child("surface(CVPixelBuffer) is macOS-only in gpui-pre 0.3.5")
}
// ANCHOR_END: surface_macos

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{TestAppContext, point};

    struct PhaseHarness(Rc<RefCell<Vec<&'static str>>>, Rc<RefCell<Option<Bounds<Pixels>>>>);

    impl Render for PhaseHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(PhaseElement::new(self.0.clone(), self.1.clone()).size_full())
        }
    }

    #[test]
    fn transform_round_trip_and_pointer_anchored_zoom() {
        let mut transform = ViewTransform::default();
        let world = point(42.0, -17.0);
        let screen = transform.world_to_screen(world);
        assert_eq!(transform.screen_to_world(screen), world);
        let pointer = point(230.0, 170.0);
        let world_under_pointer = transform.screen_to_world(pointer);
        transform.zoom_at(pointer, 1.5);
        let after = transform.world_to_screen(world_under_pointer);
        assert!((after.x - pointer.x).abs() < 0.001);
        assert!((after.y - pointer.y).abs() < 0.001);
    }

    #[gpui_pre::test]
    fn custom_element_runs_layout_prepaint_then_paint(cx: &mut TestAppContext) {
        let phases = Rc::new(RefCell::new(Vec::new()));
        let bounds = Rc::new(RefCell::new(None));
        let (_view, visual) = cx.add_window_view({
            let phases = phases.clone();
            let bounds = bounds.clone();
            move |_, _| PhaseHarness(phases, bounds)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let trace = phases.borrow();
        assert!(
            trace.len() >= 3
                && trace
                    .chunks_exact(3)
                    .all(|frame| frame == ["request_layout", "prepaint", "paint"])
        );
        let bounds = bounds.borrow().expect("prepaint receives element bounds");
        assert!(bounds.size.width > px(0.) && bounds.size.height > px(0.));
    }

    #[gpui_pre::test]
    fn pointer_drag_and_wheel_update_pan_and_zoom(cx: &mut TestAppContext) {
        let (view, visual) = cx.add_window_view(|_, _| PanZoomView::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let bounds = visual.debug_bounds("pan-zoom-viewport").unwrap();
        let start = bounds.origin + point(px(150.), px(150.));
        visual.simulate_mouse_down(start, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_move(
            start + point(px(32.), px(18.)),
            Some(gpui_pre::MouseButton::Left),
            Default::default(),
        );
        visual.simulate_mouse_up(
            start + point(px(32.), px(18.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );
        let panned = view.read_with(visual, |view, _| view.transform);
        assert_eq!((panned.offset_x, panned.offset_y), (212.0, 138.0));
        visual.simulate_event(ScrollWheelEvent {
            position: start,
            delta: ScrollDelta::Lines(point(0.0, 1.0)),
            ..Default::default()
        });
        let zoomed = view.read_with(visual, |view, _| view.transform);
        assert!(zoomed.scale > panned.scale);
        let world_at_cursor = panned.screen_to_world(point(start.x.as_f32(), start.y.as_f32()));
        let moved_cursor = zoomed.world_to_screen(world_at_cursor);
        assert!((moved_cursor.x - start.x.as_f32()).abs() < 0.01);
        assert!((moved_cursor.y - start.y.as_f32()).abs() < 0.01);
    }

    #[gpui_pre::test]
    fn anchored_overlay_fits_window_and_priority_changes_paint_not_hit_order(
        cx: &mut TestAppContext,
    ) {
        let (view, visual) = cx.add_window_view(|_, _| OverlayDemo::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let viewport = visual.update(|window, _| window.bounds());
        let low = visual.debug_bounds("overlay-low").expect("lower deferred layer bounds");
        let high = visual.debug_bounds("overlay-high").expect("higher deferred layer bounds");
        assert_eq!(low.origin, high.origin, "deferred layers overlap at the same anchor");
        assert!(high.size.width < low.size.width && high.size.height < low.size.height);

        let popover = visual.debug_bounds("anchored-popover").expect("anchored popover bounds");
        assert!(popover.origin.x >= viewport.origin.x && popover.origin.y >= viewport.origin.y);
        assert!(popover.origin.x + popover.size.width <= viewport.origin.x + viewport.size.width);
        assert!(popover.origin.y + popover.size.height <= viewport.origin.y + viewport.size.height);

        let overlap = high.origin + point(px(8.), px(8.));
        visual.simulate_mouse_down(overlap, gpui_pre::MouseButton::Left, Default::default());
        assert_eq!(
            view.read_with(visual, |view, _| view.hit_layer.get()),
            1,
            "the pinned deferred priority controls draw order only; hit dispatch follows another order"
        );
    }

    #[gpui_pre::test]
    fn virtualized_lists_render_visible_uniform_and_variable_rows(cx: &mut TestAppContext) {
        let (view, visual) = cx.add_window_view(|_, _| VirtualizedLists::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (uniform_rows, variable_rows) =
            view.read_with(visual, |view, _| (view.uniform_rows.get(), view.variable_rows.get()));
        assert!(uniform_rows > 0 && uniform_rows < 1_000);
        assert!(variable_rows > 0 && variable_rows < 1_000);
    }

    struct AnimationHarness(Rc<std::cell::Cell<f32>>);

    impl Render for AnimationHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let observed = self.0.clone();
            div().with_animation(
                "test-eased-animation",
                gpui_pre::Animation::new(std::time::Duration::from_millis(240))
                    .with_easing(|t| t * t),
                move |element, progress| {
                    observed.set(progress);
                    element.opacity(progress).child("eased")
                },
            )
        }
    }

    #[gpui_pre::test]
    fn animation_uses_custom_easing_and_reduced_motion_static_end_state(cx: &mut TestAppContext) {
        let easing =
            gpui_pre::Animation::new(std::time::Duration::from_millis(240)).with_easing(|t| t * t);
        assert!((easing.easing)(0.5) == 0.25, "custom easing maps 0.5 to 0.25");
        let progress = Rc::new(std::cell::Cell::new(-1.0));
        let (_, visual) = cx.add_window_view({
            let progress = progress.clone();
            move |_, _| AnimationHarness(progress)
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|_, cx| cx.set_reduce_motion(true));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(
            (progress.get() - 1.0).abs() < f32::EPSILON,
            "one-shot reduce-motion animation should render its end state"
        );
    }
}
