//! Shared test helpers and fixtures for pager tests.

use std::sync::Arc;

use crate::git::diff::{DiffFile, DiffHunk, DiffLine, FileStatus, LineKind};
use crate::render;
use crate::render::LineInfo;
use crate::style;

use super::super::rendering::render_scrollbar_cell;
use super::super::state::{PagerState, ReducerCtx, visible_range};
use super::super::tree::{TreeEntry, build_tree_entries};
use super::super::types::ViewScope;

use std::sync::{Mutex, OnceLock};

/// Default `ReducerCtx` for tests: 40 content height, 40 rows, 120 cols, empty files, WorkingTree.
pub fn test_ctx() -> ReducerCtx<'static> {
    ReducerCtx {
        content_height: 40,
        rows: 40,
        cols: 120,
        files: &[],
        repo: std::path::Path::new("."),
        source: &crate::git::DiffSource::WorkingTree,
    }
}

pub fn gd_debug_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct GdDebugRestore(Option<std::ffi::OsString>);
#[allow(unsafe_code)]
impl Drop for GdDebugRestore {
    fn drop(&mut self) {
        // SAFETY: Tests use a global mutex guard to serialize env mutation.
        match self.0.take() {
            Some(v) => unsafe { std::env::set_var("GD_DEBUG", v) },
            None => unsafe { std::env::remove_var("GD_DEBUG") },
        }
    }
}

