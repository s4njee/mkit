extern crate gpui_pre as gpui;
use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, Window, div,
};
use mkit::{
    core::theme,
    radio_group::{ChangeRequested, OptionItem, Orientation, RadioGroup, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    fixture: String,
    group: Option<Entity<RadioGroup>>,
    events: Rc<RefCell<Vec<String>>>,
    _subscription: Option<Subscription>,
    controlled: bool,
    disabled: bool,
}
impl Host {
    fn new(fixture: &str) -> Self {
        Self {
            fixture: fixture.to_owned(),
            group: None,
            events: Rc::new(RefCell::new(Vec::new())),
            _subscription: None,
            controlled: fixture == "controlled_fixture",
            disabled: fixture == "disabled_fixture",
        }
    }
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.group.is_none() {
            let options = vec![
                OptionItem::new("first", "First"),
                OptionItem::new("unavailable", "Unavailable").disabled(true),
                OptionItem::new("second", "Second"),
            ];
            let initial = if self.fixture.contains("second") {
                Some("second".into())
            } else if self.fixture == "no_selection_fixture" {
                None
            } else {
                Some("first".into())
            };
            let component = if self.controlled {
                RadioGroup::controlled("Plan", options, initial)
            } else {
                RadioGroup::new("Plan", options, initial)
            }
            .disabled(self.disabled)
            .orientation(if self.fixture.starts_with("horizontal") {
                Orientation::Horizontal
            } else {
                Orientation::Vertical
            });
            let group = cx.new(|_| component);
            let events = self.events.clone();
            self._subscription =
                Some(cx.subscribe(&group, move |_, _, event: &ChangeRequested, _| {
                    events.borrow_mut().push(event.0.clone())
                }));
            self.group = Some(group);
        }
        div().child(self.group.as_ref().unwrap().clone())
    }
}

fn run(case: &Value) -> Value {
    let fixture = case["load_fixture"].as_str().expect("fixture");
    let key = case["dispatch"]["key"].as_str().expect("key");
    let keystroke = match key {
        "ArrowDown" => "down",
        "ArrowUp" => "up",
        "ArrowRight" => "right",
        "ArrowLeft" => "left",
        "Home" => "home",
        "End" => "end",
        " " | "Space" => "space",
        _ => panic!("unsupported key {key}"),
    };
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host::new(fixture));
    let (group, events) = window.update(|host, window, cx| {
        let group = host.group.as_ref().unwrap().clone();
        if fixture != "disabled_fixture" {
            group.focus_handle(cx).focus(window, cx);
        }
        (group, host.events.clone())
    });
    window.simulate_keystroke(keystroke);
    let (value, focused_option, has_focus, emitted) = window.update(|_, window, cx| {
        let group = group.read(cx);
        (
            group.value().map(str::to_owned),
            group.focused_option().map(str::to_owned),
            group.focus_handle(cx).is_focused(window),
            events.borrow().clone(),
        )
    });
    let state = if fixture == "disabled_fixture" {
        "disabled"
    } else {
        match value.as_deref() {
            Some("first") => "first_selected",
            Some("second") => "second_selected",
            None => "no_selection",
            _ => "unknown",
        }
    };
    let event = emitted
        .last()
        .map(|id| format!("ChangeRequested({id})"))
        .unwrap_or_else(|| "none".to_owned());
    json!({"state": state, "value": value, "event": event, "focused_option": focused_option, "focus_target": if has_focus {"radio"} else {"none"}, "events": emitted})
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read case");
    let case: Value = serde_json::from_str(&input).expect("case JSON");
    println!("{}", json!({"passed": true, "actual": run(&case)}));
}
