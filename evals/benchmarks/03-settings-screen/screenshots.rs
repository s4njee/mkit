//! Screenshot checks for benchmark 03 (settings screen). See spec.md.

mod support;

use support::shots::{self, Outcome};

fn ac7_dark_theme_is_darker() -> Result<Outcome, String> {
    let fixture = support::fresh_fixture();
    let mut session = session_or_skip!(&fixture);
    let light = shots::capture(&mut session, "03-light")?;
    shots::require_content(&light, "light frame")?;
    shots::press(&mut session, "down tab tab space")?;
    let dark = shots::capture(&mut session, "03-dark")?;
    let (l, d) = (shots::mean_luminance(&light), shots::mean_luminance(&dark));
    if l - d < 0.15 {
        return Err(format!("dark theme luminance {d:.3} is not 0.15 below light {l:.3}"));
    }
    Ok(Outcome::Passed)
}

fn main() {
    shots::run(&[("ac7_dark_theme_is_darker", ac7_dark_theme_is_darker)]);
}
