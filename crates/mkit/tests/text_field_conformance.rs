use gpui_pre::{
    AppContext, ClipboardItem, Context, Entity, Focusable, IntoElement, ParentElement, Render,
    Subscription, TestApp, Window, div,
};
use mkit::{
    core::theme,
    text_field::{InputChanged, TextField, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    field: Option<Entity<TextField>>,
    changes: Rc<RefCell<Vec<String>>>,
    _subscription: Option<Subscription>,
}

impl Host {
    fn new() -> Self {
        Self { field: None, changes: Rc::new(RefCell::new(Vec::new())), _subscription: None }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.field.is_none() {
            let field = cx.new(|cx| {
                let mut field = TextField::new(cx).with_label("Name");
                field.set_fixture_state("A😀B", 4..4, None);
                field
            });
            let changes = self.changes.clone();
            let subscription = cx.subscribe(&field, move |_, _, event: &InputChanged, _| {
                changes.borrow_mut().push(event.0.clone());
            });
            self.field = Some(field);
            self._subscription = Some(subscription);
        }
        div().child(self.field.as_ref().expect("field initialized").clone())
    }
}

fn keyboard_case(case: &Value) -> Value {
    assert_eq!(case.get("kind"), Some(&Value::String("keyboard_cases".into())));
    assert_eq!(case["load_fixture"], "filled_fixture");
    let key = case["dispatch"]["key"].as_str().expect("dispatch key");
    let keystroke = match key {
        "Ctrl+A" => "ctrl-a",
        "Ctrl+C" => "ctrl-c",
        "Ctrl+X" => "ctrl-x",
        "Ctrl+V" => "ctrl-v",
        "Ctrl+Z" => "ctrl-z",
        "Ctrl+Shift+Z" => "ctrl-shift-z",
        _ => panic!("unsupported generated key {key}"),
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
    assert!(window.update(|_, window, cx| field.focus_handle(cx).is_focused(window)));

    if matches!(key, "Ctrl+C" | "Ctrl+X") {
        window.update(|_, _, cx| {
            field.update(cx, |field, _| field.set_fixture_state("A😀B", 1..3, None));
        });
    }
    if key == "Ctrl+V" {
        window.update(|_, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string("P\nQ".to_owned()));
        });
    }

    match key {
        "Ctrl+Z" => {
            window.simulate_input("!");
            window.simulate_input("?");
        }
        "Ctrl+Shift+Z" => {
            window.simulate_input("!");
            window.simulate_input("?");
            window.simulate_keystroke("ctrl-z");
            window.simulate_keystroke("ctrl-z");
        }
        _ => {}
    }
    window.update(|host, _, _| host.changes.borrow_mut().clear());
    window.simulate_keystroke(keystroke);

    let (text, selection, changes, focused) = window.update(|host, window, cx| {
        (
            field.read(cx).text().to_owned(),
            field.read(cx).selection_utf16().range,
            host.changes.borrow().clone(),
            field.focus_handle(cx).is_focused(window),
        )
    });
    assert!(focused, "{key} should retain field focus");
    let expected_event = case["assert"].get("event").is_some();
    assert_eq!(changes.is_empty(), !expected_event, "unexpected InputChanged events for {key}");
    match key {
        "Ctrl+A" => {
            assert_eq!(selection, 0..4, "Ctrl+A selects the complete UTF-16 range");
            assert!(changes.is_empty(), "selection-only changes do not emit InputChanged");
        }
        "Ctrl+Z" => {
            assert_eq!(text, "A😀B!", "Ctrl+Z undoes one edit");
            assert_eq!(changes, vec!["A😀B!".to_owned()], "undo emits the restored value once");
        }
        "Ctrl+Shift+Z" => {
            assert_eq!(text, "A😀B!", "Ctrl+Shift+Z redoes one edit");
            assert_eq!(changes, vec!["A😀B!".to_owned()], "redo emits the restored value once");
        }
        "Ctrl+C" => {
            assert_eq!(text, "A😀B", "copy leaves the value unchanged");
            assert_eq!(selection, 1..3, "copy preserves UTF-16 selection");
            assert!(changes.is_empty(), "copy does not emit InputChanged");
            let copied =
                window.update(|_, _, cx| cx.read_from_clipboard().and_then(|item| item.text()));
            assert_eq!(
                copied.as_deref(),
                Some("😀"),
                "copy writes the selected text to GPUI clipboard"
            );
        }
        "Ctrl+X" => {
            assert_eq!(text, "AB", "cut removes the selected emoji");
            assert_eq!(selection, 1..1, "cut places the caret at the replacement point");
            assert_eq!(changes, vec!["AB".to_owned()], "cut emits one input event");
            let copied =
                window.update(|_, _, cx| cx.read_from_clipboard().and_then(|item| item.text()));
            assert_eq!(copied.as_deref(), Some("😀"), "cut writes selected text to GPUI clipboard");
        }
        "Ctrl+V" => {
            assert_eq!(text, "A😀BP Q", "paste replaces the caret with normalized plain text");
            assert_eq!(changes, vec!["A😀BP Q".to_owned()], "paste emits one input event");
        }
        _ => unreachable!(),
    }
    json!({"state": case["assert"]["state"], "event": case["assert"].get("event"), "value": text, "selection_utf16": [selection.start, selection.end], "events": changes, "focus_target": if focused { "input" } else { "none" }})
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({
            "kind": "keyboard_cases",
            "id": "keyboard-01",
            "dispatch": {"key": "Ctrl+A", "modifiers": ["Ctrl"]},
            "load_fixture": "filled_fixture",
            "assert": {"state": "focused", "focus_target": "input"}
        })
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    let actual = keyboard_case(&case);
    println!("{}", json!({"passed": true, "actual": actual}));
}
