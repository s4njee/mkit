//! Reference solution for benchmark 01 (counter).
//!
//! Hidden from evaluation agents: the runner never copies `evals/hidden/`
//! into an agent workspace and audits workspaces for these files.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_pre::{
    App, Context, FocusHandle, IntoElement, KeyBinding, Render, Window, actions, div, prelude::*,
    rgb,
};
use std::path::Path;

actions!(counter, [Increment, Decrement, Reset]);

const KEY_CONTEXT: &str = "Counter";

/// Install GPUI Kit and the counter's rebindable key bindings.
pub fn init(cx: &mut App) {
    gpui_kit::init(cx);
    cx.bind_keys([
        KeyBinding::new("up", Increment, Some(KEY_CONTEXT)),
        KeyBinding::new("down", Decrement, Some(KEY_CONTEXT)),
        KeyBinding::new("escape", Reset, Some(KEY_CONTEXT)),
    ]);
}

/// The counter view. The focus handle is created on first render.
#[derive(Default)]
pub struct Root {
    count: u32,
    focus: Option<FocusHandle>,
}

pub fn root(_fixture: &Path) -> Root {
    Root::default()
}

impl Root {
    pub fn snapshot(&self, _cx: &App) -> serde_json::Value {
        serde_json::json!({ "count": self.count })
    }

    fn increment(&mut self, _: &Increment, _: &mut Window, cx: &mut Context<Self>) {
        self.count += 1;
        cx.notify();
    }

    fn decrement(&mut self, _: &Decrement, _: &mut Window, cx: &mut Context<Self>) {
        self.count = self.count.saturating_sub(1);
        cx.notify();
    }

    fn reset(&mut self, _: &Reset, _: &mut Window, cx: &mut Context<Self>) {
        self.count = 0;
        cx.notify();
    }
}

impl Render for Root {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus = self
            .focus
            .get_or_insert_with(|| {
                let handle = cx.focus_handle();
                window.focus(&handle, cx);
                handle
            })
            .clone();
        let view = cx.entity();
        let button = |id: &'static str, label: &'static str, action: fn(&mut Root)| {
            let view = view.clone();
            div().debug_selector(move || id.into()).child(
                Button::new(id).primary().label(label).on_click(move |_, _, cx| {
                    view.update(cx, |root, cx| {
                        action(root);
                        cx.notify();
                    });
                }),
            )
        };

        div()
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .on_action(cx.listener(Self::increment))
            .on_action(cx.listener(Self::decrement))
            .on_action(cx.listener(Self::reset))
            .size_full()
            .bg(rgb(0x1e2530))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .child(
                div().text_2xl().text_color(rgb(0xffffff)).child(format!("Count: {}", self.count)),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(button("increment", "Increment", |root| root.count += 1))
                    .child(button("decrement", "Decrement", |root| {
                        root.count = root.count.saturating_sub(1)
                    }))
                    .child(button("reset", "Reset", |root| root.count = 0)),
            )
    }
}
