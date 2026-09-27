use gpui_pre::{
    AppContext, ClipboardItem, Context, Entity, Focusable, IntoElement, ParentElement, Render,
    Subscription, TestApp, Window, div,
};
use mkit::{
    core::theme,
    text_area::{InputChanged, TextArea, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    area: Option<Entity<TextArea>>,
    disabled: bool,
    changes: Rc<RefCell<Vec<String>>>,
    _subscription: Option<Subscription>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.area.is_none() {
            let disabled = self.disabled;
            let area = cx.new(|cx| TextArea::new(cx).with_label("Notes").disabled(disabled));
            let changes = self.changes.clone();
            self._subscription = Some(cx.subscribe(&area, move |_, _, event: &InputChanged, _| {
                changes.borrow_mut().push(event.0.clone());
            }));
            self.area = Some(area);
        }
        div().child(self.area.as_ref().expect("area initialized").clone())
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind": "keyboard_cases", "id": "keyboard-01"})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "keyboard_cases");
    let id = case["id"].as_str().expect("case id");
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let changes = Rc::new(RefCell::new(Vec::new()));
    let changes_for_host = changes.clone();
    let disabled = id == "keyboard-08";
    if id == "keyboard-06" {
        app.write_to_clipboard(ClipboardItem::new_string("🌱\nnext".into()));
    }
    let mut window = app.open_window(move |_, _| Host {
        area: None,
        disabled,
        changes: changes_for_host,
        _subscription: None,
    });
    let area = window.update(|host, window, cx| {
        let area = host.area.as_ref().expect("rendered area").clone();
        area.update(cx, |area, cx| area.set_value("one", cx));
        if id == "keyboard-04" || id == "keyboard-05" {
            area.update(cx, |area, _| area.set_fixture_state("one!", 0..3, None));
        }
        if !disabled {
            area.focus_handle(cx).focus(window, cx);
        }
        area
    });

    match id {
        "keyboard-01" => window.simulate_keystroke("ctrl-a"),
        "keyboard-02" => {
            window.simulate_input(" two");
            changes.borrow_mut().clear();
            window.simulate_keystroke("ctrl-z");
        }
        "keyboard-03" => {
            window.simulate_input(" two");
            window.simulate_keystroke("ctrl-z");
            changes.borrow_mut().clear();
            window.simulate_keystroke("ctrl-shift-z");
        }
        "keyboard-04" => window.simulate_keystroke("ctrl-c"),
        "keyboard-05" => window.simulate_keystroke("ctrl-x"),
        "keyboard-06" => window.simulate_keystroke("ctrl-v"),
        "keyboard-07" => {
            window.update(|_, _, cx| {
                area.update(cx, |area, _| area.set_fixture_state("one", 3..3, None))
            });
            window.simulate_keystroke("enter");
        }
        "keyboard-08" => window.simulate_keystroke("ctrl-a"),
        _ => panic!("unmapped generated case {id}"),
    }

    let (text, selection, focused) = window.update(|_, window, cx| {
        let area = area.read(cx);
        (
            area.text().to_owned(),
            area.selection_utf16().range,
            area.focus_handle(cx).is_focused(window),
        )
    });
    let event = if changes.borrow().is_empty() { "none" } else { "input" };
    let (state, focus_target, assertion) = match id {
        "keyboard-01" => ("focused", "input", selection == (0..3)),
        "keyboard-02" => ("filled", "input", text == "one" && event == "input"),
        "keyboard-03" => ("filled", "input", text == "one two" && event == "input"),
        "keyboard-04" => (
            "selected",
            "input",
            text == "one!"
                && selection == (0..3)
                && event == "none"
                && app.read_from_clipboard().and_then(|item| item.text()).as_deref() == Some("one"),
        ),
        "keyboard-05" => (
            "filled",
            "input",
            text == "!"
                && selection == (0..0)
                && event == "input"
                && app.read_from_clipboard().and_then(|item| item.text()).as_deref() == Some("one"),
        ),
        "keyboard-06" => ("filled", "input", text == "one🌱\nnext" && event == "input"),
        "keyboard-07" => ("filled", "input", text == "one\n" && event == "input"),
        "keyboard-08" => (
            "disabled",
            "none",
            text == "one" && selection == (3..3) && event == "none" && !focused,
        ),
        _ => unreachable!(),
    };
    assert!(
        assertion,
        "case {id} failed: text={text:?}, selection={selection:?}, event={event}, focused={focused}"
    );
    let actual = json!({
        "state": state,
        "event": event,
        "focus_target": focus_target,
        "text": text,
        "selection": [selection.start, selection.end],
    });
    println!("{}", json!({"passed": true, "actual": actual}));
}
