extern crate gpui_pre as gpui;
use gpui_pre::{
    AppContext, Context, Entity, IntoElement, Modifiers, ParentElement, Render, Subscription,
    TestAppContext, Window, div,
};
use mkit::{
    core::theme,
    radio_group::{ChangeRequested, OptionItem, RadioGroup, default_key_bindings},
};
use std::{cell::RefCell, rc::Rc};

struct Host {
    group: Option<Entity<RadioGroup>>,
    events: Rc<RefCell<Vec<String>>>,
    _sub: Option<Subscription>,
    controlled: bool,
    disabled: bool,
}
impl Host {
    fn new(controlled: bool, disabled: bool) -> Self {
        Self {
            group: None,
            events: Rc::new(RefCell::new(Vec::new())),
            _sub: None,
            controlled,
            disabled,
        }
    }
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.group.is_none() {
            let options = vec![
                OptionItem::new("first", "First"),
                OptionItem::new("blocked", "Blocked").disabled(true),
                OptionItem::new("second", "Second"),
            ];
            let component = if self.controlled {
                RadioGroup::controlled("Plan", options, Some("first".into()))
            } else {
                RadioGroup::new("Plan", options, Some("first".into()))
            }
            .disabled(self.disabled);
            let group = cx.new(|_| component);
            let events = self.events.clone();
            self._sub = Some(cx.subscribe(&group, move |_, _, event: &ChangeRequested, _| {
                events.borrow_mut().push(event.0.clone())
            }));
            self.group = Some(group);
        }
        div().child(self.group.as_ref().unwrap().clone())
    }
}

#[gpui_pre::test]
fn pointer_disabled_and_controlled_radio_contract(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    cx.update(|cx| cx.bind_keys(default_key_bindings()));
    let (host, visual) = cx.add_window_view(|_, _| Host::new(false, false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let second = visual.debug_bounds("radio-option-second").expect("second option bounds").center();
    visual.simulate_click(second, Modifiers::default());
    host.read_with(visual, |host, cx| {
        let group = host.group.as_ref().unwrap().read(cx);
        assert_eq!(group.value(), Some("second"));
        assert_eq!(group.focused_option(), Some("second"));
        assert_eq!(*host.events.borrow(), vec!["second"]);
    });
    let blocked =
        visual.debug_bounds("radio-option-blocked").expect("disabled option bounds").center();
    visual.simulate_click(blocked, Modifiers::default());
    host.read_with(visual, |host, cx| {
        assert_eq!(host.group.as_ref().unwrap().read(cx).value(), Some("second"));
        assert_eq!(*host.events.borrow(), vec!["second"]);
    });

    let (controlled_host, controlled_visual) = cx.add_window_view(|_, _| Host::new(true, false));
    controlled_visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = controlled_visual
        .debug_bounds("radio-option-second")
        .expect("second option bounds")
        .center();
    controlled_visual.simulate_click(target, Modifiers::default());
    let group = controlled_host.read_with(controlled_visual, |host, cx| {
        let group = host.group.as_ref().unwrap().clone();
        assert_eq!(group.read(cx).value(), Some("first"), "controlled view waits for owner update");
        assert_eq!(group.read(cx).focused_option(), Some("second"));
        assert_eq!(*host.events.borrow(), vec!["second"]);
        group
    });
    controlled_visual.simulate_keystrokes("space");
    controlled_host.read_with(controlled_visual, |host, cx| {
        assert_eq!(host.group.as_ref().unwrap().read(cx).value(), Some("first"));
        assert_eq!(
            *host.events.borrow(),
            vec!["second", "second"],
            "Space requests the focused option"
        );
    });
    group.update(controlled_visual, |group, _| group.set_value(Some("second".into())));
    assert_eq!(
        group.read_with(controlled_visual, |group, _| group.value().map(str::to_owned)),
        Some("second".to_owned())
    );

    let (disabled_host, disabled_visual) = cx.add_window_view(|_, _| Host::new(false, true));
    disabled_visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = disabled_visual
        .debug_bounds("radio-option-second")
        .expect("disabled group option bounds")
        .center();
    disabled_visual.simulate_click(target, Modifiers::default());
    disabled_host.read_with(disabled_visual, |host, cx| {
        assert_eq!(host.group.as_ref().unwrap().read(cx).value(), Some("first"));
        assert!(host.events.borrow().is_empty());
    });
}
