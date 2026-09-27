//! A small app showing shared entities, weak handles, and global configuration.

pub mod component_state;
pub mod e4_status;
pub mod theme_tokens;

use gpui_kit::Global;

// ANCHOR: global_settings
/// Application configuration shared through GPUI's `App` context.
#[derive(Clone)]
pub struct CounterSettings {
    pub title: &'static str,
    pub increment_by: u32,
}

impl Default for CounterSettings {
    fn default() -> Self {
        Self { title: "Shared counter", increment_by: 1 }
    }
}

impl Global for CounterSettings {}
// ANCHOR_END: global_settings

// ANCHOR: entity_model
/// The retained state that can be shared by multiple views.
#[derive(Default)]
pub struct CounterModel {
    pub count: u32,
}
// ANCHOR_END: entity_model

// ANCHOR: state_entities_view
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::{Context, Entity, Render, WeakEntity, Window, div, prelude::*};

/// A window that owns a strong handle to the shared counter model.
pub struct CounterWindow {
    counter: Option<Entity<CounterModel>>,
}

impl CounterWindow {
    pub fn new(counter: Entity<CounterModel>) -> Self {
        Self { counter: Some(counter) }
    }

    /// A headless preview that creates its model when GPUI first renders it.
    pub fn preview() -> Self {
        Self { counter: None }
    }
}

impl Render for CounterWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let counter = self.counter.get_or_insert_with(|| cx.new(|_| CounterModel::default()));
        // Reading does not copy ownership of the model; the window keeps it alive.
        let count = counter.read(cx).count;
        let title = cx.global::<CounterSettings>().title;
        let strong_counter = counter.clone();
        let weak_counter: WeakEntity<CounterModel> = counter.downgrade();

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .text_color(gpui_kit::rgb(0xffffff))
            .child(div().text_2xl().child(title))
            .child(div().child(format!("Count: {count}")))
            .child(
                div().debug_selector(|| "strong-increment".into()).child(
                    Button::new("strong-increment-button")
                        .primary()
                        .label("Increment (Entity)")
                        .on_click(move |_, _, cx| {
                            let step = cx.global::<CounterSettings>().increment_by;
                            strong_counter.update(cx, |counter, cx| {
                                counter.count += step;
                                cx.notify();
                            });
                        }),
                ),
            )
            .child(div().debug_selector(|| "weak-increment".into()).child(
                Button::new("weak-increment-button").label("Increment (WeakEntity)").on_click(
                    move |_, _, cx| {
                        if let Some(counter) = weak_counter.upgrade() {
                            let step = cx.global::<CounterSettings>().increment_by;
                            counter.update(cx, |counter, cx| {
                                counter.count += step;
                                cx.notify();
                            });
                        }
                    },
                ),
            ))
    }
}
// ANCHOR_END: state_entities_view

impl CounterWindow {
    /// Read the shared model count for inspector and example verification.
    pub fn inspector_count(&self, cx: &gpui_pre::App) -> Option<u32> {
        self.counter.as_ref().map(|counter| counter.read(cx).count)
    }
}

#[cfg(test)]
mod tests {
    use super::{CounterModel, CounterSettings, CounterWindow};
    use gpui_kit::{AppContext, Modifiers, TestAppContext};

    #[gpui_kit::test]
    fn entity_updates_and_global_configuration_are_shared_with_the_view(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        cx.set_global(CounterSettings { title: "Test counter", increment_by: 3 });
        let counter = cx.update(|cx| cx.new(|_| CounterModel::default()));
        let model = counter.clone();
        let (_, visual) = cx.add_window_view(move |_, _| CounterWindow::new(model));
        visual.update(|window, cx| window.draw(cx).clear(cx));

        let entity_button =
            visual.debug_bounds("strong-increment").expect("entity button is rendered");
        visual.simulate_click(entity_button.center(), Modifiers::default());
        assert_eq!(counter.read_with(visual, |model, _| model.count), 3);

        let weak_button = visual.debug_bounds("weak-increment").expect("weak button is rendered");
        visual.simulate_click(weak_button.center(), Modifiers::default());
        assert_eq!(counter.read_with(visual, |model, _| model.count), 6);
    }

    #[gpui_kit::test]
    fn weak_handle_does_not_keep_entity_alive(cx: &mut TestAppContext) {
        let (strong, weak) = cx.update(|cx| {
            let strong = cx.new(|_| CounterModel::default());
            let weak = strong.downgrade();
            (strong, weak)
        });
        assert!(weak.upgrade().is_some());

        drop(strong);

        assert!(weak.upgrade().is_none());
        assert!(weak.update(cx, |counter, _| counter.count += 1).is_err());
    }
}
