//! Runtime regression tests: resolve_path, re_render, document swap, resize.

use crate::git::diff::LineKind;
use crate::render;
use crate::render::LineInfo;

use super::super::runtime::{re_render, resolve_path_for_editor};
use super::super::state::{
    Document, PagerState, capture_view_anchor, remap_after_document_swap, visible_range,
};
use super::super::rendering::diff_area_width;
use super::super::tree::{build_tree_entries, build_tree_lines, MIN_DIFF_WIDTH};
use super::common::{
    StateSnapshot, assert_state_invariants, make_keybinding_state, make_pager_state_for_range,
    make_pager_state_from_files, make_two_file_diff, with_gd_debug_env,
};

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
fn re_render_includes_headers_when_tree_visible() {
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
        stripped.contains('\u{2500}'),
        "with tree_visible=true, re_render should include file headers"
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
fn default_tree_hidden_for_small_flat_file_lists() {
    let files = vec![
        crate::git::diff::DiffFile::from_content("a.txt", "a"),
        crate::git::diff::DiffFile::from_content("b.txt", "b"),
    ];
    let output = render::render(&files, 80, false);
    let state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        build_tree_entries(&files),
        120,
    );
    assert!(
        !state.tree_visible,
        "tree should default hidden for a small flat file list"
    );
}

#[test]
fn default_tree_hidden_for_three_flat_files() {
    let files = vec![
        crate::git::diff::DiffFile::from_content("a.txt", "a"),
        crate::git::diff::DiffFile::from_content("b.txt", "b"),
        crate::git::diff::DiffFile::from_content("c.txt", "c"),
    ];
    let output = render::render(&files, 80, false);
    let state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        build_tree_entries(&files),
        120,
    );
    assert!(
        !state.tree_visible,
        "tree should default hidden for three flat files"
    );
}

#[test]
fn default_tree_visible_for_four_flat_files() {
    let files = vec![
        crate::git::diff::DiffFile::from_content("a.txt", "a"),
        crate::git::diff::DiffFile::from_content("b.txt", "b"),
        crate::git::diff::DiffFile::from_content("c.txt", "c"),
        crate::git::diff::DiffFile::from_content("d.txt", "d"),
    ];
    let output = render::render(&files, 80, false);
    let state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        build_tree_entries(&files),
        120,
    );
    assert!(
        state.tree_visible,
        "tree should default visible for four flat files"
    );
}

#[test]
fn default_tree_visible_for_nested_file_lists() {
    let files = vec![
        crate::git::diff::DiffFile::from_content("src/a.txt", "a"),
        crate::git::diff::DiffFile::from_content("src/b.txt", "b"),
    ];
    let output = render::render(&files, 80, false);
    let state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        build_tree_entries(&files),
        120,
    );
    assert!(
        state.tree_visible,
        "tree should default visible when directory hierarchy is present"
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
    assert_ne!(
        first_none, target,
        "need at least two None-lineno lines for the same file"
    );

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

    assert_eq!(
        snap_off, snap_on,
        "GD_DEBUG on vs off must produce identical state"
    );
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
    let single_file = crate::git::diff::parse(
        "\
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1,2 +1,1 @@
 keep
-remove
",
    );
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
    let empty_doc = Document::from_render_output(render::render(&[], 80, false));
    remap_after_document_swap(&mut state, anchor, empty_doc, &[], 120);
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
            line_kind: if i == 0 {
                None
            } else {
                Some(LineKind::Context)
            },
        })
        .collect();
    let new_doc = Document {
        lines: vec![String::new(); 20],
        line_map,
        file_starts: vec![0],
        hunk_starts: vec![],
    };

    remap_after_document_swap(&mut state, anchor, new_doc, &[], 120);

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

    remap_after_document_swap(&mut state, anchor, new_doc, &[], 120);

    assert_eq!(
        state.cursor_line, 5,
        "offset_in_file fallback should preserve position"
    );
    assert_eq!(state.top_line, 5);
}

#[test]
fn remap_after_document_swap_hides_tree_when_terminal_narrows() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, true);
    assert!(state.tree_visible, "tree should start visible");

    let anchor = capture_view_anchor(&state);
    let new_doc = Document::from_render_output(render::render(&files, 80, false));
    // Use a very narrow terminal where resolve_tree_layout returns None
    let narrow_cols = MIN_DIFF_WIDTH + 5; // too narrow for MIN_TREE_WIDTH
    remap_after_document_swap(&mut state, anchor, new_doc, &files, narrow_cols);
    assert!(
        !state.tree_visible,
        "tree should auto-hide when terminal is too narrow (cols={narrow_cols})"
    );
}

