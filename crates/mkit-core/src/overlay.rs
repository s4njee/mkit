//! Shared overlay placement and drawing order helpers.
//!
//! Placement is computed against window-coordinate rectangles. The resulting
//! rectangle can be rendered with GPUI's `anchored` element inside `deferred`.

use gpui_pre::{Anchor, IntoElement, ParentElement, anchored, deferred, point, px};

/// A rectangle in window coordinates, expressed in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverlayRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Preferred vertical side for a popover.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlaySide {
    #[default]
    Below,
    Above,
}

/// Inputs that control the shared placement policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacementOptions {
    /// Preferred side of the trigger.
    pub side: OverlaySide,
    /// Align the overlay's leading edge with the trigger's leading edge.
    pub align_start: bool,
    /// Gap between the trigger and overlay.
    pub gap: f32,
    /// Minimum distance from each window edge.
    pub margin: f32,
}

impl Default for PlacementOptions {
    fn default() -> Self {
        Self { side: OverlaySide::Below, align_start: true, gap: 4.0, margin: 8.0 }
    }
}

/// A selected overlay rectangle and the side chosen after attempting a flip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub rect: OverlayRect,
    pub side: OverlaySide,
}

/// Inputs that may request an overlay to close.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DismissalInput {
    /// A pointer press occurred outside the overlay and its trigger.
    OutsidePointerDown,
    /// The overlay's Escape action was dispatched.
    Escape,
}

/// Per-overlay dismissal behavior. Both options default to enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DismissalPolicy {
    pub outside_pointer_down: bool,
    pub escape: bool,
}

impl Default for DismissalPolicy {
    fn default() -> Self {
        Self { outside_pointer_down: true, escape: true }
    }
}

/// Return whether an input should request dismissal under this policy.
///
/// This is a policy decision only. Callers must connect it to GPUI input
/// handlers and their own open/close state. For nested overlays, send the
/// input to the topmost overlay first. GPUI exposes `on_mouse_down_out` for
/// pointer input and key/action handlers for Escape; this helper does not
/// install those handlers or consume their events.
pub fn should_dismiss(input: DismissalInput, policy: DismissalPolicy) -> bool {
    match input {
        DismissalInput::OutsidePointerDown => policy.outside_pointer_down,
        DismissalInput::Escape => policy.escape,
    }
}

/// Position an overlay, flipping vertically when that yields a better fit and
/// then shifting it into the window's margin-safe area.
///
/// If the overlay is larger than the available area, its origin is clamped to
/// the margin; callers should constrain its size separately.
pub fn place_overlay(
    trigger: OverlayRect,
    overlay_size: (f32, f32),
    window: OverlayRect,
    options: PlacementOptions,
) -> Placement {
    let (width, height) = overlay_size;
    let x_start = if options.align_start { trigger.x } else { trigger.x + trigger.width - width };
    let below_y = trigger.y + trigger.height + options.gap;
    let above_y = trigger.y - options.gap - height;
    let preferred_y = match options.side {
        OverlaySide::Below => below_y,
        OverlaySide::Above => above_y,
    };
    let flipped_y = match options.side {
        OverlaySide::Below => above_y,
        OverlaySide::Above => below_y,
    };
    let min_x = window.x + options.margin;
    let max_x = (window.x + window.width - options.margin - width).max(min_x);
    let min_y = window.y + options.margin;
    let max_y = (window.y + window.height - options.margin - height).max(min_y);
    let overflows = |y: f32| y < min_y || y > max_y;
    let side = if overflows(preferred_y) && !overflows(flipped_y) {
        match options.side {
            OverlaySide::Below => OverlaySide::Above,
            OverlaySide::Above => OverlaySide::Below,
        }
    } else {
        options.side
    };
    let y = match side {
        OverlaySide::Below => below_y,
        OverlaySide::Above => above_y,
    };
    Placement {
        rect: OverlayRect {
            x: x_start.clamp(min_x, max_x),
            y: y.clamp(min_y, max_y),
            width,
            height,
        },
        side,
    }
}

/// Drawing priorities shared by common overlay classes.
///
/// Higher GPUI deferred priorities paint over lower priorities.
pub mod layer {
    pub const TOOLTIP: usize = 10;
    pub const POPOVER: usize = 20;
    pub const MENU: usize = 30;
    pub const MODAL: usize = 40;
}

/// Wrap an already positioned element in GPUI's anchored and deferred layers.
///
/// This uses GPUI's window-edge snap as a final guard for rounding and window
/// changes after the caller computed the placement.
pub fn deferred_at(
    element: impl IntoElement,
    placement: Placement,
    margin: f32,
    priority: usize,
) -> impl IntoElement {
    deferred(
        anchored()
            .anchor(Anchor::TopLeft)
            .position(point(px(placement.rect.x), px(placement.rect.y)))
            .snap_to_window_with_margin(px(margin))
            .child(element),
    )
    .with_priority(priority)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flips_above_when_below_does_not_fit() {
        let result = place_overlay(
            OverlayRect { x: 40.0, y: 82.0, width: 20.0, height: 10.0 },
            (50.0, 30.0),
            OverlayRect { x: 0.0, y: 0.0, width: 120.0, height: 120.0 },
            PlacementOptions { margin: 4.0, gap: 2.0, ..PlacementOptions::default() },
        );
        assert_eq!(result.side, OverlaySide::Above);
        assert_eq!(result.rect.y, 50.0);
    }

    #[test]
    fn shifts_horizontally_into_safe_window_area() {
        let result = place_overlay(
            OverlayRect { x: 95.0, y: 20.0, width: 15.0, height: 10.0 },
            (40.0, 20.0),
            OverlayRect { x: 0.0, y: 0.0, width: 120.0, height: 100.0 },
            PlacementOptions { margin: 8.0, ..PlacementOptions::default() },
        );
        assert_eq!(result.rect.x, 72.0);
        assert_eq!(result.side, OverlaySide::Below);
    }

    #[test]
    fn keeps_preferred_side_when_both_sides_overflow() {
        let result = place_overlay(
            OverlayRect { x: 20.0, y: 45.0, width: 10.0, height: 8.0 },
            (30.0, 80.0),
            OverlayRect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 },
            PlacementOptions { margin: 5.0, ..PlacementOptions::default() },
        );
        assert_eq!(result.side, OverlaySide::Below);
        assert_eq!(result.rect.y, 15.0);
    }

    #[test]
    fn dismissal_policy_can_independently_enable_pointer_and_escape() {
        let policy = DismissalPolicy { outside_pointer_down: false, escape: true };
        assert!(!should_dismiss(DismissalInput::OutsidePointerDown, policy));
        assert!(should_dismiss(DismissalInput::Escape, policy));
        assert!(should_dismiss(DismissalInput::OutsidePointerDown, DismissalPolicy::default()));
    }
}
