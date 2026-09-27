use gpui_pre::TestAppContext;
use mkit_example_interaction::e4_overlay::{OverlayDismissalFixture, bind_overlay_dismiss_key};

#[gpui_pre::test]
fn e5_conformance_adapter(cx: &mut TestAppContext) {
    run_keyboard(cx);
}

fn run_keyboard(cx: &mut TestAppContext) {
    let case = std::env::var("MKIT_E5_CASE_ID").unwrap_or_else(|_| "keyboard-01".into());
    let dispatch_key = std::env::var("MKIT_E5_DISPATCH_KEY").unwrap_or_else(|_| "Escape".into());
    let modifiers = std::env::var("MKIT_E5_DISPATCH_MODIFIERS").unwrap_or_default();
    let key = gpui_keystroke(&dispatch_key, &modifiers);
    let binding = if case == "keyboard-02" { key.as_str() } else { "escape" };
    cx.update(|app| {
        mkit_core::theme::set_light_theme(app);
        bind_overlay_dismiss_key(app, binding).unwrap();
    });
    let (view, visual) = cx.add_window_view(|_, _| OverlayDismissalFixture::nested_open());
    visual.update(|window, cx| window.draw(cx).clear(cx));
    if case == "keyboard-02" {
        visual.simulate_keystrokes("escape");
        assert_eq!(view.read_with(visual, |fixture, _| fixture.state()).0, vec![1, 2]);
    }
    visual.simulate_keystrokes(&key);
    let (open, _, escapes, _) = view.read_with(visual, |fixture, _| fixture.state());
    assert_eq!(open, vec![1]);
    assert_eq!(escapes, 1);
    let state = if open == vec![1] { "outer_open" } else { "nested_open" };
    println!(
        "MKIT_E5_RESULT={{\"passed\":true,\"actual\":{{\"state\":\"{state}\",\"event\":\"dismiss_top\"}}}}"
    );
}

fn gpui_keystroke(key: &str, modifiers: &str) -> String {
    let mut parts = Vec::new();
    for modifier in modifiers.split(',').filter(|item| !item.is_empty()) {
        parts.push(match modifier {
            "Control" => "ctrl".to_owned(),
            "Alt" => "alt".to_owned(),
            "Shift" => "shift".to_owned(),
            "Platform" => "platform".to_owned(),
            other => panic!("unsupported generated modifier {other}"),
        });
    }
    parts.push(match key {
        "Escape" => "escape".to_owned(),
        "Enter" => "enter".to_owned(),
        "Tab" => "tab".to_owned(),
        other => other.to_lowercase(),
    });
    parts.join("-")
}
