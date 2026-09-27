extern crate gpui_pre as gpui;

use gpui_pre::{
    AppContext, Context, Entity, IntoElement, Modifiers, ParentElement, Render, Subscription,
    TestAppContext, Window, div,
};
use mkit::{
    core::theme,
    multi_select::{MultiSelect, OpenChanged, OptionItem, ValuesChanged},
};
use std::{cell::RefCell, rc::Rc};

struct Host {
    control: Option<Entity<MultiSelect>>,
    values_events: Rc<RefCell<Vec<Vec<String>>>>,
    open_events: Rc<RefCell<Vec<bool>>>,
    subscriptions: Vec<Subscription>,
    controlled: bool,
    disabled: bool,
}

impl Host {
    fn new(controlled: bool, disabled: bool) -> Self {
        Self {
            control: None,
            values_events: Rc::new(RefCell::new(Vec::new())),
            open_events: Rc::new(RefCell::new(Vec::new())),
            subscriptions: Vec::new(),
            controlled,
            disabled,
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.control.is_none() {
            let options = vec![
                OptionItem::new("alpha", "Alpha"),
                OptionItem::new("blocked", "Blocked").disabled(true),
                OptionItem::new("beta", "Beta"),
            ];
            let control = if self.controlled {
                MultiSelect::controlled("Topics", options, vec!["alpha".into()])
            } else {
                MultiSelect::new("Topics", options, vec!["alpha".into()])
            }
            .disabled(self.disabled);
            let control = cx.new(|_| control);
            let values_events = self.values_events.clone();
            self.subscriptions.push(cx.subscribe(
                &control,
                move |_, _, event: &ValuesChanged, _| {
                    values_events.borrow_mut().push(event.0.clone());
                },
            ));
            let open_events = self.open_events.clone();
            self.subscriptions.push(cx.subscribe(&control, move |_, _, event: &OpenChanged, _| {
                open_events.borrow_mut().push(event.0);
            }));
            self.control = Some(control);
        }
        div().child(self.control.as_ref().unwrap().clone())
    }
}

#[gpui_pre::test]
fn pointer_selection_keeps_popup_and_respects_controlled_disabled(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    let (host, visual) = cx.add_window_view(|_, _| Host::new(false, false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = visual.debug_bounds("mkit-multi-select").expect("trigger").center();
    visual.simulate_click(trigger, Modifiers::default());
    visual.update(|window, cx| window.draw(cx).clear(cx));
    host.read_with(visual, |host, cx| {
        assert!(host.control.as_ref().unwrap().read(cx).is_open());
        assert_eq!(*host.open_events.borrow(), vec![true]);
    });
    let blocked =
        visual.debug_bounds("mkit-multi-select-option-blocked").expect("disabled row").center();
    visual.simulate_click(blocked, Modifiers::default());
    host.read_with(visual, |host, cx| {
        assert_eq!(host.control.as_ref().unwrap().read(cx).values(), ["alpha"]);
        assert!(host.values_events.borrow().is_empty());
    });
    let beta = visual.debug_bounds("mkit-multi-select-option-beta").expect("beta row").center();
    visual.simulate_click(beta, Modifiers::default());
    host.read_with(visual, |host, cx| {
        let control = host.control.as_ref().unwrap().read(cx);
        assert_eq!(control.values(), ["alpha", "beta"]);
        assert!(control.is_open(), "selecting a row keeps the multi-select open");
        assert_eq!(*host.values_events.borrow(), vec![vec!["alpha".to_owned(), "beta".to_owned()]]);
        assert_eq!(*host.open_events.borrow(), vec![true]);
    });

    let (controlled_host, controlled_visual) = cx.add_window_view(|_, _| Host::new(true, false));
    controlled_visual.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = controlled_visual.debug_bounds("mkit-multi-select").unwrap().center();
    controlled_visual.simulate_click(trigger, Modifiers::default());
    controlled_visual.update(|window, cx| window.draw(cx).clear(cx));
    let beta = controlled_visual.debug_bounds("mkit-multi-select-option-beta").unwrap().center();
    controlled_visual.simulate_click(beta, Modifiers::default());
    let control = controlled_host.read_with(controlled_visual, |host, cx| {
        let control = host.control.as_ref().unwrap().clone();
        assert_eq!(control.read(cx).values(), ["alpha"], "owner must apply controlled request");
        assert_eq!(*host.values_events.borrow(), vec![vec!["alpha".to_owned(), "beta".to_owned()]]);
        control
    });
    control.update(controlled_visual, |control, cx| {
        control.set_values(vec!["alpha".into(), "beta".into()], cx)
    });
    assert_eq!(
        control.read_with(controlled_visual, |control, _| control.values().to_vec()),
        ["alpha", "beta"]
    );

    let (disabled_host, disabled_visual) = cx.add_window_view(|_, _| Host::new(false, true));
    disabled_visual.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = disabled_visual.debug_bounds("mkit-multi-select").unwrap().center();
    disabled_visual.simulate_click(trigger, Modifiers::default());
    disabled_host.read_with(disabled_visual, |host, cx| {
        assert!(!host.control.as_ref().unwrap().read(cx).is_open());
        assert!(host.open_events.borrow().is_empty());
        assert!(host.values_events.borrow().is_empty());
    });
}
