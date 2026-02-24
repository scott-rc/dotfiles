//! Reducer (handle_key) integration tests.

use crate::git::diff::{DiffFile, LineKind};
use insta::assert_debug_snapshot;
use tui::pager::Key;
use tui::search::{find_matches, find_nearest_match};

use super::super::content::is_content_line;
use super::super::reducer::handle_key;
use super::super::runtime::re_render;
use super::super::state::visible_range;
use super::super::types::{KeyResult, Mode};
use super::common::{
    StateSnapshot, add_leading_context_before_hunk_changes, assert_state_invariants,
    make_diff_file, make_keybinding_state, make_mixed_content_state, make_pager_state_from_files,
    make_two_file_diff,
};

// ---- Navigation: j/k scroll ----

#[test]
fn key_j_next_content_line() {
    let mut state = make_keybinding_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_k_prev_content_line() {
    let mut state = make_keybinding_state();
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char('k'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_j_skips_headers() {
    let mut state = make_keybinding_state();
    state.cursor_line = 4;
    handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_g_jumps_to_first_content() {
    let mut state = make_keybinding_state();
    state.cursor_line = 15;
    handle_key(&mut state, Key::Char('g'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_G_jumps_to_last_content() {
    let mut state = make_keybinding_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('G'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Navigation: d/u half page ----

#[test]
fn key_d_half_page_down() {
    let mut state = make_keybinding_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('d'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_u_half_page_up() {
    let mut state = make_keybinding_state();
    state.cursor_line = 25;
    handle_key(&mut state, Key::Char('u'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Navigation: z center viewport ----

#[test]
fn key_z_centers_viewport() {
    let mut state = make_keybinding_state();
    state.cursor_line = 40;
    state.top_line = 0;
    handle_key(&mut state, Key::Char('z'), 20, 40, 120, &[]);
    assert!(
        state.top_line > 0,
        "z should center viewport around cursor, moving top_line from 0"
    );
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Diff nav: ]/[ hunk ----

#[test]
fn key_bracket_next_hunk_same_file() {
    let mut state = make_keybinding_state();
    state.cursor_line = 8;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_prev_hunk_same_file() {
    let mut state = make_keybinding_state();
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_prev_hunk_from_first_content_line() {
    let mut state = make_keybinding_state();
    state.cursor_line = 16;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[]);
    assert_eq!(state.cursor_line, 6);
}

#[test]
fn key_bracket_prev_hunk_cross_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.cursor_line = 36;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[]);
    assert_eq!(state.cursor_line, 16);
}

#[test]
fn key_bracket_next_hunk_cross_file_boundary() {
    let mut state = make_keybinding_state();
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_next_hunk_scrolloff_binding() {
    let mut state = make_keybinding_state();
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char(']'), 15, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_next_hunk_at_last_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 76;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_prev_hunk_at_first_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_no_active_file_does_not_stick() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 5;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Diff nav: }/{ file ----

#[test]
fn key_brace_next_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_prev_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('{'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_prev_file_no_active_stuck_cursor() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('{'), 50, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_next_file_no_active_file_does_not_stick() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 0;
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_next_file_at_last_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 66;
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_prev_file_at_first_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('{'), 50, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_next_file_single_file_switches() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[]);
    assert_eq!(
        state.active_file(),
        Some(1),
        "}} in single-file mode should switch to next file"
    );
    assert!(state.cursor_line >= 30 && state.cursor_line < 60);
}

#[test]
fn key_brace_prev_file_single_file_switches() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('{'), 40, 40, 120, &[]);
    assert_eq!(
        state.active_file(),
        Some(0),
        "{{ in single-file mode should switch to prev file"
    );
    assert!(state.cursor_line < 30);
}

#[test]
fn key_brace_next_file_single_file_last_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(2));
    state.cursor_line = 61;
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[]);
    assert_eq!(state.active_file(), Some(2));
}

#[test]
fn key_brace_prev_file_single_file_first_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('{'), 40, 40, 120, &[]);
    assert_eq!(state.active_file(), Some(0));
}

#[test]
fn key_brace_shows_file_status_message() {
    let mut state = make_mixed_content_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_prev_shows_file_status_message() {
    let mut state = make_mixed_content_state();
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('{'), 50, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Hunk nav in single-file mode ----

#[test]
fn key_bracket_single_file_jumps_to_next_file_hunk() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    assert_eq!(state.cursor_line, 36);
    assert_eq!(state.active_file(), Some(1));
}

#[test]
fn key_bracket_single_file_jumps_to_prev_file_hunk() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.cursor_line = 36;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[]);
    assert_eq!(state.cursor_line, 16);
    assert_eq!(state.active_file(), Some(0));
}

#[test]
fn key_bracket_single_file_within_file_works() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    assert_eq!(state.cursor_line, 16);
}

#[test]
fn key_bracket_single_file_clamps_top_line_to_active_file_range() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    let (range_start, _range_end) = visible_range(&state);
    assert!(state.top_line >= range_start);
}

// ---- Full context ----

#[test]
fn key_bracket_full_context_single_file_navigates_changes() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    for i in 0..30 {
        state.doc.line_map[i].line_kind = if [0, 5, 15].contains(&i) {
            None
        } else if (7..=8).contains(&i) {
            Some(LineKind::Added)
        } else if (20..=21).contains(&i) {
            Some(LineKind::Deleted)
        } else {
            Some(LineKind::Context)
        };
    }
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    assert_state_invariants(&state);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_hunk_context_skips_leading_context_to_first_change() {
    let mut state = make_mixed_content_state();
    add_leading_context_before_hunk_changes(&mut state);
    state.full_context = false;
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    assert_eq!(state.cursor_line, 8);
}

#[test]
fn key_bracket_prev_hunk_context_skips_leading_context() {
    let mut state = make_mixed_content_state();
    add_leading_context_before_hunk_changes(&mut state);
    state.full_context = false;
    state.cursor_line = 17;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[]);
    assert_eq!(state.cursor_line, 8);
}

#[test]
fn key_bracket_full_context_single_file_lands_on_change_group() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    assert_eq!(state.cursor_line, 6);
}

#[test]
fn key_bracket_prev_full_context_single_file_at_first_change_is_noop() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 7;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[]);
    assert_eq!(state.cursor_line, 6);
}

#[test]
fn key_bracket_then_prev_round_trip_full_context_single_file() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    let after_next = state.cursor_line;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[]);
    let after_prev = state.cursor_line;
    assert!(
        after_next > 6,
        "] should move forward from 6, got {after_next}"
    );
    assert!(
        after_prev <= 8,
        "[ should return near first change group, got {after_prev}"
    );
    assert_eq!(after_prev, 6);
}

