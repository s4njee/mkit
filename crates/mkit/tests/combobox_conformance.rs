use gpui_pre::{
    AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, Subscription, TestApp, Window, div,
};
use mkit::{
    combobox::{
        Cancelled, Combobox, Dismissed, OpenChanged, OptionItem, OptionSelected, ValueChanged,
        default_key_bindings,
    },
    core::theme,
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

const COUNTRIES: &[(&str, &str)] = &[
    ("us", "United States"),
    ("cameroon", "Cameroon"),
    ("canada", "Canada"),
    ("cambodia", "Cambodia"),
];

struct Host {
    fixture: String,
    combobox: Option<Entity<Combobox>>,
    previous_focus: Option<FocusHandle>,
    next_focus: Option<FocusHandle>,
    events: Rc<RefCell<Vec<&'static str>>>,
    _subscriptions: Vec<Subscription>,
}

impl Host {
    fn new(fixture: &str) -> Self {
        Self {
            fixture: fixture.to_owned(),
            combobox: None,
            previous_focus: None,
            next_focus: None,
            events: Rc::new(RefCell::new(Vec::new())),
            _subscriptions: Vec::new(),
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let previous_focus = self
            .previous_focus
            .get_or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(true))
            .clone();
        let next_focus = self
            .next_focus
            .get_or_insert_with(|| cx.focus_handle().tab_index(2).tab_stop(true))
            .clone();
        if self.combobox.is_none() {
            let options =
                COUNTRIES.iter().map(|(id, label)| OptionItem::new(*id, *label)).collect();
            let value = (self.fixture == "closed_committed_fixture"
                || self.fixture == "open_filtered_fixture")
                .then(|| "us".to_owned());
            let mut component = Combobox::new("Country", options, value);
            if self.fixture == "disabled_fixture" {
                component = component.disabled(true);
            }
            let combobox = cx.new(|_| component);
            let opened = self.events.clone();
            let open_subscription = cx.subscribe(&combobox, move |_, _, event: &OpenChanged, _| {
                opened.borrow_mut().push(if event.0 { "open" } else { "close" });
            });
            let values = self.events.clone();
            let value_subscription = cx.subscribe(&combobox, move |_, _, _: &ValueChanged, _| {
                values.borrow_mut().push("commit");
            });
            let selections = self.events.clone();
            let selected_subscription =
                cx.subscribe(&combobox, move |_, _, _: &OptionSelected, _| {
                    selections.borrow_mut().push("select");
                });
            let cancelled = self.events.clone();
            let cancel_subscription = cx.subscribe(&combobox, move |_, _, _: &Cancelled, _| {
                cancelled.borrow_mut().push("cancel");
            });
            let dismissed = self.events.clone();
            let dismiss_subscription = cx.subscribe(&combobox, move |_, _, _: &Dismissed, _| {
                dismissed.borrow_mut().push("dismiss");
            });
            self.combobox = Some(combobox);
            self._subscriptions = vec![
                open_subscription,
                value_subscription,
                selected_subscription,
                cancel_subscription,
                dismiss_subscription,
            ];
        }
        div()
            .child(div().id("previous-focus-target").track_focus(&previous_focus))
            .child(self.combobox.as_ref().expect("initialized").clone())
            .child(div().id("next-focus-target").track_focus(&next_focus))
    }
}

fn keyboard_case(case: &Value) -> Value {
    let case_id = case.get("id").and_then(Value::as_str).expect("case id");
    let dispatch = case.get("dispatch").expect("generated case dispatch");
    let key = dispatch.get("key").and_then(Value::as_str).expect("dispatch key");
    let modifiers = dispatch.get("modifiers").and_then(Value::as_array).expect("modifiers");
    let modifier_names =
        modifiers.iter().map(|value| value.as_str().expect("modifier name")).collect::<Vec<_>>();
    let keystroke = match (key, modifier_names.as_slice()) {
        ("ArrowDown", []) => "down",
        ("ArrowUp", []) => "up",
        ("Enter", []) => "enter",
        ("Escape", []) => "escape",
        ("Tab", []) => "tab",
        ("Tab", ["Shift"]) => "shift-tab",
        ("ArrowDown", ["Alt"]) => "alt-down",
        _ => panic!("unsupported generated dispatch {key} {modifier_names:?}"),
    };
    let fixture = case.get("load_fixture").and_then(Value::as_str).expect("fixture name");

    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host::new(fixture));
    let (combobox, events, previous_focus, next_focus) = window.update(|host, window, cx| {
        let combobox = host.combobox.as_ref().expect("rendered combobox").clone();
        if fixture == "open_filtered_fixture" {
            combobox.update(cx, |combobox, cx| combobox.set_query("Cam", cx));
        }
        if fixture != "disabled_fixture" {
            combobox.focus_handle(cx).focus(window, cx);
        }
        (
            combobox,
            host.events.clone(),
            host.previous_focus.as_ref().expect("rendered previous focus target").clone(),
            host.next_focus.as_ref().expect("rendered next focus target").clone(),
        )
    });
    if fixture == "open_filtered_fixture" {
        // The fixture declares Cameroon active before dispatch; Cambodia is
        // also a matching row, so Up/Down verifies wrapped option navigation.
        window.simulate_keystroke("down");
        events.borrow_mut().clear();
    }
    let focused_before =
        window.update(|_, window, cx| combobox.focus_handle(cx).is_focused(window));
    window.simulate_keystroke(keystroke);

    let (open, query, value, active, focused_after, previous_focused, next_focused, emitted) =
        window.update(|_, window, cx| {
            (
                combobox.read(cx).is_open(),
                combobox.read(cx).query().to_owned(),
                combobox.read(cx).value().map(str::to_owned),
                combobox.read(cx).active_option().map(str::to_owned),
                combobox.focus_handle(cx).is_focused(window),
                previous_focus.is_focused(window),
                next_focus.is_focused(window),
                events.borrow().clone(),
            )
        });
    let expected_event = case["assert"]["event"].as_str();
    if let Some(expected) = expected_event {
        assert!(emitted.contains(&expected), "expected {expected} event; got {emitted:?}");
    }
    let focus_target = if focused_after { Some("input") } else { None };
    let state = if open {
        if query.is_empty() {
            if active.is_some() { "open_unfiltered_active" } else { "open_empty" }
        } else {
            "open_filtered"
        }
    } else if let Some(committed) = value.as_deref() {
        let committed_label = COUNTRIES
            .iter()
            .find(|(id, _)| *id == committed)
            .map_or(committed, |(_, label)| *label);
        if query == committed_label { "closed_committed" } else { "closed_draft" }
    } else if query.is_empty() {
        "closed_empty"
    } else {
        "closed_draft"
    };
    match case_id {
        "keyboard-06" | "keyboard-07" => {
            assert!(!open, "Tab dismisses the popup");
            assert_eq!(query, "Cam", "Tab preserves the typed draft");
            assert_eq!(value.as_deref(), Some("us"), "Tab does not commit the active suggestion");
            assert!(emitted.contains(&"dismiss"), "Tab emits Dismissed; got {emitted:?}");
        }
        "keyboard-02" => {
            assert_eq!(active.as_deref(), Some("cambodia"), "fixture navigation target");
        }
        "keyboard-03" => {
            assert_eq!(active.as_deref(), Some("cambodia"), "ArrowUp wraps to the last option");
        }
        "keyboard-01" => assert_eq!(active.as_deref(), Some("us")),
        "keyboard-08" => assert_eq!(active.as_deref(), Some("us")),
        "keyboard-09" => assert_eq!(active, None, "explicit open leaves no active option"),
        _ => {}
    }
    let traversal_target = if next_focused {
        Some("next")
    } else if previous_focused {
        Some("previous")
    } else if focused_after {
        Some("input")
    } else {
        None
    };
    json!({
        "state": state,
        "event": expected_event,
        "focus_target": if case_id == "keyboard-06" || case_id == "keyboard-07" { traversal_target } else { focus_target },
        "focused_before": focused_before,
        "query": query,
        "value": value,
        "active_option": active,
        "events": emitted,
    })
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case from stdin");
    let case: Value = if input.trim().is_empty() {
        json!({
            "kind": "keyboard_cases",
            "id": "keyboard-01",
            "dispatch": {"key": "ArrowDown", "modifiers": []},
            "load_fixture": "closed_empty_fixture",
            "assert": {"state": "open_unfiltered_active", "event": "open", "focus_target": "input"}
        })
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "keyboard_cases", "only generated keyboard cases are supported");
    let actual = keyboard_case(&case);
    println!("{}", json!({"passed": true, "actual": actual}));
}
