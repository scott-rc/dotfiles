//! Runtime regression tests: resolve_path, re_render, document swap, resize.

use crate::git::diff::LineKind;
use crate::render;
use crate::render::LineInfo;

use super::super::runtime::{re_render, resolve_path_for_editor};
use super::super::state::{capture_view_anchor, remap_after_document_swap, visible_range, Document};
use super::common::{
    assert_state_invariants, make_keybinding_state, make_pager_state_for_range,
    make_pager_state_from_files, make_two_file_diff, with_gd_debug_env, StateSnapshot,
};
use super::super::tree::build_tree_lines;

#[test]
fn test_resolve_path_relative_joins_repo_root() {
    let repo = std::path::Path::new("/tmp/my_repo");
    let path = "src/foo.rs";
    assert_eq!(
        resolve_path_for_editor(path, repo),
        std::path::PathBuf::from("/tmp/my_repo/src/foo.rs")
    );
}

#[test]
fn test_resolve_path_absolute_unchanged() {
    let repo = std::path::Path::new("/tmp/my_repo");
    let path = "/absolute/path/to/file.rs";
    assert_eq!(
        resolve_path_for_editor(path, repo),
        std::path::PathBuf::from(path)
    );
}

#[test]
fn test_resolve_path_simple_filename() {
    let repo = std::path::Path::new("/home/user/repo");
    let path = "README.md";
    assert_eq!(
        resolve_path_for_editor(path, repo),
        std::path::PathBuf::from("/home/user/repo/README.md")
    );
}

#[test]
fn re_render_passes_skip_headers_when_tree_visible() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, true);
    re_render(&mut state, &files, false, 80);
    let stripped: String = state
        .doc
        .lines
        .iter()
        .map(|l| crate::ansi::strip_ansi(l))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !stripped.contains('\u{2500}'),
        "with tree_visible=true, re_render should skip file headers"
    );
}

#[test]
fn re_render_includes_headers_when_tree_hidden() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, false);
    re_render(&mut state, &files, false, 80);
    let stripped: String = state
        .doc
        .lines
        .iter()
        .map(|l| crate::ansi::strip_ansi(l))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        stripped.contains('\u{2500}'),
        "with tree_visible=false, re_render should include file headers"
    );
}

#[test]
fn re_render_preserves_position_on_header_line() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, false);

    let target = state
        .doc
        .line_map
        .iter()
        .enumerate()
        .rev()
        .find(|(_, li)| li.file_idx > 0 && li.new_lineno.is_none())
        .map(|(i, _)| i)
        .expect("should have a new_lineno=None line with file_idx > 0");

    let first_none = state
        .doc
        .line_map
        .iter()
        .position(|li| {
            li.file_idx == state.doc.line_map[target].file_idx && li.new_lineno.is_none()
        })
        .unwrap();
    assert_ne!(first_none, target, "need at least two None-lineno lines for the same file");

    state.top_line = target;
    re_render(&mut state, &files, false, 80);
    assert_eq!(
        state.top_line, target,
        "re_render should preserve top_line on a header/None-lineno line"
    );
}

#[test]
fn debug_toggle_does_not_change_reducer_output() {
    let files = make_two_file_diff();
    let snap_off = with_gd_debug_env(None, || {
        let mut state_off = make_pager_state_from_files(&files, true);
        re_render(&mut state_off, &files, false, 80);
        StateSnapshot::from(&state_off)
    });

    let snap_on = with_gd_debug_env(Some("1"), || {
        let mut state_on = make_pager_state_from_files(&files, true);
        re_render(&mut state_on, &files, false, 80);
        StateSnapshot::from(&state_on)
    });

    assert_eq!(snap_off, snap_on, "GD_DEBUG on vs off must produce identical state");
}

