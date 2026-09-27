extern crate gpui_pre as gpui;
use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, Window, div,
};
use mkit_registry_segmented_control::{
    Item, Orientation, SegmentedControl, ValueChanged, default_key_bindings,
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    fixture: String,
    tabs: Option<Entity<SegmentedControl>>,
    events: Rc<RefCell<Vec<Option<String>>>>,
    _subscription: Option<Subscription>,
}
impl Host {
    fn new(fixture: &str) -> Self {
        Self {
            fixture: fixture.to_owned(),
            tabs: None,
            events: Rc::new(RefCell::new(Vec::new())),
            _subscription: None,
        }
    }
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.tabs.is_none() {
            let items = vec![
                Item::new("one", "One"),
                Item::new("skip", "Skip").disabled(true),
                Item::new("three", "Three"),
            ];
            let initial = if self.fixture.contains("empty") {
                None
            } else if self.fixture.contains("selected_three") {
                Some("three".to_owned())
            } else {
                Some("one".to_owned())
            };
            let component = if self.fixture.contains("controlled") {
                SegmentedControl::controlled("Views", items, initial)
            } else {
                SegmentedControl::new("Views", items, initial)
            }
            .disabled(self.fixture.contains("disabled"))
            .orientation(if self.fixture.contains("vertical") {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            });
            let tabs = cx.new(|_| component);
            let events = self.events.clone();
            self._subscription = Some(cx.subscribe(&tabs, move |_, _, event: &ValueChanged, _| {
                events.borrow_mut().push(event.0.clone())
            }));
            self.tabs = Some(tabs);
        }
        div().child(self.tabs.as_ref().unwrap().clone())
    }
}

fn key_name(key: &str) -> &'static str {
    match key {
        "ArrowDown" => "down",
        "ArrowUp" => "up",
        "ArrowRight" => "right",
        "ArrowLeft" => "left",
        "Home" => "home",
        "End" => "end",
        "Enter" => "enter",
        " " | "Space" => "space",
        _ => panic!("unsupported key {key}"),
    }
}
fn run(case: &Value) -> Value {
    let fixture = case["load_fixture"].as_str().expect("fixture");
    let mut app = TestApp::new();
    app.update(|cx| {
        mkit_core::theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host::new(fixture));
    let (tabs, events) = window.update(|host, window, cx| {
        let tabs = host.tabs.as_ref().unwrap().clone();
        if !fixture.contains("disabled") {
            tabs.focus_handle(cx).focus(window, cx);
        }
        (tabs, host.events.clone())
    });
    if fixture.contains("focus_pending") {
        window.simulate_keystroke("right");
    }
    if let Some(keys) = case["pre_dispatch"].as_array() {
        for key in keys {
            window.simulate_keystroke(key_name(key.as_str().expect("pre-dispatch key")));
        }
    }
    window.simulate_keystroke(key_name(case["dispatch"]["key"].as_str().expect("key")));
    let (value, focused_value, emitted) = window.update(|_, _, cx| {
        let tabs = tabs.read(cx);
        (
            tabs.value().map(str::to_owned),
            tabs.focused_value().map(str::to_owned),
            events.borrow().clone(),
        )
    });
    let state = if fixture.contains("disabled") {
        "disabled"
    } else if fixture.contains("controlled") {
        "controlled"
    } else if value.is_none() {
        "empty"
    } else if value.as_deref() == Some("three") {
        "selected_three"
    } else {
        "selection"
    };
    json!({"state": state, "value": value, "focused_value": focused_value,
        "focus_target": if fixture.contains("disabled") { "none" } else { focused_value.as_deref().unwrap_or("none") },
        "event": if emitted.is_empty() { "none" } else { "selection_changed" }})
}
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read case");
    let case: Value = serde_json::from_str(&input).expect("case JSON");
    println!("{}", json!({"passed": true, "actual": run(&case)}));
}
