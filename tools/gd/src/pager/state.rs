#![allow(dead_code)]

use crate::git::diff::DiffFile;
use crate::render::{LineInfo, RenderOutput};

use super::content::{next_content_line, snap_to_content};
use super::tree::{build_tree_lines, build_tree_entries, compute_tree_width, file_idx_to_entry_idx, TreeEntry};
use super::types::{FileIx, Focus, KeyResult, Mode, TreeEntryIx, ViewScope};

/// Context passed into the reducer (content height, total rows, files for tree/editor).
#[derive(Debug)]
pub(crate) struct ReducerCtx<'a> {
    pub content_height: usize,
    pub rows: u16,
    pub files: &'a [DiffFile],
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
}

/// Immutable document encapsulating all rendered diff content. Swapped atomically
/// on regenerate/rerender/resize; view state (cursor, top, scope) is remapped via remap_after_document_swap.
#[derive(Debug, Clone)]
pub(crate) struct Document {
    pub(crate) lines: Vec<String>,
    pub(crate) line_map: Vec<LineInfo>,
    pub(crate) file_starts: Vec<usize>,
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

/// Anchor for preserving view position across document swaps. Captured before swap,
/// used to remap cursor/top to nearest valid content in the new document.
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
    pub(crate) visual_anchor: usize,
    pub(crate) search_query: String,
    pub(crate) search_matches: Vec<usize>,
    pub(crate) current_match: isize,
    pub(crate) mode: Mode,
    pub(crate) search_input: String,
    pub(crate) search_cursor: usize,
    pub(crate) status_message: String,
    pub(crate) tree_visible: bool,
    /// Typed focus: Diff = diff panel focused, Tree = tree panel focused.
    /// Invariant: when tree has no entries, focus must be Diff.
    pub(crate) focus: Focus,
    /// Valid tree selection when tree has entries. None when tree empty.
    pub(crate) tree_selection: Option<TreeEntryIx>,
    pub(crate) tree_width: usize,
    pub(crate) tree_scroll: usize,
    pub(crate) tree_lines: Vec<String>,
    pub(crate) tree_entries: Vec<TreeEntry>,
    /// Maps visible tree line index to original `tree_entries` index
    pub(crate) tree_visible_to_entry: Vec<usize>,
    /// Typed view scope. Invariant: SingleFile(ix) implies ix is valid for file_starts.
    pub(crate) view_scope: ViewScope,
    pub(crate) full_context: bool,
}

impl PagerState {
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

    /// Checked accessor for file start line. Returns None if idx is out of bounds.
    pub(crate) fn file_start(&self, idx: usize) -> Option<usize> {
        self.doc.file_start(idx)
    }

    /// Checked accessor for file end line (exclusive). Returns line_count for last file.
    pub(crate) fn file_end(&self, idx: usize) -> usize {
        self.doc.file_end(idx)
    }

    /// Checked accessor for tree entry. Returns None if idx is out of bounds.
    pub(crate) fn tree_entry(&self, idx: usize) -> Option<&TreeEntry> {
        self.tree_entries.get(idx)
    }

    /// Checked mutable accessor for tree entry.
    pub(crate) fn tree_entry_mut(&mut self, idx: usize) -> Option<&mut TreeEntry> {
        self.tree_entries.get_mut(idx)
    }

    /// Start line of the active file, or 0 if all-files view.
    #[allow(dead_code)]
    pub(crate) fn active_file_start(&self) -> usize {
        match self.view_scope {
            ViewScope::AllFiles => 0,
            ViewScope::SingleFile(ix) => self.file_start(ix.get()).unwrap_or(0),
        }
    }

    /// End line (exclusive) of the active file, or line_count if all-files view.
    #[allow(dead_code)]
    pub(crate) fn active_file_end(&self) -> usize {
        match self.view_scope {
            ViewScope::AllFiles => self.doc.line_count(),
            ViewScope::SingleFile(ix) => self.file_end(ix.get()),
        }
    }