#[test]
fn re_render_resize_diff_lines_fit_within_new_tree_width() {
    // Nested files with content_width ~30 so tree_width gets clamped at 80 cols
    // but gets the full width at 120 cols. This triggers the stale tree_width bug
    // when widening: old tree_width=19 → new tree_width=30, diff rendered too wide.
    let mut files = vec![
        crate::git::diff::DiffFile::from_content("src/pager/handler.rs", "fn handle() {}"),
        crate::git::diff::DiffFile::from_content("src/pager/rendering.rs", "fn render() {}"),
        crate::git::diff::DiffFile::from_content("src/pager/state.rs", "fn state() {}"),
        crate::git::diff::DiffFile::from_content("src/lib.rs", "mod pager;"),
    ];
    crate::git::sort_files_for_display(&mut files);
    let tree_entries = build_tree_entries(&files);

    // Start at 100 cols where tree_width is clamped (100 - 80 - 1 = 19)
    let output = render::render(&files, 80, false);
    let mut state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        tree_entries,
        100,
    );
    assert!(state.tree_visible, "tree should be visible at 100 cols");
    let narrow_tree_width = state.tree_width;

    // Re-render at 140 cols — tree_width should increase
    re_render(&mut state, &files, false, 140);
    assert!(state.tree_visible, "tree should still be visible at 140 cols");
    assert!(
        state.tree_width > narrow_tree_width,
        "tree_width should increase from {narrow_tree_width} at wider terminal, got {}",
        state.tree_width
    );

    let expected_diff_width =
        diff_area_width(140, state.tree_width, state.tree_visible, state.full_context);

    // Every rendered line must fit within the diff area after resize
    for (i, line) in state.doc.lines.iter().enumerate() {
        let vis_w = crate::ansi::visible_width(line);
        assert!(
            vis_w <= expected_diff_width,
            "line {i} has visible_width {vis_w} but diff_area_width is {expected_diff_width} \
             (cols=140, tree_width={}, narrow_tree_width={narrow_tree_width})",
            state.tree_width
        );
    }
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

    remap_after_document_swap(&mut state, None, new_doc, &[], 120);

    assert_eq!(state.cursor_line, 0);
    assert_eq!(state.top_line, 0);
}

#[test]
fn resize_tree_width_changes_but_stays_visible() {
    // Deeply nested files with content_width ~44 so tree gets clamped at 100 cols
    let mut files = vec![
        crate::git::diff::DiffFile::from_content(
            "src/pager/components/handler.rs",
            "fn handle() {}",
        ),
        crate::git::diff::DiffFile::from_content(
            "src/pager/components/rendering.rs",
            "fn render() {}",
        ),
        crate::git::diff::DiffFile::from_content("src/pager/state.rs", "fn state() {}"),
        crate::git::diff::DiffFile::from_content("src/lib.rs", "mod pager;"),
    ];
    crate::git::sort_files_for_display(&mut files);
    let tree_entries = build_tree_entries(&files);
    let content_width = super::super::tree::compute_tree_width(&tree_entries);

    // Start at 100 cols — tree_width clamped to 100 - 80 - 1 = 19
    let output = render::render(&files, 80, false);
    let mut state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        tree_entries,
        100,
    );
    assert!(state.tree_visible, "tree should be visible at 100 cols");
    let width_at_100 = state.tree_width;
    assert!(width_at_100 < content_width, "tree_width should be clamped at 100 cols");

    // Resize to 140 — tree still visible, width should increase
    re_render(&mut state, &files, false, 140);
    assert!(state.tree_visible, "tree should still be visible at 140 cols");
    assert!(
        state.tree_width > width_at_100,
        "tree_width should increase from {width_at_100} to {}, content_width={content_width}",
        state.tree_width
    );
    let daw = diff_area_width(140, state.tree_width, state.tree_visible, state.full_context);
    for (i, line) in state.doc.lines.iter().enumerate() {
        let vis_w = crate::ansi::visible_width(line);
        assert!(
            vis_w <= daw,
            "line {i} overflows at 140 cols: {vis_w} > {daw}"
        );
    }

    // Resize back to 100 — no overflow
    re_render(&mut state, &files, false, 100);
    assert!(state.tree_visible, "tree should still be visible at 100 cols");
    let daw = diff_area_width(100, state.tree_width, state.tree_visible, state.full_context);
    for (i, line) in state.doc.lines.iter().enumerate() {
        let vis_w = crate::ansi::visible_width(line);
        assert!(
            vis_w <= daw,
            "line {i} overflows at 100 cols: {vis_w} > {daw}"
        );
    }
}

