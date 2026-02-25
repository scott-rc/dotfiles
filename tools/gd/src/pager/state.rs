use crate::git::diff::DiffFile;
use crate::render::{LineInfo, RenderOutput};

use super::content::{next_content_line, snap_to_content};
use super::tree::{
    TreeEntry, build_tree_entries, build_tree_lines, compute_tree_width, file_idx_to_entry_idx,
    resolve_tree_layout,
};
use super::types::{FileIx, KeyResult, Mode, TreeEntryIx, ViewScope};

/// Context passed into the reducer (content height, total rows, terminal cols, files for tree/editor).
#[derive(Debug)]
pub(crate) struct ReducerCtx<'a> {
    pub content_height: usize,
    #[allow(dead_code)]
    pub rows: u16,
    pub cols: u16,
    pub files: &'a [DiffFile],
    pub repo: &'a std::path::Path,
}

/// Effect emitted by the reducer. Mirrors KeyResult.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ReducerEffect {
    Continue,
    ReRender,
    ReGenerate,
    Quit,
    OpenEditor { path: String, lineno: Option<u32> },
}

impl From<ReducerEffect> for KeyResult {
    fn from(e: ReducerEffect) -> Self {
        match e {
            ReducerEffect::Continue => KeyResult::Continue,
            ReducerEffect::ReRender => KeyResult::ReRender,
            ReducerEffect::ReGenerate => KeyResult::ReGenerate,
            ReducerEffect::Quit => KeyResult::Quit,
            ReducerEffect::OpenEditor { path, lineno } => KeyResult::OpenEditor { path, lineno },
        }
    }
}

pub struct DiffContext {
    pub repo: std::path::PathBuf,
    pub source: crate::git::DiffSource,
    pub no_untracked: bool,
    pub ignore_whitespace: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Document {
    pub(crate) lines: Vec<String>,
    pub(crate) line_map: Vec<LineInfo>,
    pub(crate) file_starts: Vec<usize>,
    #[allow(dead_code)]
    pub(crate) hunk_starts: Vec<usize>,
}

impl Document {
    pub(crate) fn from_render_output(output: RenderOutput) -> Self {
        Document {
            lines: output.lines,
            line_map: output.line_map,
            file_starts: output.file_starts,
            hunk_starts: output.hunk_starts,
        }
    }

    pub(crate) fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn file_count(&self) -> usize {
        self.file_starts.len()
    }

    pub(crate) fn file_start(&self, idx: usize) -> Option<usize> {
        self.file_starts.get(idx).copied()
    }