    /// Visible tree entry at cursor, if tree has entries and cursor is valid.
    pub(crate) fn visible_tree_entry(&self) -> Option<&TreeEntry> {
        self.tree_selection.and_then(|ix| self.tree_entry(ix.get()))
    }

    /// LineInfo at cursor, if in range.
    #[allow(dead_code)]
    pub(crate) fn cursor_line_info(&self) -> Option<&LineInfo> {
        self.doc.line_map.get(self.cursor_line)
    }

    // Compatibility adapters (for migration in later chunks)
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
                .map(ViewScope::SingleFile)
                .unwrap_or(ViewScope::AllFiles),
        };
    }

    pub(crate) fn tree_focused(&self) -> bool {
        matches!(self.focus, Focus::Tree)
    }

    pub(crate) fn set_tree_focused(&mut self, focused: bool) {
        self.focus = if focused { Focus::Tree } else { Focus::Diff };
        // Invariant: tree focus invalid when tree empty
        if focused && self.tree_entries.is_empty() {
            self.focus = Focus::Diff;
        }
    }

    pub(crate) fn tree_cursor(&self) -> usize {
        self.tree_selection.map(|s| s.get()).unwrap_or(0)
    }

    pub(crate) fn set_tree_cursor(&mut self, idx: usize) {
        self.tree_selection = TreeEntryIx::new(idx, self.tree_entry_count());
        if self.tree_selection.is_none() && self.focus == Focus::Tree {
            self.focus = Focus::Diff;
        }
    }

    /// Build a valid initial state. Tree starts visible, focused; scope all-files.
    /// Tree panel falls back to hidden when no entries.
    pub(crate) fn new(
        lines: Vec<String>,
        line_map: Vec<LineInfo>,
        file_starts: Vec<usize>,
        hunk_starts: Vec<usize>,
        tree_entries: Vec<TreeEntry>,
    ) -> Self {
        let doc = Document {
            lines,
            line_map,
            file_starts,
            hunk_starts,
        };
        Self::from_doc(doc, tree_entries)
    }

    /// Build state from a Document and tree entries. Used by new() and after document swaps.
    pub(crate) fn from_doc(doc: Document, tree_entries: Vec<TreeEntry>) -> Self {
        let entry_count = tree_entries.len();
        let tree_width = compute_tree_width(&tree_entries);

        let (tree_selection, focus, tree_visible) = if entry_count > 0 {
            let sel = TreeEntryIx::new(0, entry_count).unwrap();
            let (_tl, _tv) = build_tree_lines(&tree_entries, 0, tree_width);
            (Some(sel), Focus::Tree, true)
        } else {
            (None, Focus::Diff, false)
        };

        let (tree_lines, tree_visible_to_entry) = if tree_visible {
            let (tl, tv) = build_tree_lines(&tree_entries, 0, tree_width);
            (tl, tv)
        } else {
            (Vec::new(), Vec::new())
        };

        PagerState {
            doc,
            top_line: 0,
            cursor_line: 0,
            visual_anchor: 0,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match: -1,
            mode: Mode::Normal,
            search_input: String::new(),
            search_cursor: 0,
            status_message: String::new(),
            tree_visible,
            focus,
            tree_selection,
            tree_width: if tree_visible { tree_width } else { 0 },
            tree_scroll: 0,
            tree_lines,
            tree_entries,
            tree_visible_to_entry,
            view_scope: ViewScope::AllFiles,
            full_context: false,
        }
    }
}

/// Invariant guard: cursor/top in range, tree focus valid, single-file scope valid.
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
        state.top_line
            >= rs && state.top_line <= re.saturating_sub(1).min(state.doc.line_count().saturating_sub(1)),
        "top_line {} out of range",
        state.top_line
    );
    if state.tree_focused() {
        assert!(!state.tree_entries.is_empty(), "tree focus invalid when tree empty");
        assert!(
            state.tree_selection.is_some() && state.tree_selection.unwrap().get() < state.tree_entry_count(),
            "tree_selection invalid"
        );
    }
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

