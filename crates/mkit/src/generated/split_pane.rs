//! Resizable two-panel layout with a keyboard-accessible splitter.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnyView, App, Bounds, Context, EventEmitter, FocusHandle, Focusable, KeyBinding, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Render, Window, actions, canvas, div,
    prelude::*, px, relative,
};
use mkit_core::theme::Theme;

pub const KEY_CONTEXT: &str = "MkitSplitPane";
actions!(split_pane, [Increase, Decrease, FineIncrease, FineDecrease, Minimum, Maximum]);

pub fn default_key_bindings() -> [KeyBinding; 10] {
    [
        KeyBinding::new("right", Increase, Some(KEY_CONTEXT)),
        KeyBinding::new("down", Increase, Some(KEY_CONTEXT)),
        KeyBinding::new("left", Decrease, Some(KEY_CONTEXT)),
        KeyBinding::new("up", Decrease, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-right", FineIncrease, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-down", FineIncrease, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-left", FineDecrease, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-up", FineDecrease, Some(KEY_CONTEXT)),
        KeyBinding::new("home", Minimum, Some(KEY_CONTEXT)),
        KeyBinding::new("end", Maximum, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RatioChanged(pub f32);
impl EventEmitter<RatioChanged> for SplitPane {}

pub struct SplitPane {
    leading: AnyView,
    trailing: AnyView,
    ratio: f32,
    controlled: bool,
    min_ratio: f32,
    max_ratio: f32,
    orientation: Orientation,
    disabled: bool,
    dragging: bool,
    bounds: Option<Bounds<Pixels>>,
    focus: Option<FocusHandle>,
}

impl SplitPane {
    pub fn new(
        leading: impl Into<AnyView>,
        trailing: impl Into<AnyView>,
        default_ratio: f32,
    ) -> Self {
        Self {
            leading: leading.into(),
            trailing: trailing.into(),
            ratio: default_ratio.clamp(0.0, 1.0),
            controlled: false,
            min_ratio: 0.1,
            max_ratio: 0.9,
            orientation: Orientation::Horizontal,
            disabled: false,
            dragging: false,
            bounds: None,
            focus: None,
        }
    }
    pub fn controlled(
        leading: impl Into<AnyView>,
        trailing: impl Into<AnyView>,
        ratio: f32,
    ) -> Self {
        let mut pane = Self::new(leading, trailing, ratio);
        pane.controlled = true;
        pane
    }
    pub fn bounds(mut self, min_ratio: f32, max_ratio: f32) -> Self {
        self.min_ratio = min_ratio.clamp(0.0, 1.0);
        self.max_ratio = max_ratio.clamp(self.min_ratio, 1.0);
        self.ratio = self.ratio.clamp(self.min_ratio, self.max_ratio);
        self
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn ratio(&self) -> f32 {
        self.ratio
    }
    pub fn set_ratio(&mut self, ratio: f32, cx: &mut Context<Self>) {
        self.ratio = ratio.clamp(self.min_ratio, self.max_ratio);
        cx.notify();
    }
    fn request(&mut self, ratio: f32, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let next = ratio.clamp(self.min_ratio, self.max_ratio);
        if (next - self.ratio).abs() < f32::EPSILON {
            return;
        }
        if !self.controlled {
            self.ratio = next;
            cx.notify();
        }
        cx.emit(RatioChanged(next));
    }
    fn increase(&mut self, _: &Increase, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.ratio + 0.02, cx)
    }
    fn decrease(&mut self, _: &Decrease, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.ratio - 0.02, cx)
    }
    fn fine_increase(&mut self, _: &FineIncrease, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.ratio + 0.005, cx)
    }
    fn fine_decrease(&mut self, _: &FineDecrease, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.ratio - 0.005, cx)
    }
    fn minimum(&mut self, _: &Minimum, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.min_ratio, cx)
    }
    fn maximum(&mut self, _: &Maximum, _: &mut Window, cx: &mut Context<Self>) {
        self.request(self.max_ratio, cx)
    }
    fn pointer_down(&mut self, event: &MouseDownEvent, _: &mut Window, _: &mut Context<Self>) {
        if !self.disabled && event.button == MouseButton::Left {
            self.dragging = true;
        }
    }
    fn pointer_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.dragging || !event.dragging() {
            return;
        }
        let Some(bounds) = self.bounds else { return };
        let fraction = match self.orientation {
            Orientation::Horizontal => {
                (f32::from(event.position.x) - f32::from(bounds.origin.x))
                    / f32::from(bounds.size.width).max(1.0)
            }
            Orientation::Vertical => {
                (f32::from(event.position.y) - f32::from(bounds.origin.y))
                    / f32::from(bounds.size.height).max(1.0)
            }
        };
        self.request(fraction, cx);
    }
    fn pointer_up(&mut self, event: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        if event.button == MouseButton::Left {
            self.dragging = false;
        }
    }
}

