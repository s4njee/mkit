//! Runnable example of mkit's controlled and uncontrolled state contract.

extern crate gpui_pre as gpui;

// ANCHOR: component_state_pattern
use gpui_pre::{
    Context, Entity, EventEmitter, IntoElement, Render, Subscription, Window, div, prelude::*,
};
use mkit_core::{
    state::{ComponentState, StateChange},
    theme::Theme,
};

/// A component-specific typed event. The payload is a user request; for a
/// controlled model it is a proposal until the owner sends the accepted value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToggleChangeRequested(pub StateChange<bool>);

/// Model owned by GPUI as an entity. Its event type is declared on this model,
/// as required by GPUI's `Context::emit` contract.
pub struct ToggleModel {
    state: ComponentState<bool>,
}

impl ToggleModel {
    pub fn uncontrolled(initial: bool) -> Self {
        Self { state: ComponentState::uncontrolled(initial) }
    }

    pub fn controlled(value_from_owner: bool) -> Self {
        Self { state: ComponentState::controlled(value_from_owner) }
    }

    pub fn value(&self) -> bool {
        *self.state.value()
    }
}

impl EventEmitter<ToggleChangeRequested> for ToggleModel {}

/// Parent view containing one locally owned toggle and one controlled toggle.
#[derive(Default)]
pub struct ComponentStateDemo {
    uncontrolled: Option<Entity<ToggleModel>>,
    controlled: Option<Entity<ToggleModel>>,
    uncontrolled_value: bool,
    controlled_value: bool,
    uncontrolled_requests: u32,
    controlled_requests: u32,
    last_controlled_request: Option<(bool, bool)>,
    _subscriptions: Vec<Subscription>,
}

impl ComponentStateDemo {
    fn initialize(&mut self, cx: &mut Context<Self>) {
        if self.uncontrolled.is_some() {
            return;
        }

        let uncontrolled = cx.new(|_| ToggleModel::uncontrolled(false));
        let controlled = cx.new(|_| ToggleModel::controlled(false));

        let uncontrolled_observer = cx.observe(&uncontrolled, |this, model, cx| {
            this.uncontrolled_value = model.read(cx).value();
            cx.notify();
        });
        let controlled_observer = cx.observe(&controlled, |this, model, cx| {
            this.controlled_value = model.read(cx).value();
            cx.notify();
        });

        // The uncontrolled event reports a committed change. The owner records
        // it, while the entity itself remains responsible for notifying views.
        let uncontrolled_subscription = cx.subscribe(&uncontrolled, |this, _, event, cx| {
            this.uncontrolled_requests += 1;
            debug_assert!(event.0.was_committed());
            cx.notify();
        });

        // The controlled event is a proposal. This example's parent accepts it
        // and sends the new value back to the child's entity.
        let controlled_subscription = cx.subscribe(&controlled, |this, model, event, cx| {
            this.controlled_requests += 1;
            this.last_controlled_request = Some((*event.0.previous(), *event.0.value()));
            let accepted_value = *event.0.value();
            model.update(cx, |model, cx| {
                model.state.set_value(accepted_value);
                cx.notify();
            });
        });

        self.uncontrolled = Some(uncontrolled);
        self.controlled = Some(controlled);
        self._subscriptions = vec![
            uncontrolled_observer,
            controlled_observer,
            uncontrolled_subscription,
            controlled_subscription,
        ];
    }

    fn request_toggle(model: Entity<ToggleModel>, cx: &mut gpui_pre::App) {
        model.update(cx, |model, cx| {
            let requested_value = !model.value();
            let change = model.state.request_change(requested_value);
            let was_committed = change.was_committed();
            cx.emit(ToggleChangeRequested(change));
            if was_committed {
                cx.notify();
            }
        });
    }

    /// Snapshot values and request metadata for examples and tests.
    pub fn inspector_snapshot(&self) -> (bool, bool, u32, u32, Option<(bool, bool)>) {
        (
            self.uncontrolled_value,
            self.controlled_value,
            self.uncontrolled_requests,
            self.controlled_requests,
            self.last_controlled_request,
        )
    }
}

impl Render for ComponentStateDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.initialize(cx);
        let uncontrolled = self.uncontrolled.as_ref().expect("initialized").clone();
        let controlled = self.controlled.as_ref().expect("initialized").clone();
        let uncontrolled_value = self.uncontrolled_value;
        let controlled_value = self.controlled_value;
        let colors = &cx.global::<Theme>().colors;

        div()
            .size_full()
            .bg(colors.background)
            .text_color(colors.text)
            .flex()
            .flex_col()
            .items_start()
            .gap_3()
            .p_6()
            .child(div().text_2xl().child("State patterns"))
            .child(
                div()
                    .debug_selector(move || format!("uncontrolled-value-{}", uncontrolled_value))
                    .child(format!("Uncontrolled: {uncontrolled_value}")),
            )
            .child(
                div()
                    .debug_selector(move || format!("controlled-value-{}", controlled_value))
                    .child(format!("Controlled: {controlled_value}")),
            )
            .child(
                div()
                    .id("uncontrolled-toggle")
                    .debug_selector(|| "uncontrolled-toggle".into())
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(colors.accent)
                    .text_color(colors.accent_text)
                    .child("Toggle uncontrolled")
                    .on_click(move |_, _, cx| Self::request_toggle(uncontrolled.clone(), cx)),
            )
            .child(
                div()
                    .id("controlled-toggle")
                    .debug_selector(|| "controlled-toggle".into())
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(colors.accent)
                    .text_color(colors.accent_text)
                    .child("Toggle controlled")
                    .on_click(move |_, _, cx| Self::request_toggle(controlled.clone(), cx)),
            )
    }
}
// ANCHOR_END: component_state_pattern

#[cfg(test)]
mod tests {
    use super::ComponentStateDemo;
    use gpui_pre::{Modifiers, TestAppContext};

    #[gpui_pre::test]
    fn uncontrolled_commits_and_controlled_waits_for_parent_event_response(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            mkit_core::theme::set_light_theme(cx);
        });
        let (demo, visual) = cx.add_window_view(|_, _| ComponentStateDemo::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));

        let uncontrolled =
            visual.debug_bounds("uncontrolled-toggle").expect("uncontrolled toggle rendered");
        visual.simulate_click(uncontrolled.center(), Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("uncontrolled-value-true").is_some());

        let controlled =
            visual.debug_bounds("controlled-toggle").expect("controlled toggle rendered");
        visual.simulate_click(controlled.center(), Modifiers::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("controlled-value-true").is_some());

        let snapshot = demo.read_with(visual, |demo, _| demo.inspector_snapshot());
        assert_eq!(snapshot, (true, true, 1, 1, Some((false, true))));
    }

    #[test]
    fn mode_contract_marks_controlled_event_as_proposal() {
        use mkit_core::state::{ComponentState, StateMode};

        let mut controlled = ComponentState::controlled(false);
        let event = controlled.request_change(true);

        assert_eq!(event.mode(), StateMode::Controlled);
        assert!(!event.was_committed());
        assert!(!*controlled.value());
        assert!(*event.value());
    }
}
