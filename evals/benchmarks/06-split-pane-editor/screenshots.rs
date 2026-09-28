//! Screenshot checks for benchmark 06 (split-pane editor). See spec.md.

mod support;

use support::shots::{self, Outcome};

fn ac7_editing_is_visible() -> Result<Outcome, String> {
    let fixture = support::fresh_fixture();
    let mut session = session_or_skip!(&fixture);
    let initial = shots::capture(&mut session, "06-initial")?;
    shots::require_content(&initial, "initial frame")?;
    shots::press(&mut session, "enter")?;
    let opened = shots::capture(&mut session, "06-opened")?;
    shots::type_text(&mut session, "hello")?;
    let edited = shots::capture(&mut session, "06-edited")?;
    shots::require_change(&opened, &edited, "typing hello")?;
    Ok(Outcome::Passed)
}

fn main() {
    shots::run(&[("ac7_editing_is_visible", ac7_editing_is_visible)]);
}
