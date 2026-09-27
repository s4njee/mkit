extern crate gpui_pre as gpui;

use gpui_pre::{
    AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, Subscription, TestApp, Window, div,
};
use mkit::{
    core::theme,
    multi_select::{MultiSelect, OpenChanged, OptionItem, ValuesChanged, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    fixture: String,
    control: Option<Entity<MultiSelect>>,
    events: Rc<RefCell<Vec<String>>>,
    subscriptions: Vec<Subscription>,
    tab_stops: Option<(FocusHandle, FocusHandle)>,
}

impl Host {
    fn new(fixture: &str) -> Self {
        Self {
            fixture: fixture.to_owned(),
            control: None,
            events: Rc::new(RefCell::new(Vec::new())),
            subscriptions: Vec::new(),
            tab_stops: None,
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.tab_stops.is_none() {
            self.tab_stops =
                Some((cx.focus_handle().tab_stop(true), cx.focus_handle().tab_stop(true)));
        }
        if self.control.is_none() {
            let mut options = vec![
                OptionItem::new("alpha", "Alpha"),
                OptionItem::new("skip", "Unavailable").disabled(true),
                OptionItem::new("beta", "Beta"),
            ];
            options
                .extend((3..12).map(|index| {
                    OptionItem::new(format!("item-{index}"), format!("Item {index}"))
                }));
            if self.fixture == "all_options_disabled_fixture" {
                for option in &mut options {
                    option.disabled = true;
                }
            }
            let initial = if self.fixture == "open_fixture" {
                vec!["beta".to_owned()]
            } else {
                vec!["alpha".to_owned()]
            };
            let control = cx.new(|_| {
                MultiSelect::new("Topics", options, initial)
                    .disabled(self.fixture == "disabled_fixture")
            });
            let events = self.events.clone();
            self.subscriptions.push(cx.subscribe(&control, move |_, _, event: &OpenChanged, _| {
                events.borrow_mut().push(if event.0 { "open" } else { "close" }.to_owned());
            }));
            let events = self.events.clone();
            self.subscriptions.push(cx.subscribe(
                &control,
                move |_, _, event: &ValuesChanged, _| {
                    events.borrow_mut().push(format!("change:{}", event.0.join(",")));
                },
            ));
            self.control = Some(control);
        }
        let (before, after) = self.tab_stops.as_ref().unwrap();
        div()
            .child(div().id("before_tab_stop").track_focus(before).tab_stop(true))
            .child(self.control.as_ref().unwrap().clone())
            .child(div().id("after_tab_stop").track_focus(after).tab_stop(true))
    }
}

fn run(case: &Value) -> Value {
    let fixture = case["load_fixture"].as_str().expect("fixture");
    let key = case["dispatch"]["key"].as_str().expect("key");
    let keystroke = match key {
        "ArrowDown" => "down",
        "Space" => "space",
        "Escape" => "escape",
        "Home" => "home",
        "End" => "end",
        "Tab"
            if case["dispatch"]["modifiers"]
                .as_array()
                .is_some_and(|mods| mods.iter().any(|modifier| modifier == "Shift")) =>
        {
            "shift-tab"
        }
        "Tab" => "tab",
        _ => panic!("unsupported key {key}"),
    };
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host::new(fixture));
    let (control, events) = window.update(|host, window, cx| {
        let control = host.control.as_ref().unwrap().clone();
        if fixture != "disabled_fixture" {
            control.focus_handle(cx).focus(window, cx);
        }
        (control, host.events.clone())
    });
    if fixture == "open_fixture" {
        window.simulate_keystroke("space");
        events.borrow_mut().clear();
    }
    let before = window.update(|_, _, cx| control.read(cx).values().to_vec());
    window.simulate_keystroke(keystroke);
    let (open, active, values, focused, focus_target) = window.update(|host, window, cx| {
        let control = control.read(cx);
        let target = host
            .tab_stops
            .as_ref()
            .map(|(before, after)| {
                if after.is_focused(window) {
                    "next_tab_stop"
                } else if before.is_focused(window) {
                    "previous_tab_stop"
                } else if control.focus_handle(cx).is_focused(window) {
                    "multi_select"
                } else {
                    "none"
                }
            })
            .unwrap_or("none");
        (
            control.is_open(),
            control.active_option_id().map(str::to_owned),
            control.values().to_vec(),
            control.focus_handle(cx).is_focused(window),
            target,
        )
    });
    let emitted = events.borrow().clone();
    let event = emitted
        .last()
        .map(|event| if event.starts_with("change:") { "change" } else { event.as_str() })
        .unwrap_or("none");
    json!({
        "state": if fixture == "disabled_fixture" { "disabled" } else if fixture == "all_options_disabled_fixture" { "all_options_disabled" } else if open { "open" } else { "closed" },
        "event": event,
        "focus_target": if key == "Tab" { focus_target } else if focused { "multi_select" } else { "none" },
        "active_option": active,
        "values": values,
        "values_before": before,
        "events": emitted,
    })
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read case");
    let case: Value = serde_json::from_str(&input).expect("case JSON");
    println!("{}", json!({"passed": true, "actual": run(&case)}));
}
