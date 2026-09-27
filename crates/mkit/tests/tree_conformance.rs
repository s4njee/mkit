use gpui_pre::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Subscription, TestApp, Window,
    div, point, px,
};
use mkit::{
    core::theme,
    tree::{ActiveChanged, ExpansionChanged, Tree, TreeNode, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    tree: Option<Entity<Tree>>,
    events: Rc<RefCell<Vec<&'static str>>>,
    _subscriptions: Vec<Subscription>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.tree.is_none() {
            let tree = cx.new(|_| {
                Tree::new(
                    "Files",
                    vec![
                        TreeNode::branch("a", "Alpha", vec![TreeNode::leaf("child", "Child")]),
                        TreeNode::leaf("b", "Beta"),
                    ],
                )
            });
            let events = self.events.clone();
            self._subscriptions.push(cx.subscribe(&tree, move |_, _, _: &ActiveChanged, _| {
                events.borrow_mut().push("ActiveChanged");
            }));
            let events = self.events.clone();
            self._subscriptions.push(cx.subscribe(&tree, move |_, _, _: &ExpansionChanged, _| {
                events.borrow_mut().push("ExpansionChanged");
            }));
            self.tree = Some(tree);
        }
        div().child(self.tree.as_ref().expect("tree initialized").clone())
    }
}

fn run_case(case: &Value) -> Value {
    assert_eq!(case["load_fixture"], "collapsed_fixture");
    assert_eq!(case["dispatch"]["modifiers"], json!([]));
    let (keystroke, active, expanded) = match case["dispatch"]["key"].as_str() {
        Some("ArrowDown") => ("down", "b", false),
        Some("ArrowRight") => ("right", "a", true),
        key => panic!("unsupported tree key {key:?}"),
    };
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host {
        tree: None,
        events: Rc::new(RefCell::new(Vec::new())),
        _subscriptions: Vec::new(),
    });
    window.draw();
    // Click the row label, outside the separate disclosure hit target, to
    // establish keyboard focus without expanding the initially collapsed node.
    window.simulate_click(point(px(50.0), px(10.0)), gpui_pre::MouseButton::Left);
    window.simulate_keystroke(keystroke);
    let (current, is_expanded, events) = window.update(|host, _, cx| {
        let tree = host.tree.as_ref().expect("rendered tree").read(cx);
        (tree.active().map(str::to_owned), tree.is_expanded("a"), host.events.borrow().clone())
    });
    assert_eq!(current.as_deref(), Some(active));
    assert_eq!(is_expanded, expanded);
    let expected_event = case["assert"]["event"].as_str().expect("event assertion");
    assert!(events.contains(&expected_event), "expected {expected_event}, got {events:?}");
    json!({"event":expected_event})
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"keyboard_cases","id":"keyboard-01","dispatch":{"key":"ArrowDown","modifiers":[]},"load_fixture":"collapsed_fixture","assert":{"event":"ActiveChanged"}})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "keyboard_cases");
    let actual = run_case(&case);
    println!("{}", json!({"passed":true,"actual":actual}));
}
