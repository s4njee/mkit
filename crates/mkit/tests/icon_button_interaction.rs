extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, IntoElement, Modifiers, ParentElement, Render, TestAppContext, Window, div,
};
use mkit::{core::theme, icon_button::IconButton};
use std::{cell::RefCell, rc::Rc};

struct Host {
    disabled: bool,
    events: Rc<RefCell<Vec<&'static str>>>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let clicks = self.events.clone();
        let activations = self.events.clone();
        div().child(
            IconButton::new("Close", div())
                .id(1)
                .disabled(self.disabled)
                .on_click(move |_, _, _| clicks.borrow_mut().push("click"))
                .on_activate(move |_, _| activations.borrow_mut().push("activate")),
        )
    }
}

#[gpui::test]
fn pointer_click_runs_callbacks_in_order_and_disabled_suppresses_them(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    for (disabled, expected) in [(false, vec!["click", "activate"]), (true, vec![])] {
        let (host, visual) =
            cx.add_window_view(|_, _| Host { disabled, events: Rc::new(RefCell::new(Vec::new())) });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let target = visual.debug_bounds("mkit-icon-button").expect("icon button bounds").center();
        visual.simulate_click(target, Modifiers::default());
        host.read_with(visual, |host, _| assert_eq!(*host.events.borrow(), expected));
    }
}
