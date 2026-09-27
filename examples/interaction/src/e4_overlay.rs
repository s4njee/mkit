//! Live GPUI fixture for overlay dismissal and stacking contracts.
//!
//! This fixture intentionally does not treat deferred paint priority as an
//! input barrier. Its background button remains interactive wherever GPUI hit
//! testing routes a press.

use gpui_pre::prelude::*;
use gpui_pre::{
    App, Context, FocusHandle, IntoElement, KeyBinding, Render, Window, actions, div, px,
};
use mkit_core::overlay::{
    DismissalInput, DismissalPolicy, OverlayRect, OverlaySide, Placement, deferred_at, layer,
    should_dismiss,
};

const KEY_CONTEXT: &str = "E4Overlay";

actions!(e4_overlay, [DismissTopOverlay]);

/// Register Escape (or a caller-selected replacement) for the overlay's key context.
pub fn bind_overlay_dismiss_key(cx: &mut App, keystroke: &str) -> Result<(), String> {
    gpui_pre::Keystroke::parse(keystroke)
        .map_err(|error| format!("invalid overlay dismissal keystroke `{keystroke}`: {error}"))?;
    cx.bind_keys([KeyBinding::new(keystroke, DismissTopOverlay, Some(KEY_CONTEXT))]);
    Ok(())
}

/// Small interactive fixture with a trigger, nested overlays, and an underlying button.
#[derive(Default)]
pub struct OverlayDismissalFixture {
    open: Vec<u8>,
    focus: Option<FocusHandle>,
    outside_dismissals: u32,
    escape_dismissals: u32,
    underlying_presses: u32,
}

impl OverlayDismissalFixture {
    /// Construct the stable nested-open state used by the E4.3 screenshot.
    pub fn nested_open() -> Self {
        Self { open: vec![1, 2], ..Self::default() }
    }

    fn open_outer(&mut self) {
        if self.open.is_empty() {
            self.open.push(1);
        }
    }

    fn open_inner(&mut self) {
        if self.open.last() == Some(&1) {
            self.open.push(2);
        }
    }

    fn dismiss_top(&mut self, input: DismissalInput) {
        if let Some(id) = self.open.last().copied() {
            self.dismiss_layer(id, input);
        }
    }

    fn dismiss_layer(&mut self, id: u8, input: DismissalInput) {
        let policy = DismissalPolicy::default();
        if self.open.last() == Some(&id) && should_dismiss(input, policy) {
            self.open.pop();
            match input {
                DismissalInput::OutsidePointerDown => self.outside_dismissals += 1,
                DismissalInput::Escape => self.escape_dismissals += 1,
            }
        }
    }

    /// Exposes stable fixture state for harness assertions.
    pub fn state(&self) -> (Vec<u8>, u32, u32, u32) {
        (
            self.open.clone(),
            self.outside_dismissals,
            self.escape_dismissals,
            self.underlying_presses,
        )
    }
}

impl Render for OverlayDismissalFixture {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<mkit_core::theme::Theme>().colors;
        let spacing = cx.global::<mkit_core::theme::Theme>().spacing;
        if self.focus.is_none() {
            let focus = cx.focus_handle();
            window.focus(&focus, cx);
            self.focus = Some(focus);
        }

