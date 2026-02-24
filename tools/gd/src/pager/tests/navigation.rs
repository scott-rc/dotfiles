//! Navigation, viewport, content-line and scrollbar tests.

use crate::render::LineInfo;

use super::super::content::{is_content_line, next_content_line, prev_content_line};
use super::super::navigation::{
    change_group_starts, jump_next, jump_prev, nav_du_up, nav_U_up, nav_status_message,
    sync_tree_cursor, viewport_bounds,
};
use super::super::state::{
    Document, PagerState, capture_view_anchor, remap_after_document_swap, visible_range,
};
use super::super::tree::TreeEntry;
use super::super::types::{FileIx, LineIx, TreeEntryIx};
use super::common::{
    make_keybinding_state, make_line_map, make_line_map_with_headers, make_pager_state_for_range,
    scrollbar_thumb_range,
};
use crate::git::diff::{FileStatus, LineKind};

#[test]
fn test_change_group_starts_empty_line_map() {
    assert_eq!(change_group_starts(&[], 0, 0), Vec::<usize>::new());
    assert_eq!(change_group_starts(&[], 0, 10), Vec::<usize>::new());
}

#[test]
fn test_change_group_starts_no_changes() {
    let line_map = make_line_map(&[
        None,
        Some(LineKind::Context),
        Some(LineKind::Context),
        None,
        Some(LineKind::Context),
    ]);
    assert_eq!(
        change_group_starts(&line_map, 0, line_map.len()),
        Vec::<usize>::new()
    );
}

#[test]
fn test_change_group_starts_single_group() {
    let line_map = make_line_map(&[
        Some(LineKind::Context),
        Some(LineKind::Added),
        Some(LineKind::Added),
        Some(LineKind::Context),
    ]);
    assert_eq!(change_group_starts(&line_map, 0, line_map.len()), vec![1]);
}

#[test]
fn test_change_group_starts_adjacent_added_deleted_one_group() {
    let line_map = make_line_map(&[
        Some(LineKind::Context),
        Some(LineKind::Added),
        Some(LineKind::Deleted),
        Some(LineKind::Added),
        Some(LineKind::Context),
    ]);
    assert_eq!(change_group_starts(&line_map, 0, line_map.len()), vec![1]);
}

#[test]
fn test_change_group_starts_range_boundaries() {
    let line_map = make_line_map(&[
        Some(LineKind::Added),   // 0: change group at 0
        Some(LineKind::Context), // 1
        Some(LineKind::Context), // 2
        Some(LineKind::Context), // 3
        Some(LineKind::Context), // 4
        Some(LineKind::Deleted), // 5: change group at 5
        Some(LineKind::Context), // 6
        Some(LineKind::Context), // 7
        Some(LineKind::Context), // 8
        Some(LineKind::Context), // 9
        Some(LineKind::Added),   // 10: change group at 10
        Some(LineKind::Context), // 11
    ]);
    assert_eq!(change_group_starts(&line_map, 2, 8), vec![5]);
    assert_eq!(change_group_starts(&line_map, 6, 12), vec![10]);
    assert_eq!(change_group_starts(&line_map, 0, 3), vec![0]);
}

#[test]
fn test_change_group_starts_range_end_beyond_len() {
    let line_map = make_line_map(&[
        Some(LineKind::Context), // 0
        Some(LineKind::Context), // 1
        Some(LineKind::Added),   // 2
        Some(LineKind::Context), // 3
        Some(LineKind::Deleted), // 4
    ]);
    assert_eq!(change_group_starts(&line_map, 0, 100), vec![2, 4]);
}

#[test]
fn change_group_starts_finds_change_boundaries() {
    let line_map: Vec<LineInfo> = [
        Some(LineKind::Context),
        Some(LineKind::Context),
        Some(LineKind::Context),
        Some(LineKind::Context),
        Some(LineKind::Context),
        Some(LineKind::Added),
        Some(LineKind::Added),
        Some(LineKind::Added),
        Some(LineKind::Context),
        Some(LineKind::Context),
        Some(LineKind::Context),
        Some(LineKind::Context),
        Some(LineKind::Deleted),
        Some(LineKind::Deleted),
        Some(LineKind::Context),
        Some(LineKind::Context),
        Some(LineKind::Context),
        Some(LineKind::Added),
        Some(LineKind::Context),
        Some(LineKind::Context),
    ]
    .iter()
    .enumerate()
    .map(|(i, kind)| LineInfo {
        file_idx: 0,
        path: "test.rs".into(),
        new_lineno: Some(i as u32 + 1),
        old_lineno: None,
        line_kind: *kind,
    })
    .collect();

    let starts = change_group_starts(&line_map, 0, line_map.len());
    assert_eq!(starts, vec![5, 12, 17]);
}

