//! Stateless tooltip wrapper using GPUI's delayed hover tooltip support.
extern crate gpui_pre as gpui;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use gpui_pre::{AnyElement, IntoElement, Render, RenderOnce, Window, div, prelude::*, px};
use mkit_core::theme::Theme;

const SHOW_DELAY: Duration = Duration::from_millis(500);
static NEXT_TOOLTIP_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(IntoElement)]
pub struct Tooltip {
    label: String,
    child: AnyElement,
    disabled: bool,
    id: usize,
}

impl Tooltip {
    pub fn new(label: impl Into<String>, child: impl IntoElement) -> Self {
        Self {
            label: label.into(),
            child: child.into_any_element(),
            disabled: false,
            id: NEXT_TOOLTIP_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
}

struct TooltipLabel(String);

impl Render for TooltipLabel {
    fn render(&mut self, _: &mut Window, cx: &mut gpui_pre::Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        div()
            .px(px(t.spacing.small))
            .py(px(t.spacing.xsmall))
            .rounded(px(t.radii.small))
            .border(px(t.borders.hairline))
            .border_color(t.colors.border)
            .bg(t.colors.elevated_surface)
            .text_color(t.colors.text)
            .text_size(px(t.typography.caption))
            .child(self.0.clone())
    }
}

impl RenderOnce for Tooltip {
    fn render(self, _: &mut Window, _: &mut gpui_pre::App) -> impl IntoElement {
        let mut root = div().id(("mkit-tooltip-trigger", self.id)).relative().child(self.child);
        if !self.disabled {
            let label = self.label;
            root = root
                .aria_description(label.clone())
                .tooltip_show_delay(SHOW_DELAY)
                .tooltip(move |_, cx| cx.new(|_| TooltipLabel(label.clone())).into());
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Context, FocusHandle, Focusable, KeyBinding, TestAppContext, actions};

    const KEY_CONTEXT: &str = "TooltipFocusTest";
    actions!(tooltip_test, [FocusNext]);

    struct FocusChild {
        focus: FocusHandle,
    }

    impl Focusable for FocusChild {
        fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
            self.focus.clone()
        }
    }

    impl Render for FocusChild {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .id("tooltip-focus-child")
                .track_focus(&self.focus)
                .tab_stop(true)
                .role(gpui_pre::accesskit::Role::Button)
                .aria_label("Focus child")
                .w(px(40.0))
                .h(px(40.0))
        }
    }

    struct TooltipHost {
        child: gpui_pre::Entity<FocusChild>,
        before: FocusHandle,
        after: FocusHandle,
    }

    impl Render for TooltipHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .key_context(KEY_CONTEXT)
                .on_action(cx.listener(|_, _: &FocusNext, window, cx| window.focus_next(cx)))
                .child(div().id("before").track_focus(&self.before).tab_stop(true))
                .child(Tooltip::new("Focus child help", self.child.clone()))
                .child(div().id("after").track_focus(&self.after).tab_stop(true))
        }
    }

    #[test]
    fn disabled_is_stored() {
        let x = Tooltip::new("Help", div()).disabled(true);
        assert!(x.disabled);
    }

    #[test]
    fn hover_delay_is_half_second() {
        assert_eq!(SHOW_DELAY, Duration::from_millis(500));
    }

    #[gpui_pre::test]
    fn tab_focus_stays_on_the_wrapped_child(cx: &mut TestAppContext) {
        cx.update(|app| {
            mkit_core::theme::set_light_theme(app);
            app.bind_keys([KeyBinding::new("tab", FocusNext, Some(KEY_CONTEXT))]);
        });
        let (host, visual) = cx.add_window_view(|_, cx| {
            let focus = cx.focus_handle().tab_stop(true);
            let child = cx.new(|_| FocusChild { focus });
            TooltipHost {
                child,
                before: cx.focus_handle().tab_stop(true),
                after: cx.focus_handle().tab_stop(true),
            }
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let (before, after) =
            host.read_with(visual, |host, _| (host.before.clone(), host.after.clone()));
        visual.update(|window, cx| before.focus(window, cx));
        visual.simulate_keystrokes("tab");
        assert!(visual.update(|window, cx| host.read(cx).child.read(cx).focus.is_focused(window)));
        visual.simulate_keystrokes("tab");
        assert!(visual.update(|window, _| after.is_focused(window)));
    }
}
