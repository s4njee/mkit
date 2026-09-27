//! Headless rendering and interaction support for mkit examples and components.

pub mod accessibility;
pub mod input;
pub mod screenshot;

pub use accessibility::{AccessibilityError, AccessibilitySnapshot};
pub use input::{InputScript, ScriptAction, ScriptError, parse_script};
pub use screenshot::{HeadlessSession, PixelTolerance, ScreenshotError, screenshot};
