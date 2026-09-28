//! Screenshot checks for benchmark 01 (counter). See spec.md.

mod support;

use support::shots::{self, Outcome};

fn ac5_count_change_is_visible() -> Result<Outcome, String> {
    let fixture = support::fresh_fixture();
    let mut session = session_or_skip!(&fixture);
    let initial = shots::capture(&mut session, "01-initial")?;
    shots::require_content(&initial, "initial frame")?;
    shots::press(&mut session, "up up up")?;
    let after = shots::capture(&mut session, "01-count-3")?;
    shots::require_change(&initial, &after, "count 0 -> 3")?;
    Ok(Outcome::Passed)
}

fn main() {
    shots::run(&[("ac5_count_change_is_visible", ac5_count_change_is_visible)]);
}