#[test]
fn full_context_with_tree_visible_accounts_for_scrollbar() {
    let mut files = vec![
        crate::git::diff::DiffFile::from_content("src/pager/handler.rs", "fn handle() {}"),
        crate::git::diff::DiffFile::from_content("src/pager/rendering.rs", "fn render() {}"),
        crate::git::diff::DiffFile::from_content("src/pager/state.rs", "fn state() {}"),
        crate::git::diff::DiffFile::from_content("src/lib.rs", "mod pager;"),
    ];
    crate::git::sort_files_for_display(&mut files);
    let tree_entries = build_tree_entries(&files);
    let output = render::render(&files, 80, false);
    let mut state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        tree_entries,
        120,
    );
    assert!(state.tree_visible);

    // Enable full_context (scrollbar visible)
    state.full_context = true;
    re_render(&mut state, &files, false, 120);

    // diff_area_width should subtract both separator and scrollbar
    let expected = diff_area_width(120, state.tree_width, true, true);
    assert_eq!(
        expected,
        120 - state.tree_width - 2,
        "diff_area_width with tree + scrollbar should be cols - tree_width - 2"
    );
    for (i, line) in state.doc.lines.iter().enumerate() {
        let vis_w = crate::ansi::visible_width(line);
        assert!(
            vis_w <= expected,
            "line {i} overflows with full_context+tree: {vis_w} > {expected}"
        );
    }
}

#[test]
fn default_tree_hidden_at_80_cols_flat_files() {
    let files = vec![
        crate::git::diff::DiffFile::from_content("a.txt", "a"),
        crate::git::diff::DiffFile::from_content("b.txt", "b"),
        crate::git::diff::DiffFile::from_content("c.txt", "c"),
        crate::git::diff::DiffFile::from_content("d.txt", "d"),
    ];
    let output = render::render(&files, 80, false);
    let state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        build_tree_entries(&files),
        80,
    );
    assert!(
        !state.tree_visible,
        "tree should be hidden at 80 cols (MIN_DIFF_WIDTH=80 leaves no room)"
    );
}

#[test]
fn default_tree_hidden_at_90_cols_nested_files() {
    let mut files = vec![
        crate::git::diff::DiffFile::from_content("src/a.txt", "a"),
        crate::git::diff::DiffFile::from_content("src/b.txt", "b"),
    ];
    crate::git::sort_files_for_display(&mut files);
    let output = render::render(&files, 80, false);
    let state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        build_tree_entries(&files),
        90,
    );
    // 90 - 80 - 1 = 9, which is < MIN_TREE_WIDTH (15)
    assert!(
        !state.tree_visible,
        "tree should be hidden at 90 cols (not enough room for min tree width)"
    );
}

#[test]
fn default_tree_visible_at_100_cols_nested_files() {
    let mut files = vec![
        crate::git::diff::DiffFile::from_content("src/a.txt", "a"),
        crate::git::diff::DiffFile::from_content("src/b.txt", "b"),
    ];
    crate::git::sort_files_for_display(&mut files);
    let output = render::render(&files, 80, false);
    let state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        build_tree_entries(&files),
        100,
    );
    // 100 - 80 - 1 = 19 >= MIN_TREE_WIDTH (15), and has_directories=true
    assert!(
        state.tree_visible,
        "tree should be visible at 100 cols with nested files"
    );
}

#[test]
fn default_tree_visibility_exact_threshold_boundary() {
    use super::super::tree::{MIN_DIFF_WIDTH, MIN_TREE_WIDTH};
    // Exact threshold: MIN_DIFF_WIDTH + MIN_TREE_WIDTH + 1 = 80 + 15 + 1 = 96
    let threshold = MIN_DIFF_WIDTH + MIN_TREE_WIDTH + 1;
    assert_eq!(threshold, 96, "threshold should be 96 with current constants");

    let mut files = vec![
        crate::git::diff::DiffFile::from_content("src/a.txt", "a"),
        crate::git::diff::DiffFile::from_content("src/b.txt", "b"),
    ];
    crate::git::sort_files_for_display(&mut files);

    // At threshold: tree visible
    let output = render::render(&files, 80, false);
    let state = PagerState::new(
        output.lines.clone(),
        output.line_map.clone(),
        output.file_starts.clone(),
        output.hunk_starts.clone(),
        build_tree_entries(&files),
        threshold,
    );
    assert!(state.tree_visible, "tree should be visible at exact threshold ({threshold})");

    // One below threshold: tree hidden
    let state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        build_tree_entries(&files),
        threshold - 1,
    );
    assert!(
        !state.tree_visible,
        "tree should be hidden one below threshold ({})",
        threshold - 1
    );
}