impl Focusable for SplitPane {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone().expect("split pane focus initialized by render")
    }
}

impl Render for SplitPane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let tab_stop = !self.disabled;
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_stop(tab_stop)).clone();
        let entity = cx.entity();
        let horizontal = self.orientation == Orientation::Horizontal;
        let ratio = self.ratio.clamp(self.min_ratio, self.max_ratio);
        let divider = div()
            .id("mkit-split-pane-divider")
            .debug_selector(|| "mkit-split-pane-divider".into())
            .key_context(KEY_CONTEXT)
            .when(!self.disabled, |el| el.track_focus(&focus))
            .role(gpui_pre::accesskit::Role::Splitter)
            .aria_label("Resize panels")
            .aria_orientation(if horizontal {
                gpui_pre::accesskit::Orientation::Vertical
            } else {
                gpui_pre::accesskit::Orientation::Horizontal
            })
            .aria_numeric_value((ratio * 100.0) as f64)
            .aria_min_numeric_value((self.min_ratio * 100.0) as f64)
            .aria_max_numeric_value((self.max_ratio * 100.0) as f64)
            .when(horizontal, |el| el.w(px(theme.spacing.xsmall)).h_full())
            .when(!horizontal, |el| el.h(px(theme.spacing.xsmall)).w_full())
            .bg(if self.disabled { theme.colors.disabled } else { theme.colors.border })
            .focus_visible(|el| el.bg(theme.colors.focus))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::pointer_down));
        div()
            .id("mkit-split-pane")
            .debug_selector(|| "mkit-split-pane".into())
            .flex()
            .when(horizontal, |el| el.flex_row())
            .when(!horizontal, |el| el.flex_col())
            .size_full()
            .on_mouse_move(cx.listener(Self::pointer_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::pointer_up))
            .on_action(cx.listener(Self::increase))
            .on_action(cx.listener(Self::decrease))
            .on_action(cx.listener(Self::fine_increase))
            .on_action(cx.listener(Self::fine_decrease))
            .on_action(cx.listener(Self::minimum))
            .on_action(cx.listener(Self::maximum))
            .child(
                div()
                    .when(horizontal, |el| el.w(relative(ratio)))
                    .when(!horizontal, |el| el.h(relative(ratio)))
                    .overflow_hidden()
                    .child(self.leading.clone()),
            )
            .child(divider)
            .child(div().flex_1().overflow_hidden().child(self.trailing.clone()))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, (), _, cx| {
                        entity.update(cx, |pane, _| pane.bounds = Some(bounds));
                    },
                )
                .absolute()
                .inset_0(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Modifiers, TestAppContext, point};

    struct Panel;
    impl Render for Panel {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full()
        }
    }

    #[gpui_pre::test]
    fn divider_supports_keyboard_and_pointer_resizing(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (pane, visual) =
            cx.add_window_view(|_, cx| SplitPane::new(cx.new(|_| Panel), cx.new(|_| Panel), 0.5));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| pane.focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("right");
        assert!((pane.read_with(visual, |pane, _| pane.ratio()) - 0.52).abs() < 0.001);

        visual.update(|window, cx| window.draw(cx).clear(cx));
        let divider = visual.debug_bounds("mkit-split-pane-divider").unwrap().center();
        let end = divider + point(px(50.), px(0.));
        visual.simulate_mouse_down(divider, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        visual.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        assert!(pane.read_with(visual, |pane, _| pane.ratio()) > 0.52);
    }
}
