use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, Window, div,
};
use mkit::{
    core::theme,
    toggle_button::{ToggleButton, ValueChanged, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    toggle: Option<Entity<ToggleButton>>,
    changes: Rc<RefCell<Vec<bool>>>,
    _subscription: Option<Subscription>,
}

impl Host {
    fn new() -> Self {
        Self { toggle: None, changes: Rc::new(RefCell::new(Vec::new())), _subscription: None }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.toggle.is_none() {
            let toggle = cx.new(|_| ToggleButton::new("Bold", false));
            let changes = self.changes.clone();
            let subscription = cx.subscribe(&toggle, move |_, _, event: &ValueChanged, _| {
                changes.borrow_mut().push(event.0);
            });
            self.toggle = Some(toggle);
            self._subscription = Some(subscription);
        }
        div().child(self.toggle.as_ref().expect("toggle initialized").clone())
    }
}

fn keyboard_case(case: &Value) -> Value {
    assert_eq!(case.get("kind"), Some(&Value::String("keyboard_cases".into())));
    assert_eq!(case["load_fixture"], "toggle_off");
    let dispatch = case.get("dispatch").expect("dispatch");
    assert_eq!(dispatch["modifiers"], json!([]));
    let key = dispatch["key"].as_str().expect("key");
    let keystroke = match key {
        "Enter" => "enter",
        "Space" => "space",
        _ => panic!("unsupported generated key {key}"),
    };

    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host::new());
    let toggle = window.update(|host, window, cx| {
        let toggle = host.toggle.as_ref().expect("rendered toggle").clone();
        toggle.focus_handle(cx).focus(window, cx);
        toggle
    });
    let focused_before = window.update(|_, window, cx| toggle.focus_handle(cx).is_focused(window));
    assert!(focused_before, "toggle should be focused before key dispatch");
    window.simulate_keystroke(keystroke);
    let (pressed, changes, focused_after) = window.update(|host, window, cx| {
        (
            toggle.read(cx).is_pressed(),
            host.changes.borrow().clone(),
            toggle.focus_handle(cx).is_focused(window),
        )
    });
    assert!(pressed, "{key} should press the uncontrolled toggle");
    assert_eq!(changes, vec![true], "{key} should emit one ValueChanged(true)");
    assert!(focused_after, "toggle retains focus after activation");
    let expected = case["assert"]["event"].as_str().expect("expected event");
    assert_eq!(expected, "value_changed");
    assert_eq!(case["assert"]["state"], "on");
    json!({"state": if pressed { "on" } else { "off" }, "event": expected, "events": changes})
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({
            "kind": "keyboard_cases",
            "id": "keyboard-01",
            "dispatch": {"key": "Enter", "modifiers": []},
            "load_fixture": "toggle_off",
            "assert": {"state": "on", "event": "value_changed"}
        })
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    let actual = keyboard_case(&case);
    println!("{}", json!({"passed": true, "actual": actual}));
}