// ---- tree auto-show on resize ----

#[test]
fn re_render_auto_shows_tree_when_terminal_widens() {
    use super::super::tree::MIN_TREE_WIDTH;
    let mut files = vec![
        crate::git::diff::DiffFile::from_content("src/pager/handler.rs", "fn handle() {}"),
        crate::git::diff::DiffFile::from_content("src/pager/rendering.rs", "fn render() {}"),
        crate::git::diff::DiffFile::from_content("src/pager/state.rs", "fn state() {}"),
        crate::git::diff::DiffFile::from_content("src/lib.rs", "mod pager;"),
    ];
    crate::git::sort_files_for_display(&mut files);
    let tree_entries = build_tree_entries(&files);

    // Start narrow — tree should be hidden
    let narrow_cols: usize = MIN_DIFF_WIDTH + 5;
    let output = render::render(&files, narrow_cols, false);
    let state = PagerState::new(
        output.lines.clone(),
        output.line_map.clone(),
        output.file_starts.clone(),
        output.hunk_starts.clone(),
        tree_entries,
        narrow_cols,
    );
    assert!(!state.tree_visible, "tree should start hidden at {narrow_cols} cols");
    assert!(!state.tree_user_hidden, "tree_user_hidden should be false (auto-hidden)");

    // Re-render at wide terminal — tree should auto-show
    let mut state = state;
    re_render(&mut state, &files, false, 120);
    assert!(
        state.tree_visible,
        "tree should auto-show when terminal widens to 120 cols"
    );
    assert!(state.tree_width >= MIN_TREE_WIDTH);
}

#[test]
fn re_render_does_not_auto_show_tree_when_user_hidden() {
    let mut files = vec![
        crate::git::diff::DiffFile::from_content("src/pager/handler.rs", "fn handle() {}"),
        crate::git::diff::DiffFile::from_content("src/pager/rendering.rs", "fn render() {}"),
        crate::git::diff::DiffFile::from_content("src/pager/state.rs", "fn state() {}"),
        crate::git::diff::DiffFile::from_content("src/lib.rs", "mod pager;"),
    ];
    crate::git::sort_files_for_display(&mut files);
    let tree_entries = build_tree_entries(&files);

    // Start wide — tree visible
    let output = render::render(&files, 80, false);
    let mut state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        tree_entries,
        120,
    );
    assert!(state.tree_visible, "tree should start visible at 120 cols");

    // Simulate user pressing `l` to hide
    state.tree_visible = false;
    state.tree_user_hidden = true;

    // Re-render at wide terminal — tree should stay hidden
    re_render(&mut state, &files, false, 120);
    assert!(
        !state.tree_visible,
        "tree should stay hidden when user explicitly hid it"
    );
}

#[test]
fn re_render_auto_shows_after_auto_hide_cycle() {
    let mut files = vec![
        crate::git::diff::DiffFile::from_content("src/pager/handler.rs", "fn handle() {}"),
        crate::git::diff::DiffFile::from_content("src/pager/rendering.rs", "fn render() {}"),
        crate::git::diff::DiffFile::from_content("src/pager/state.rs", "fn state() {}"),
        crate::git::diff::DiffFile::from_content("src/lib.rs", "mod pager;"),
    ];
    crate::git::sort_files_for_display(&mut files);
    let tree_entries = build_tree_entries(&files);

    // Start wide — tree visible
    let output = render::render(&files, 80, false);
    let mut state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        tree_entries,
        120,
    );
    assert!(state.tree_visible);

    // Narrow → tree auto-hides
    re_render(&mut state, &files, false, (MIN_DIFF_WIDTH + 5) as u16);
    assert!(!state.tree_visible, "tree should auto-hide when narrowed");
    assert!(!state.tree_user_hidden, "auto-hide should not set user_hidden");

    // Widen → tree auto-shows
    re_render(&mut state, &files, false, 120);
    assert!(
        state.tree_visible,
        "tree should auto-show after auto-hide when terminal widens again"
    );
}

#[test]
fn re_render_does_not_auto_show_for_small_flat_diffs() {
    // Only 2 files, no directories — resolve_tree_layout returns None regardless of width
    let files = make_two_file_diff();
    let tree_entries = build_tree_entries(&files);
    let output = render::render(&files, 80, false);
    let mut state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        tree_entries,
        60,
    );
    assert!(!state.tree_visible, "tree should be hidden for 2 flat files");

    // Widen — should still not show (content doesn't qualify)
    re_render(&mut state, &files, false, 200);
    assert!(
        !state.tree_visible,
        "tree should stay hidden for small flat diffs even at wide terminal"
    );
}
