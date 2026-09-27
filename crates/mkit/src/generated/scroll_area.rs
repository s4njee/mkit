//! A themed, independently scrollable viewport backed by GPUI's ScrollHandle.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnyElement, App, IntoElement, KeyBinding, RenderOnce, ScrollHandle, Window, actions, div,
    point, prelude::*, px,
};
use mkit_core::theme::Theme;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
pub const KEY_CONTEXT: &str = "MkitScrollArea";
actions!(scroll_area, [PageUp, PageDown, LineUp, LineDown, Start, End]);

pub fn default_key_bindings() -> [KeyBinding; 6] {
    [
        KeyBinding::new("pageup", PageUp, Some(KEY_CONTEXT)),
        KeyBinding::new("pagedown", PageDown, Some(KEY_CONTEXT)),
        KeyBinding::new("up", LineUp, Some(KEY_CONTEXT)),
        KeyBinding::new("down", LineDown, Some(KEY_CONTEXT)),
        KeyBinding::new("home", Start, Some(KEY_CONTEXT)),
        KeyBinding::new("end", End, Some(KEY_CONTEXT)),
    ]
}

fn set_vertical_offset(handle: &ScrollHandle, desired: f32, window: &mut Window) {
    let offset = handle.offset();
    let limit = f32::from(handle.max_offset().y);
    handle.set_offset(point(offset.x, px(desired.clamp(-limit, 0.0))));
    window.refresh();
}

#[derive(IntoElement)]
pub struct ScrollArea {
    id: usize,
    label: Option<String>,
    height: f32,
    width: Option<f32>,
    handle: ScrollHandle,
    content: AnyElement,
}

impl ScrollArea {
    pub fn new(height: f32, content: impl IntoElement) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            label: None,
            height: height.max(0.0),
            width: None,
            handle: ScrollHandle::new(),
            content: content.into_any_element(),
        }
    }
    pub fn id(mut self, id: usize) -> Self {
        self.id = id;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.0));
        self
    }
    pub fn handle(mut self, handle: ScrollHandle) -> Self {
        self.handle = handle;
        self
    }
}

impl RenderOnce for ScrollArea {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let page_up = self.handle.clone();
        let page_down = self.handle.clone();
        let line_up = self.handle.clone();
        let line_down = self.handle.clone();
        let start = self.handle.clone();
        let end = self.handle.clone();
        div()
            .id(("mkit-scroll-area", self.id))
            .key_context(KEY_CONTEXT)
            .tab_index(0)
            .role(gpui_pre::accesskit::Role::ScrollView)
            .when_some(self.label, |el, label| el.aria_label(label))
            .h(px(self.height))
            .when_some(self.width, |el, width| el.w(px(width)))
            .overflow_x_scroll()
            .overflow_y_scroll()
            .track_scroll(&self.handle)
            .bg(theme.colors.surface)
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .rounded(px(theme.radii.medium))
            .focus_visible(|el| el.border_color(theme.colors.focus))
            .on_action(move |_: &PageUp, window, _| {
                let y = f32::from(page_up.offset().y) + f32::from(page_up.bounds().size.height);
                set_vertical_offset(&page_up, y, window);
            })
            .on_action(move |_: &PageDown, window, _| {
                let y = f32::from(page_down.offset().y) - f32::from(page_down.bounds().size.height);
                set_vertical_offset(&page_down, y, window);
            })
            .on_action(move |_: &LineUp, window, _| {
                let y = f32::from(line_up.offset().y) + theme.spacing.large * 2.0;
                set_vertical_offset(&line_up, y, window);
            })
            .on_action(move |_: &LineDown, window, _| {
                let y = f32::from(line_down.offset().y) - theme.spacing.large * 2.0;
                set_vertical_offset(&line_down, y, window);
            })
            .on_action(move |_: &Start, window, _| set_vertical_offset(&start, 0.0, window))
            .on_action(move |_: &End, window, _| {
                let y = -f32::from(end.max_offset().y);
                set_vertical_offset(&end, y, window);
            })
            .child(self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Context, Modifiers, Render, TestAppContext};

    struct Host {
        handle: ScrollHandle,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            ScrollArea::new(80.0, div().h(px(800.0)).child("Long content"))
                .id(1)
                .label("Details")
                .width(240.0)
                .handle(self.handle.clone())
        }
    }

    #[gpui_pre::test]
    fn page_down_scrolls_the_focused_viewport(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (host, visual) = cx.add_window_view(|_, _| Host { handle: ScrollHandle::new() });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_click(point(px(12.0), px(12.0)), Modifiers::default());
        visual.simulate_keystrokes("pagedown");
        visual.update(|window, cx| window.draw(cx).clear(cx));
        host.read_with(visual, |host, _| {
            assert!(f32::from(host.handle.offset().y) < 0.0);
        });
    }
}
