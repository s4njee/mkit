use gpui_pre::{Context, IntoElement, ParentElement, Render, TestApp, Window, div};
use mkit::{button::Button, core::theme};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    fixture: &'static str,
    activations: Rc<RefCell<usize>>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let activations = self.activations.clone();
        div().child(
            Button::new("Save")
                .id(1)
                .disabled(self.fixture == "button_disabled")
                .loading(self.fixture == "button_loading")
                .on_activate(move |_, _| {
                    *activations.borrow_mut() += 1;
                }),
        )
    }
}

fn activation_count(fixture: &'static str, keystroke: &str) -> usize {
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(mkit::button::default_key_bindings());
    });
    let mut window =
        app.open_window(|_, _| Host { fixture, activations: Rc::new(RefCell::new(0)) });
    let activations = window.update(|host, _, _| host.activations.clone());
    window.update(|_, window, cx| window.focus_next(cx));
    window.simulate_keystroke(keystroke);
    *activations.borrow()
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind": "keyboard_cases", "dispatch": {"key": "Enter", "modifiers": []}, "load_fixture": "button_idle"})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "keyboard_cases");
    assert_eq!(case["load_fixture"], "button_idle");
    assert_eq!(case["dispatch"]["modifiers"], json!([]));
    let key = case["dispatch"]["key"].as_str().expect("dispatch key");
    let keystroke = match key {
        "Enter" => "enter",
        "Space" => "space",
        _ => panic!("unsupported key {key}"),
    };

    assert_eq!(
        activation_count("button_idle", keystroke),
        1,
        "button must activate once from {keystroke}"
    );
    assert_eq!(
        activation_count("button_disabled", keystroke),
        0,
        "disabled button must remain inert"
    );
    assert_eq!(
        activation_count("button_loading", keystroke),
        0,
        "loading button must remain inert"
    );
    println!("{}", json!({"passed": true, "actual": {"event": "activate"}}));
}
