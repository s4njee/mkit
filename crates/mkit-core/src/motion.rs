//! Shared transition durations for mkit components.
//!
//! Pass [`App::reduce_motion`](gpui_pre::App::reduce_motion) to
//! [`transition_duration`] and use the result with GPUI's
//! [`AnimationExt::with_animation`](gpui_pre::AnimationExt::with_animation).
//! GPUI handles the static end state when its reduce-motion flag is enabled.
//! On macOS, [`crate::reduced_motion::watch_system_reduced_motion`] can keep
//! that flag synchronized with the system accessibility preference.

use std::time::Duration;

use gpui_pre::{Animation, App};

use crate::theme::Theme;

/// Common short interaction transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionKind {
    /// Pointer entering or leaving an interactive component.
    Hover,
    /// Pointer or keyboard press feedback.
    Press,
    /// A component becoming visible or expanded.
    Open,
    /// A component becoming hidden or collapsed.
    Close,
}

/// Resolve a transition duration from theme tokens and the app's motion flag.
///
/// Hover and press use `Theme.motion.fast_ms`; open and close use
/// `Theme.motion.normal_ms`. The returned duration is zero when
/// `reduce_motion` is true or the selected theme token is zero. Passing
/// `app.reduce_motion()` applies GPUI's configured reduced-motion preference.
///
/// This helper only selects a duration. It does not add an animation to an
/// element or schedule frames; callers compose the returned duration with
/// GPUI's `Animation` and `AnimationExt` APIs.
pub fn transition_duration(theme: &Theme, kind: TransitionKind, reduce_motion: bool) -> Duration {
    if reduce_motion {
        return Duration::ZERO;
    }

    let milliseconds = match kind {
        TransitionKind::Hover | TransitionKind::Press => theme.motion.fast_ms,
        TransitionKind::Open | TransitionKind::Close => theme.motion.normal_ms,
    };
    Duration::from_millis(u64::from(milliseconds))
}

/// Create a GPUI animation with the theme duration for a common transition.
///
/// Use this with `with_animation`; GPUI's `AnimationExt` checks the app's
/// `reduce_motion` setting itself and renders the appropriate static state.
pub fn transition_animation(theme: &Theme, kind: TransitionKind) -> Animation {
    Animation::new(transition_duration(theme, kind, false))
}

/// Resolve a transition duration using the current app-level motion setting.
///
/// Use this when the caller needs to inspect or store the resolved duration.
/// When constructing an animation for `with_animation`, prefer
/// [`transition_animation`], because that GPUI wrapper independently honors
/// `App::reduce_motion` while rendering.
pub fn app_transition_duration(app: &App, theme: &Theme, kind: TransitionKind) -> Duration {
    transition_duration(theme, kind, app.reduce_motion())
}

#[cfg(test)]
mod tests {
    use super::{TransitionKind, transition_animation, transition_duration};
    use crate::theme::{DARK, HIGH_CONTRAST, LIGHT};
    use std::time::Duration;

    #[test]
    fn interaction_kinds_use_their_theme_duration_tokens() {
        assert_eq!(
            transition_duration(&LIGHT, TransitionKind::Hover, false),
            Duration::from_millis(u64::from(LIGHT.motion.fast_ms))
        );
        assert_eq!(
            transition_duration(&DARK, TransitionKind::Press, false),
            Duration::from_millis(u64::from(DARK.motion.fast_ms))
        );
        assert_eq!(
            transition_duration(&LIGHT, TransitionKind::Open, false),
            Duration::from_millis(u64::from(LIGHT.motion.normal_ms))
        );
        assert_eq!(
            transition_duration(&DARK, TransitionKind::Close, false),
            Duration::from_millis(u64::from(DARK.motion.normal_ms))
        );
    }

    #[test]
    fn reduce_motion_and_zero_theme_tokens_disable_transitions() {
        for kind in [
            TransitionKind::Hover,
            TransitionKind::Press,
            TransitionKind::Open,
            TransitionKind::Close,
        ] {
            assert_eq!(transition_duration(&LIGHT, kind, true), Duration::ZERO);
            assert_eq!(transition_duration(&HIGH_CONTRAST, kind, false), Duration::ZERO);
        }
    }

    #[test]
    fn animation_helper_uses_the_resolved_theme_duration() {
        let animation = transition_animation(&LIGHT, TransitionKind::Open);
        assert_eq!(animation.duration, Duration::from_millis(u64::from(LIGHT.motion.normal_ms)));
        assert!(animation.oneshot);
    }
}