#[test]
fn key_bracket_full_context_all_context_file_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- s toggle single file ----

#[test]
fn key_s_toggles_off_single_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_s_toggles_on_single_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_s_still_toggles_single_file() {
    let mut state = make_keybinding_state();
    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &[]);
    assert_eq!(state.active_file(), Some(0));
    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &[]);
    assert_eq!(state.active_file(), None);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn normal_s_toggle_on_snaps_cursor_to_content() {
    let mut state = make_keybinding_state();
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &[]);
    assert!(
        is_content_line(&state.doc.line_map, state.cursor_line),
        "cursor_line {} is not a content line",
        state.cursor_line
    );
}

// ---- o toggle full context ----

#[test]
fn key_o_toggles_full_context() {
    let mut state = make_keybinding_state();
    handle_key(&mut state, Key::Char('o'), 40, 40, 120, &[]);
    assert!(state.full_context);
}

#[test]
fn key_o_toggles_hunk_context() {
    let mut state = make_keybinding_state();
    state.full_context = true;
    handle_key(&mut state, Key::Char('o'), 40, 40, 120, &[]);
    assert!(!state.full_context);
}

#[test]
fn key_space_is_noop_for_full_context_toggle() {
    let mut state = make_keybinding_state();
    state.full_context = false;
    handle_key(&mut state, Key::Char(' '), 40, 40, 120, &[]);
    assert!(!state.full_context);
}

