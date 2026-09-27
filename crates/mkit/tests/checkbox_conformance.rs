use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, Window, div,
};
use mkit::{
    checkbox::{ChangeRequested, Checkbox, default_key_bindings},
    core::theme,
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    checkbox: Option<Entity<Checkbox>>,
    changes: Rc<RefCell<Vec<bool>>>,
    _subscription: Option<Subscription>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.checkbox.is_none() {
            let checkbox = cx.new(|_| Checkbox::new("Remember me", false));
            let changes = self.changes.clone();
            self._subscription =
                Some(cx.subscribe(&checkbox, move |_, _, event: &ChangeRequested, _| {
                    changes.borrow_mut().push(event.0);
                }));
            self.checkbox = Some(checkbox);
        }
        div().child(self.checkbox.as_ref().expect("checkbox initialized").clone())
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = if input.trim().is_empty() {
        json!({"kind":"keyboard_cases","id":"keyboard-01","dispatch":{"key":"Space","modifiers":[]},"load_fixture":"unchecked_fixture"})
    } else {
        serde_json::from_str(&input).expect("generated case JSON")
    };
    assert_eq!(case["kind"], "keyboard_cases");
    assert_eq!(case["id"], "keyboard-01");
    assert_eq!(case["load_fixture"], "unchecked_fixture");
    assert_eq!(case["dispatch"], json!({"key":"Space","modifiers":[]}));
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host {
        checkbox: None,
        changes: Rc::new(RefCell::new(Vec::new())),
        _subscription: None,
    });
    let checkbox = window.update(|host, window, cx| {
        window.focus_next(cx);
        let checkbox = host.checkbox.as_ref().expect("rendered checkbox").clone();
        assert!(checkbox.focus_handle(cx).is_focused(window), "enabled checkbox is a tab stop");
        checkbox
    });
    window.simulate_keystroke("space");
    let (checked, mixed, changes, focused) = window.update(|host, window, cx| {
        let checkbox = checkbox.read(cx);
        (
            checkbox.is_checked(),
            checkbox.is_indeterminate(),
            host.changes.borrow().clone(),
            checkbox.focus_handle(cx).is_focused(window),
        )
    });
    assert!(checked && !mixed, "Space changes unchecked to checked");
    assert_eq!(changes, vec![true], "Space emits one ChangeRequested(true)");
    assert!(focused, "checkbox retains focus");
    println!(
        "{}",
        json!({"passed":true,"actual":{"state":"checked","event":"change","checked":checked,"events":changes}})
    );
}
