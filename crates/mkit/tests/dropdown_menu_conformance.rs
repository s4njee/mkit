use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, Window, div,
};
use mkit::{
    core::theme,
    dropdown_menu::{self, DropdownMenu, MenuItem, OpenChanged},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    menu: Option<Entity<DropdownMenu>>,
    requests: Rc<RefCell<Vec<bool>>>,
    selections: Rc<RefCell<Vec<String>>>,
    checked: Rc<RefCell<Vec<(String, bool)>>>,
    _subscriptions: Vec<Subscription>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.menu.is_none() {
            let items = vec![
                MenuItem::new("more", "More").submenu(vec![MenuItem::new("nested", "Nested")]),
                MenuItem::new("blocked", "Blocked").disabled(true),
                MenuItem::new("pin", "Pin").checked(false),
                MenuItem::new("save", "Save").shortcut("⌘S"),
            ];
            let menu = cx.new(|_| DropdownMenu::controlled(items, true));
            let requests = self.requests.clone();
            let open_subscription = cx.subscribe(&menu, move |_, _, event: &OpenChanged, _| {
                requests.borrow_mut().push(event.0)
            });
            let selections = self.selections.clone();
            let selected_subscription =
                cx.subscribe(&menu, move |_, _, event: &dropdown_menu::ItemSelected, _| {
                    selections.borrow_mut().push(event.0.clone())
                });
            let checked = self.checked.clone();
            let checked_subscription =
                cx.subscribe(&menu, move |_, _, event: &dropdown_menu::CheckedChanged, _| {
                    checked.borrow_mut().push((event.0.clone(), event.1))
                });
            self._subscriptions =
                vec![open_subscription, selected_subscription, checked_subscription];
            self.menu = Some(menu);
        }
        div().child(self.menu.as_ref().unwrap().clone())
    }
}

fn case_evidence(case: &Value) -> Value {
    assert_eq!(case["component"], "dropdown-menu");
    assert_eq!(case["kind"], "keyboard_cases");
    let fixture = case["load_fixture"].as_str().expect("fixture");
    assert!(matches!(fixture, "dropdown-menu_open" | "dropdown-menu_submenu"));
    assert_eq!(case["dispatch"]["modifiers"], json!([]));
    let key = case["dispatch"]["key"].as_str().expect("key");
    let keystroke = match key {
        "Escape" => "escape",
        "ArrowRight" => "right",
        "ArrowLeft" => "left",
        "Home" => "home",
        "End" => "end",
        _ => panic!("unsupported dropdown-menu key {key}"),
    };

    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(dropdown_menu::default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host {
        menu: None,
        requests: Rc::new(RefCell::new(Vec::new())),
        selections: Rc::new(RefCell::new(Vec::new())),
        checked: Rc::new(RefCell::new(Vec::new())),
        _subscriptions: Vec::new(),
    });
    window.draw();
    let (menu, requests, selections) = window.update(|host, window, cx| {
        let menu = host.menu.as_ref().unwrap().clone();
        menu.focus_handle(cx).focus(window, cx);
        (menu, host.requests.clone(), host.selections.clone())
    });
    if fixture == "dropdown-menu_submenu" {
        window.simulate_keystroke("right");
        window.draw();
    }
    window.simulate_keystroke(keystroke);
    window.draw();

    let expected = case["assert"].as_object().expect("assert mapping");
    let mut actual = serde_json::Map::new();
    if expected.contains_key("event") {
        assert_eq!(
            requests.borrow().as_slice(),
            &[false],
            "Escape emits a controlled close request"
        );
        let still_open = window.update(|_, _, cx| menu.read(cx).is_open());
        assert!(still_open, "controlled menu applies close only when owner calls set_open");
        actual.insert("event".into(), json!("open_changed"));
        actual.insert("requested_open".into(), json!(false));
        actual.insert("controlled_open_before_owner_update".into(), json!(still_open));
        window.update(|_, _, cx| menu.update(cx, |menu, cx| menu.set_open(false, cx)));
        let closed = window.update(|_, _, cx| !menu.read(cx).is_open());
        assert!(closed, "owner update applies controlled close");
        actual.insert("owner_applied_close".into(), json!(closed));
    }
    if let Some(target) = expected.get("focus_target").and_then(Value::as_str) {
        let steps = match target {
            "submenu-first-enabled" => {
                window.simulate_keystroke("enter");
                assert_eq!(selections.borrow().as_slice(), &["nested"]);
                1
            }
            "submenu-parent" | "first-enabled" => {
                window.simulate_keystroke("enter");
                window.simulate_keystroke("enter");
                assert_eq!(selections.borrow().as_slice(), &["nested"]);
                2
            }
            "last-enabled" => {
                window.simulate_keystroke("enter");
                assert_eq!(selections.borrow().as_slice(), &["save"]);
                1
            }
            _ => panic!("unmapped focus target {target}"),
        };
        actual.insert("focus_target".into(), json!(target));
        actual.insert("active_target_confirmed_by_activation".into(), json!(true));
        actual.insert("activation_steps".into(), json!(steps));
        actual.insert("selected_item".into(), json!(selections.borrow()[0]));
    }
    Value::Object(actual)
}

fn verify_disabled_item_is_skipped() -> bool {
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(dropdown_menu::default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host {
        menu: None,
        requests: Rc::new(RefCell::new(Vec::new())),
        selections: Rc::new(RefCell::new(Vec::new())),
        checked: Rc::new(RefCell::new(Vec::new())),
        _subscriptions: Vec::new(),
    });
    window.draw();
    let checked = window.update(|host, window, cx| {
        let menu = host.menu.as_ref().unwrap().clone();
        menu.focus_handle(cx).focus(window, cx);
        host.checked.clone()
    });
    window.simulate_keystroke("down");
    window.simulate_keystroke("enter");
    assert_eq!(
        checked.borrow().as_slice(),
        &[("pin".to_owned(), true)],
        "Down skips disabled row and toggles Pin"
    );
    true
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read conformance case");
    let case: Value = if input.trim().is_empty() {
        json!({"component":"dropdown-menu","kind":"keyboard_cases","id":"keyboard-01","dispatch":{"key":"Escape","modifiers":[]},"load_fixture":"dropdown-menu_open","assert":{"event":"open_changed"}})
    } else {
        serde_json::from_str(&input).expect("conformance case JSON")
    };
    let mut actual = case_evidence(&case);
    assert!(verify_disabled_item_is_skipped());
    actual["disabled_item_skipped"] = json!(true);
    println!("{}", json!({"passed":true,"actual":actual}));
}
