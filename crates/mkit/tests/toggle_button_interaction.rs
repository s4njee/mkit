extern crate gpui_pre as gpui;
use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, TestAppWindow, Window, div,
};
use mkit::{
    core::theme,
    toggle_button::{ToggleButton, ValueChanged, default_key_bindings},
};
use std::{cell::RefCell, rc::Rc};

struct Host {
    controlled: bool,
    disabled: bool,
    toggle: Option<Entity<ToggleButton>>,
    changes: Rc<RefCell<Vec<bool>>>,
    _subscription: Option<Subscription>,
}

impl Host {
    fn new(controlled: bool, disabled: bool) -> Self {
        Self {
            controlled,
            disabled,
            toggle: None,
            changes: Rc::new(RefCell::new(Vec::new())),
            _subscription: None,
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.toggle.is_none() {
            let toggle = cx.new(|_| {
                let toggle = if self.controlled {
                    ToggleButton::controlled("Bold", false)
                } else {
                    ToggleButton::new("Bold", false)
                };
                toggle.disabled(self.disabled)
            });
            let changes = self.changes.clone();
            let subscription = cx.subscribe(&toggle, move |_, _, event: &ValueChanged, _| {
                changes.borrow_mut().push(event.0);
            });
            self.toggle = Some(toggle);
            self._subscription = Some(subscription);
        }
        div().child(self.toggle.as_ref().expect("toggle initialized").clone())
    }
}

fn new_window(controlled: bool, disabled: bool) -> (TestApp, TestAppWindow<Host>) {
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let window = app.open_window(|_, _| Host::new(controlled, disabled));
    (app, window)
}

#[test]
fn uncontrolled_activation_updates_state_and_emits_requested_value() {
    let (_app, mut window) = new_window(false, false);
    let toggle = window.update(|host, window, cx| {
        let toggle = host.toggle.as_ref().expect("rendered toggle").clone();
        toggle.focus_handle(cx).focus(window, cx);
        toggle
    });

    window.simulate_keystroke("enter");
    window.update(|host, _, cx| {
        assert!(toggle.read(cx).is_pressed());
        assert_eq!(*host.changes.borrow(), vec![true]);
    });
}

#[test]
fn controlled_activation_requests_changes_until_parent_applies_them() {
    let (_app, mut window) = new_window(true, false);
    let toggle = window.update(|host, window, cx| {
        let toggle = host.toggle.as_ref().expect("rendered toggle").clone();
        toggle.focus_handle(cx).focus(window, cx);
        toggle
    });

    window.simulate_keystroke("enter");
    window.update(|host, _, cx| {
        assert!(!toggle.read(cx).is_pressed(), "controlled value stays with the parent");
        assert_eq!(*host.changes.borrow(), vec![true]);
        toggle.update(cx, |toggle, cx| toggle.set_pressed(true, cx));
    });
    window.simulate_keystroke("space");
    window.update(|host, _, cx| {
        assert!(toggle.read(cx).is_pressed(), "the parent-applied value is displayed");
        assert_eq!(*host.changes.borrow(), vec![true, false]);
    });
}

#[test]
fn disabled_toggle_suppresses_keyboard_requests_and_state_changes() {
    let (_app, mut window) = new_window(false, true);
    let toggle = window.update(|host, window, cx| {
        let toggle = host.toggle.as_ref().expect("rendered toggle").clone();
        // The disabled toggle is excluded from normal tab traversal. Directly
        // focusing its handle here also exercises its action guard.
        toggle.focus_handle(cx).focus(window, cx);
        toggle
    });

    window.simulate_keystroke("enter");
    window.simulate_keystroke("space");
    window.update(|host, _, cx| {
        assert!(!toggle.read(cx).is_pressed());
        assert!(host.changes.borrow().is_empty());
    });
}
