//! Screenshot checks for benchmark 09 (canvas drawing). See spec.md.

mod support;

use gpui_pre::{point, px};
use support::shots::{self, Outcome};

fn ac6_stroke_is_visible() -> Result<Outcome, String> {
    let fixture = support::fresh_fixture();
    let mut session = session_or_skip!(&fixture);
    let blank = shots::capture(&mut session, "09-blank")?;
    shots::require_content(&blank, "initial frame")?;
    let err = |e: mkit_harness::ScreenshotError| e.to_string();
    session.simulate_pointer_down(point(px(300.), px(300.))).map_err(err)?;
    for step in 1..=10 {
        let t = step as f32 / 10.0;
        let p = point(px(300. + 200. * t), px(300. + 50. * t));
        session.simulate_pointer_drag_to(p).map_err(err)?;
    }
    session.simulate_pointer_up(point(px(500.), px(350.))).map_err(err)?;
    let drawn = shots::capture(&mut session, "09-stroke")?;
    shots::require_change(&blank, &drawn, "pen stroke")?;
    Ok(Outcome::Passed)
}

fn main() {
    shots::run(&[("ac6_stroke_is_visible", ac6_stroke_is_visible)]);
}
