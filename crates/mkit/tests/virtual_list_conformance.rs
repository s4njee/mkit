use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, Window, div,
};
use mkit::{
    core::theme,
    virtual_list::{ActiveChanged, ListItem, SelectionChanged, VirtualList, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    list: Option<Entity<VirtualList>>,
    events: Rc<RefCell<Vec<&'static str>>>,
    _subscriptions: Vec<Subscription>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.list.is_none() {
            let list = cx.new(|_| {
                VirtualList::new(
                    "Files",
                    vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Beta")],
                )
            });
            let events = self.events.clone();
            self._subscriptions.push(cx.subscribe(&list, move |_, _, _: &ActiveChanged, _| {
                events.borrow_mut().push("ActiveChanged");
            }));
            let events = self.events.clone();
            self._subscriptions.push(cx.subscribe(&list, move |_, _, _: &SelectionChanged, _| {
                events.borrow_mut().push("SelectionChanged");
            }));
            self.list = Some(list);
        }
        div().child(self.list.as_ref().expect("list initialized").clone())
    }
}

fn run_case(case: &Value) -> Value {
    assert_eq!(case["load_fixture"], "idle_fixture");
    assert_eq!(case["dispatch"]["modifiers"], json!([]));
    let (keystroke, expected_active, expected_selected) = match case["dispatch"]["key"].as_str() {
        Some("ArrowDown") => ("down", Some("b"), Vec::<String>::new()),
        Some("Space") => ("space", Some("a"), vec!["a".to_owned()]),
        key => panic!("unsupported virtual-list key {key:?}"),
    };
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host {
        list: None,
        events: Rc::new(RefCell::new(Vec::new())),
        _subscriptions: Vec::new(),
    });
    window.draw();
    let (list, events) = window.update(|host, window, cx| {
        let list = host.list.as_ref().expect("rendered list").clone();
        list.focus_handle(cx).focus(window, cx);
        (list, host.events.clone())
    });
    window.simulate_keystroke(keystroke);
    let (active, selected) = window.update(|_, _, cx| {
        let list = list.read(cx);
        (list.active().map(str::to_owned), list.selected().to_vec())
    });
    assert_eq!(active.as_deref(), expected_active);
    assert_eq!(selected, expected_selected);
    let expected_event = case["assert"]["event"].as_str().expect("event assertion");
    assert!(events.borrow().contains(&expected_event), "expected {expected_event}, got {events:?}");
    json!({"event": expected_event})
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"keyboard_cases","id":"keyboard-01","dispatch":{"key":"ArrowDown","modifiers":[]},"load_fixture":"idle_fixture","assert":{"event":"ActiveChanged"}})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "keyboard_cases");
    let actual = run_case(&case);
    println!("{}", json!({"passed":true,"actual":actual}));
}
