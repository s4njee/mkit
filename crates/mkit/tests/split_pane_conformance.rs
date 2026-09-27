use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, Render, Subscription, TestApp, Window, div,
};
use mkit::{
    core::theme,
    split_pane::{Orientation, RatioChanged, SplitPane, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Panel;
impl Render for Panel {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

struct Host {
    vertical: bool,
    pane: Option<Entity<SplitPane>>,
    changes: Rc<RefCell<Vec<f32>>>,
    _subscription: Option<Subscription>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.pane.is_none() {
            let leading = cx.new(|_| Panel);
            let trailing = cx.new(|_| Panel);
            let pane = cx.new(|_| {
                SplitPane::new(leading, trailing, 0.5).orientation(if self.vertical {
                    Orientation::Vertical
                } else {
                    Orientation::Horizontal
                })
            });
            let changes = self.changes.clone();
            self._subscription = Some(cx.subscribe(&pane, move |_, _, event: &RatioChanged, _| {
                changes.borrow_mut().push(event.0);
            }));
            self.pane = Some(pane);
        }
        self.pane.as_ref().expect("pane initialized").clone()
    }
}

fn run_case(case: &Value) -> Value {
    assert_eq!(case["kind"], "keyboard_cases");
    let vertical = match case["load_fixture"].as_str() {
        Some("centered_fixture") => false,
        Some("vertical_centered_fixture") => true,
        other => panic!("unsupported split-pane fixture {other:?}"),
    };
    let key = case["dispatch"]["key"].as_str().expect("key");
    let shift = case["dispatch"]["modifiers"] == json!(["Shift"]);
    let (keystroke, expected_ratio) = match (key, shift) {
        ("ArrowRight", false) if !vertical => ("right", 0.52),
        ("ArrowLeft", false) if !vertical => ("left", 0.48),
        ("ArrowRight", true) if !vertical => ("shift-right", 0.505),
        ("ArrowLeft", true) if !vertical => ("shift-left", 0.495),
        ("Home", false) if !vertical => ("home", 0.1),
        ("End", false) if !vertical => ("end", 0.9),
        ("ArrowDown", false) if vertical => ("down", 0.52),
        ("ArrowUp", false) if vertical => ("up", 0.48),
        ("ArrowDown", true) if vertical => ("shift-down", 0.505),
        ("ArrowUp", true) if vertical => ("shift-up", 0.495),
        other => panic!("unsupported split-pane key {other:?} for vertical={vertical}"),
    };
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host {
        vertical,
        pane: None,
        changes: Rc::new(RefCell::new(Vec::new())),
        _subscription: None,
    });
    let (pane, changes) = window.update(|host, window, cx| {
        let pane = host.pane.as_ref().expect("pane rendered").clone();
        pane.focus_handle(cx).focus(window, cx);
        (pane, host.changes.clone())
    });
    window.simulate_keystroke(keystroke);
    let ratio = window.update(|_, _, cx| pane.read(cx).ratio());
    assert!((ratio - expected_ratio).abs() < 0.001, "{keystroke} ratio {ratio}");
    assert_eq!(*changes.borrow(), vec![ratio], "one typed ratio event");
    json!({"state":"resized","event":"ratio_changed","ratio":ratio})
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"keyboard_cases","dispatch":{"key":"ArrowRight","modifiers":[]},"load_fixture":"centered_fixture","assert":{"state":"resized","event":"ratio_changed"}})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    println!("{}", json!({"passed":true,"actual":run_case(&case)}));
}
