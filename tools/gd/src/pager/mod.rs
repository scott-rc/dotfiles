mod content;
mod keymap;
mod navigation;
mod reducer;
mod rendering;
mod runtime;
mod search;
mod state;

pub mod tree;
mod types;

#[cfg(test)]
mod tests;

// Public API for main.rs
pub use runtime::run_pager;
pub use runtime::run_pager_replay;
pub use state::DiffContext;

/// Render one frame of the content area into a `Write` sink.
/// Builds a minimal `PagerState` from the given `RenderOutput`. For benchmarks.
#[allow(dead_code)]
pub fn bench_render_frame(
    output: crate::render::RenderOutput,
    files: &[crate::git::diff::DiffFile],
    out: &mut impl std::io::Write,
    cols: u16,
    rows: u16,
) {
    let tree_entries = tree::build_tree_entries(files);
    let doc = state::Document::from_render_output(output);
    let st = state::PagerState::from_doc(doc, tree_entries, cols as usize);
    let ch = rendering::content_height(rows, &st);
    rendering::render_content_area(out, &st, cols, ch as u16);
}

/// Style all files (Phase 1 only). For benchmarks.
#[allow(dead_code)]
pub fn bench_style_files(
    files: &[crate::git::diff::DiffFile],
    color: bool,
) -> Vec<crate::render::StyledFile> {
    crate::render::style_files(files, color)
}

/// Layout styled files at a specific width (Phase 2 only). For benchmarks.
#[allow(dead_code)]
pub fn bench_layout(
    styled_files: &[crate::render::StyledFile],
    width: usize,
    color: bool,
) -> crate::render::LayoutOutput {
    crate::render::layout(styled_files, width, color)
}
