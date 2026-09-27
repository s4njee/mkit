use std::error::Error;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
mod cli;
mod diff;
pub mod doctor;
mod installer;
mod lock;
mod registry;
mod support;
mod update;
pub use cli::cli_main;
pub use diff::diff_components;
pub use installer::add_components;
pub use update::{ComponentUpdate, UpdateReport, update_components};
#[cfg(test)]
mod tests;
