use gpui_pre::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Subscription, TestApp, Window,
    div,
};
use mkit::{
    core::theme,
    sidebar::{Item, Sidebar, ValueChanged, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    sidebar: Option<Entity<Sidebar>>,
    changes: Rc<RefCell<Vec<Option<String>>>>,
    _subscription: Option<Subscription>,
}
impl Host {
    fn new() -> Self {
        Self { sidebar: None, changes: Rc::new(RefCell::new(Vec::new())), _subscription: None }
    }
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.sidebar.is_none() {
            let items = vec![
                Item::new("home", "Home"),
                Item::new("blocked", "Blocked").disabled(true),
                Item::new("settings", "Settings"),
            ];
            let sidebar = cx.new(|_| Sidebar::new("Primary", items, Some("home".into())));
            let changes = self.changes.clone();
            self._subscription =
                Some(cx.subscribe(&sidebar, move |_, _, event: &ValueChanged, _| {
                    changes.borrow_mut().push(event.0.clone())
                }));
            self.sidebar = Some(sidebar);
        }
        div().child(self.sidebar.as_ref().unwrap().clone())
    }
}
fn keyboard_case(case: &Value) -> Value {
    assert_eq!(case["kind"], "keyboard_cases");
    assert_eq!(case["load_fixture"], "sidebar_selection");
    assert_eq!(case["dispatch"]["modifiers"], json!([]));
    let key = match case["dispatch"]["key"].as_str().unwrap() {
        "Enter" => "enter",
        other => panic!("unsupported key {other}"),
    };
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host::new());
    window.update(|_, window, cx| window.focus_next(cx));
    window.simulate_keystroke(key);
    let (value, events) = window.update(|host, _, cx| {
        (
            host.sidebar.as_ref().unwrap().read(cx).value().map(str::to_owned),
            host.changes.borrow().clone(),
        )
    });
    assert_eq!(value.as_deref(), Some("home"));
    assert_eq!(events, vec![Some("home".into())]);
    json!({"event": "value_changed", "state": "selection"})
}
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"keyboard_cases","dispatch":{"key":"Enter","modifiers":[]},"load_fixture":"sidebar_selection"})
    } else {
        serde_json::from_str(&input).expect("case JSON")
    };
    println!("{}", json!({"passed":true,"actual":keyboard_case(&case)}));
}
