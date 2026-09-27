//! Shared foundations for mkit components.

extern crate gpui_pre as gpui;

pub mod a11y;
pub mod civil_date;
pub use civil_date::CivilDate;
pub mod focus;
pub mod motion;
pub mod overlay;
pub mod reduced_motion;
pub mod state;
pub mod theme;

/// GPUI version this workspace is pinned to.
pub const GPUI_PRE_VERSION: &str = "0.3.5";