#[test]
fn test_file_ix_constructor_boundaries() {
    assert_eq!(FileIx::new(0, 3), Some(FileIx(0)));
    assert_eq!(FileIx::new(2, 3), Some(FileIx(2)));
    assert_eq!(FileIx::new(3, 3), None);
    assert_eq!(FileIx::new(0, 0), None);
}

#[test]
fn test_line_ix_constructor_boundaries() {
    assert_eq!(LineIx::new(0, 10), Some(LineIx(0)));
    assert_eq!(LineIx::new(9, 10), Some(LineIx(9)));
    assert_eq!(LineIx::new(10, 10), None);
    assert_eq!(LineIx::new(0, 0), None);
}

#[test]
fn test_tree_entry_ix_constructor_boundaries() {
    assert_eq!(TreeEntryIx::new(0, 5), Some(TreeEntryIx(0)));
    assert_eq!(TreeEntryIx::new(4, 5), Some(TreeEntryIx(4)));
    assert_eq!(TreeEntryIx::new(5, 5), None);
    assert_eq!(TreeEntryIx::new(0, 0), None);
}

#[test]
fn test_normalize_after_document_swap_clamps_view_scope_when_file_count_shrinks() {
    let mut state = make_pager_state_for_range(vec![0, 10, 20], 30, Some(2));
    assert_eq!(state.active_file(), Some(2));
    let anchor = capture_view_anchor(&state);
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
    remap_after_document_swap(&mut state, anchor, new_doc, &[], 120);
    assert_eq!(
        state.active_file(),
        None,
        "view scope should clamp to AllFiles when file 2 no longer exists"
    );
}

#[test]
fn nav_status_message_positions() {
    let state = make_keybinding_state();
    assert_eq!(
        nav_status_message("Hunk", 6, &state.doc.hunk_starts, &state.doc.line_map),
        "Hunk 1/6 · a.rs"
    );
    assert_eq!(
        nav_status_message("Hunk", 36, &state.doc.hunk_starts, &state.doc.line_map),
        "Hunk 3/6 · b.rs"
    );
    assert_eq!(
        nav_status_message("Hunk", 76, &state.doc.hunk_starts, &state.doc.line_map),
        "Hunk 6/6 · c.rs"
    );
    assert_eq!(
        nav_status_message("Hunk", 45, &state.doc.hunk_starts, &state.doc.line_map),
        "Hunk 4/6 · b.rs"
    );
}

#[test]
fn test_jump_next_finds_first_target_after_top() {
    assert_eq!(jump_next(&[2, 5, 9], 3), Some(5));
}

#[test]
fn test_jump_next_returns_none_when_all_before() {
    assert_eq!(jump_next(&[1, 2], 5), None);
}

#[test]
fn test_jump_prev_finds_last_target_before_top() {
    assert_eq!(jump_prev(&[0, 3, 7], 5), Some(3));
}

#[test]
fn test_jump_prev_returns_none_when_all_after() {
    assert_eq!(jump_prev(&[5, 8], 3), None);
}

#[test]
fn test_visible_range_no_active_file() {
    let state = make_pager_state_for_range(vec![0, 5], 10, None);
    assert_eq!(visible_range(&state), (0, 10));
}

#[test]
fn test_visible_range_active_file_middle() {
    let state = make_pager_state_for_range(vec![0, 4, 9], 12, Some(1));
    assert_eq!(visible_range(&state), (4, 9));
}

#[test]
fn test_visible_range_active_file_last() {
    let state = make_pager_state_for_range(vec![0, 5], 10, Some(1));
    assert_eq!(visible_range(&state), (5, 10));
}

#[test]
fn test_viewport_bounds_active_file() {
    let state = make_pager_state_for_range(vec![0, 30, 60], 90, Some(1));
    let (range_start, range_end, max_line, max_top) = viewport_bounds(&state, 40);
    assert_eq!((range_start, range_end), (30, 60));
    assert_eq!(max_line, 59);
    assert_eq!(max_top, 30);
}

