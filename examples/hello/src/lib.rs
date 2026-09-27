//! Renderable fixture for the GPUI book's first example and inspector.

// ANCHOR: hello_view
use gpui_pre::{Context, FocusHandle, KeyDownEvent, Render, Window, div, prelude::*};

/// A small stateful view that can be launched by mkit-inspector.
pub struct InspectorFixture {
    pub clicks: u64,
    pub typed: String,
    pub last_key: String,
    pub heading: &'static str,
    pub detail: &'static str,
    focus_handle: Option<FocusHandle>,
}

impl InspectorFixture {
    pub fn hello() -> Self {
        Self::new("Hello from mkit", "Type into the window or click the counter.")
    }

    pub fn gallery() -> Self {
        Self::new("mkit component gallery", "Inspector fixture: component preview canvas.")
    }

    fn new(heading: &'static str, detail: &'static str) -> Self {
        Self {
            clicks: 0,
            typed: String::new(),
            last_key: String::new(),
            heading,
            detail,
            focus_handle: None,
        }
    }
}

impl Render for InspectorFixture {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_handle.is_none() {
            self.focus_handle = Some(cx.focus_handle());
            window.focus(self.focus_handle.as_ref().expect("just initialized"), cx);
        }
        let focus_handle = self.focus_handle.as_ref().expect("focus handle initialized").clone();
        let clicks = self.clicks;
        let typed = self.typed.clone();
        let last_key = self.last_key.clone();
        let background = if clicks & 1 == 0 { 0x243b53 } else { 0x167a63 };
        div()
            .id("mkit-inspector-example")
            .size_full()
            .bg(gpui_pre::rgb(background))
            .text_color(gpui_pre::rgb(0xffffff))
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                this.last_key = event.keystroke.key.to_string();
                if event.keystroke.key.chars().count() == 1 {
                    this.typed.push_str(&event.keystroke.key);
                }
                cx.notify();
            }))
            .child(div().child(self.heading))
            .child(div().child(self.detail))
            .child(
                div()
                    .id("increment")
                    .absolute()
                    .left(gpui_pre::px(16.))
                    .top(gpui_pre::px(80.))
                    .w(gpui_pre::px(140.))
                    .h(gpui_pre::px(40.))
                    .px_3()
                    .py_2()
                    .border_1()
                    .border_color(gpui_pre::rgb(0xffffff))
                    .flex()
                    .items_center()
                    .child(format!("Clicks: {clicks}"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.clicks += 1;
                        cx.notify();
                    })),
            )
            .child(div().h(gpui_pre::px(42.)))
            .child(format!("Typed: {typed}"))
            .child(format!("Last key: {last_key}"))
    }
}
// ANCHOR_END: hello_view
