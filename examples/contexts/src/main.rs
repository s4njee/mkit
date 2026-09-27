//! This example is intentionally nonvisual; it demonstrates app and entity contexts.
//! This binary is a compile fixture: a real app retains the model's window or owner
//! and keeps its event loop alive; the unit test deterministically drives completion.

// ANCHOR: contexts_main
use gpui_platform::application;
use gpui_pre::App;
use mkit_example_contexts::setup;

fn main() {
    application().run(|cx: &mut App| {
        let _refresh_state = setup(cx);
    });
}
// ANCHOR_END: contexts_main