        let outer_open = self.open.contains(&1);
        let inner_open = self.open.contains(&2);
        let mut root = div()
            .id("overlay-root")
            .size_full()
            .p(px(spacing.large))
            .bg(colors.background)
            .text_color(colors.text)
            .key_context(KEY_CONTEXT)
            .track_focus(self.focus.as_ref().expect("focus initialized"))
            .on_action(cx.listener(|this, _: &DismissTopOverlay, _, cx| {
                this.dismiss_top(DismissalInput::Escape);
                cx.notify();
            }))
            .child(
                div()
                    .id("open-outer")
                    .debug_selector(|| "open-outer".into())
                    .px(px(spacing.medium))
                    .py(px(spacing.small))
                    .bg(colors.accent)
                    .text_color(colors.accent_text)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open_outer();
                        cx.notify();
                    }))
                    .child("Open outer overlay"),
            )
            .child(
                div()
                    .id("underlying-button")
                    .debug_selector(|| "underlying-button".into())
                    .mt(px(spacing.xlarge))
                    .px(px(spacing.medium))
                    .py(px(spacing.small))
                    .bg(colors.surface)
                    .text_color(colors.text)
                    .border_1()
                    .border_color(colors.border)
                    .on_mouse_down(
                        gpui_pre::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.underlying_presses += 1;
                            cx.notify();
                        }),
                    )
                    .child("Underlying input target"),
            );

        if outer_open {
            let placement = Placement {
                rect: OverlayRect { x: 180.0, y: 120.0, width: 240.0, height: 150.0 },
                side: OverlaySide::Below,
            };
            let mut outer_panel = div()
                .id("outer-overlay")
                .debug_selector(|| "outer-overlay".into())
                .w(px(240.0))
                .h(px(150.0))
                .p(px(spacing.medium))
                .bg(colors.elevated_surface)
                .text_color(colors.text)
                .border_1()
                .border_color(colors.border)
                .child("Outer overlay")
                .child(
                    div()
                        .id("open-inner")
                        .debug_selector(|| "open-inner".into())
                        .mt(px(spacing.small))
                        .px(px(spacing.medium))
                        .py(px(spacing.small))
                        .bg(colors.surface)
                        .text_color(colors.text)
                        .border_1()
                        .border_color(colors.border)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.open_inner();
                            cx.notify();
                        }))
                        .child("Open inner overlay"),
                );
            if self.open.last() == Some(&1) {
                outer_panel = outer_panel.on_mouse_down_out(cx.listener(|this, _, _, cx| {
                    this.dismiss_layer(1, DismissalInput::OutsidePointerDown);
                    cx.notify();
                }));
            }
            root = root.child(deferred_at(outer_panel, placement, 8.0, layer::POPOVER));
        }

        if inner_open {
            let placement = Placement {
                rect: OverlayRect { x: 320.0, y: 225.0, width: 180.0, height: 100.0 },
                side: OverlaySide::Above,
            };
            let inner_panel = div()
                .id("inner-overlay")
                .debug_selector(|| "inner-overlay".into())
                .w(px(180.0))
                .h(px(100.0))
                .p(px(spacing.medium))
                .bg(colors.accent)
                .text_color(colors.accent_text)
                .border_1()
                .border_color(colors.border)
                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                    this.dismiss_layer(2, DismissalInput::OutsidePointerDown);
                    cx.notify();
                }))
                .child("Inner overlay");
            root = root.child(deferred_at(inner_panel, placement, 8.0, layer::MENU));
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Modifiers, TestAppContext, point};

    fn setup(cx: &mut TestAppContext, escape: &str) {
        cx.update(|app| {
            mkit_core::theme::set_light_theme(app);
            bind_overlay_dismiss_key(app, escape).unwrap();
        });
    }

    fn open_nested(
        view: &gpui_pre::Entity<OverlayDismissalFixture>,
        visual: &mut gpui_pre::VisualTestContext,
    ) {
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let outer = visual.debug_bounds("open-outer").unwrap().center();
        visual.simulate_click(outer, Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let inner = visual.debug_bounds("open-inner").unwrap().center();
        visual.simulate_click(inner, Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(view.read_with(visual, |fixture, _| fixture.open.clone()), vec![1, 2]);
    }

    #[gpui_pre::test]
    fn outside_press_dismisses_only_topmost_nested_overlay(cx: &mut TestAppContext) {
        setup(cx, "escape");
        let (view, visual) = cx.add_window_view(|_, _| OverlayDismissalFixture::default());
        open_nested(&view, visual);

        let outside = point(px(740.0), px(540.0));
        visual.simulate_mouse_down(outside, gpui_pre::MouseButton::Left, Modifiers::default());
        assert_eq!(view.read_with(visual, |fixture, _| fixture.state()), (vec![1], 1, 0, 0));

        visual.simulate_mouse_down(outside, gpui_pre::MouseButton::Left, Modifiers::default());
        assert_eq!(view.read_with(visual, |fixture, _| fixture.state()), (vec![], 2, 0, 0));
    }

    #[gpui_pre::test]
    fn escape_binding_is_rebindable_and_dismisses_one_layer(cx: &mut TestAppContext) {
        setup(cx, "ctrl-e");
        let (view, visual) = cx.add_window_view(|_, _| OverlayDismissalFixture::default());
        open_nested(&view, visual);

        visual.simulate_keystrokes("escape");
        assert_eq!(view.read_with(visual, |fixture, _| fixture.open.clone()), vec![1, 2]);
        visual.simulate_keystrokes("ctrl-e");
        assert_eq!(view.read_with(visual, |fixture, _| fixture.state()), (vec![1], 0, 1, 0));
    }

    #[gpui_pre::test]
    fn higher_deferred_priority_paints_the_inner_overlay_last(cx: &mut TestAppContext) {
        setup(cx, "escape");
        let (view, visual) = cx.add_window_view(|_, _| OverlayDismissalFixture::default());
        open_nested(&view, visual);
        let outer = visual.debug_bounds("outer-overlay").unwrap();
        let inner = visual.debug_bounds("inner-overlay").unwrap();
        let overlaps = inner.origin.x < outer.origin.x + outer.size.width
            && inner.origin.x + inner.size.width > outer.origin.x
            && inner.origin.y < outer.origin.y + outer.size.height
            && inner.origin.y + inner.size.height > outer.origin.y;
        assert!(overlaps, "fixture panels overlap so draw order is observable");

        let (quads, scale_factor) = visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            (window.painted_quads(), window.scale_factor())
        });
        let max_background_order = |bounds: gpui_pre::Bounds<gpui_pre::Pixels>| {
            quads
                .iter()
                .filter(|quad| {
                    (quad.bounds.origin.x.as_f32() - bounds.origin.x.as_f32() * scale_factor).abs()
                        < 0.01
                        && (quad.bounds.origin.y.as_f32() - bounds.origin.y.as_f32() * scale_factor)
                            .abs()
                            < 0.01
                        && (quad.bounds.size.width.as_f32()
                            - bounds.size.width.as_f32() * scale_factor)
                            .abs()
                            < 0.01
                        && (quad.bounds.size.height.as_f32()
                            - bounds.size.height.as_f32() * scale_factor)
                            .abs()
                            < 0.01
                })
                .map(|quad| quad.order)
                .max()
                .expect("overlay background quad is present")
        };
        assert!(
            max_background_order(inner) > max_background_order(outer),
            "the menu layer's background quad must be painted after the popover layer"
        );
    }

    #[gpui_pre::test]
    fn outside_press_reaches_an_uncovered_underlying_control(cx: &mut TestAppContext) {
        setup(cx, "escape");
        let (view, visual) = cx.add_window_view(|_, _| OverlayDismissalFixture::default());
        open_nested(&view, visual);
        let target = visual.debug_bounds("underlying-button").unwrap().center();
        visual.simulate_mouse_down(target, gpui_pre::MouseButton::Left, Modifiers::default());
        assert_eq!(
            view.read_with(visual, |fixture, _| fixture.underlying_presses),
            1,
            "deferred drawing does not promise modal input blocking"
        );
    }
}
