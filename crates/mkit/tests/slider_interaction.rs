extern crate gpui_pre as gpui;
use gpui_pre::{
    AppContext, Context, Entity, IntoElement, Modifiers, MouseButton, ParentElement, Render,
    Styled, Subscription, TestAppContext, Window, div, point, px,
};
use mkit::{
    core::theme,
    slider::{ChangeRequested, Slider},
};
use std::{cell::RefCell, rc::Rc};

struct Host {
    range: bool,
    disabled: bool,
    slider: Option<Entity<Slider>>,
    events: Rc<RefCell<Vec<Vec<f64>>>>,
    _subscription: Option<Subscription>,
}

impl Host {
    fn new(range: bool, disabled: bool) -> Self {
        Self {
            range,
            disabled,
            slider: None,
            events: Rc::new(RefCell::new(Vec::new())),
            _subscription: None,
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.slider.is_none() {
            let component = if self.range {
                Slider::range("Range", 20.0, 80.0, 0.0, 100.0, 5.0)
            } else {
                Slider::new("Level", 20.0, 0.0, 100.0, 5.0)
            }
            .disabled(self.disabled);
            let slider = cx.new(|_| component);
            let events = self.events.clone();
            self._subscription =
                Some(cx.subscribe(&slider, move |_, _, event: &ChangeRequested, _| {
                    events.borrow_mut().push(event.0.clone())
                }));
            self.slider = Some(slider);
        }
        div().w(px(300.0)).child(self.slider.as_ref().expect("slider initialized").clone())
    }
}

#[gpui_pre::test]
fn pointer_drag_selects_thumb_and_clamps_outside_track(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    let (host, visual) = cx.add_window_view(|_, _| Host::new(true, false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let bounds = visual.debug_bounds("mkit-slider-track").expect("slider track bounds");
    let y = f32::from(bounds.origin.y) + f32::from(bounds.size.height) / 2.0;
    let at = |fraction: f32| {
        point(px(f32::from(bounds.origin.x) + f32::from(bounds.size.width) * fraction), px(y))
    };
    let start = at(0.7);
    visual.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
    host.read_with(visual, |host, cx| {
        assert_eq!(host.slider.as_ref().expect("slider").read(cx).values(), &[20.0, 70.0]);
        assert_eq!(*host.events.borrow(), vec![vec![20.0, 70.0]]);
    });
    visual.simulate_mouse_move(at(1.25), Some(MouseButton::Left), Modifiers::default());
    visual.simulate_mouse_up(at(1.25), MouseButton::Left, Modifiers::default());
    host.read_with(visual, |host, cx| {
        assert_eq!(host.slider.as_ref().expect("slider").read(cx).values(), &[20.0, 100.0]);
        assert_eq!(*host.events.borrow(), vec![vec![20.0, 70.0], vec![20.0, 100.0]]);
    });

    let (disabled_host, disabled_visual) = cx.add_window_view(|_, _| Host::new(false, true));
    disabled_visual.update(|window, cx| window.draw(cx).clear(cx));
    let bounds = disabled_visual.debug_bounds("mkit-slider-track").expect("disabled slider track");
    let target = point(
        px(f32::from(bounds.origin.x) + f32::from(bounds.size.width) * 0.8),
        px(f32::from(bounds.origin.y) + f32::from(bounds.size.height) / 2.0),
    );
    disabled_visual.simulate_click(target, Modifiers::default());
    disabled_host.read_with(disabled_visual, |host, cx| {
        assert_eq!(host.slider.as_ref().expect("slider").read(cx).values(), &[20.0]);
        assert!(host.events.borrow().is_empty());
    });
}
