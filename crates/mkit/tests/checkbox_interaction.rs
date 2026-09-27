extern crate gpui_pre as gpui;
use gpui_pre::{
    AppContext, Context, Entity, IntoElement, Modifiers, ParentElement, Render, Subscription,
    TestAppContext, Window, div,
};
use mkit::{
    checkbox::{ChangeRequested, Checkbox},
    core::theme,
};
use std::{cell::RefCell, rc::Rc};

struct Host {
    controlled: bool,
    mixed: bool,
    disabled: bool,
    checkbox: Option<Entity<Checkbox>>,
    events: Rc<RefCell<Vec<bool>>>,
    _subscription: Option<Subscription>,
}

impl Host {
    fn new(controlled: bool, mixed: bool, disabled: bool) -> Self {
        Self {
            controlled,
            mixed,
            disabled,
            checkbox: None,
            events: Rc::new(RefCell::new(Vec::new())),
            _subscription: None,
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.checkbox.is_none() {
            let component = if self.controlled {
                Checkbox::controlled("Remember me", false)
            } else {
                Checkbox::new("Remember me", false)
            }
            .indeterminate(self.mixed)
            .disabled(self.disabled);
            let checkbox = cx.new(|_| component);
            let events = self.events.clone();
            self._subscription =
                Some(cx.subscribe(&checkbox, move |_, _, event: &ChangeRequested, _| {
                    events.borrow_mut().push(event.0)
                }));
            self.checkbox = Some(checkbox);
        }
        div().child(self.checkbox.as_ref().expect("checkbox initialized").clone())
    }
}

#[gpui_pre::test]
fn pointer_and_controlled_checkbox_contract(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);

    let (host, visual) = cx.add_window_view(|_, _| Host::new(false, false, false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = visual.debug_bounds("mkit-checkbox").expect("checkbox bounds").center();
    visual.simulate_click(target, Modifiers::default());
    host.read_with(visual, |host, cx| {
        let checkbox = host.checkbox.as_ref().expect("checkbox").read(cx);
        assert!(checkbox.is_checked());
        assert_eq!(*host.events.borrow(), vec![true]);
    });
    visual.simulate_click(target, Modifiers::default());
    host.read_with(visual, |host, cx| {
        let checkbox = host.checkbox.as_ref().expect("checkbox").read(cx);
        assert!(!checkbox.is_checked());
        assert_eq!(*host.events.borrow(), vec![true, false]);
    });

    let (mixed_host, mixed_visual) = cx.add_window_view(|_, _| Host::new(false, true, false));
    mixed_visual.update(|window, cx| window.draw(cx).clear(cx));
    let target =
        mixed_visual.debug_bounds("mkit-checkbox").expect("mixed checkbox bounds").center();
    mixed_visual.simulate_click(target, Modifiers::default());
    mixed_host.read_with(mixed_visual, |host, cx| {
        let checkbox = host.checkbox.as_ref().expect("checkbox").read(cx);
        assert!(checkbox.is_checked());
        assert!(!checkbox.is_indeterminate());
        assert_eq!(*host.events.borrow(), vec![true]);
    });

    let (controlled_host, controlled_visual) =
        cx.add_window_view(|_, _| Host::new(true, false, false));
    controlled_visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = controlled_visual
        .debug_bounds("mkit-checkbox")
        .expect("controlled checkbox bounds")
        .center();
    controlled_visual.simulate_click(target, Modifiers::default());
    let controlled = controlled_host.read_with(controlled_visual, |host, cx| {
        assert_eq!(*host.events.borrow(), vec![true]);
        let checkbox = host.checkbox.as_ref().expect("checkbox").clone();
        assert!(!checkbox.read(cx).is_checked());
        checkbox
    });
    controlled.update(controlled_visual, |checkbox, cx| checkbox.set_checked(true, cx));
    assert!(controlled.read_with(controlled_visual, |checkbox, _| checkbox.is_checked()));

    let (controlled_mixed_host, controlled_mixed_visual) =
        cx.add_window_view(|_, _| Host::new(true, true, false));
    controlled_mixed_visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = controlled_mixed_visual
        .debug_bounds("mkit-checkbox")
        .expect("controlled mixed checkbox bounds")
        .center();
    controlled_mixed_visual.simulate_click(target, Modifiers::default());
    let controlled_mixed = controlled_mixed_host.read_with(controlled_mixed_visual, |host, cx| {
        assert_eq!(*host.events.borrow(), vec![true]);
        let checkbox = host.checkbox.as_ref().expect("checkbox").clone();
        assert!(!checkbox.read(cx).is_checked());
        assert!(checkbox.read(cx).is_indeterminate());
        checkbox
    });
    controlled_mixed.update(controlled_mixed_visual, |checkbox, cx| checkbox.set_checked(true, cx));
    controlled_mixed.read_with(controlled_mixed_visual, |checkbox, _| {
        assert!(checkbox.is_checked());
        assert!(!checkbox.is_indeterminate());
    });

    let (disabled_host, disabled_visual) = cx.add_window_view(|_, _| Host::new(false, false, true));
    disabled_visual.update(|window, cx| window.draw(cx).clear(cx));
    let target =
        disabled_visual.debug_bounds("mkit-checkbox").expect("disabled checkbox bounds").center();
    disabled_visual.simulate_click(target, Modifiers::default());
    disabled_host.read_with(disabled_visual, |host, cx| {
        assert!(!host.checkbox.as_ref().expect("checkbox").read(cx).is_checked());
        assert!(host.events.borrow().is_empty());
    });
}
