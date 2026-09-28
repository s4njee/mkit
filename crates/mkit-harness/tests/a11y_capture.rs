//! End-to-end check that `AccessibilitySession` records the AccessKit tree GPUI
//! delivers to an active platform adapter. Runs on the main thread (macOS text
//! system requirement), so this target uses `harness = false`.

#[cfg(target_os = "macos")]
mod macos {
    use gpui_pre::{
        Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, Role,
        StatefulInteractiveElement as _, Window, div, prelude::FluentBuilder as _, px, size,
    };
    use mkit_harness::{AccessibilitySession, AccessibilitySnapshot, AccessibilityValue};

    struct Fixture {
        pressed: bool,
    }

    impl Render for Fixture {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(div().id("label").role(Role::Label).aria_value("Volume").child("Volume"))
                .child(
                    div()
                        .id("toggle")
                        .role(Role::Button)
                        .aria_label("Mute")
                        .aria_toggled(gpui_pre::Toggled::from(self.pressed))
                        .when(self.pressed, |el| el.aria_description("On")),
                )
        }
    }

    pub fn session_captures_the_tree_gpui_delivers_to_the_platform_adapter() {
        let mut session =
            AccessibilitySession::new(Fixture { pressed: false }, size(px(200.), px(100.)), |_| {})
                .expect("capture session");
        assert!(session.update_count() >= 1, "GPUI delivered no tree update");
        let tree = session.tree().expect("tree");
        assert_eq!(
            tree.as_text(),
            "Window\n  Label value=\"Volume\"\n  Button name=\"Mute\" toggled=False"
        );
        let button = tree.top_level().nth(1).expect("button node");
        assert_eq!(button.aria_role(), "button");
        assert_eq!(
            button.aria_properties().get("aria-pressed"),
            Some(&AccessibilityValue::Bool(false))
        );

        // GPUI's own debug dump agrees now that the adapter is active.
        let debug = session
            .update(|_, window, _| AccessibilitySnapshot::capture(window))
            .unwrap()
            .expect("debug snapshot from an active window");
        assert!(debug.as_text().contains("Button name=\"Mute\""), "{}", debug.as_text());

        session.update(|root, _, cx| root.update(cx, |fixture, _| fixture.pressed = true)).unwrap();
        assert_eq!(
            session.tree().unwrap().as_text(),
            "Window\n  Label value=\"Volume\"\n  Button name=\"Mute\" description=\"On\" toggled=True"
        );

        // Deactivation stops delivery; the retained tree is the last one sent.
        let delivered = session.update_count();
        session.deactivate().unwrap();
        session.update(|_, _, _| ()).unwrap();
        assert_eq!(session.update_count(), delivered);
    }
}

fn main() {
    #[cfg(target_os = "macos")]
    {
        macos::session_captures_the_tree_gpui_delivers_to_the_platform_adapter();
        println!("a11y_capture: 1 passed");
    }
    #[cfg(not(target_os = "macos"))]
    println!("a11y_capture: skipped; AccessibilitySession requires macOS");
}
