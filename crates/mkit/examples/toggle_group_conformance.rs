use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, Window, div,
};
use mkit::{
    core::theme,
    toggle_group::{Item, Orientation, ToggleGroup, ValueChanged, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    orientation: Orientation,
    initial_value: &'static str,
    disabled: bool,
    controlled: bool,
    group: Option<Entity<ToggleGroup>>,
    events: Rc<RefCell<Vec<Option<String>>>>,
    _subscription: Option<Subscription>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.group.is_none() {
            let items = vec![
                Item::new("left", "Left"),
                Item::new("center", "Center").disabled(true),
                Item::new("right", "Right"),
                Item::new("justify", "Justify"),
            ];
            let group = cx.new(|_| {
                let value = Some(self.initial_value.to_owned());
                let component = if self.controlled {
                    ToggleGroup::controlled("Alignment", items, value)
                } else {
                    ToggleGroup::new("Alignment", items, value)
                };
                component.orientation(self.orientation).disabled(self.disabled)
            });
            let events = self.events.clone();
            self._subscription =
                Some(cx.subscribe(&group, move |_, _, event: &ValueChanged, _| {
                    events.borrow_mut().push(event.0.clone())
                }));
            self.group = Some(group);
        }
        div().child(self.group.as_ref().expect("group initialized").clone())
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"keyboard_cases","id":"keyboard-01","dispatch":{"key":"ArrowRight","modifiers":[]},"load_fixture":"toggle_group_selection"})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "keyboard_cases");
    assert_eq!(case["load_fixture"], "toggle_group_selection");
    assert_eq!(case["dispatch"]["modifiers"], json!([]));
    let key = case["dispatch"]["key"].as_str().expect("dispatch key");
    let (keystroke, orientation, initial, target, focus_target) = match key {
        "ArrowRight" => ("right", Orientation::Horizontal, "left", "right", "next_enabled_item"),
        "ArrowLeft" => {
            ("left", Orientation::Horizontal, "left", "justify", "previous_enabled_item")
        }
        "ArrowDown" => ("down", Orientation::Vertical, "left", "right", "next_enabled_item"),
        "ArrowUp" => ("up", Orientation::Vertical, "left", "justify", "previous_enabled_item"),
        "Home" => ("home", Orientation::Horizontal, "right", "left", "first_enabled_item"),
        "End" => ("end", Orientation::Horizontal, "left", "justify", "last_enabled_item"),
        _ => panic!("unsupported key {key}"),
    };
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host {
        orientation,
        initial_value: initial,
        disabled: false,
        controlled: false,
        group: None,
        events: Rc::new(RefCell::new(Vec::new())),
        _subscription: None,
    });
    let (group, events) = window.update(|host, window, cx| {
        let group = host.group.as_ref().expect("rendered group").clone();
        group.focus_handle(cx).focus(window, cx);
        (group, host.events.clone())
    });
    let cross_axis = if orientation == Orientation::Horizontal { "down" } else { "right" };
    window.simulate_keystroke(cross_axis);
    let after_cross_axis = window.update(|_, _, cx| group.read(cx).value().map(str::to_owned));
    assert_eq!(after_cross_axis.as_deref(), Some(initial));
    assert!(events.borrow().is_empty(), "cross-axis arrow must not select an item");
    window.simulate_keystroke(keystroke);
    let (value, focused) = window.update(|_, window, cx| {
        (group.read(cx).value().map(str::to_owned), group.focus_handle(cx).is_focused(window))
    });
    let emitted = events.borrow().clone();
    assert_eq!(value.as_deref(), Some(target), "{key} must select {target}");
    assert!(focused, "{key} must focus the selected item");
    assert_eq!(emitted, vec![Some(target.to_owned())], "{key} event contract");

    let mut disabled_window = app.open_window(|_, _| Host {
        orientation,
        initial_value: initial,
        disabled: true,
        controlled: false,
        group: None,
        events: Rc::new(RefCell::new(Vec::new())),
        _subscription: None,
    });
    let (disabled_group, disabled_events) = disabled_window.update(|host, window, cx| {
        let group = host.group.as_ref().expect("rendered disabled group").clone();
        group.focus_handle(cx).focus(window, cx);
        (group, host.events.clone())
    });
    disabled_window.simulate_keystroke(keystroke);
    let disabled_value =
        disabled_window.update(|_, _, cx| disabled_group.read(cx).value().map(str::to_owned));
    assert_eq!(disabled_value.as_deref(), Some(initial), "disabled group must retain its value");
    assert!(disabled_events.borrow().is_empty(), "disabled group must not emit");

    let mut controlled_window = app.open_window(|_, _| Host {
        orientation,
        initial_value: initial,
        disabled: false,
        controlled: true,
        group: None,
        events: Rc::new(RefCell::new(Vec::new())),
        _subscription: None,
    });
    let (controlled_group, controlled_events) = controlled_window.update(|host, window, cx| {
        let group = host.group.as_ref().expect("rendered controlled group").clone();
        group.focus_handle(cx).focus(window, cx);
        (group, host.events.clone())
    });
    controlled_window.simulate_keystroke(keystroke);
    let controlled_value =
        controlled_window.update(|_, _, cx| controlled_group.read(cx).value().map(str::to_owned));
    assert_eq!(controlled_value.as_deref(), Some(initial), "controlled group retains parent value");
    assert_eq!(
        *controlled_events.borrow(),
        vec![Some(target.to_owned())],
        "controlled group requests next value"
    );
    println!(
        "{}",
        json!({
            "passed": true,
            "actual": {
                "event": "value_changed",
                "focus_target": focus_target,
                "selected_value": value,
                "focused": focused,
                "events": emitted,
            }
        })
    );
}
