//! A themed, independently scrollable viewport backed by GPUI's ScrollHandle.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnyElement, App, Bounds, IntoElement, KeyBinding, Pixels, RenderOnce, Rgba, ScrollHandle,
    Window, actions, canvas, div, fill, point, prelude::*, px, size,
};
use mkit_core::{contrast::relative_luminance, theme::Theme};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Colours derived from theme tokens; see the spec's "Theme tokens used" table.
struct Look {
    border: Rgba,
    thumb: Rgba,
    focus: Rgba,
    ring: Rgba,
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look { border: c.border, thumb: c.border, focus: c.focus, ring: c.focus };
    }
    // shadcn "border": the theme border in light themes, `text` at 10% in dark themes.
    let line = if relative_luminance(c.background) < 0.5 { c.text.opacity(0.1) } else { c.border };
    Look { border: line, thumb: line, focus: c.focus, ring: c.focus.opacity(0.5) }
}
/// shadcn/ui focus ring width, drawn outside the viewport.
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
/// Paint overlay scrollbar thumbs for the viewport tracked by `handle`. The thumbs only
/// indicate position; wheel, trackpad, and keyboard scrolling stay with GPUI and the actions.
/// `inset` is the viewport border, `track` the track thickness and `pad` the gap around a thumb.
fn paint_thumbs(
    handle: &ScrollHandle,
    inset: f32,
    track: f32,
    pad: f32,
    min_thumb: f32,
    color: Rgba,
    window: &mut Window,
) {
    let bounds = handle.bounds();
    let max = handle.max_offset();
    let offset = handle.offset();
    let (inset, track, pad) = (px(inset), px(track), px(pad));
    let inner = Bounds::new(
        bounds.origin + point(inset, inset),
        size(bounds.size.width - inset * 2.0, bounds.size.height - inset * 2.0),
    );
    let (vertical, horizontal) = (max.y > px(0.5), max.x > px(0.5));
    // One axis's thumb stops short of the other's track, like a scrollbar corner.
    let thumb = |length: Pixels, visible: Pixels, extent: Pixels, scrolled: Pixels| {
        let length = length - pad * 2.0;
        let size = (length * (visible / (visible + extent))).max(px(min_thumb)).min(length);
        let travel = length - size;
        let at = pad + travel * (-scrolled / extent).clamp(0.0, 1.0);
        (at, size)
    };
    if vertical {
        let length = inner.size.height - if horizontal { track } else { px(0.) };
        let (at, size_y) = thumb(length, inner.size.height, max.y, offset.y);
        let origin = point(inner.right() - track + pad, inner.top() + at);
        let quad = fill(Bounds::new(origin, size(track - pad * 2.0, size_y)), color);
        window.paint_quad(quad.corner_radii((track - pad * 2.0) / 2.0));
    }
    if horizontal {
        let length = inner.size.width - if vertical { track } else { px(0.) };
        let (at, size_x) = thumb(length, inner.size.width, max.x, offset.x);
        let origin = point(inner.left() + at, inner.bottom() - track + pad);
        let quad = fill(Bounds::new(origin, size(size_x, track - pad * 2.0)), color);
        window.paint_quad(quad.corner_radii((track - pad * 2.0) / 2.0));
    }
}

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
        let look = look(&theme);
        let thumb_handle = self.handle.clone();
        let hairline = theme.borders.hairline;
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
            .border_color(look.border)
            .rounded(px(theme.radii.medium))
            .focus_visible(|el| el.border_color(look.focus).shadow(vec![focus_ring(look.ring)]))
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
            .child(
                // Painted last so the thumbs sit above the content. The zero-size canvas adds
                // nothing to the scrollable content size; it paints against the handle's bounds.
                canvas(
                    |_, _, _| (),
                    move |_, (), window, _| {
                        paint_thumbs(
                            &thumb_handle,
                            hairline,
                            // shadcn's 10px track (`w-2.5`) with a 1px (`p-px`) gap: an 8px
                            // (`spacing.small`) thumb inside a hairline on each side.
                            theme.spacing.small + hairline * 2.0,
                            hairline,
                            theme.spacing.large,
                            look.thumb,
                            window,
                        )
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .size_0(),
            )
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
