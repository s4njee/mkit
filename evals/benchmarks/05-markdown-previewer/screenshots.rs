//! Screenshot checks for benchmark 05 (markdown previewer). See spec.md.

mod support;

use support::shots::{self, Outcome};

fn ac6_preview_and_source_differ() -> Result<Outcome, String> {
    let fixture = support::fresh_fixture();
    let mut session = session_or_skip!(&fixture);
    let preview = shots::capture(&mut session, "05-preview")?;
    shots::require_content(&preview, "preview frame")?;
    shots::press(&mut session, "secondary-e")?;
    let source = shots::capture(&mut session, "05-source")?;
    shots::require_change(&preview, &source, "preview -> source")?;
    Ok(Outcome::Passed)
}

fn main() {
    shots::run(&[("ac6_preview_and_source_differ", ac6_preview_and_source_differ)]);
}
