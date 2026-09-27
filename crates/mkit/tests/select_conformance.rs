use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled,
    Subscription, TestApp, Window, div, px,
};
use mkit::{
    core::theme,
    select::{OpenChanged, OptionItem, Select, ValueChanged, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

const OPTIONS: &[(&str, &str, bool)] = &[
    ("a", "Alpha", false),
    ("b", "Blocked", true),
    ("c", "Charlie", false),
    ("d", "Delta", false),
];

struct Host {
    fixture: String,
    controlled: bool,
    select: Option<Entity<Select>>,
    events: Rc<RefCell<Vec<String>>>,
    requests: Rc<RefCell<Vec<Option<String>>>>,
    _subscriptions: Vec<Subscription>,
}

impl Host {
    fn new(fixture: &str, controlled: bool) -> Self {
        Self {
            fixture: fixture.to_owned(),
            controlled,
            select: None,
            events: Rc::new(RefCell::new(Vec::new())),
            requests: Rc::new(RefCell::new(Vec::new())),
            _subscriptions: Vec::new(),
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.select.is_none() {
            let options = OPTIONS
                .iter()
                .map(|(id, label, disabled)| OptionItem::new(*id, *label).disabled(*disabled))
                .collect();
            let initial_value = Some("a".to_owned());
            let mut select = if self.controlled {
                Select::controlled("Choice", options, initial_value)
            } else {
                Select::new("Choice", options, initial_value)
            };
            if self.fixture == "disabled_fixture" {
                select = select.disabled(true);
            }
            let entity = cx.new(|_| select);
            let events = self.events.clone();
            let open_subscription = cx.subscribe(&entity, move |_, _, event: &OpenChanged, _| {
                events.borrow_mut().push(if event.0 { "open" } else { "close" }.to_owned());
            });
            let events = self.events.clone();
            let requests = self.requests.clone();
            let value_subscription = cx.subscribe(&entity, move |_, _, event: &ValueChanged, _| {
                events.borrow_mut().push("change".to_owned());
                requests.borrow_mut().push(event.0.clone());
            });
            self.select = Some(entity);
            self._subscriptions = vec![open_subscription, value_subscription];
        }
        div().w(px(320.)).child(self.select.as_ref().expect("initialized").clone())
    }
}

fn fixture_app(fixture: &str, controlled: bool) -> (TestApp, gpui_pre::TestAppWindow<Host>) {
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let window = app.open_window(|_, _| Host::new(fixture, controlled));
    (app, window)
}

fn keyboard_case(case: &Value) -> Value {
    let fixture = case["load_fixture"].as_str().expect("fixture");
    assert!(
        fixture == "closed_fixture" || fixture == "open_fixture" || fixture == "disabled_fixture"
    );
    let key = case["dispatch"]["key"].as_str().expect("dispatch key");
    assert_eq!(case["dispatch"]["modifiers"], json!([]), "Select generated keys have no modifiers");
    let keystroke = match key {
        "ArrowDown" => "down",
        "Enter" => "enter",
        "Escape" => "escape",
        "Home" => "home",
        "End" => "end",
        _ => panic!("unsupported generated Select key {key}"),
    };
    let (_app, mut window) = fixture_app(fixture, false);
    let (select, events) = window.update(|host, window, cx| {
        let select = host.select.as_ref().expect("rendered Select").clone();
        select.focus_handle(cx).focus(window, cx);
        (select, host.events.clone())
    });
    if fixture == "open_fixture" {
        window.simulate_keystroke("enter");
        events.borrow_mut().clear();
        // Opening by Enter must not commit a value; focus remains on the trigger.
    }
    window.simulate_keystroke(keystroke);
    let (open, value, emitted) = window.update(|_, _, cx| {
        let select = select.read(cx);
        (select.is_open(), select.value().map(str::to_owned), events.borrow().clone())
    });
    let expected_event = case["assert"]["event"].as_str();
    if let Some(expected) = expected_event {
        assert!(
            emitted.iter().any(|event| event == expected),
            "expected {expected} event, got {emitted:?}"
        );
    }
    let state = if open { "open" } else { "closed" };
    match case["id"].as_str().unwrap_or("") {
        "keyboard-01" => {
            assert_eq!(state, "open");
            window.simulate_keystroke("enter");
            let committed = window.update(|_, _, cx| select.read(cx).value().map(str::to_owned));
            assert_eq!(committed.as_deref(), Some("c"), "ArrowDown skips disabled option");
        }
        "keyboard-02" => assert_eq!(value.as_deref(), Some("a"), "Enter commits active value"),
        "keyboard-03" => {
            assert_eq!(value.as_deref(), Some("a"), "Escape leaves committed value unchanged")
        }
        "keyboard-04" => {
            assert_eq!(state, "open");
            window.simulate_keystroke("enter");
            let committed = window.update(|_, _, cx| select.read(cx).value().map(str::to_owned));
            assert_eq!(committed.as_deref(), Some("a"), "Home selects first enabled option");
        }
        "keyboard-05" => {
            assert_eq!(state, "open");
            window.simulate_keystroke("enter");
            let committed = window.update(|_, _, cx| select.read(cx).value().map(str::to_owned));
            assert_eq!(committed.as_deref(), Some("d"), "End selects last enabled option");
        }
        id => panic!("unmapped generated Select case {id}"),
    }
    json!({ "state": state, "event": expected_event })
}

fn check_pointer_controlled_disabled() {
    // Pointer row selection must commit and close exactly once.
    let (_app, mut window) = fixture_app("open_fixture", false);
    let (select, events, requests) = window.update(|host, _, _| {
        (host.select.as_ref().unwrap().clone(), host.events.clone(), host.requests.clone())
    });
    window.draw();
    window.simulate_click(gpui_pre::point(px(18.), px(16.)), gpui_pre::MouseButton::Left);
    let open = window.update(|_, _, cx| select.read(cx).is_open());
    assert!(open, "trigger pointer opens popup");
    events.borrow_mut().clear();
    let (disabled_row_y, enabled_row_y) = window.update(|_, _, cx| {
        let tokens = cx.global::<theme::Theme>();
        let list_top = tokens.controls.medium + tokens.spacing.small + tokens.borders.regular;
        (list_top + tokens.controls.medium * 1.5, list_top + tokens.controls.medium * 2.5)
    });
    window.draw();
    window
        .simulate_click(gpui_pre::point(px(18.), px(disabled_row_y)), gpui_pre::MouseButton::Left);
    let after_disabled_row = window
        .update(|_, _, cx| (select.read(cx).is_open(), select.read(cx).value().map(str::to_owned)));
    assert_eq!(
        after_disabled_row,
        (true, Some("a".to_owned())),
        "disabled option pointer input is inert"
    );
    assert!(events.borrow().is_empty(), "disabled option emits no value event");
    window.draw();
    window.simulate_click(gpui_pre::point(px(18.), px(enabled_row_y)), gpui_pre::MouseButton::Left);
    let (open, value) = window
        .update(|_, _, cx| (select.read(cx).is_open(), select.read(cx).value().map(str::to_owned)));
    assert!(!open, "selecting a row closes popup without parent-click reopening it");
    assert_eq!(value.as_deref(), Some("c"), "enabled row commits its option");
    assert_eq!(events.borrow().iter().filter(|event| *event == "change").count(), 1);
    assert_eq!(*requests.borrow(), vec![Some("c".to_owned())]);

    // Controlled requests emit while the displayed value waits for the owner.
    let (_app, mut window) = fixture_app("open_fixture", true);
    let (select, events, requests) = window.update(|host, window, cx| {
        let select = host.select.as_ref().unwrap().clone();
        select.focus_handle(cx).focus(window, cx);
        (select, host.events.clone(), host.requests.clone())
    });
    window.simulate_keystroke("enter");
    window.simulate_keystroke("down");
    window.simulate_keystroke("enter");
    let before_owner = window.update(|_, _, cx| select.read(cx).value().map(str::to_owned));
    assert_eq!(before_owner.as_deref(), Some("a"));
    assert!(
        events.borrow().contains(&"change".to_owned()),
        "controlled selection emits a value request"
    );
    assert_eq!(*requests.borrow(), vec![Some("c".to_owned())]);
    window
        .update(|_, _, cx| select.update(cx, |select, cx| select.set_value(Some("c".into()), cx)));
    let applied = window.update(|_, _, cx| select.read(cx).value().map(str::to_owned));
    assert_eq!(applied.as_deref(), Some("c"));

    // Disabled controls ignore keyboard navigation and pointer activation.
    let (_app, mut window) = fixture_app("disabled_fixture", false);
    let (select, events) =
        window.update(|host, _, _| (host.select.as_ref().unwrap().clone(), host.events.clone()));
    window.draw();
    window.simulate_keystroke("down");
    window.simulate_click(gpui_pre::point(px(18.), px(20.)), gpui_pre::MouseButton::Left);
    let remains_closed = window.update(|_, _, cx| !select.read(cx).is_open());
    assert!(remains_closed, "disabled Select remains closed");
    assert!(events.borrow().is_empty(), "disabled Select emits no events");
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"keyboard_cases","id":"keyboard-01","dispatch":{"key":"ArrowDown","modifiers":[]},"load_fixture":"closed_fixture","assert":{"state":"open","event":"open"}})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "keyboard_cases");
    let actual = keyboard_case(&case);
    check_pointer_controlled_disabled();
    println!("{}", json!({"passed":true,"actual":actual}));
}
