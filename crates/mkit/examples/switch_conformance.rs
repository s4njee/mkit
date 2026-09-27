extern crate gpui_pre as gpui;
use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, Window, div,
};
use mkit::{
    core::theme,
    switch::{ChangeRequested, Switch, default_key_bindings},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

struct Host {
    switch: Option<Entity<Switch>>,
    changes: Rc<RefCell<Vec<bool>>>,
    _subscription: Option<Subscription>,
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.switch.is_none() {
            let switch = cx.new(|_| Switch::new("Wi-Fi", false));
            let changes = self.changes.clone();
            self._subscription =
                Some(cx.subscribe(&switch, move |_, _, event: &ChangeRequested, _| {
                    changes.borrow_mut().push(event.0);
                }));
            self.switch = Some(switch);
        }
        div().child(self.switch.as_ref().expect("switch initialized").clone())
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read generated case");
    let case: Value = serde_json::from_str(&input).expect("generated conformance case JSON");
    assert_eq!(case["kind"], "keyboard_cases");
    assert_eq!(case["load_fixture"], "off_fixture");
    assert_eq!(case["dispatch"], json!({"key":"Space","modifiers":[]}));

    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host {
        switch: None,
        changes: Rc::new(RefCell::new(Vec::new())),
        _subscription: None,
    });
    let switch = window.update(|host, window, cx| {
        window.focus_next(cx);
        let switch = host.switch.as_ref().expect("rendered switch").clone();
        assert!(switch.focus_handle(cx).is_focused(window), "enabled switch is a tab stop");
        switch
    });
    window.simulate_keystroke("space");
    let (checked, events, focused) = window.update(|host, window, cx| {
        (
            switch.read(cx).is_checked(),
            host.changes.borrow().clone(),
            switch.focus_handle(cx).is_focused(window),
        )
    });
    assert!(checked, "Space changes off to on");
    assert_eq!(events, vec![true], "Space emits one ChangeRequested(true)");
    assert!(focused, "switch retains focus after activation");
    println!("{}", json!({"passed":true,"actual":{"state":"on","event":"change","events":events}}));
}
