use gpui_pre::{Context, IntoElement, ParentElement, Render, TestApp, Window, div};
use mkit::{core::theme, icon_button::IconButton};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    disabled: bool,
    activations: Rc<RefCell<usize>>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let activations = self.activations.clone();
        div().child(
            IconButton::new("Close", div())
                .id(1)
                .disabled(self.disabled)
                .on_activate(move |_, _| *activations.borrow_mut() += 1),
        )
    }
}

fn activation_count(disabled: bool, keystroke: &str) -> usize {
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(mkit::icon_button::default_key_bindings());
    });
    let mut window =
        app.open_window(|_, _| Host { disabled, activations: Rc::new(RefCell::new(0)) });
    let activations = window.update(|host, _, _| host.activations.clone());
    window.update(|_, window, cx| window.focus_next(cx));
    window.simulate_keystroke(keystroke);
    *activations.borrow()
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"keyboard_cases", "dispatch":{"key":"Enter", "modifiers":[]}, "load_fixture":"icon_button_idle"})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "keyboard_cases");
    assert_eq!(case["load_fixture"], "icon_button_idle");
    assert_eq!(case["dispatch"]["modifiers"], json!([]));
    let key = case["dispatch"]["key"].as_str().expect("dispatch key");
    let keystroke = match key {
        "Enter" => "enter",
        "Space" => "space",
        _ => panic!("unsupported key {key}"),
    };
    assert_eq!(
        activation_count(false, keystroke),
        1,
        "enabled icon button activates from {keystroke}"
    );
    assert_eq!(activation_count(true, keystroke), 0, "disabled icon button suppresses {keystroke}");
    println!("{}", json!({"passed":true, "actual":{"event":"activate"}}));
}
