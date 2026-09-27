extern crate gpui_pre as gpui;
use gpui_pre::{
    AppContext, Context, Entity, IntoElement, Modifiers, ParentElement, Render, Subscription,
    TestAppContext, Window, div,
};
use mkit::{
    core::theme,
    switch::{ChangeRequested, Switch},
};
use std::{cell::RefCell, rc::Rc};

struct Host {
    controlled: bool,
    disabled: bool,
    switch: Option<Entity<Switch>>,
    events: Rc<RefCell<Vec<bool>>>,
    _subscription: Option<Subscription>,
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.switch.is_none() {
            let switch = cx.new(|_| {
                if self.controlled {
                    Switch::controlled("Wi-Fi", false)
                } else {
                    Switch::new("Wi-Fi", false)
                }
                .disabled(self.disabled)
            });
            let events = self.events.clone();
            self._subscription =
                Some(cx.subscribe(&switch, move |_, _, event: &ChangeRequested, _| {
                    events.borrow_mut().push(event.0)
                }));
            self.switch = Some(switch);
        }
        div().child(self.switch.as_ref().expect("switch initialized").clone())
    }
}
fn host(controlled: bool, disabled: bool) -> Host {
    Host {
        controlled,
        disabled,
        switch: None,
        events: Rc::new(RefCell::new(Vec::new())),
        _subscription: None,
    }
}

#[gpui_pre::test]
fn pointer_controlled_and_disabled_switch_contract(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    let (uncontrolled_host, visual) = cx.add_window_view(|_, _| host(false, false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = visual.debug_bounds("mkit-switch").expect("switch bounds").center();
    visual.simulate_click(target, Modifiers::default());
    uncontrolled_host.read_with(visual, |host, cx| {
        assert!(host.switch.as_ref().unwrap().read(cx).is_checked());
        assert_eq!(*host.events.borrow(), vec![true]);
    });

    let (controlled_host, controlled_visual) = cx.add_window_view(|_, _| host(true, false));
    controlled_visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = controlled_visual.debug_bounds("mkit-switch").expect("switch bounds").center();
    controlled_visual.simulate_click(target, Modifiers::default());
    let switch = controlled_host.read_with(controlled_visual, |host, cx| {
        assert_eq!(*host.events.borrow(), vec![true]);
        let switch = host.switch.as_ref().unwrap().clone();
        assert!(!switch.read(cx).is_checked(), "controlled switch waits for set_checked");
        switch
    });
    switch.update(controlled_visual, |switch, _| switch.set_checked(true));
    assert!(switch.read_with(controlled_visual, |switch, _| switch.is_checked()));

    let (disabled_host, disabled_visual) = cx.add_window_view(|_, _| host(false, true));
    disabled_visual.update(|window, cx| window.draw(cx).clear(cx));
    let target =
        disabled_visual.debug_bounds("mkit-switch").expect("disabled switch bounds").center();
    disabled_visual.simulate_click(target, Modifiers::default());
    disabled_host.read_with(disabled_visual, |host, cx| {
        assert!(!host.switch.as_ref().unwrap().read(cx).is_checked());
        assert!(host.events.borrow().is_empty(), "disabled switch emits no event");
    });
}