#[test]
fn test_is_content_line_true_for_added_deleted_context() {
    let map = make_line_map(&[
        None,
        Some(LineKind::Added),
        Some(LineKind::Deleted),
        Some(LineKind::Context),
    ]);
    assert!(!is_content_line(&map, 0), "None should not be content");
    assert!(is_content_line(&map, 1), "Added should be content");
    assert!(is_content_line(&map, 2), "Deleted should be content");
    assert!(is_content_line(&map, 3), "Context should be content");
}

#[test]
fn test_next_content_line_skips_header() {
    let map = make_line_map(&[None, None, Some(LineKind::Context)]);
    assert_eq!(next_content_line(&map, 0, 2), 2);
}

#[test]
fn test_next_content_line_returns_from_when_none_found() {
    let map = make_line_map(&[None, None]);
    assert_eq!(next_content_line(&map, 0, 1), 0);
}

#[test]
fn test_prev_content_line_scans_backward() {
    let map = make_line_map(&[Some(LineKind::Context), None, None]);
    assert_eq!(prev_content_line(&map, 2, 0), 0);
}

#[test]
fn test_is_content_line() {
    let lm = make_line_map_with_headers();
    assert!(!is_content_line(&lm, 0), "file header is not content");
    assert!(!is_content_line(&lm, 1), "hunk header is not content");
    assert!(is_content_line(&lm, 2), "context is content");
    assert!(is_content_line(&lm, 3), "added is content");
    assert!(is_content_line(&lm, 4), "deleted is content");
    assert!(!is_content_line(&lm, 5), "blank sep is not content");
    assert!(!is_content_line(&lm, 6), "hunk header is not content");
    assert!(!is_content_line(&lm, 7), "file header is not content");
    assert!(is_content_line(&lm, 8), "added is content");
}

#[test]
fn test_j_skips_headers() {
    let lm = make_line_map_with_headers();
    let result = next_content_line(&lm, 5, 8);
    assert_eq!(result, 8);
}

#[test]
fn test_k_skips_headers() {
    let lm = make_line_map_with_headers();
    let result = prev_content_line(&lm, 7, 0);
    assert_eq!(result, 4);
}

#[test]
fn test_scrollbar_thumb_fills_screen_when_content_equals_viewport() {
    let (thumb_start, thumb_end) = scrollbar_thumb_range(20, 20, 0, 0);
    assert_eq!(thumb_start, 0, "thumb should start at row 0");
    assert_eq!(thumb_end, 19, "thumb should end at row 19");
}

#[test]
fn test_scrollbar_thumb_half_height_when_content_double_viewport() {
    let (thumb_start, thumb_end) = scrollbar_thumb_range(20, 40, 0, 0);
    let height = thumb_end - thumb_start + 1;
    assert!(
        (9..=11).contains(&height),
        "thumb height should be ~10, got {height}"
    );
}

#[test]
fn test_scrollbar_thumb_at_bottom_when_scrolled_to_end() {
    let (thumb_start, _thumb_end) = scrollbar_thumb_range(20, 40, 20, 0);
    assert!(
        thumb_start >= 10,
        "thumb should be in the lower half when scrolled to end, got thumb_start={thumb_start}"
    );
}

#[test]
fn test_scrollbar_thumb_minimum_height_one_row() {
    let (thumb_start, thumb_end) = scrollbar_thumb_range(20, 10000, 0, 0);
    assert!(
        thumb_end >= thumb_start,
        "thumb must occupy at least 1 row (min-height guard)"
    );
}

#[test]
fn test_scrollbar_no_crash_on_zero_range() {
    let line_map: Vec<LineInfo> = (0..5)
        .map(|_| LineInfo {
            file_idx: 0,
            path: String::new(),
            new_lineno: None,
            old_lineno: None,
            line_kind: None,
        })
        .collect();
    let cell = super::super::rendering::render_scrollbar_cell(0, 20, 5, 5, 0, &line_map);
    assert!(
        cell.contains(crate::style::BG_SCROLLBAR_TRACK),
        "zero-range should return a track cell"
    );
}

#[test]
fn test_scrollbar_no_crash_on_empty_line_map() {
    let cell = super::super::rendering::render_scrollbar_cell(0, 20, 0, 40, 0, &[]);
    assert!(
        cell.contains(crate::style::BG_SCROLLBAR_TRACK)
            || cell.contains(crate::style::BG_SCROLLBAR_THUMB),
        "empty line_map should return a valid scrollbar cell without panicking"
    );
}

