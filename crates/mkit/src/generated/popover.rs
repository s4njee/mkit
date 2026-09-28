//! Stateful popover with a component-owned trigger and a GPUI anchored surface.
extern crate gpui_pre as gpui;
use std::{cell::RefCell, rc::Rc};

use gpui_pre::{
    Anchor, AnchoredPositionMode, Bounds, BoxShadow, ClickEvent, Context, EventEmitter,
    FocusHandle, Focusable, FontWeight, IntoElement, KeyBinding, MouseDownEvent, Pixels, Render,
    Rgba, Size, WeakFocusHandle, Window, actions, anchored, deferred, div, point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};

pub const KEY_CONTEXT: &str = "Popover";
actions!(popover, [Dismiss]);
pub fn default_key_bindings() -> [KeyBinding; 1] {
    [KeyBinding::new("escape", Dismiss, Some(KEY_CONTEXT))]
}

/// Resolved colours for the trigger and surface; see the spec's theme table.
#[derive(Clone, Copy)]
struct Look {
    trigger_border: Rgba,
    trigger_hover_bg: Option<Rgba>,
    trigger_hover_border: Option<Rgba>,
    ring: Rgba,
    surface_bg: Rgba,
    border: Rgba,
    /// Whether the trigger and surface draw their shadow tokens.
    shadows: bool,
}

/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight, ..foreground }, Rgba { a: 1.0, ..base })
}

fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            trigger_border: c.border,
            trigger_hover_bg: None,
            trigger_hover_border: Some(c.accent),
            ring: c.focus,
            surface_bg: c.background,
            border: c.border,
            shadows: false,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let hairline = if dark { c.text.opacity(0.1) } else { c.border };
    Look {
        trigger_border: hairline,
        trigger_hover_bg: Some(mix(c.text, c.background, if dark { 0.12 } else { 0.04 })),
        trigger_hover_border: None,
        ring: c.focus.opacity(0.5),
        surface_bg: c.surface,
        border: hairline,
        shadows: true,
    }
}

fn box_shadow(shadow: ShadowToken) -> BoxShadow {
    BoxShadow {
        color: shadow.color.into(),
        offset: point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}

/// shadcn/ui focus ring width, drawn outside the trigger (the same value Button uses).
const FOCUS_RING_WIDTH: f32 = 3.0;

fn focus_ring(color: Rgba) -> BoxShadow {
    BoxShadow {
        color: color.into(),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(FOCUS_RING_WIDTH),
        inset: false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenChanged(pub bool);
impl EventEmitter<OpenChanged> for Popover {}

pub struct Popover {
    title: String,
    content: String,
    trigger: String,
    open: bool,
    controlled: bool,
    busy: bool,
    anchor: Option<gpui_pre::Point<gpui_pre::Pixels>>,
    trigger_bounds: Rc<RefCell<Option<Bounds<Pixels>>>>,
    surface_size: Rc<RefCell<Option<Size<Pixels>>>>,
    anchor_from_trigger: bool,
    trigger_focus: Option<FocusHandle>,
    return_focus_to: Option<WeakFocusHandle>,
    last_open: bool,
    trigger_mouse_down: bool,
}

impl Popover {
    /// Create a closed, uncontrolled popover. Use [`Self::trigger`] to set its trigger label.
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            trigger: "Open popover".into(),
            open: false,
            controlled: false,
            busy: false,
            anchor: None,
            trigger_bounds: Rc::new(RefCell::new(None)),
            surface_size: Rc::new(RefCell::new(None)),
            anchor_from_trigger: false,
            trigger_focus: None,
            return_focus_to: None,
            last_open: false,
            trigger_mouse_down: false,
        }
    }

    /// Create a popover whose open state is owned by the caller.
    pub fn controlled(title: impl Into<String>, content: impl Into<String>, open: bool) -> Self {
        Self { open, controlled: true, ..Self::new(title, content) }
    }

    /// Set the visible label for the focusable trigger.
    pub fn trigger(mut self, label: impl Into<String>) -> Self {
        self.trigger = label.into();
        self
    }

    /// Provide the initial window-coordinate anchor for an initially open surface.
    ///
    /// A trigger click replaces this with the trigger's measured bounds. Until then,
    /// the point is used for an initially open controlled surface.
    pub fn anchor_at(mut self, position: gpui_pre::Point<gpui_pre::Pixels>) -> Self {
        self.anchor = Some(position);
        self
    }

    /// Set the handle that receives focus when the surface closes.
    pub fn return_focus_to(mut self, handle: &FocusHandle) -> Self {
        self.return_focus_to = Some(handle.downgrade());
        self
    }

    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.open = open;
        cx.notify();
    }

    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    fn dismiss(&mut self, _: &Dismiss, window: &mut Window, cx: &mut Context<Self>) {
        self.request_dismiss(window, cx);
    }

    fn outside_down(&mut self, _: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        // The floating surface is a sibling of the trigger. Its outside listener therefore
        // observes trigger presses too; let the trigger's click handler perform the toggle.
        if !self.trigger_mouse_down {
            self.request_dismiss(window, cx);
        }
    }

    fn trigger_down(&mut self, _: &MouseDownEvent, _: &mut Window, _: &mut Context<Self>) {
        self.trigger_mouse_down = true;
    }

    fn trigger_click(&mut self, event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.trigger_mouse_down = false;
        self.anchor = Some(event.position());
        self.anchor_from_trigger = true;
        if self.open {
            self.request_dismiss(window, cx);
        } else {
            if !self.controlled {
                self.open = true;
            }
            cx.emit(OpenChanged(true));
            cx.notify();
        }
    }

    fn request_dismiss(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if !self.open || self.busy {
            return;
        }
        if !self.controlled {
            self.open = false;
        }
        cx.emit(OpenChanged(false));
        cx.notify();
    }

    fn restore_focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(handle) = self
            .return_focus_to
            .as_ref()
            .and_then(WeakFocusHandle::upgrade)
            .or_else(|| self.trigger_focus.clone())
        {
            handle.focus(window, cx);
        }
    }
}

