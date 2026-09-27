extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, IntoElement, Modifiers, ParentElement, Render, TestAppContext, Window, div,
};
use mkit::{button::Button, core::theme};
use std::{cell::RefCell, rc::Rc};

struct Host {
    disabled: bool,
    loading: bool,
    with_click: bool,
    events: Rc<RefCell<Vec<&'static str>>>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let clicks = self.events.clone();
        let activations = self.events.clone();
        let mut button = Button::new("Save")
            .id(1)
            .disabled(self.disabled)
            .loading(self.loading)
            .on_activate(move |_, _| activations.borrow_mut().push("activate"));
        if self.with_click {
            button = button.on_click(move |_, _, _| clicks.borrow_mut().push("click"));
        }
        div().child(button)
    }
}

#[gpui_pre::test]
fn pointer_activation_fires_callbacks_in_order_and_suppresses_unavailable_states(
    cx: &mut TestAppContext,
) {
    cx.update(theme::set_light_theme);
    for (disabled, loading, with_click, expected) in [
        (false, false, true, vec!["click", "activate"]),
        (false, false, false, vec!["activate"]),
        (true, false, true, vec![]),
        (false, true, true, vec![]),
    ] {
        let (host, visual) = cx.add_window_view(|_, _| Host {
            disabled,
            loading,
            with_click,
            events: Rc::new(RefCell::new(Vec::new())),
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let target = visual.debug_bounds("mkit-button").expect("button bounds").center();
        visual.simulate_click(target, Modifiers::default());
        host.read_with(visual, |host, _| assert_eq!(*host.events.borrow(), expected));
    }
}
