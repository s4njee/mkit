//! Screenshot checks for benchmark 04 (file browser). See spec.md.

mod support;

use support::shots::{self, Outcome};

fn ac7_navigation_is_visible() -> Result<Outcome, String> {
    let fixture = support::fresh_fixture();
    let mut session = session_or_skip!(&fixture);
    let root = shots::capture(&mut session, "04-root")?;
    shots::require_content(&root, "root listing")?;
    shots::press(&mut session, "enter")?;
    let docs = shots::capture(&mut session, "04-docs")?;
    shots::require_change(&root, &docs, "enter docs")?;
    Ok(Outcome::Passed)
}

fn main() {
    shots::run(&[("ac7_navigation_is_visible", ac7_navigation_is_visible)]);
}
