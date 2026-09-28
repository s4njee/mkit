//! Benchmark app. Replace this placeholder with your implementation.
//!
//! The items below are the grading contract described in CONTRACT.md. Keep
//! their names and signatures; everything else is yours to design.

use gpui_pre::{App, Context, IntoElement, Render, Window, div};
use std::path::{Path, PathBuf};

/// Runs once before any window opens: key bindings, globals, and themes.
pub fn init(_cx: &mut App) {}

/// The root view. The grader opens it directly.
pub struct Root {
    #[allow(dead_code)]
    fixture: PathBuf,
}

/// Build the root view without an `App`; create entities lazily in `render`.
pub fn root(fixture: &Path) -> Root {
    Root { fixture: fixture.to_path_buf() }
}

impl Root {
    /// State readout described in the task's Snapshot section.
    pub fn snapshot(&self, _cx: &App) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Render for Root {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
