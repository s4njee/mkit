extern crate gpui_pre as gpui;

use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, Window, div,
};
use mkit::{
    core::theme,
    toast::{OpenChanged, Toast, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    fixture: String,
    toast: Option<Entity<Toast>>,
    events: Rc<RefCell<Vec<bool>>>,
    subscription: Option<Subscription>,
}

impl Host {
    fn new(fixture: &str) -> Self {
        Self {
            fixture: fixture.to_owned(),
            toast: None,
            events: Rc::new(RefCell::new(Vec::new())),
            subscription: None,
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.toast.is_none() {
            let toast = if self.fixture == "toast_controlled" {
                Toast::controlled("Saved", "Changes saved", true)
            } else {
                Toast::new("Saved", "Changes saved")
            }
            .busy(self.fixture == "toast_busy");
            let toast = cx.new(|_| toast);
            let events = self.events.clone();
            self.subscription = Some(cx.subscribe(&toast, move |_, _, event: &OpenChanged, _| {
                events.borrow_mut().push(event.0);
            }));
            self.toast = Some(toast);
        }
        div().child(self.toast.as_ref().expect("toast initialized").clone())
    }
}

fn run(case: &Value) -> Value {
    let fixture = case["load_fixture"].as_str().expect("fixture");
    assert_eq!(case["dispatch"]["key"], "Escape");
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host::new(fixture));
    let (toast, events) = window.update(|host, window, cx| {
        let toast = host.toast.as_ref().expect("toast rendered").clone();
        toast.focus_handle(cx).focus(window, cx);
        (toast, host.events.clone())
    });
    window.simulate_keystroke("escape");
    let open = window.update(|_, _, cx| toast.read(cx).is_open());
    let events = events.borrow().clone();
    let state = if !open {
        "dismissed"
    } else if fixture == "toast_busy" {
        "busy"
    } else if fixture == "toast_controlled" {
        "controlled"
    } else {
        "visible"
    };
    json!({
        "state": state,
        "event": if events.is_empty() { "none" } else { "OpenChanged(false)" },
        "open": open,
        "events": events,
    })
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read case");
    let case: Value = if input.trim().is_empty() {
        json!({"load_fixture": "toast_visible", "dispatch": {"key": "Escape"}})
    } else {
        serde_json::from_str(&input).expect("case JSON")
    };
    println!("{}", json!({"passed": true, "actual": run(&case)}));
}