#[allow(unsafe_code)]
pub fn with_gd_debug_env<T>(value: Option<&str>, f: impl FnOnce() -> T) -> T {
    let _guard = gd_debug_env_lock()
        .lock()
        .expect("failed to lock GD_DEBUG test mutex");
    let _restore = GdDebugRestore(std::env::var_os("GD_DEBUG"));
    match value {
        Some(v) => unsafe { std::env::set_var("GD_DEBUG", v) },
        None => unsafe { std::env::remove_var("GD_DEBUG") },
    }
    f()
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct StateSnapshot {
    pub cursor_line: usize,
    pub top_line: usize,
    pub active_file: Option<usize>,
    pub tree_visible: bool,
    pub visual_anchor: Option<usize>,
    pub tooltip_visible: bool,
    pub mode: super::super::types::Mode,
    pub status_message: String,
    pub full_context: bool,
}

impl From<&PagerState> for StateSnapshot {
    fn from(s: &PagerState) -> Self {
        StateSnapshot {
            cursor_line: s.cursor_line,
            top_line: s.top_line,
            active_file: s.active_file(),
            tree_visible: s.tree_visible,
            visual_anchor: s.visual_anchor,
            tooltip_visible: s.tooltip_visible,
            mode: s.mode.clone(),
            status_message: s.status_message.clone(),
            full_context: s.full_context,
        }
    }
}

/// Invariant test harness: validates PagerState after transitions.
pub fn assert_state_invariants(state: &PagerState) {
    let line_count = state.doc.line_count();
    let (rs, re) = visible_range(state);
    let range_max = re.saturating_sub(1);

    assert!(
        state.cursor_line >= rs && state.cursor_line <= range_max,
        "cursor_line {} out of visible range [{}, {}]",
        state.cursor_line,
        rs,
        range_max
    );
    assert!(
        state.top_line >= rs && state.top_line <= range_max,
        "top_line {} out of range [rs={}, max={}]",
        state.top_line,
        rs,
        range_max
    );

    if let ViewScope::SingleFile(ix) = state.view_scope {
        assert!(
            ix.get() < state.file_count(),
            "SingleFile index {} out of bounds (file_count={})",
            ix.get(),
            state.file_count()
        );
    }

    for &idx in &state.search_matches {
        assert!(
            idx < line_count,
            "search_match index {idx} out of doc bounds (line_count={line_count})"
        );
    }
}

/// Build a 90-line PagerState for keybinding snapshot tests.
pub fn make_keybinding_state() -> PagerState {
    let mut line_map = Vec::with_capacity(90);
    let header_indices: &[usize] = &[0, 5, 15, 30, 35, 45, 60, 65, 75];
    for i in 0..90 {
        let (file_idx, path) = if i < 30 {
            (0, "a.rs")
        } else if i < 60 {
            (1, "b.rs")
        } else {
            (2, "c.rs")
        };
        let line_kind = if header_indices.contains(&i) {
            None
        } else {
            Some(LineKind::Context)
        };
        line_map.push(LineInfo {
            file_idx,
            path: path.into(),
            new_lineno: Some(i as u32 + 1),
            old_lineno: None,
            line_kind,
            hunk_idx: None,
        });
    }

    let tree_entries = vec![
        entry("a.rs", 0, Some(0)),
        entry("b.rs", 0, Some(1)),
        entry("c.rs", 0, Some(2)),
    ];
    let mut state = PagerState::new(
        vec!["line".into(); 90],
        line_map,
        vec![0, 30, 60],
        vec![5, 15, 35, 45, 65, 75],
        tree_entries,
        120,
    );
    state.cursor_line = 1;
    state
}

pub fn make_search_state(input: &str, cursor: usize) -> PagerState {
    let mut state = make_keybinding_state();
    state.mode = super::super::types::Mode::Search;
    state.search_input = input.to_string();
    state.search_cursor = cursor;
    state
}

pub fn entry(label: &str, depth: usize, file_idx: Option<usize>) -> TreeEntry {
    TreeEntry {
        label: label.to_string(),
        depth,
        file_idx,
        status: file_idx.map(|_| FileStatus::Modified),
        collapsed: false,
    }
}

pub fn make_diff_file(path: &str) -> DiffFile {
    DiffFile {
        old_path: Some(path.to_string()),
        new_path: Some(path.to_string()),
        status: FileStatus::Modified,
        hunks: Vec::new(),
    }
}

pub fn make_pager_state_for_range(
    file_starts: Vec<usize>,
    lines_len: usize,
    active_file: Option<usize>,
) -> PagerState {
    let line_map = vec![
        LineInfo {
            file_idx: 0,
            path: Arc::from(""),
            new_lineno: None,
            old_lineno: None,
            line_kind: None,
            hunk_idx: None,
        };
        lines_len
    ];
    let mut state = PagerState::new(
        vec![String::new(); lines_len],
        line_map,
        file_starts,
        Vec::new(),
        Vec::new(),
        120,
    );
    state.set_active_file(active_file);
    state
}

pub fn make_line_map(kinds: &[Option<LineKind>]) -> Vec<LineInfo> {
    kinds
        .iter()
        .map(|&kind| LineInfo {
            file_idx: 0,
            path: "test.rs".into(),
            new_lineno: None,
            old_lineno: None,
            line_kind: kind,
            hunk_idx: None,
        })
        .collect()
}

/// Build a line_map with headers at known positions for content-line tests.
pub fn make_line_map_with_headers() -> Vec<LineInfo> {
    let kinds: Vec<Option<LineKind>> = vec![
        None,                    // 0: file header
        None,                    // 1: hunk header
        Some(LineKind::Context), // 2: context
        Some(LineKind::Added),   // 3: added
        Some(LineKind::Deleted), // 4: deleted
        None,                    // 5: blank sep
        None,                    // 6: hunk header
        None,                    // 7: file header
        Some(LineKind::Added),   // 8: added
    ];
    kinds
        .into_iter()
        .map(|kind| LineInfo {
            file_idx: 0,
            path: "test.rs".into(),
            new_lineno: Some(1),
            old_lineno: None,
            line_kind: kind,
            hunk_idx: None,
        })
        .collect()
}

pub fn entry_with_status(
    label: &str,
    depth: usize,
    file_idx: usize,
    status: FileStatus,
) -> TreeEntry {
    TreeEntry {
        label: label.to_string(),
        depth,
        file_idx: Some(file_idx),
        status: Some(status),
        collapsed: false,
    }
}

/// Calls `render_scrollbar_cell` for every row and returns (first_thumb_row, last_thumb_row).
pub fn scrollbar_thumb_range(
    content_height: usize,
    range: usize,
    top: usize,
    vis_start: usize,
) -> (usize, usize) {
    let vis_end = vis_start + range;
    let line_map: Vec<LineInfo> = (0..vis_end.max(content_height))
        .map(|_| LineInfo {
            file_idx: 0,
            path: Arc::from(""),
            new_lineno: None,
            old_lineno: None,
            line_kind: None,
            hunk_idx: None,
        })
        .collect();
    let mut first = None;
    let mut last = None;
    for row in 0..content_height {
        let cell = render_scrollbar_cell(row, content_height, vis_start, vis_end, top, &line_map);
        if cell.contains(style::BG_SCROLLBAR_THUMB) {
            if first.is_none() {
                first = Some(row);
            }
            last = Some(row);
        }
    }
    (
        first.expect("no thumb rows found"),
        last.expect("no thumb rows found"),
    )
}

pub fn strip(s: &str) -> String {
    crate::ansi::strip_ansi(s)
}

pub fn make_two_file_diff() -> Vec<DiffFile> {
    let raw = "\
diff --git a/a.txt b/a.txt
--- a/a.txt
+++ b/a.txt
@@ -1,1 +1,2 @@
 first
+second
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1,2 +1,1 @@
 keep
-remove
";
    crate::git::diff::parse(raw)
}

pub fn make_pager_state_from_files(files: &[DiffFile], tree_visible: bool) -> PagerState {
    let output = render::render(files, 80, false);
    let tree_entries = build_tree_entries(files);
    let doc = super::super::state::Document::from_render_output(output);
    let mut state = PagerState::from_doc(doc, tree_entries, 120);
    state.tree_visible = tree_visible;
    state
}

/// Build a 90-line PagerState with mixed content (Added/Deleted/Context).
pub fn make_mixed_content_state() -> PagerState {
    let header_indices: &[usize] = &[0, 5, 15, 30, 35, 45, 60, 65, 75];
    let mut line_map = Vec::with_capacity(90);
    for i in 0..90 {
        let (file_idx, path) = if i < 30 {
            (0, "a.rs")
        } else if i < 60 {
            (1, "b.rs")
        } else {
            (2, "c.rs")
        };
        let line_kind = if header_indices.contains(&i) {
            None
        } else if matches!(i, 6..=8) || matches!(i, 36..=37) || matches!(i, 76..=78) {
            Some(LineKind::Added)
        } else if matches!(i, 10..=11) || matches!(i, 46..=48) || matches!(i, 66..=67) {
            Some(LineKind::Deleted)
        } else {
            Some(LineKind::Context)
        };
        let lineno = if matches!(line_kind, Some(LineKind::Deleted)) {
            None
        } else if line_kind.is_some() {
            Some(i as u32 + 1)
        } else {
            None
        };
        let old_lineno = if matches!(line_kind, Some(LineKind::Deleted)) {
            Some(i as u32 + 1)
        } else {
            None
        };
        line_map.push(LineInfo {
            file_idx,
            path: path.into(),
            new_lineno: lineno,
            old_lineno,
            line_kind,
            hunk_idx: None,
        });
    }

    let tree_entries = vec![
        entry("a.rs", 0, Some(0)),
        entry("b.rs", 0, Some(1)),
        entry("c.rs", 0, Some(2)),
    ];
    let mut state = PagerState::new(
        vec!["line".into(); 90],
        line_map,
        vec![0, 30, 60],
        vec![5, 15, 35, 45, 65, 75],
        tree_entries,
        120,
    );
    state.cursor_line = 1;
    state
}

/// Build DiffFiles suitable for staging tests: one file with one hunk containing
/// context + added + deleted lines.
pub fn make_staging_files() -> Vec<DiffFile> {
    vec![DiffFile {
        old_path: Some("test.rs".to_string()),
        new_path: Some("test.rs".to_string()),
        status: FileStatus::Modified,
        hunks: vec![DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![
                DiffLine {
                    kind: LineKind::Context,
                    content: "ctx1".into(),
                    old_lineno: Some(1),
                    new_lineno: Some(1),
                },
                DiffLine {
                    kind: LineKind::Deleted,
                    content: "old".into(),
                    old_lineno: Some(2),
                    new_lineno: None,
                },
                DiffLine {
                    kind: LineKind::Added,
                    content: "new".into(),
                    old_lineno: None,
                    new_lineno: Some(2),
                },
                DiffLine {
                    kind: LineKind::Context,
                    content: "ctx2".into(),
                    old_lineno: Some(3),
                    new_lineno: Some(3),
                },
            ],
        }],
    }]
}