    pub(crate) fn file_end(&self, idx: usize) -> usize {
        self.file_starts
            .get(idx + 1)
            .copied()
            .unwrap_or(self.lines.len())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.line_map.is_empty()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ViewAnchor {
    pub(crate) file_idx: usize,
    pub(crate) new_lineno: Option<u32>,
    pub(crate) offset_in_file: usize,
}

#[derive(Debug)]
pub(crate) struct PagerState {
    pub(crate) doc: Document,
    pub(crate) top_line: usize,
    pub(crate) cursor_line: usize,
    pub(crate) visual_anchor: Option<usize>,
    pub(crate) search_query: String,
    pub(crate) search_matches: Vec<usize>,
    pub(crate) current_match: isize,
    pub(crate) mode: Mode,
    pub(crate) search_input: String,
    pub(crate) search_cursor: usize,
    pub(crate) status_message: String,
    pub(crate) tooltip_visible: bool,
    pub(crate) tree_visible: bool,
    pub(crate) tree_user_hidden: bool,
    pub(crate) tree_selection: Option<TreeEntryIx>,
    pub(crate) tree_width: usize,
    pub(crate) tree_scroll: usize,
    pub(crate) tree_lines: Vec<String>,
    pub(crate) tree_entries: Vec<TreeEntry>,
    pub(crate) tree_visible_to_entry: Vec<usize>,
    pub(crate) view_scope: ViewScope,
    pub(crate) full_context: bool,
}

impl PagerState {
    #[allow(dead_code)]
    pub(crate) fn file_count(&self) -> usize {
        self.doc.file_count()
    }

    #[allow(dead_code)]
    pub(crate) fn line_count(&self) -> usize {
        self.doc.line_count()
    }

    pub(crate) fn tree_entry_count(&self) -> usize {
        self.tree_entries.len()
    }

    pub(crate) fn file_start(&self, idx: usize) -> Option<usize> {
        self.doc.file_start(idx)
    }

    pub(crate) fn file_end(&self, idx: usize) -> usize {
        self.doc.file_end(idx)
    }

    pub(crate) fn tree_entry(&self, idx: usize) -> Option<&TreeEntry> {
        self.tree_entries.get(idx)
    }

    #[allow(dead_code)]
    pub(crate) fn tree_entry_mut(&mut self, idx: usize) -> Option<&mut TreeEntry> {
        self.tree_entries.get_mut(idx)
    }

    #[allow(dead_code)]
    pub(crate) fn active_file_start(&self) -> usize {
        match self.view_scope {
            ViewScope::AllFiles => 0,
            ViewScope::SingleFile(ix) => self.file_start(ix.get()).unwrap_or(0),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn active_file_end(&self) -> usize {
        match self.view_scope {
            ViewScope::AllFiles => self.doc.line_count(),
            ViewScope::SingleFile(ix) => self.file_end(ix.get()),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn cursor_line_info(&self) -> Option<&LineInfo> {
        self.doc.line_map.get(self.cursor_line)
    }

    pub(crate) fn active_file(&self) -> Option<usize> {
        match self.view_scope {
            ViewScope::AllFiles => None,
            ViewScope::SingleFile(ix) => Some(ix.get()),
        }
    }

    pub(crate) fn set_active_file(&mut self, v: Option<usize>) {
        self.view_scope = match v {
            None => ViewScope::AllFiles,
            Some(idx) => FileIx::new(idx, self.doc.file_count())
                .map_or(ViewScope::AllFiles, ViewScope::SingleFile),
        };
    }

    pub(crate) fn tree_cursor(&self) -> usize {
        self.tree_selection
            .map_or(0, super::types::TreeEntryIx::get)
    }

    pub(crate) fn set_tree_cursor(&mut self, idx: usize) {
        self.tree_selection = TreeEntryIx::new(idx, self.tree_entry_count());
    }

    pub(crate) fn rebuild_tree_lines(&mut self) {
        let (tl, tv) = build_tree_lines(&self.tree_entries, self.tree_cursor(), self.tree_width);
        self.tree_lines = tl;
        self.tree_visible_to_entry = tv;
    }

    pub(crate) fn new(
        lines: Vec<String>,
        line_map: Vec<LineInfo>,
        file_starts: Vec<usize>,
        hunk_starts: Vec<usize>,
        tree_entries: Vec<TreeEntry>,
        terminal_cols: usize,
    ) -> Self {
        let doc = Document {
            lines,
            line_map,
            file_starts,
            hunk_starts,
        };
        Self::from_doc(doc, tree_entries, terminal_cols)
    }

    pub(crate) fn from_doc(
        doc: Document,
        tree_entries: Vec<TreeEntry>,
        terminal_cols: usize,
    ) -> Self {
        let entry_count = tree_entries.len();
        let content_width = compute_tree_width(&tree_entries);
        let has_directories = tree_entries.iter().any(|e| e.file_idx.is_none());
        let file_count = doc.file_count();
        let layout = resolve_tree_layout(content_width, terminal_cols, has_directories, file_count);
        let tree_visible = layout.is_some();
        let tree_width = layout.unwrap_or(0);
        let tree_selection = TreeEntryIx::new(0, entry_count);

        let (tree_lines, tree_visible_to_entry) = if tree_visible {
            build_tree_lines(&tree_entries, 0, tree_width)
        } else {
            (Vec::new(), Vec::new())
        };

        PagerState {
            doc,
            top_line: 0,
            cursor_line: 0,
            visual_anchor: None,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match: -1,
            mode: Mode::Normal,
            search_input: String::new(),
            search_cursor: 0,
            status_message: String::new(),
            tooltip_visible: false,
            tree_visible,
            tree_user_hidden: false,
            tree_selection,
            tree_width,
            tree_scroll: 0,
            tree_lines,
            tree_entries,
            tree_visible_to_entry,
            view_scope: ViewScope::AllFiles,
            full_context: false,
        }
    }
}

#[cfg(debug_assertions)]
pub(crate) fn debug_assert_valid_state(state: &PagerState) {
    let (rs, re) = visible_range(state);
    let max_cursor = re.saturating_sub(1);
    assert!(
        state.cursor_line >= rs && state.cursor_line <= max_cursor,
        "cursor_line {} out of visible range [{}, {}]",
        state.cursor_line,
        rs,
        max_cursor
    );
    assert!(
        state.top_line >= rs
            && state.top_line <= re.saturating_sub(1),
        "top_line {} out of range",
        state.top_line
    );
    if let ViewScope::SingleFile(ix) = state.view_scope {
        assert!(
            ix.get() < state.file_count(),
            "SingleFile index {} out of bounds",
            ix.get()
        );
    }
}

#[cfg(not(debug_assertions))]
pub(crate) fn debug_assert_valid_state(_state: &PagerState) {}

pub(crate) fn clamp_cursor_and_top(state: &mut PagerState) {
    let (rs, re) = visible_range(state);
    let range_max = re.saturating_sub(1);
    state.cursor_line = state.cursor_line.clamp(rs, range_max);
    state.top_line = state.top_line.clamp(rs, range_max);
}

pub(crate) fn visible_range(state: &PagerState) -> (usize, usize) {
    match state.active_file() {
        Some(idx) => {
            let start = state.file_start(idx).unwrap_or(0);
            let end = state.file_end(idx);
            (start, end)
        }
        None => (0, state.doc.line_count()),
    }
}

pub(crate) fn capture_view_anchor(state: &PagerState) -> Option<ViewAnchor> {
    if state.doc.is_empty() {
        return None;
    }
    let top = state
        .top_line
        .min(state.doc.line_map.len().saturating_sub(1));
    let info = &state.doc.line_map[top];
    let file_start = state.file_start(info.file_idx).unwrap_or(0);
    let offset_in_file = top - file_start;
    Some(ViewAnchor {
        file_idx: info.file_idx,
        new_lineno: info.new_lineno,
        offset_in_file,
    })
}

pub(crate) fn remap_after_document_swap(
    state: &mut PagerState,
    anchor: Option<ViewAnchor>,
    new_doc: Document,
    files: &[DiffFile],
    terminal_cols: usize,
) {
    state.doc = new_doc;

    let file_count = state.doc.file_count();
    state.view_scope = match state.view_scope {
        ViewScope::AllFiles => ViewScope::AllFiles,
        ViewScope::SingleFile(ix) => {
            if ix.get() < file_count {
                ViewScope::SingleFile(ix)
            } else {
                ViewScope::AllFiles
            }
        }
    };

    let entry_count = state.tree_entry_count();
    if entry_count > 0 {
        let current = state.tree_cursor();
        let clamped = current.min(entry_count.saturating_sub(1));
        state.set_tree_cursor(clamped);
    } else {
        state.tree_selection = None;
        state.tree_visible_to_entry.clear();
        state.tree_lines.clear();
    }

    let line_count = state.doc.line_count();
    let (rs, re) = visible_range(state);
    let range_max = re.saturating_sub(1);

    if state.doc.is_empty() {
        state.top_line = 0;
        state.cursor_line = 0;
    } else if let Some(a) = anchor {
        let target_line = if a.file_idx >= file_count {
            next_content_line(&state.doc.line_map, 0, line_count.saturating_sub(1))
        } else if let Some(lineno) = a.new_lineno {
            state
                .doc
                .line_map
                .iter()
                .position(|li| li.file_idx == a.file_idx && li.new_lineno == Some(lineno))
                .unwrap_or_else(|| {
                    let file_start = state.doc.file_start(a.file_idx).unwrap_or(0);
                    let file_end = state.doc.file_end(a.file_idx).saturating_sub(1);
                    (file_start + a.offset_in_file)
                        .min(file_end)
                        .min(line_count.saturating_sub(1))
                })
        } else {
            let file_start = state.doc.file_start(a.file_idx).unwrap_or(0);
            let file_end = state.doc.file_end(a.file_idx).saturating_sub(1);
            (file_start + a.offset_in_file)
                .min(file_end)
                .min(line_count.saturating_sub(1))
        };
        state.top_line = target_line.min(range_max);
        state.cursor_line = state.top_line.min(range_max);
        state.top_line = state.top_line.clamp(rs, range_max);
        state.cursor_line = state.cursor_line.clamp(rs, range_max);
        // In single file mode, keep cursor at the file header (range start)
        // so ] can find the first change group via jump_next (strictly >).
        // In all-files mode, snap to the nearest content line as usual.
        if state.cursor_line != rs || state.active_file().is_none() {
            state.cursor_line =
                snap_to_content(&state.doc.line_map, state.cursor_line, rs, range_max);
        }
    } else {
        state.top_line = 0;
        state.cursor_line = 0;
    }

    if state.tree_visible && !files.is_empty() {
        state.tree_entries = build_tree_entries(files);
        let content_width = compute_tree_width(&state.tree_entries);
        let has_directories = state.tree_entries.iter().any(|e| e.file_idx.is_none());
        let file_count = state.doc.file_count();
        // Account for scrollbar column when full_context is active
        let effective_cols = if state.full_context {
            terminal_cols.saturating_sub(1)
        } else {
            terminal_cols
        };
        if let Some(w) = resolve_tree_layout(content_width, effective_cols, has_directories, file_count) {
            state.tree_width = w;
            let cursor_file_idx = state
                .doc
                .line_map
                .get(state.cursor_line)
                .map_or(0, |li| li.file_idx);
            let cursor_entry_idx = file_idx_to_entry_idx(&state.tree_entries, cursor_file_idx);
            state.set_tree_cursor(cursor_entry_idx);
            let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
            state.tree_lines = tl;
            state.tree_visible_to_entry = tv;
        } else {
            state.tree_visible = false;
            state.tree_lines.clear();
            state.tree_visible_to_entry.clear();
        }
    }

    if !state.search_query.is_empty() {
        state.search_matches = tui::search::find_matches(&state.doc.lines, &state.search_query);
        state.current_match =
            tui::search::find_nearest_match(&state.search_matches, state.top_line);
    }
}