#[test]
fn test_sync_tree_cursor_collapsed_parent_lands_on_directory() {
    let tree_entries = vec![
        TreeEntry {
            label: "src".into(),
            depth: 0,
            file_idx: None,
            status: None,
            collapsed: true,
        },
        TreeEntry {
            label: "a.rs".into(),
            depth: 1,
            file_idx: Some(0),
            status: Some(FileStatus::Modified),
            collapsed: false,
        },
        TreeEntry {
            label: "b.rs".into(),
            depth: 1,
            file_idx: Some(1),
            status: Some(FileStatus::Modified),
            collapsed: false,
        },
    ];

    let line_map: Vec<LineInfo> = (0..10)
        .map(|i| LineInfo {
            file_idx: usize::from(i >= 5),
            path: if i < 5 { "src/a.rs" } else { "src/b.rs" }.into(),
            new_lineno: Some(i as u32 + 1),
            old_lineno: None,
            line_kind: Some(LineKind::Context),
        })
        .collect();

    let mut state = PagerState::new(
        vec!["line".into(); 10],
        line_map,
        vec![0, 5],
        vec![],
        tree_entries,
        120,
    );

    assert_eq!(
        state.tree_visible_to_entry,
        vec![0],
        "only collapsed dir should be visible"
    );

    state.cursor_line = 2;

    sync_tree_cursor(&mut state, 20);

    assert_eq!(
        state.tree_cursor(),
        0,
        "should land on parent directory entry when child is collapsed"
    );
}

#[test]
fn test_sync_tree_cursor_all_visible_normal_sync() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.rebuild_tree_lines();
    state.cursor_line = 31;

    sync_tree_cursor(&mut state, 20);

    assert_eq!(
        state.tree_cursor(),
        1,
        "should sync to entry 1 (b.rs) when cursor is in file 1"
    );
}

#[test]
fn test_sync_tree_cursor_empty_tree_no_panic() {
    let mut state = PagerState::new(
        vec!["line".into(); 5],
        vec![
            LineInfo {
                file_idx: 0,
                path: "a".into(),
                new_lineno: None,
                old_lineno: None,
                line_kind: None,
            };
            5
        ],
        vec![0],
        vec![],
        vec![],
        120,
    );
    state.tree_visible = true;

    sync_tree_cursor(&mut state, 20);
}

#[test]
fn test_nav_du_up_at_first_hunk_reports_not_moved() {
    let mut state = make_keybinding_state();
    // Line 6 is the first content line after hunk_start 5 (first hunk).
    state.cursor_line = 6;
    let result = nav_du_up(&state);
    assert!(
        !result.moved,
        "nav_du_up from first hunk should report moved=false, got cursor={}",
        result.cursor_line
    );
}

#[test]
fn test_nav_du_up_fallback_to_anchor_reports_not_moved() {
    let mut state = make_keybinding_state();
    // Line 6 is the first content line in the first hunk. jump_prev finds
    // hunk_start 5, next_content_line(5) returns 6 which is >= anchor (6),
    // second jump_prev(5) returns None, so cursor resets to anchor.
    state.cursor_line = 6;
    let result = nav_du_up(&state);
    assert_eq!(
        result.cursor_line, 6,
        "cursor should remain at anchor when fallback resets"
    );
    assert!(
        !result.moved,
        "nav_du_up should report moved=false when fallback resets to anchor"
    );
}

#[test]
#[allow(non_snake_case)]
fn test_scrollbar_no_panic_on_vis_end_less_than_vis_start() {
    let line_map: Vec<LineInfo> = (0..20)
        .map(|_| LineInfo {
            file_idx: 0,
            path: String::new(),
            new_lineno: None,
            old_lineno: None,
            line_kind: None,
        })
        .collect();
    // vis_start=10 > vis_end=5 should not panic on subtraction underflow.
    let cell =
        super::super::rendering::render_scrollbar_cell(0, 20, 10, 5, 0, &line_map);
    assert!(
        cell.contains(crate::style::BG_SCROLLBAR_TRACK),
        "inverted range should return a track cell without panicking"
    );
}

#[test]
#[allow(non_snake_case)]
fn test_nav_U_up_at_first_file_reports_not_moved() {
    let mut state = make_keybinding_state();
    // Line 1 is the first content line in file 0.
    state.cursor_line = 1;
    let result = nav_U_up(&state, 40);
    assert!(
        !result.moved,
        "nav_U_up from first file should report moved=false, got cursor={}",
        result.cursor_line
    );
}
