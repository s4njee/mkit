use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled,
    Subscription, TestApp, Window, div,
};
use mkit::{
    core::theme,
    slider::{ChangeRequested, InteractionEnded, InteractionStarted, Slider, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    range: bool,
    controlled: bool,
    disabled: bool,
    slider: Option<Entity<Slider>>,
    events: Rc<RefCell<Vec<Vec<f64>>>>,
    sequence: Rc<RefCell<Vec<&'static str>>>,
    _subscriptions: Vec<Subscription>,
}

impl Host {
    fn new(range: bool, controlled: bool, disabled: bool) -> Self {
        Self {
            range,
            controlled,
            disabled,
            slider: None,
            events: Rc::new(RefCell::new(Vec::new())),
            sequence: Rc::new(RefCell::new(Vec::new())),
            _subscriptions: Vec::new(),
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.slider.is_none() {
            let component = if self.controlled {
                Slider::controlled(
                    "Volume",
                    if self.range { vec![20.0, 80.0] } else { vec![50.0] },
                    0.0,
                    100.0,
                    5.0,
                )
            } else if self.range {
                Slider::range("Volume", 20.0, 80.0, 0.0, 100.0, 5.0)
            } else {
                Slider::new("Volume", 50.0, 0.0, 100.0, 5.0)
            }
            .disabled(self.disabled);
            let slider = cx.new(|_| component);
            let events = self.events.clone();
            let sequence = self.sequence.clone();
            let started = self.sequence.clone();
            let ended = self.sequence.clone();
            self._subscriptions = vec![
                cx.subscribe(&slider, move |_, _, _: &InteractionStarted, _| {
                    started.borrow_mut().push("interaction_started")
                }),
                cx.subscribe(&slider, move |_, _, event: &ChangeRequested, _| {
                    sequence.borrow_mut().push("change_requested");
                    events.borrow_mut().push(event.0.clone())
                }),
                cx.subscribe(&slider, move |_, _, _: &InteractionEnded, _| {
                    ended.borrow_mut().push("interaction_ended")
                }),
            ];
            self.slider = Some(slider);
        }
        div()
            .w(gpui_pre::px(320.0))
            .child(self.slider.as_ref().expect("slider initialized").clone())
    }
}

fn assert_values(actual: &[f64], expected: &[f64], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} length");
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() < 1e-8, "{context}: expected {e}, got {a}");
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"keyboard_cases","id":"keyboard-01","dispatch":{"key":"ArrowRight","modifiers":[]},"load_fixture":"middle_fixture"})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "keyboard_cases");
    let key = case["dispatch"]["key"].as_str().expect("dispatch key");
    let modifiers = case["dispatch"]["modifiers"].as_array().expect("modifiers");
    let modifier_names = modifiers
        .iter()
        .map(|modifier| modifier.as_str().expect("modifier name"))
        .collect::<Vec<_>>();
    let (keystroke, range, expected): (&str, bool, Vec<f64>) =
        match (key, modifier_names.as_slice()) {
            ("ArrowRight", []) => ("right", false, vec![55.0]),
            ("ArrowLeft", []) => ("left", true, vec![15.0, 80.0]),
            ("ArrowRight", ["Shift"]) => ("shift-right", false, vec![50.5]),
            ("ArrowRight", ["Alt"]) => ("alt-right", false, vec![50.05]),
            ("PageUp", []) => ("pageup", false, vec![100.0]),
            ("Home", []) => ("home", false, vec![0.0]),
            _ => panic!("unsupported generated key {key} {modifier_names:?}"),
        };
    assert_eq!(case["load_fixture"], if range { "range_fixture" } else { "middle_fixture" });

    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host::new(range, false, false));
    let (slider, events, sequence) = window.update(|host, window, cx| {
        let slider = host.slider.as_ref().expect("rendered slider").clone();
        window.focus_next(cx);
        assert!(slider.focus_handle(cx).is_focused(window), "first thumb is a tab stop");
        (slider, host.events.clone(), host.sequence.clone())
    });
    window.simulate_keystroke(keystroke);
    let values = window.update(|_, _, cx| slider.read(cx).values().to_vec());
    let emitted = events.borrow().clone();
    assert_values(&values, &expected, keystroke);
    assert_eq!(emitted.len(), 1, "one keyboard change request");
    assert_values(&emitted[0], &expected, "emitted change request");
    let event_sequence = sequence.borrow().join(" \u{2192} ");

    let mut controlled_window = app.open_window(|_, _| Host::new(range, true, false));
    let (controlled_slider, controlled_events) = controlled_window.update(|host, window, cx| {
        let slider = host.slider.as_ref().expect("rendered controlled slider").clone();
        slider.focus_handle(cx).focus(window, cx);
        (slider, host.events.clone())
    });
    controlled_window.simulate_keystroke(keystroke);
    let controlled_values =
        controlled_window.update(|_, _, cx| controlled_slider.read(cx).values().to_vec());
    assert_values(
        &controlled_values,
        if range { &[20.0, 80.0] } else { &[50.0] },
        "controlled display",
    );
    let controlled_events = controlled_events.borrow();
    assert_eq!(controlled_events.len(), 1, "controlled request count");
    assert_values(&controlled_events[0], &expected, "controlled request");
    drop(controlled_events);
    controlled_window.update(|_, _, cx| {
        controlled_slider.update(cx, |slider, cx| slider.set_value(expected.clone(), cx));
    });
    let applied = controlled_window.update(|_, _, cx| controlled_slider.read(cx).values().to_vec());
    assert_values(&applied, &expected, "owner-applied controlled value");

    let mut disabled_window = app.open_window(|_, _| Host::new(range, false, true));
    let (disabled_slider, disabled_events) = disabled_window.update(|host, window, cx| {
        let slider = host.slider.as_ref().expect("rendered disabled slider").clone();
        slider.focus_handle(cx).focus(window, cx);
        (slider, host.events.clone())
    });
    disabled_window.simulate_keystroke(keystroke);
    let disabled_values =
        disabled_window.update(|_, _, cx| disabled_slider.read(cx).values().to_vec());
    assert_values(
        &disabled_values,
        if range { &[20.0, 80.0] } else { &[50.0] },
        "disabled display",
    );
    assert!(disabled_events.borrow().is_empty(), "disabled slider emits no change");

    println!(
        "{}",
        json!({"passed":true,"actual":{"event":event_sequence,"values":values,"events":emitted}})
    );
}