#[test]
fn document_swap_multi_to_single_file_preserves_valid_cursor() {
    let raw3 = "\
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
diff --git a/c.txt b/c.txt
--- /dev/null
+++ b/c.txt
@@ -0,0 +1,1 @@
+new
";
    let three_files = crate::git::diff::parse(raw3);
    let mut state = make_pager_state_from_files(&three_files, true);
    state.set_active_file(Some(1));
    state.cursor_line = 35;
    let single_file = crate::git::diff::parse("\
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1,2 +1,1 @@
 keep
-remove
");
    re_render(&mut state, &single_file, false, 80);
    let (rs, re) = visible_range(&state);
    assert!(
        state.cursor_line >= rs && state.cursor_line < re,
        "cursor {} must be in visible range [{}, {})",
        state.cursor_line,
        rs,
        re
    );
    assert!(state.doc.line_map.get(state.cursor_line).is_some());
}

#[test]
fn document_swap_to_empty_exits_cleanly() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, false);
    state.cursor_line = 5;
    state.top_line = 3;
    let anchor = capture_view_anchor(&state);
    let empty_doc = Document::from_render_output(render::render(&[], 80, false, false));
    remap_after_document_swap(&mut state, anchor, empty_doc, &[]);
    assert_eq!(state.cursor_line, 0);
    assert_eq!(state.top_line, 0);
}

#[test]
fn resize_with_tree_visible_keeps_valid_selection_and_cursor() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, true);
    state.set_tree_cursor(1);
    state.set_active_file(Some(1));
    state.cursor_line = 4;
    let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
    state.tree_lines = tl;
    state.tree_visible_to_entry = tv;
    re_render(&mut state, &files, false, 40);
    assert_state_invariants(&state);
}

#[test]
fn test_remap_anchor_file_idx_beyond_new_doc_lands_on_first_content() {
    let mut state = make_keybinding_state();
    state.tree_visible = false;
    state.top_line = 65;
    state.cursor_line = 65;

    let anchor = capture_view_anchor(&state);
    assert!(anchor.is_some());
    assert_eq!(
        anchor.as_ref().unwrap().file_idx,
        2,
        "anchor should reference file 2"
    );

    let line_map: Vec<LineInfo> = (0..20)
        .map(|i| LineInfo {
            file_idx: 0,
            path: "x.txt".into(),
            new_lineno: if i == 0 { None } else { Some(i as u32) },
            old_lineno: None,
            line_kind: if i == 0 { None } else { Some(LineKind::Context) },
        })
        .collect();
    let new_doc = Document {
        lines: vec![String::new(); 20],
        line_map,
        file_starts: vec![0],
        hunk_starts: vec![],
    };

    remap_after_document_swap(&mut state, anchor, new_doc, &[]);

    assert!(
        state.cursor_line > 0,
        "should skip header and land on content"
    );
    assert!(state.cursor_line < 20);
}

#[test]
fn test_remap_anchor_new_lineno_none_uses_offset_in_file() {
    let mut state = make_pager_state_for_range(vec![0, 10, 20], 30, None);
    state.top_line = 5;
    state.cursor_line = 5;

    let anchor = capture_view_anchor(&state);
    assert!(anchor.is_some());
    let a = anchor.as_ref().unwrap();
    assert_eq!(a.new_lineno, None, "header line should have no lineno");
    assert_eq!(a.offset_in_file, 5);

    let new_doc = Document {
        lines: vec![String::new(); 30],
        line_map: vec![
            LineInfo {
                file_idx: 0,
                path: String::new(),
                new_lineno: None,
                old_lineno: None,
                line_kind: None,
            };
            30
        ],
        file_starts: vec![0, 10, 20],
        hunk_starts: vec![],
    };

    remap_after_document_swap(&mut state, anchor, new_doc, &[]);

    assert_eq!(
        state.cursor_line, 5,
        "offset_in_file fallback should preserve position"
    );
    assert_eq!(state.top_line, 5);
}

#[test]
fn test_remap_anchor_none_resets_cursor_and_top() {
    let mut state = make_pager_state_for_range(vec![0, 10], 20, None);
    state.cursor_line = 10;
    state.top_line = 5;

    let new_doc = Document {
        lines: vec![String::new(); 20],
        line_map: vec![
            LineInfo {
                file_idx: 0,
                path: String::new(),
                new_lineno: None,
                old_lineno: None,
                line_kind: None,
            };
            20
        ],
        file_starts: vec![0, 10],
        hunk_starts: vec![],
    };

    remap_after_document_swap(&mut state, None, new_doc, &[]);

    assert_eq!(state.cursor_line, 0);
    assert_eq!(state.top_line, 0);
}