/// Clamp cursor and top_line to visible range. Call before debug_assert_valid_state.
pub(crate) fn clamp_cursor_and_top(state: &mut PagerState) {
    let (rs, re) = visible_range(state);
    let range_max = re.saturating_sub(1);
    let max_top = range_max.saturating_sub(1).min(state.doc.line_count().saturating_sub(1));
    state.cursor_line = state.cursor_line.clamp(rs, range_max);
    state.top_line = state.top_line.clamp(rs, max_top.min(range_max));
}

/// Return the `(start, end)` line range for the active file, or the full
/// document range when no file is selected.
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

/// Capture view anchor from current state for remap after document swap.
/// Returns None when document is empty.
pub(crate) fn capture_view_anchor(state: &PagerState) -> Option<ViewAnchor> {
    if state.doc.is_empty() {
        return None;
    }
    let top = state.top_line.min(state.doc.line_map.len().saturating_sub(1));
    let info = &state.doc.line_map[top];
    let file_start = state.file_start(info.file_idx).unwrap_or(0);
    let offset_in_file = top - file_start;
    Some(ViewAnchor {
        file_idx: info.file_idx,
        new_lineno: info.new_lineno,
        offset_in_file,
    })
}

/// Remap cursor/top/scope/tree/overlay after swapping in a new document.
/// Handles file removal and collapsed ranges by falling back to nearest valid content.
pub(crate) fn remap_after_document_swap(
    state: &mut PagerState,
    anchor: Option<ViewAnchor>,
    new_doc: Document,
    files: &[DiffFile],
) {
    state.doc = new_doc;

    // 1. Normalize scope: downgrade single-file to all-files if target file gone
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

    // 2. Normalize tree selection when tree has entries
    let entry_count = state.tree_entry_count();
    if entry_count > 0 {
        let current = state.tree_cursor();
        let clamped = current.min(entry_count.saturating_sub(1));
        state.set_tree_cursor(clamped);
    } else {
        state.tree_selection = None;
        state.focus = Focus::Diff;
        state.tree_visible_to_entry.clear();
        state.tree_lines.clear();
    }

    // 3. Remap cursor and top from anchor
    let line_count = state.doc.line_count();
    let (rs, re) = visible_range(state);
    let range_max = re.saturating_sub(1);

    if state.doc.is_empty() {
        state.top_line = 0;
        state.cursor_line = 0;
        state.visual_anchor = 0;
    } else if let Some(a) = anchor {
        // Resolve anchor to line index; fall back to nearest valid on file removal/collapsed
        let target_line = if a.file_idx >= file_count {
            // Target file no longer exists: use first content line of doc
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
        state.cursor_line = snap_to_content(&state.doc.line_map, state.cursor_line, rs, range_max);
        state.visual_anchor = state.cursor_line;
    } else {
        state.top_line = 0;
        state.cursor_line = 0;
        state.visual_anchor = 0;
    }

    // 4. Rebuild tree entries and sync tree selection from cursor
    if state.tree_visible && !files.is_empty() {
        state.tree_entries = build_tree_entries(files);
        state.tree_width = compute_tree_width(&state.tree_entries);
        let cursor_file_idx = state
            .doc
            .line_map
            .get(state.cursor_line)
            .map(|li| li.file_idx)
            .unwrap_or(0);
        let cursor_entry_idx = file_idx_to_entry_idx(&state.tree_entries, cursor_file_idx);
        state.set_tree_cursor(cursor_entry_idx);
        let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
        state.tree_lines = tl;
        state.tree_visible_to_entry = tv;
    }

    // 5. Re-run search against new lines
    if !state.search_query.is_empty() {
        state.search_matches = tui::search::find_matches(&state.doc.lines, &state.search_query);
        state.current_match = tui::search::find_nearest_match(&state.search_matches, state.top_line);
    }
}