impl Focusable for Popover {
    fn focus_handle(&self, cx: &gpui_pre::App) -> FocusHandle {
        self.trigger_focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}

impl Render for Popover {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let trigger_focus =
            self.trigger_focus.get_or_insert_with(|| cx.focus_handle().tab_stop(true)).clone();
        if self.open && !self.last_open {
            if self.return_focus_to.is_none()
                && let Some(previous) = window.focused(cx)
                && previous != trigger_focus
            {
                self.return_focus_to = Some(previous.downgrade());
            }
        } else if !self.open && self.last_open {
            self.restore_focus(window, cx);
        }
        self.last_open = self.open;

        let look = look(&theme);
        let trigger = div()
            .id("mkit-popover-trigger")
            .debug_selector(|| "mkit-popover-trigger".into())
            .role(gpui_pre::accesskit::Role::Button)
            .track_focus(&trigger_focus)
            .tab_stop(true)
            // shadcn outline button, matching the Button component's outline variant.
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(px(theme.controls.medium))
            .px(px(theme.spacing.large))
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.regular))
            .border_color(look.trigger_border)
            .bg(theme.colors.background)
            .when(look.shadows, |e| e.shadow(vec![box_shadow(theme.shadows.small)]))
            .text_color(theme.colors.text)
            .text_size(px(theme.typography.body))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .hover(move |s| {
                let s = match look.trigger_hover_bg {
                    Some(color) => s.bg(color),
                    None => s,
                };
                match look.trigger_hover_border {
                    Some(color) => s.border_color(color),
                    None => s,
                }
            })
            .focus_visible(move |s| {
                s.border_color(theme.colors.focus).shadow(vec![focus_ring(look.ring)])
            })
            .on_mouse_down(gpui_pre::MouseButton::Left, cx.listener(Self::trigger_down))
            .on_click(cx.listener(Self::trigger_click))
            .child(self.trigger.clone());

        let trigger_bounds = self.trigger_bounds.borrow().as_ref().copied();
        let surface_height = self.surface_size.borrow().map(|size| size.height);
        let surface = div()
            .id("mkit-popover-surface")
            .debug_selector(|| "mkit-popover-surface".into())
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::dismiss))
            .on_mouse_down_out(cx.listener(Self::outside_down))
            .role(gpui_pre::accesskit::Role::Dialog)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.xsmall))
            .p(px(theme.spacing.large))
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.regular))
            .border_color(look.border)
            .bg(look.surface_bg)
            .when(look.shadows, |e| e.shadow(vec![box_shadow(theme.shadows.medium)]))
            .text_color(theme.colors.text)
            .text_size(px(theme.typography.body))
            // The deferred child is measured during its first prepaint, after anchored
            // placement has already been selected. Keep that measurement frame invisible
            // so the first visible frame can use the measured height and flip correctly.
            .when(surface_height.is_none(), |surface| surface.opacity(0.))
            .child(div().font_weight(FontWeight::SEMIBOLD).child(self.title.clone()))
            .child(div().text_color(theme.colors.text_muted).child(self.content.clone()));

        let measured_anchor = self
            .trigger_bounds
            .borrow()
            .as_ref()
            .map(|bounds| point(bounds.left(), bounds.bottom() + px(theme.spacing.small)));
        let should_flip = trigger_bounds.zip(surface_height).is_some_and(|(trigger, height)| {
            let viewport_height = window.viewport_size().height;
            let margin = px(theme.spacing.medium);
            let gap = px(theme.spacing.small);
            let below = viewport_height - trigger.bottom() - gap - margin;
            let above = trigger.top() - gap - margin;
            below < height && above > below
        });
        let anchor =
            if self.anchor_from_trigger { measured_anchor.or(self.anchor) } else { self.anchor };
        let position = if should_flip {
            trigger_bounds
                .map(|bounds| point(bounds.left(), bounds.top() - px(theme.spacing.small)))
        } else {
            anchor
        };
        let surface_size = Rc::clone(&self.surface_size);
        let entity = cx.entity();
        let measured_surface = div()
            .on_children_prepainted(move |children, _, cx| {
                if let Some(bounds) = children.first() {
                    let changed = *surface_size.borrow() != Some(bounds.size);
                    if changed {
                        *surface_size.borrow_mut() = Some(bounds.size);
                        entity.update(cx, |_, cx| cx.notify());
                    }
                }
            })
            .child(surface);
        let floating = self.open.then(|| {
            deferred(
                anchored()
                    .anchor(if should_flip { Anchor::BottomLeft } else { Anchor::TopLeft })
                    .position_mode(AnchoredPositionMode::Window)
                    .position(position.unwrap_or_else(|| point(px(0.), px(0.))))
                    .snap_to_window_with_margin(px(theme.spacing.medium))
                    .child(measured_surface),
            )
            .with_priority(mkit_core::overlay::layer::POPOVER)
        });

        let trigger_bounds = Rc::clone(&self.trigger_bounds);
        div()
            .on_children_prepainted(move |children, _, _| {
                if let Some(bounds) = children.first() {
                    *trigger_bounds.borrow_mut() = Some(*bounds);
                }
            })
            .id(("mkit-popover", cx.entity().entity_id()))
            // A row lets the outline trigger size to its label instead of stretching.
            .flex()
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::dismiss))
            .child(trigger)
            .children(floating)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Context, IntoElement, ParentElement, Render, TestAppContext, div, px};

    struct Host {
        popover: Option<gpui_pre::Entity<Popover>>,
        outside: FocusHandle,
    }

    struct BottomHost {
        popover: Option<gpui_pre::Entity<Popover>>,
        outside: FocusHandle,
    }

    impl Render for BottomHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.popover.is_none() {
                self.popover = Some(cx.new(|_| {
                    Popover::new("title", "body\nbody\nbody\nbody\nbody").trigger("Details")
                }));
            }
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(div().h(px(950.)))
                .child(self.popover.as_ref().unwrap().clone())
                .child(div().id("outside").track_focus(&self.outside).tab_stop(true))
        }
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if self.popover.is_none() {
                self.popover = Some(cx.new(|_| Popover::new("title", "body").trigger("Details")));
            }
            div()
                .size_full()
                .child(self.popover.as_ref().unwrap().clone())
                .child(div().id("outside").track_focus(&self.outside).tab_stop(true))
        }
    }

    #[gpui_pre::test]
    fn trigger_opens_escape_closes_and_returns_focus(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, cx| Host {
            popover: None,
            outside: cx.focus_handle().tab_stop(true),
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let popover = host.read_with(visual, |host, _| host.popover.as_ref().unwrap().clone());
        let trigger_focus = popover.read_with(visual, |popover, cx| popover.focus_handle(cx));
        visual.simulate_mouse_down(
            point(px(10.), px(10.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );
        visual.simulate_mouse_up(
            point(px(10.), px(10.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );
        assert!(popover.read_with(visual, |popover, _| popover.is_open()));
        visual.simulate_keystrokes("escape");
        assert!(!popover.read_with(visual, |popover, _| popover.is_open()));
        assert!(visual.update(|window, _| trigger_focus.is_focused(window)));
    }

    #[gpui_pre::test]
    fn outside_pointer_down_dismisses_and_busy_suppresses_it(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (view, visual) = cx.add_window_view(|_, _| {
            Popover::new("title", "body").anchor_at(point(px(10.), px(10.)))
        });
        view.update(visual, |popover, cx| popover.set_open(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_mouse_down(
            point(px(400.), px(400.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );
        assert!(view.read_with(visual, |popover, _| !popover.is_open()));

        let (busy, visual) = cx.add_window_view(|_, _| {
            Popover::new("title", "body").busy(true).anchor_at(point(px(10.), px(10.)))
        });
        busy.update(visual, |popover, cx| popover.set_open(true, cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_mouse_down(
            point(px(400.), px(400.)),
            gpui_pre::MouseButton::Left,
            Default::default(),
        );
        assert!(busy.read_with(visual, |popover, _| popover.is_open()));
    }

    #[gpui_pre::test]
    fn trigger_click_positions_surface_below_its_measured_left_edge(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) = cx.add_window_view(|_, cx| Host {
            popover: None,
            outside: cx.focus_handle().tab_stop(true),
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let view = host.read_with(visual, |host, _| host.popover.as_ref().unwrap().clone());
        let trigger = visual.debug_bounds("mkit-popover-trigger").expect("trigger bounds");
        let center = trigger.center();
        visual.simulate_mouse_down(center, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_up(center, gpui_pre::MouseButton::Left, Default::default());
        assert!(view.read_with(visual, |popover, _| popover.is_open()));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let surface = visual.debug_bounds("mkit-popover-surface").expect("open surface bounds");
        let gap = visual.update(|_, cx| cx.global::<Theme>().spacing.small);
        assert!((f32::from(surface.left()) - f32::from(trigger.left())).abs() <= 1.0);
        assert!(
            f32::from(surface.top()) >= f32::from(trigger.bottom()) + gap - 1.0,
            "surface should clear the trigger by the theme gap"
        );
    }

    #[gpui_pre::test]
    fn surface_flips_above_near_viewport_bottom_and_keeps_gap(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (host, visual) = cx.add_window_view(|_, cx| BottomHost {
            popover: None,
            outside: cx.focus_handle().tab_stop(true),
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let view = host.read_with(visual, |host, _| host.popover.as_ref().unwrap().clone());
        let trigger = visual.debug_bounds("mkit-popover-trigger").expect("trigger bounds");
        let center = trigger.center();
        visual.simulate_mouse_down(center, gpui_pre::MouseButton::Left, Default::default());
        visual.simulate_mouse_up(center, gpui_pre::MouseButton::Left, Default::default());
        assert!(view.read_with(visual, |popover, _| popover.is_open()));
        // The first open frame measures the deferred surface while transparent. The first
        // visible frame then uses that measurement for placement.
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let trigger = visual.debug_bounds("mkit-popover-trigger").expect("trigger bounds");
        let surface = visual.debug_bounds("mkit-popover-surface").expect("open surface bounds");
        let (gap, viewport_height) = visual.update(|window, cx| {
            (cx.global::<Theme>().spacing.small, window.viewport_size().height)
        });
        assert!(
            f32::from(surface.bottom()) <= f32::from(trigger.top()) - gap + 1.0,
            "surface should flip above the trigger with the theme gap: {surface:?} / {trigger:?}"
        );
        assert!(f32::from(surface.top()) >= 0.0);
        assert!(f32::from(surface.bottom()) <= f32::from(viewport_height));
    }

    #[test]
    fn controlled_mode_remains_authoritative() {
        let popover = Popover::controlled("title", "body", false);
        assert!(popover.controlled);
        assert!(!popover.open);
    }
}
