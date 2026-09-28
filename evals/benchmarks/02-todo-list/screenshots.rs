//! Screenshot checks for benchmark 02 (todo list). See spec.md.

mod support;

use support::shots::{self, Outcome};

fn ac7_toggle_is_visible() -> Result<Outcome, String> {
    let fixture = support::fresh_fixture();
    let mut session = session_or_skip!(&fixture);
    let initial = shots::capture(&mut session, "02-initial")?;
    shots::require_content(&initial, "initial frame")?;
    shots::press(&mut session, "tab")?;
    let list_focused = shots::capture(&mut session, "02-list-focused")?;
    shots::press(&mut session, "space")?;
    let toggled = shots::capture(&mut session, "02-first-done")?;
    shots::require_change(&list_focused, &toggled, "toggle first item done")?;
    Ok(Outcome::Passed)
}

fn main() {
    shots::run(&[("ac7_toggle_is_visible", ac7_toggle_is_visible)]);
}
