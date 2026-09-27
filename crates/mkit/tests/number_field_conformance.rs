use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, Window, div,
};
use mkit::{
    core::theme,
    scrubbable_number_field::{
        ChangeSource, ScrubbableNumberField, ValueChanged, ValueCommitted, default_key_bindings,
    },
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    field: Option<Entity<ScrubbableNumberField>>,
    changes: Rc<RefCell<Vec<ValueChanged>>>,
    commits: Rc<RefCell<Vec<ValueCommitted>>>,
    _subscriptions: Vec<Subscription>,
}

impl Host {
    fn new() -> Self {
        Self {
            field: None,
            changes: Rc::new(RefCell::new(Vec::new())),
            commits: Rc::new(RefCell::new(Vec::new())),
            _subscriptions: Vec::new(),
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.field.is_none() {
            let field = cx.new(|_| {
                ScrubbableNumberField::new(12.5)
                    .bounds(Some(0.0), Some(100.0))
                    .step(1.0)
                    .label("Number")
            });
            let changes = self.changes.clone();
            let changed =
                cx.subscribe(&field, move |_, _, event, _| changes.borrow_mut().push(*event));
            let commits = self.commits.clone();
            let committed =
                cx.subscribe(&field, move |_, _, event, _| commits.borrow_mut().push(*event));
            self.field = Some(field);
            self._subscriptions = vec![changed, committed];
        }
        div().child(self.field.as_ref().expect("field initialized").clone())
    }
}

fn keyboard_case(case: &Value) -> Value {
    let dispatch = case.get("dispatch").expect("generated case dispatch");
    let key = dispatch.get("key").and_then(Value::as_str).expect("dispatch key");
    let modifiers =
        dispatch.get("modifiers").and_then(Value::as_array).expect("dispatch modifiers");
    let modifier_names =
        modifiers.iter().map(|value| value.as_str().expect("modifier name")).collect::<Vec<_>>();
    let keystroke = match (key, modifier_names.as_slice()) {
        ("ArrowUp", []) => "up",
        ("ArrowDown", []) => "down",
        ("PageUp", []) => "pageup",
        ("PageDown", []) => "pagedown",
        ("Home", []) => "home",
        ("End", []) => "end",
        ("Enter", []) => "enter",
        ("Escape", []) => "escape",
        ("ArrowUp", ["Shift"]) => "shift-up",
        ("ArrowDown", ["Shift"]) => "shift-down",
        _ => panic!("unsupported generated dispatch: key={key}, modifiers={modifiers:?}"),
    };

    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host::new());
    let field = window.update(|host, window, cx| {
        let field = host.field.as_ref().expect("rendered field").clone();
        field.focus_handle(cx).focus(window, cx);
        field
    });
    let focused_before = window.update(|_, window, cx| field.focus_handle(cx).is_focused(window));
    assert!(focused_before, "field should be focused before dispatch");
    let text_fixture = match key {
        "Enter" => Some("13.5"),
        "Escape" => Some("-"),
        _ => None,
    };
    if let Some(draft) = text_fixture {
        window.update(|_, _, cx| field.update(cx, |field, _| field.select_all()));
        window.simulate_input(draft);
        let (shown_draft, committed) = window
            .update(|_, _, cx| (field.read(cx).draft_text().to_owned(), field.read(cx).value()));
        assert_eq!(shown_draft, draft, "fixture must enter a real GPUI text draft");
        assert_eq!(committed, 12.5, "draft must not commit before the key dispatch");
    }
    window.simulate_keystroke(keystroke);

    let (value, draft, focused_after, changes, commits) = window.update(|host, window, cx| {
        (
            field.read(cx).value(),
            field.read(cx).draft_text().to_owned(),
            field.focus_handle(cx).is_focused(window),
            host.changes.borrow().clone(),
            host.commits.borrow().clone(),
        )
    });
    let expected_value = match (key, modifier_names.as_slice()) {
        ("ArrowUp", []) => 13.5,
        ("ArrowDown", []) => 11.5,
        ("PageUp", []) => 22.5,
        ("PageDown", []) => 2.5,
        ("Home", []) => 0.0,
        ("End", []) => 100.0,
        ("Enter", []) => 13.5,
        ("Escape", []) => 12.5,
        ("ArrowUp", ["Shift"]) => 12.6,
        ("ArrowDown", ["Shift"]) => 12.4,
        _ => unreachable!("dispatch validated above"),
    };
    assert_eq!(value, expected_value);
    assert!(focused_after, "focus should remain on the number field");
    if key == "Escape" {
        assert_eq!(draft, "12.5", "Escape restores the committed display");
        assert!(changes.is_empty(), "Escape must not change the value");
        assert!(commits.is_empty(), "Escape must not commit the invalid draft");
    } else {
        assert_eq!(draft, expected_value.to_string(), "display follows the committed value");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].value, expected_value);
        assert_eq!(
            changes[0].source,
            if key == "Enter" { ChangeSource::Text } else { ChangeSource::Keyboard }
        );
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].value, expected_value);
        assert_eq!(
            commits[0].source,
            if key == "Enter" { ChangeSource::Text } else { ChangeSource::Keyboard }
        );
    }

    json!({
        "state": "focused",
        "event": if key == "Enter" { "value_committed" } else if key == "Escape" { "none" } else { "value_changed" },
        "focus_target": "number_field",
        "value": value,
        "draft": draft,
        "commit_event": commits.first().map(|_| "value_committed"),
        "commit_value": commits.first().map(|event| event.value),
    })
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case from stdin");
    let case: Value = if input.trim().is_empty() {
        json!({"kind": "keyboard_cases", "id": "keyboard-01", "dispatch": {"key": "ArrowUp", "modifiers": []}})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    let actual = match case.get("kind").and_then(Value::as_str) {
        Some("keyboard_cases") => keyboard_case(&case),
        kind => panic!("unsupported case kind {kind:?}"),
    };
    println!("{}", json!({"passed": true, "actual": actual}));
}
