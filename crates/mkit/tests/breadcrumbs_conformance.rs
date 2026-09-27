use gpui_pre::{Context, IntoElement, ParentElement, Render, TestApp, Window, div};
use mkit::{
    breadcrumbs::{Breadcrumbs, Crumb, default_key_bindings},
    core::theme,
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    routes: Rc<RefCell<Vec<String>>>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let routes = self.routes.clone();
        div().child(
            Breadcrumbs::new(
                "Breadcrumb",
                vec![Crumb::new("Home").target("/home"), Crumb::new("Current")],
            )
            .on_navigate(move |target| routes.borrow_mut().push(target)),
        )
    }
}

fn run_case(case: &Value) -> Value {
    assert_eq!(case["kind"], "keyboard_cases");
    assert_eq!(case["load_fixture"], "breadcrumbs_selection");
    assert_eq!(case["dispatch"]["key"], "Enter");
    assert_eq!(case["dispatch"]["modifiers"], json!([]));
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host { routes: Rc::new(RefCell::new(Vec::new())) });
    window.update(|_, window, cx| window.focus_next(cx));
    window.simulate_keystroke("enter");
    let routes = window.update(|host, _, _| host.routes.borrow().clone());
    assert_eq!(routes, vec!["/home".to_owned()]);
    json!({"event":"navigate"})
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"keyboard_cases","dispatch":{"key":"Enter","modifiers":[]},"load_fixture":"breadcrumbs_selection","assert":{"event":"navigate"}})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    println!("{}", json!({"passed":true,"actual":run_case(&case)}));
}
