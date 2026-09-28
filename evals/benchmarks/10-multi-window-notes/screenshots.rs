//! Screenshot checks for benchmark 10 (multi-window notes). See spec.md.

mod support;

use support::shots::{self, Outcome};

fn ac7_list_is_rendered() -> Result<Outcome, String> {
    let fixture = support::fresh_fixture();
    let mut session = session_or_skip!(&fixture);
    let list = shots::capture(&mut session, "10-list")?;
    shots::require_content(&list, "main window")?;
    Ok(Outcome::Passed)
}

fn main() {
    shots::run(&[("ac7_list_is_rendered", ac7_list_is_rendered)]);
}
