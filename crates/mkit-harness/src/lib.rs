//! Headless rendering and interaction support for mkit examples and components.

#[cfg(target_os = "macos")]
pub mod a11y_capture;
pub mod accessibility;
pub mod input;
pub mod screenshot;

#[cfg(target_os = "macos")]
pub use a11y_capture::AccessibilitySession;
pub use accessibility::{
    AccessibilityError, AccessibilityNode, AccessibilitySnapshot, AccessibilityTree,
    AccessibilityValue,
};
pub use input::{InputScript, ScriptAction, ScriptError, parse_script};
pub use screenshot::{HeadlessSession, PixelTolerance, ScreenshotError, screenshot};
