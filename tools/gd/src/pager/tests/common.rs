//! Shared test helpers and fixtures for pager tests.

use crate::git::diff::{DiffFile, FileStatus, LineKind};
use crate::render;
use crate::render::LineInfo;
use crate::style;

use super::super::rendering::render_scrollbar_cell;
use super::super::state::{visible_range, PagerState};
use super::super::tree::{build_tree_entries, TreeEntry};
use super::super::types::ViewScope;
use std::sync::{Mutex, OnceLock};

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
    pub mark_line: Option<usize>,
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
            mark_line: s.mark_line,
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
    let max_top = range_max.saturating_sub(1).min(line_count.saturating_sub(1));
    assert!(
        state.top_line >= rs && state.top_line <= max_top.min(range_max),
        "top_line {} out of range [rs={}, max_top={}]",
        state.top_line,
        rs,
        max_top
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
            path: String::new(),
            new_lineno: None,
            old_lineno: None,
            line_kind: None,
        };
        lines_len
    ];
    let mut state = PagerState::new(
        vec![String::new(); lines_len],
        line_map,
        file_starts,
        Vec::new(),
        Vec::new(),
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
        })
        .collect()
}

pub fn entry_with_status(label: &str, depth: usize, file_idx: usize, status: FileStatus) -> TreeEntry {
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
            path: String::new(),
            new_lineno: None,
            old_lineno: None,
            line_kind: None,
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
    let output = render::render(files, 80, false, tree_visible);
    let tree_entries = build_tree_entries(files);
    let mut state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        tree_entries,
    );
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
    );
    state.cursor_line = 1;
    state
}

pub fn add_leading_context_before_hunk_changes(state: &mut PagerState) {
    state.doc.line_map[6].line_kind = Some(LineKind::Context);
    state.doc.line_map[7].line_kind = Some(LineKind::Context);
    state.doc.line_map[8].line_kind = Some(LineKind::Added);

    state.doc.line_map[16].line_kind = Some(LineKind::Context);
    state.doc.line_map[17].line_kind = Some(LineKind::Deleted);
}
