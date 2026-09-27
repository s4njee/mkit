use gpui_pre::{
    Context, IntoElement, MouseButton, ParentElement, Render, ScrollHandle, Styled, TestApp,
    Window, div, point, px,
};
use mkit::{
    core::theme,
    scroll_area::{ScrollArea, default_key_bindings},
};
use serde_json::{Value, json};
use std::io::{self, Read};

struct Host {
    handle: ScrollHandle,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        ScrollArea::new(80.0, div().h(px(800.0)).child("Long content"))
            .id(1)
            .label("Details")
            .width(240.0)
            .handle(self.handle.clone())
    }
}

fn run_case(case: &Value) -> Value {
    assert_eq!(case["dispatch"]["modifiers"], json!([]));
    let fixture = case["load_fixture"].as_str().expect("fixture");
    assert!(matches!(fixture, "idle_fixture" | "scrolled_fixture"));
    let (key, expected_state) = match case["dispatch"]["key"].as_str() {
        Some("PageDown") => ("pagedown", "scrolled"),
        Some("PageUp") => ("pageup", "idle"),
        Some("End") => ("end", "scrolled"),
        Some("Home") => ("home", "idle"),
        Some("ArrowDown") => ("down", "scrolled"),
        Some("ArrowUp") => ("up", "idle"),
        other => panic!("unsupported ScrollArea key {other:?}"),
    };
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host { handle: ScrollHandle::new() });
    window.draw();
    if fixture == "scrolled_fixture" {
        window.update(|host, window, _| {
            host.handle.set_offset(point(px(0.0), px(-32.0)));
            window.refresh();
        });
        window.draw();
    }
    window.simulate_click(point(px(12.0), px(12.0)), MouseButton::Left);
    window.simulate_keystroke(key);
    let (offset, max_offset) = window.update(|host, _, _| {
        (f32::from(host.handle.offset().y), f32::from(host.handle.max_offset().y))
    });
    let actual_state = if offset < -0.1 { "scrolled" } else { "idle" };
    assert_eq!(actual_state, expected_state, "{key} produced offset {offset}");
    match key {
        "down" => assert!((offset + 32.0).abs() < 0.1, "ArrowDown moves one token line"),
        "end" => assert!((offset + max_offset).abs() < 0.1, "End reaches bottom"),
        _ => {}
    }
    json!({"state":actual_state})
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"keyboard_cases","id":"keyboard-01","dispatch":{"key":"PageDown","modifiers":[]},"load_fixture":"idle_fixture","assert":{"state":"scrolled"}})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "keyboard_cases");
    let actual = run_case(&case);
    println!("{}", json!({"passed":true,"actual":actual}));
}