/// Build a PagerState with line_map entries that have hunk_idx set, suitable for
/// staging action tests. Returns (state, files).
pub fn make_staging_state() -> (PagerState, Vec<DiffFile>) {
    let files = make_staging_files();
    let output = render::render(&files, 80, false);
    let tree_entries = build_tree_entries(&files);
    let state = PagerState::new(
        output.lines().to_vec(),
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        tree_entries,
        120,
    );
    (state, files)
}

/// Build a `Document` from plain strings for tests that construct Documents directly.
pub fn make_test_document(
    lines: Vec<String>,
    line_map: Vec<LineInfo>,
    file_starts: Vec<usize>,
    hunk_starts: Vec<usize>,
) -> super::super::state::Document {
    let raw_texts = lines.clone();
    super::super::state::Document {
        styled_files: Vec::new(),
        display_lines: lines,
        raw_texts,
        line_map,
        file_starts,
        hunk_starts,
    }
}

pub fn add_leading_context_before_hunk_changes(state: &mut PagerState) {
    state.doc.line_map[6].line_kind = Some(LineKind::Context);
    state.doc.line_map[7].line_kind = Some(LineKind::Context);
    state.doc.line_map[8].line_kind = Some(LineKind::Added);

    state.doc.line_map[16].line_kind = Some(LineKind::Context);
    state.doc.line_map[17].line_kind = Some(LineKind::Deleted);
}
