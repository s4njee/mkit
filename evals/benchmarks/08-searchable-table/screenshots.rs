//! Screenshot checks for benchmark 08 (searchable table). See spec.md.

mod support;

use support::shots::{self, Outcome};

fn ac7_filter_is_visible() -> Result<Outcome, String> {
    let fixture = support::fresh_fixture();
    let mut session = session_or_skip!(&fixture);
    let all = shots::capture(&mut session, "08-all")?;
    shots::require_content(&all, "unfiltered frame")?;
    shots::type_text(&mut session, "osl")?;
    let filtered = shots::capture(&mut session, "08-osl")?;
    shots::require_change(&all, &filtered, "filter osl")?;
    Ok(Outcome::Passed)
}

fn main() {
    shots::run(&[("ac7_filter_is_visible", ac7_filter_is_visible)]);
}
