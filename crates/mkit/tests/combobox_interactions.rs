extern crate gpui_pre as gpui;

use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled,
    Subscription, TestAppContext, Window, div, px,
};
use mkit::{
    combobox::{Cancelled, Combobox, OptionItem, OptionSelected, ValueChanged},
    core::theme,
};
use std::{cell::RefCell, rc::Rc};

struct Host {
    combobox: Option<Entity<Combobox>>,
    controlled: bool,
    disabled_option: bool,
    cancellations: Rc<RefCell<Vec<Option<String>>>>,
    changes: Rc<RefCell<Vec<String>>>,
    selected: Rc<RefCell<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.combobox.is_none() {
            let controlled = self.controlled;
            let disabled_option = self.disabled_option;
            let combobox = cx.new(move |_| {
                let options = vec![
                    OptionItem::new("cameroon", "Cameroon").disabled(disabled_option),
                    OptionItem::new("canada", "Canada"),
                    OptionItem::new("cambodia", "Cambodia"),
                ];
                if controlled {
                    Combobox::controlled("Country", options, Some("canada".into()))
                } else {
                    Combobox::new("Country", options, None)
                }
            });
            let cancellations = self.cancellations.clone();
            let cancelled = cx.subscribe(&combobox, move |_, _, event: &Cancelled, _| {
                cancellations.borrow_mut().push(event.0.clone());
            });
            let changes = self.changes.clone();
            let changed = cx.subscribe(&combobox, move |_, _, event: &ValueChanged, _| {
                changes.borrow_mut().push(event.0.clone());
            });
            let selected = self.selected.clone();
            let selected_event = cx.subscribe(&combobox, move |_, _, event: &OptionSelected, _| {
                selected.borrow_mut().push(event.0.clone());
            });
            self.combobox = Some(combobox);
            self._subscriptions = vec![cancelled, changed, selected_event];
        }
        div().w(px(320.0)).child(self.combobox.as_ref().unwrap().clone())
    }
}

fn host(controlled: bool, disabled_option: bool) -> Host {
    Host {
        combobox: None,
        controlled,
        disabled_option,
        cancellations: Rc::new(RefCell::new(Vec::new())),
        changes: Rc::new(RefCell::new(Vec::new())),
        selected: Rc::new(RefCell::new(Vec::new())),
        _subscriptions: Vec::new(),
    }
}

#[gpui_pre::test]
fn controlled_cancel_keeps_the_parent_value_authoritative(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    cx.update(|app| app.bind_keys(mkit::combobox::default_key_bindings()));
    let (host, visual) = cx.add_window_view(|_, _| host(true, false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let (combobox, cancellations, changes, selected) = host.read_with(visual, |host, _| {
        (
            host.combobox.as_ref().unwrap().clone(),
            host.cancellations.clone(),
            host.changes.clone(),
            host.selected.clone(),
        )
    });
    visual.update(|window, cx| combobox.focus_handle(cx).focus(window, cx));
    combobox.update(visual, |combobox, cx| combobox.set_query("Cam", cx));
    combobox.update(visual, |combobox, cx| combobox.set_value(Some("cameroon".into()), cx));
    visual.simulate_keystrokes("escape");

    combobox.read_with(visual, |combobox, _| {
        assert_eq!(combobox.value(), Some("cameroon"));
        assert_eq!(combobox.query(), "Cameroon");
        assert!(!combobox.is_open());
    });
    assert_eq!(&*cancellations.borrow(), &[Some("canada".into())]);
    assert!(changes.borrow().is_empty());
    assert!(selected.borrow().is_empty());
}

#[gpui_pre::test]
fn controlled_selection_emits_a_request_without_changing_the_value_prop(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    cx.update(|app| app.bind_keys(mkit::combobox::default_key_bindings()));
    let (host, visual) = cx.add_window_view(|_, _| host(true, false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let (combobox, changes, selected) = host.read_with(visual, |host, _| {
        (host.combobox.as_ref().unwrap().clone(), host.changes.clone(), host.selected.clone())
    });
    visual.update(|window, cx| combobox.focus_handle(cx).focus(window, cx));
    combobox.update(visual, |combobox, cx| combobox.set_query("Cam", cx));
    visual.simulate_keystrokes("down enter");

    combobox.read_with(visual, |combobox, _| {
        assert_eq!(combobox.value(), Some("canada"));
        assert_eq!(combobox.query(), "Canada");
        assert!(!combobox.is_open());
    });
    assert_eq!(&*changes.borrow(), &["cameroon"]);
    assert_eq!(&*selected.borrow(), &["cameroon"]);
}

#[gpui_pre::test]
fn pointer_selects_enabled_rows_and_ignores_disabled_rows(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    let (host, visual) = cx.add_window_view(|_, _| host(false, true));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let (combobox, changes, selected) = host.read_with(visual, |host, _| {
        (host.combobox.as_ref().unwrap().clone(), host.changes.clone(), host.selected.clone())
    });
    combobox.update(visual, |combobox, cx| combobox.set_query("Cam", cx));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    assert!(combobox.read_with(visual, |combobox, _| combobox.is_open()));

    let disabled =
        visual.debug_bounds("combobox-option-cameroon").expect("disabled option row bounds");
    visual.simulate_click(disabled.center(), Default::default());
    assert_eq!(combobox.read_with(visual, |combobox, _| combobox.value().map(str::to_owned)), None);
    assert!(changes.borrow().is_empty());
    assert!(selected.borrow().is_empty());

    let enabled =
        visual.debug_bounds("combobox-option-cambodia").expect("enabled option row bounds");
    visual.simulate_click(enabled.center(), Default::default());
    assert_eq!(
        combobox.read_with(visual, |combobox, _| combobox.value().map(str::to_owned)),
        Some("cambodia".into())
    );
    assert_eq!(&*changes.borrow(), &["cambodia"]);
    assert_eq!(&*selected.borrow(), &["cambodia"]);
}
