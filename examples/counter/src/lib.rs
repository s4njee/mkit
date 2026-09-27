//! A minimal stateful counter for the GPUI book.

// ANCHOR: counter_view
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::{Context, Render, Window, div, prelude::*};

/// A view that retains its count between renders.
#[derive(Default)]
pub struct Counter {
    count: u32,
}

impl Render for Counter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.count;
        let counter = cx.entity();

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .text_color(gpui_kit::rgb(0xffffff))
                    .child(format!("Count: {count}")),
            )
            .child(div().debug_selector(|| "increment".into()).child(
                Button::new("increment-button").primary().label("Increment").on_click(
                    move |_, _, cx| {
                        counter.update(cx, |this, cx| {
                            this.count += 1;
                            cx.notify();
                        });
                    },
                ),
            ))
    }
}
// ANCHOR_END: counter_view

impl Counter {
    /// Return the retained count for inspector and example verification.
    pub fn inspector_count(&self) -> u32 {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::Counter;
    use gpui_kit::{Modifiers, TestAppContext};

    // ANCHOR: counter_interaction_test
    #[gpui_kit::test]
    fn clicking_increment_button_updates_retained_count(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (counter, visual) = cx.add_window_view(|_, _| Counter::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));

        let button = visual.debug_bounds("increment").expect("increment button is rendered");
        visual.simulate_click(button.center(), Modifiers::default());

        assert_eq!(counter.read_with(visual, |counter, _| counter.count), 1);
    }
    // ANCHOR_END: counter_interaction_test
}
