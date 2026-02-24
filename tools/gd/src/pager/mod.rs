mod content;
mod keymap;
mod navigation;
mod reducer;
mod rendering;
mod runtime;
mod search;
mod state;
mod text;
pub mod tree;
mod types;

#[cfg(test)]
mod tests;

// Public API for main.rs
pub use runtime::run_pager;
pub use state::DiffContext;
