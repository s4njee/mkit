//! A small compiling example that separates retained views, notifications, and components.

// ANCHOR: reactivity_views_component
// The pinned IntoElement derive uses the conventional `gpui` crate path.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnyElement, App, Context, EventEmitter, IntoElement, Render, RenderOnce, Subscription, Window,
    div, prelude::*,
};

/// A reusable, stateless recipe. It owns its display text and renders once into elements.
#[derive(IntoElement)]
pub struct StatusPill {
    text: String,
}

impl StatusPill {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for StatusPill {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &gpui_kit::component::ActiveTheme::theme(cx).colors;
        div()
            .px_3()
            .py_1()
            .rounded_md()
            .bg(colors.primary)
            .text_color(colors.primary_foreground)
            .child(self.text)
    }
}

/// A theme-backed surface which gives its children the application's foreground.
#[derive(IntoElement)]
pub struct DemoSurface {
    child: AnyElement,
}

impl RenderOnce for DemoSurface {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &gpui_kit::component::ActiveTheme::theme(cx).colors;
        div().size_full().bg(colors.background).text_color(colors.foreground).child(self.child)
    }
}
// ANCHOR_END: reactivity_views_component

// ANCHOR: reactivity_views_events
/// A typed message emitted whenever the source count changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountChanged {
    pub count: u32,
}

#[derive(Default)]
struct Source {
    count: u32,
}

impl EventEmitter<CountChanged> for Source {}
// ANCHOR_END: reactivity_views_events

// ANCHOR: reactivity_views_view
/// A stateful view that observes one entity and subscribes to its typed event.
#[derive(Default)]
pub struct ReactivityDemo {
    source: Option<gpui_pre::Entity<Source>>,
    observed_count: u32,
    notification_count: u32,
    event_count: u32,
    last_event_count: u32,
    _observation: Option<Subscription>,
    _event_subscription: Option<Subscription>,
}

impl ReactivityDemo {
    fn initialize(&mut self, cx: &mut Context<Self>) {
        if self.source.is_some() {
            return;
        }
        let source = cx.new(|_| Source::default());
        let observation = cx.observe(&source, |this, source, cx| {
            this.observed_count = source.read(cx).count;
            this.notification_count += 1;
            cx.notify();
        });
        let event_subscription = cx.subscribe(&source, |this, _, event, cx| {
            this.event_count += 1;
            this.last_event_count = event.count;
            cx.notify();
        });

        self.source = Some(source);
        self._observation = Some(observation);
        self._event_subscription = Some(event_subscription);
    }
}

impl Render for ReactivityDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.initialize(cx);
        let source = self.source.as_ref().expect("initialized").clone();
        let observed_count = self.observed_count;
        let notification_count = self.notification_count;
        let event_count = self.event_count;
        let last_event_count = self.last_event_count;

        DemoSurface {
            child: div()
                .p_6()
                .flex()
                .flex_col()
                .gap_3()
                .child(div().text_2xl().child("Views and reactivity"))
                .child(
                    div()
                        .debug_selector(move || format!("observed-count-{observed_count}"))
                        .child(format!("Observed count: {observed_count}")),
                )
                .child(div().child(format!("Notifications: {notification_count}")))
                .child(
                    div().child(format!("Typed events: {event_count} (last {last_event_count})")),
                )
                .child(StatusPill::new("RenderOnce component"))
                .child(
                    div()
                        .id("increment")
                        .debug_selector(|| "increment".into())
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .child("Change source")
                        .on_click(move |_, _, cx| {
                            source.update(cx, |source, cx| {
                                source.count += 1;
                                cx.emit(CountChanged { count: source.count });
                                cx.notify();
                            });
                        }),
                )
                .into_any_element(),
        }
    }
}
// ANCHOR_END: reactivity_views_view

impl ReactivityDemo {
    /// Return observer and typed-event counters for inspector verification.
    pub fn inspector_snapshot(&self) -> (u32, u32, u32, u32) {
        (self.observed_count, self.notification_count, self.event_count, self.last_event_count)
    }
}

#[cfg(test)]
mod tests {
    use super::ReactivityDemo;
    use gpui_pre::{Modifiers, TestAppContext};

    #[gpui_pre::test]
    fn click_notifies_observer_and_emits_typed_event(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (demo, visual) = cx.add_window_view(|_, _| ReactivityDemo::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("observed-count-0").is_some());

        let button = visual.debug_bounds("increment").expect("increment is rendered");
        visual.simulate_click(button.center(), Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("observed-count-1").is_some());

        let snapshot = demo.read_with(visual, |demo, _| {
            (demo.observed_count, demo.notification_count, demo.event_count, demo.last_event_count)
        });
        assert_eq!(snapshot, (1, 1, 1, 1));
    }
}
