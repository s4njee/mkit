//! Screenshot checks for benchmark 07 (image viewer). See spec.md.

mod support;

use support::shots::{self, Outcome};

fn ac7_zoom_is_visible() -> Result<Outcome, String> {
    let fixture = support::fresh_fixture();
    let mut session = session_or_skip!(&fixture);
    let fit = shots::capture(&mut session, "07-fit")?;
    shots::require_content(&fit, "fit frame")?;
    shots::press(&mut session, "secondary-=")?;
    let zoomed = shots::capture(&mut session, "07-zoomed")?;
    shots::require_change(&fit, &zoomed, "zoom in")?;
    Ok(Outcome::Passed)
}

fn main() {
    shots::run(&[("ac7_zoom_is_visible", ac7_zoom_is_visible)]);
}