#[test]
fn key_space_is_noop_for_context_toggle() {
    let mut state = make_keybinding_state();
    state.full_context = true;
    handle_key(&mut state, Key::Char(' '), 40, 40, 120, &[]);
    assert!(state.full_context);
}

// ---- / search ----

#[test]
fn key_slash_enters_search() {
    let mut state = make_keybinding_state();
    handle_key(&mut state, Key::Char('/'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- n/N match navigation ----

#[test]
fn key_n_wraps_within_single_file() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![6, 36, 66];
    state.current_match = 0;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('n'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_N_wraps_within_single_file() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![6, 36, 66];
    state.current_match = 0;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('N'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_n_no_matches_in_active_file() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![36, 66];
    state.current_match = -1;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('n'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_n_after_toggling_single_file_off_cycles_globally() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![6, 36, 66];
    state.current_match = 0;
    state.set_active_file(None);
    handle_key(&mut state, Key::Char('n'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn test_key_n_single_file_moves_to_next_match() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.search_matches = vec![5, 15];
    state.current_match = 0;
    handle_key(&mut state, Key::Char('n'), 40, 40, 120, &[]);
    assert_eq!(state.current_match, 1);
    assert_eq!(state.cursor_line, 15);
}

#[test]
#[allow(non_snake_case)]
fn test_key_N_single_file_moves_to_prev_match() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.search_matches = vec![5, 15];
    state.current_match = 1;
    handle_key(&mut state, Key::Char('N'), 40, 40, 120, &[]);
    assert_eq!(state.current_match, 0);
    assert_eq!(state.cursor_line, 5);
}

#[test]
fn test_key_n_empty_matches_noop() {
    let mut state = make_keybinding_state();
    state.search_matches = vec![];
    state.current_match = -1;
    handle_key(&mut state, Key::Char('n'), 40, 40, 120, &[]);
    assert_eq!(state.current_match, -1);
}

#[test]
#[allow(non_snake_case)]
fn test_key_N_empty_matches_noop() {
    let mut state = make_keybinding_state();
    state.search_matches = vec![];
    state.current_match = -1;
    handle_key(&mut state, Key::Char('N'), 40, 40, 120, &[]);
    assert_eq!(state.current_match, -1);
}

// ---- l toggle tree ----

#[test]
fn key_l_toggles_tree_on() {
    let mut state = make_keybinding_state();
    state.tree_visible = false;
    let files = vec![
        make_diff_file("a.rs"),
        make_diff_file("b.rs"),
        make_diff_file("c.rs"),
    ];
    handle_key(&mut state, Key::Char('l'), 40, 40, 120, &files);
    assert!(state.tree_visible, "l should show tree");
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_l_toggles_tree_off() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Char('l'), 40, 40, 120, &[]);
    assert!(!state.tree_visible, "l should hide tree");
}

// ---- tree width clamping ----

#[test]
fn key_l_toggle_tree_clamps_width_on_narrow_terminal() {
    use super::super::tree::MIN_DIFF_WIDTH;
    let mut state = make_keybinding_state();
    state.tree_visible = false;
    let mut files = vec![
        make_diff_file("src/components/very_long_path/deeply_nested/file.rs"),
        make_diff_file("src/components/very_long_path/deeply_nested/fourth.rs"),
        make_diff_file("src/components/very_long_path/deeply_nested/other.rs"),
        make_diff_file("src/components/very_long_path/deeply_nested/third.rs"),
    ];
    crate::git::sort_files_for_display(&mut files);
    let cols: u16 = 100;
    handle_key(&mut state, Key::Char('l'), 40, 40, cols, &files);
    assert!(state.tree_visible, "tree should be visible even on narrow terminal");
    let max_tree = cols as usize - MIN_DIFF_WIDTH - 1;
    assert!(
        state.tree_width <= max_tree,
        "tree_width {} should leave room for diff (max_tree={max_tree})",
        state.tree_width
    );
}

#[test]
fn key_s_toggle_single_file_clamps_tree_width_on_narrow_terminal() {
    use super::super::tree::MIN_DIFF_WIDTH;
    let mut state = make_keybinding_state();
    state.tree_entries.clear();
    let mut files = vec![
        make_diff_file("src/components/very_long_path/deeply_nested/file.rs"),
        make_diff_file("src/components/very_long_path/deeply_nested/fourth.rs"),
        make_diff_file("src/components/very_long_path/deeply_nested/other.rs"),
        make_diff_file("src/components/very_long_path/deeply_nested/third.rs"),
    ];
    crate::git::sort_files_for_display(&mut files);
    let cols: u16 = 100;
    handle_key(&mut state, Key::Char('s'), 40, 40, cols, &files);
    let max_tree = cols as usize - MIN_DIFF_WIDTH - 1;
    assert!(
        state.tree_width <= max_tree,
        "tree_width {} should be clamped (max_tree={max_tree})",
        state.tree_width
    );
}

// ---- tree toggle fallback ----

#[test]
fn key_l_toggle_tree_fallback_on_very_narrow_terminal() {
    use super::super::tree::MIN_DIFF_WIDTH;
    let mut state = make_keybinding_state();
    state.tree_visible = false;
    let files = vec![
        make_diff_file("src/a.rs"),
        make_diff_file("src/b.rs"),
    ];
    // Very narrow terminal where resolve_tree_layout returns None,
    // triggering the fallback: terminal_cols.saturating_sub(MIN_DIFF_WIDTH + 1)
    let cols: u16 = 85;
    handle_key(&mut state, Key::Char('l'), 40, 40, cols, &files);
    assert!(state.tree_visible, "l should still toggle tree on");
    // Fallback tree_width must not make the diff unusable
    assert!(
        state.tree_width + MIN_DIFF_WIDTH + 1 <= cols as usize,
        "tree_width {} + MIN_DIFF_WIDTH + 1 should not exceed cols {}",
        state.tree_width,
        cols
    );
}

// ---- v visual select / y yank selection ----

#[test]
fn key_v_starts_visual_select() {
    let mut state = make_keybinding_state();
    state.cursor_line = 10;
    handle_key(&mut state, Key::Char('v'), 40, 40, 120, &[]);
    assert_eq!(state.visual_anchor, Some(10));
    assert!(state.status_message.contains("VISUAL"));
}

#[test]
fn key_y_without_selection_shows_error() {
    let mut state = make_keybinding_state();
    state.visual_anchor = None;
    handle_key(&mut state, Key::Char('y'), 40, 40, 120, &[]);
    assert!(state.status_message.contains("No selection"));
}

#[test]
fn key_y_with_selection_clears_anchor() {
    let mut state = make_keybinding_state();
    state.visual_anchor = Some(5);
    state.cursor_line = 10;
    handle_key(&mut state, Key::Char('y'), 40, 40, 120, &[]);
    assert_eq!(state.visual_anchor, None, "yank should clear visual anchor");
}

#[test]
fn key_esc_clears_visual_selection() {
    let mut state = make_keybinding_state();
    state.visual_anchor = Some(5);
    handle_key(&mut state, Key::Escape, 40, 40, 120, &[]);
    assert_eq!(state.visual_anchor, None, "Esc should clear visual anchor");
}

#[test]
fn key_esc_without_selection_is_noop() {
    let mut state = make_keybinding_state();
    let cursor_before = state.cursor_line;
    handle_key(&mut state, Key::Escape, 40, 40, 120, &[]);
    assert_eq!(state.cursor_line, cursor_before);
    assert_eq!(state.visual_anchor, None);
}

// ---- ? toggle tooltip ----

#[test]
fn key_question_toggles_tooltip() {
    let mut state = make_keybinding_state();
    assert!(!state.tooltip_visible);
    handle_key(&mut state, Key::Char('?'), 40, 40, 120, &[]);
    assert!(state.tooltip_visible, "? should show tooltip");
    handle_key(&mut state, Key::Char('?'), 40, 40, 120, &[]);
    assert!(!state.tooltip_visible, "? again should hide tooltip");
}

// ---- Single-file boundary tests ----

#[test]
fn key_g_single_file_lands_on_file_start() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(1));
    state.cursor_line = 50;
    handle_key(&mut state, Key::Char('g'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_G_single_file_lands_on_file_end() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('G'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_d_single_file_clamps_to_file_end() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.cursor_line = 25;
    handle_key(&mut state, Key::Char('d'), 20, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_u_single_file_clamps_to_file_start() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(1));
    state.cursor_line = 32;
    handle_key(&mut state, Key::Char('u'), 20, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_j_at_last_content_line_of_single_file_is_noop() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.cursor_line = 29;
    handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_prev_no_active_file_at_file_boundary() {
    let mut state = make_mixed_content_state();
    state.set_active_file(None);
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('{'), 50, 40, 120, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Initial state ----

#[test]
fn test_initial_state_no_active_file() {
    let state = make_keybinding_state();
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Sequences ----

#[test]
fn sequence_toggle_single_file_context_regenerate() {
    let files: Vec<DiffFile> = vec![
        make_diff_file("a.rs"),
        make_diff_file("b.rs"),
        make_diff_file("c.rs"),
    ];
    let mut state = make_keybinding_state();

    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &files);
    assert_state_invariants(&state);

    let result = handle_key(&mut state, Key::Char('o'), 40, 40, 120, &files);
    assert_state_invariants(&state);
    if matches!(result, KeyResult::ReGenerate) {
        re_render(&mut state, &files, false, 80);
    }
    assert_state_invariants(&state);
}

#[test]
fn sequence_hunk_nav_in_both_context_modes() {
    let mut state = make_mixed_content_state();
    let files: Vec<DiffFile> = vec![];

    state.full_context = false;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &files);
    assert_state_invariants(&state);
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &files);
    assert_state_invariants(&state);

    state.full_context = true;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &files);
    assert_state_invariants(&state);
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &files);
    assert_state_invariants(&state);
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &files);
    assert_state_invariants(&state);

    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &files);
    assert_state_invariants(&state);
}

#[test]
fn sequence_resize_rerender_in_search() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, true);

    handle_key(&mut state, Key::Char('/'), 40, 40, 120, &files);
    state.search_input = "first".to_string();
    state.search_cursor = 5;
    state.search_query = "first".to_string();
    state.search_matches = find_matches(&state.doc.lines, "first");
    state.current_match = find_nearest_match(&state.search_matches, state.top_line);
    state.mode = Mode::Search;
    assert_state_invariants(&state);

    re_render(&mut state, &files, false, 40);
    assert_state_invariants(&state);

    handle_key(&mut state, Key::Escape, 40, 40, 120, &files);
    assert_state_invariants(&state);
}

#[test]
fn property_bounded_random_transitions() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, true);

    let mut rng: u64 = 12345;
    let keys: &[Key] = &[
        Key::Char('j'),
        Key::Char('k'),
        Key::Char(']'),
        Key::Char('['),
        Key::Char('g'),
        Key::Char('G'),
        Key::Char('d'),
        Key::Char('u'),
        Key::Char('z'),
        Key::Escape,
        Key::Char('s'),
        Key::Char('o'),
        Key::Char('l'),
        Key::Char('m'),
        Key::Char('y'),
        Key::Char('}'),
        Key::Char('{'),
        Key::Char('?'),
    ];

    for step in 0..72 {
        let key_idx = (rng as usize) % keys.len();
        let key = keys[key_idx];
        rng = rng.wrapping_mul(1_103_515_245).wrapping_add(12_345);

        let ch = 24 + ((rng >> 16) as usize % 20);
        let rows = 40;
        let _ = handle_key(&mut state, key, ch, rows, 120, &files);
        assert_state_invariants(&state);

        if step > 0 && step % 12 == 0 {
            let cols = 40 + ((rng >> 8) as u16 % 40);
            re_render(&mut state, &files, false, cols);
            assert_state_invariants(&state);
        } else if step > 0 && step % 18 == 0 {
            re_render(&mut state, &files, false, 80);
            assert_state_invariants(&state);
        }
    }
}
