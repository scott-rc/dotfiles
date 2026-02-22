//! Reducer (handle_key) integration tests.

use crate::git::diff::{DiffFile, LineKind};
use insta::assert_debug_snapshot;
use tui::pager::Key;
use tui::search::{find_matches, find_nearest_match};

use super::super::content::is_content_line;
use super::super::state::visible_range;
use super::super::reducer::handle_key;
use super::super::runtime::re_render;
use super::super::tree::{build_tree_lines, file_idx_to_entry_idx};
use super::super::types::{KeyResult, Mode};
use super::common::{
    add_leading_context_before_hunk_changes, assert_state_invariants, entry,
    make_diff_file, make_keybinding_state, make_mixed_content_state,
    make_pager_state_from_files, make_two_file_diff, StateSnapshot,
};

#[test]
fn key_d_full_context_single_file_navigates_changes() {
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
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_state_invariants(&state);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_j_next_content_line() {
    let mut state = make_keybinding_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_k_prev_content_line() {
    let mut state = make_keybinding_state();
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_j_skips_headers() {
    let mut state = make_keybinding_state();
    state.cursor_line = 4;
    handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_g_jumps_to_first_content() {
    let mut state = make_keybinding_state();
    state.cursor_line = 15;
    handle_key(&mut state, Key::Char('g'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_G_jumps_to_last_content() {
    let mut state = make_keybinding_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('G'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_ctrl_d_half_page_down() {
    let mut state = make_keybinding_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::CtrlD, 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_ctrl_u_half_page_up() {
    let mut state = make_keybinding_state();
    state.cursor_line = 25;
    handle_key(&mut state, Key::CtrlU, 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_d_next_hunk_same_file() {
    let mut state = make_keybinding_state();
    state.cursor_line = 8;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_u_prev_hunk_same_file() {
    let mut state = make_keybinding_state();
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_u_prev_hunk_from_first_content_line() {
    let mut state = make_keybinding_state();
    state.cursor_line = 16;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 6);
}

#[test]
fn key_u_cross_file_from_first_hunk() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.cursor_line = 36;
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 16);
}

#[test]
fn key_u_tree_focused_from_first_content_line() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.cursor_line = 16;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 6);
}

#[test]
fn key_d_cross_file_boundary() {
    let mut state = make_keybinding_state();
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_u_cross_file_boundary() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.cursor_line = 36;
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 16);
}

#[test]
fn key_d_next_hunk_scrolloff_binding() {
    let mut state = make_keybinding_state();
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char('d'), 15, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_D_next_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('D'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_U_prev_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('U'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_U_no_active_file_stuck_cursor() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('U'), 50, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_d_no_active_file_does_not_stick() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 5;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_next_file_no_active_file_does_not_stick() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 0;
    handle_key(&mut state, Key::Char('D'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_d_at_last_hunk_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 76;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_u_at_first_hunk_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_D_at_last_file_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 66;
    handle_key(&mut state, Key::Char('D'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_U_at_first_file_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('U'), 50, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_d_single_file_jumps_to_next_file_hunk() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 36);
    assert_eq!(state.active_file(), Some(1));
}

#[test]
fn key_u_single_file_jumps_to_prev_file_hunk() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.cursor_line = 36;
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 16);
    assert_eq!(state.active_file(), Some(0));
}

#[test]
fn key_next_file_single_file_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('D'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_prev_file_single_file_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(2));
    state.cursor_line = 61;
    handle_key(&mut state, Key::Char('U'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_d_single_file_within_file_works() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 16);
}

#[test]
fn key_d_tree_focused_single_file_jumps_globally() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_active_file(Some(0));
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 36);
    assert_eq!(state.active_file(), Some(1));
    assert_eq!(state.tree_cursor(), file_idx_to_entry_idx(&state.tree_entries, 1));
}

#[test]
fn key_u_tree_focused_single_file_jumps_globally() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_active_file(Some(1));
    state.cursor_line = 36;
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 16);
    assert_eq!(state.active_file(), Some(0));
    assert_eq!(state.tree_cursor(), file_idx_to_entry_idx(&state.tree_entries, 0));
}

#[test]
fn key_d_single_file_clamps_top_line_to_active_file_range() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    let (range_start, _range_end) = visible_range(&state);
    assert!(state.top_line >= range_start);
}

#[test]
fn key_u_tree_focused_single_file_clamps_top_line_to_active_file_range() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_active_file(Some(1));
    state.cursor_line = 36;
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    let (range_start, range_end) = visible_range(&state);
    let max_top: usize = range_end.saturating_sub(40).max(range_start);
    assert!(state.top_line >= range_start);
    assert!(state.top_line <= max_top);
}

#[test]
fn key_slash_enters_search() {
    let mut state = make_keybinding_state();
    handle_key(&mut state, Key::Char('/'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_question_enters_help() {
    let mut state = make_keybinding_state();
    handle_key(&mut state, Key::Char('?'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_v_enters_visual() {
    let mut state = make_keybinding_state();
    handle_key(&mut state, Key::Char('v'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_esc_exits_visual() {
    let mut state = make_keybinding_state();
    state.mode = Mode::Visual;
    state.visual_anchor = 10;
    handle_key(&mut state, Key::Escape, 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_h_in_tree_defocuses() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    handle_key(&mut state, Key::Char('h'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_esc_in_tree_defocuses() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    handle_key(&mut state, Key::Escape, 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_tab_in_tree_defocuses() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    handle_key(&mut state, Key::Tab, 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_1_in_tree_defocuses() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    handle_key(&mut state, Key::Char('1'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_j_in_tree_next_entry() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_tree_cursor(0);
    let initial_top = state.top_line;
    let initial_cursor = state.cursor_line;
    handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
    assert_eq!(state.tree_cursor(), 1);
    assert_eq!(state.top_line, initial_top);
    assert_eq!(state.cursor_line, initial_cursor);
}

#[test]
fn key_k_in_tree_prev_entry() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_tree_cursor(state.tree_visible_to_entry[1]);
    let initial_top = state.top_line;
    let initial_cursor = state.cursor_line;
    handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
    assert_eq!(state.tree_cursor(), 0);
    assert_eq!(state.top_line, initial_top);
    assert_eq!(state.cursor_line, initial_cursor);
}

#[test]
fn key_g_in_tree_first_entry() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_tree_cursor(*state.tree_visible_to_entry.last().unwrap());
    handle_key(&mut state, Key::Char('g'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_G_in_tree_last_entry() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_tree_cursor(state.tree_visible_to_entry[0]);
    handle_key(&mut state, Key::Char('G'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_enter_on_file_in_tree() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_tree_cursor(state.tree_visible_to_entry[1]);
    handle_key(&mut state, Key::Enter, 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_enter_on_dir_in_tree() {
    let mut state = make_keybinding_state();
    state.tree_entries = vec![
        super::super::tree::TreeEntry {
            label: "src".to_string(),
            depth: 0,
            file_idx: None,
            status: None,
            collapsed: false,
        },
        entry("a.rs", 1, Some(0)),
        entry("b.rs", 1, Some(1)),
    ];
    state.tree_width = super::super::tree::compute_tree_width(&state.tree_entries);
    let (tl, tv) = build_tree_lines(&state.tree_entries, 0, state.tree_width);
    state.tree_lines = tl;
    state.tree_visible_to_entry = tv;
    state.set_tree_focused(true);
    state.set_tree_cursor(0);
    handle_key(&mut state, Key::Enter, 40, 40, &[]);
    assert!(state.tree_entries[0].collapsed);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_e_opens_and_focuses_tree() {
    let mut state = make_keybinding_state();
    state.tree_visible = false;
    state.set_tree_focused(false);
    let files = vec![make_diff_file("a.rs"), make_diff_file("b.rs"), make_diff_file("c.rs")];
    handle_key(&mut state, Key::Char('e'), 40, 40, &files);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_e_focuses_open_tree() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.set_tree_focused(false);
    handle_key(&mut state, Key::Char('e'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_e_closes_focused_tree() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.set_tree_focused(true);
    handle_key(&mut state, Key::Char('e'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn e_close_tree_preserves_active_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.tree_visible = true;
    state.set_tree_focused(true);
    handle_key(&mut state, Key::Char('e'), 40, 40, &[]);
    assert_eq!(state.active_file(), Some(1), "single-file view must be preserved");
    assert!(!state.tree_visible);
    assert!(!state.tree_focused());
}

#[test]
fn e_close_tree_preserves_active_file_none() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.tree_visible = true;
    state.set_tree_focused(true);
    handle_key(&mut state, Key::Char('e'), 40, 40, &[]);
    assert_eq!(state.active_file(), None, "active_file must stay None");
}

#[test]
fn key_tab_focuses_tree() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.set_tree_focused(false);
    handle_key(&mut state, Key::Tab, 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_l_shows_and_focuses_tree() {
    let mut state = make_keybinding_state();
    state.tree_visible = false;
    state.set_tree_focused(false);
    let files = vec![make_diff_file("a.rs"), make_diff_file("b.rs"), make_diff_file("c.rs")];
    handle_key(&mut state, Key::Char('l'), 40, 40, &files);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_a_toggles_off_single_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('a'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_a_toggles_on_single_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    handle_key(&mut state, Key::Char('a'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_space_is_noop_for_full_context_toggle() {
    let mut state = make_keybinding_state();
    state.full_context = false;
    handle_key(&mut state, Key::Char(' '), 40, 40, &[]);
    assert!(!state.full_context);
}

#[test]
fn key_space_is_noop_for_context_toggle() {
    let mut state = make_keybinding_state();
    state.full_context = true;
    handle_key(&mut state, Key::Char(' '), 40, 40, &[]);
    assert!(state.full_context);
}

#[test]
fn key_z_toggles_full_context() {
    let mut state = make_keybinding_state();
    handle_key(&mut state, Key::Char('z'), 40, 40, &[]);
    assert!(state.full_context);
}

#[test]
fn key_z_toggles_hunk_context() {
    let mut state = make_keybinding_state();
    state.full_context = true;
    handle_key(&mut state, Key::Char('z'), 40, 40, &[]);
    assert!(!state.full_context);
}

#[test]
fn test_initial_state_no_active_file() {
    let state = make_keybinding_state();
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn test_tree_j_without_active_file_moves_tree_only() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    let initial_top = state.top_line;
    let initial_cursor = state.cursor_line;
    handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
    assert_eq!(state.tree_cursor(), 1);
    assert_eq!(state.top_line, initial_top);
    assert_eq!(state.cursor_line, initial_cursor);
}

#[test]
fn test_tree_j_single_file_moves_tree_only() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_active_file(Some(0));
    let initial_top = state.top_line;
    let initial_cursor = state.cursor_line;
    handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
    assert_eq!(state.tree_cursor(), 1);
    assert_eq!(state.top_line, initial_top);
    assert_eq!(state.cursor_line, initial_cursor);
    assert_eq!(state.active_file(), Some(0));
}

#[test]
fn test_tree_k_without_active_file_moves_tree_only() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_tree_cursor(1);
    let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
    state.tree_lines = tl;
    state.tree_visible_to_entry = tv;
    let initial_top = state.top_line;
    let initial_cursor = state.cursor_line;
    handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
    assert_eq!(state.tree_cursor(), 0);
    assert_eq!(state.top_line, initial_top);
    assert_eq!(state.cursor_line, initial_cursor);
}

#[test]
fn test_tree_k_single_file_moves_tree_only() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_active_file(Some(1));
    state.set_tree_cursor(1);
    state.top_line = 30;
    state.cursor_line = 31;
    let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
    state.tree_lines = tl;
    state.tree_visible_to_entry = tv;
    let initial_top = state.top_line;
    let initial_cursor = state.cursor_line;
    handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
    let (vis_start, vis_end) = visible_range(&state);
    assert_eq!(state.tree_cursor(), 0);
    assert_eq!(state.top_line, initial_top);
    assert_eq!(state.cursor_line, initial_cursor);
    assert!(state.cursor_line >= vis_start && state.cursor_line < vis_end);
    assert!(is_content_line(&state.doc.line_map, state.cursor_line));
    assert_eq!(state.active_file(), Some(1));
}

#[test]
fn test_tree_enter_scrolls_without_active_file() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_tree_cursor(1);
    let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
    state.tree_lines = tl;
    state.tree_visible_to_entry = tv;
    handle_key(&mut state, Key::Enter, 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn test_tree_enter_single_file_switches_active_file() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_active_file(Some(0));
    state.set_tree_cursor(1);
    let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
    state.tree_lines = tl;
    state.tree_visible_to_entry = tv;
    handle_key(&mut state, Key::Enter, 40, 40, &[]);
    assert_eq!(state.active_file(), Some(1));
    assert_eq!(state.top_line, 30);
    assert!(state.cursor_line >= 30 && state.cursor_line < 60);
}

#[test]
fn test_a_still_toggles_single_file() {
    let mut state = make_keybinding_state();
    handle_key(&mut state, Key::Char('a'), 40, 40, &[]);
    assert_eq!(state.active_file(), Some(0));
    handle_key(&mut state, Key::Char('a'), 40, 40, &[]);
    assert_eq!(state.active_file(), None);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_d_hunk_context_skips_leading_context_to_first_change() {
    let mut state = make_mixed_content_state();
    add_leading_context_before_hunk_changes(&mut state);
    state.full_context = false;
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 8);
}

#[test]
fn key_u_hunk_context_skips_leading_context_to_prev_first_change() {
    let mut state = make_mixed_content_state();
    add_leading_context_before_hunk_changes(&mut state);
    state.full_context = false;
    state.cursor_line = 17;
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 8);
}

#[test]
fn key_d_tree_focused_hunk_context_skips_leading_context() {
    let mut state = make_mixed_content_state();
    add_leading_context_before_hunk_changes(&mut state);
    state.full_context = false;
    state.set_tree_focused(true);
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 8);
}

#[test]
fn key_d_full_context_single_file_lands_on_change_group() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 6);
}

#[test]
fn key_u_full_context_single_file_at_first_change_is_noop() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 7;
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 6);
}

#[test]
fn key_d_then_u_round_trip_full_context_single_file() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    let after_d = state.cursor_line;
    handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
    let after_u = state.cursor_line;
    assert!(after_d > 6, "d should move forward from 6, got {after_d}");
    assert!(after_u <= 8, "u should return near first change group, got {after_u}");
    assert_eq!(after_u, 6);
}

#[test]
fn key_d_full_context_all_context_file_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_d_tree_focused_full_context_single_file() {
    let mut state = make_mixed_content_state();
    state.set_tree_focused(true);
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 6);
}

#[test]
fn key_g_single_file_lands_on_file_start() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(1));
    state.cursor_line = 50;
    handle_key(&mut state, Key::Char('g'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_G_single_file_lands_on_file_end() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('G'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_ctrl_d_single_file_clamps_to_file_end() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.cursor_line = 25;
    handle_key(&mut state, Key::CtrlD, 20, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_ctrl_u_single_file_clamps_to_file_start() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(1));
    state.cursor_line = 32;
    handle_key(&mut state, Key::CtrlU, 20, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_j_at_last_content_line_of_single_file_is_noop() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.cursor_line = 29;
    handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_U_no_active_file_at_file_boundary() {
    let mut state = make_mixed_content_state();
    state.set_active_file(None);
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('U'), 50, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_n_wraps_within_single_file() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![6, 36, 66];
    state.current_match = 0;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('n'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_N_wraps_within_single_file() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![6, 36, 66];
    state.current_match = 0;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('N'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_n_no_matches_in_active_file() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![36, 66];
    state.current_match = -1;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('n'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_n_after_toggling_single_file_off_cycles_globally() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![6, 36, 66];
    state.current_match = 0;
    state.set_active_file(None);
    handle_key(&mut state, Key::Char('n'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn visual_j_clamps_at_file_boundary() {
    let mut state = make_mixed_content_state();
    state.mode = Mode::Visual;
    state.visual_anchor = 28;
    state.cursor_line = 28;
    handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
    assert_eq!(state.cursor_line, 29, "first j should move to 29");
    handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn visual_k_clamps_at_file_boundary() {
    let mut state = make_mixed_content_state();
    state.mode = Mode::Visual;
    state.visual_anchor = 31;
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
    let after_first_k = state.cursor_line;
    handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
    let file_idx = state.doc.line_map[state.cursor_line].file_idx;
    assert_eq!(file_idx, 1, "cursor must remain in file 1");
    let _ = after_first_k;
}

#[test]
fn visual_y_with_mixed_content_lines() {
    let mut state = make_mixed_content_state();
    state.mode = Mode::Visual;
    state.visual_anchor = 6;
    state.cursor_line = 11;
    handle_key(&mut state, Key::Char('y'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn visual_escape_snaps_cursor_to_content() {
    let mut state = make_keybinding_state();
    state.mode = Mode::Visual;
    state.visual_anchor = 1;
    state.cursor_line = 3;
    state.top_line = 0;
    handle_key(&mut state, Key::Escape, 40, 40, &[]);
    assert!(
        is_content_line(&state.doc.line_map, state.cursor_line),
        "cursor_line {} is not a content line",
        state.cursor_line
    );
}

#[test]
fn visual_yank_snaps_cursor_to_content() {
    let mut state = make_keybinding_state();
    state.mode = Mode::Visual;
    state.visual_anchor = 1;
    state.cursor_line = 3;
    state.top_line = 0;
    handle_key(&mut state, Key::Char('y'), 40, 40, &[]);
    assert!(
        is_content_line(&state.doc.line_map, state.cursor_line),
        "cursor_line {} is not a content line",
        state.cursor_line
    );
}

#[test]
fn tree_j_snaps_cursor_to_content() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_tree_cursor(0);
    handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
    assert!(
        is_content_line(&state.doc.line_map, state.cursor_line),
        "cursor_line {} is not a content line",
        state.cursor_line
    );
}

#[test]
fn tree_enter_snaps_cursor_to_content() {
    let mut state = make_keybinding_state();
    state.set_tree_focused(true);
    state.set_tree_cursor(1);
    handle_key(&mut state, Key::Enter, 40, 40, &[]);
    assert!(
        is_content_line(&state.doc.line_map, state.cursor_line),
        "cursor_line {} is not a content line",
        state.cursor_line
    );
}

#[test]
fn tree_g_snaps_cursor_to_content() {
    let mut state = make_keybinding_state();
    state.cursor_line = 31;
    state.set_tree_cursor(1);
    handle_key(&mut state, Key::Char('g'), 40, 40, &[]);
    assert!(
        is_content_line(&state.doc.line_map, state.cursor_line),
        "cursor_line {} is not a content line",
        state.cursor_line
    );
}

#[test]
#[allow(non_snake_case)]
fn key_D_shows_file_status_message() {
    let mut state = make_mixed_content_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('D'), 40, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_U_shows_file_status_message() {
    let mut state = make_mixed_content_state();
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('U'), 50, 40, &[]);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn normal_a_toggle_on_snaps_cursor_to_content() {
    let mut state = make_keybinding_state();
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('a'), 40, 40, &[]);
    assert!(
        is_content_line(&state.doc.line_map, state.cursor_line),
        "cursor_line {} is not a content line",
        state.cursor_line
    );
}

#[test]
fn sequence_toggle_single_file_tree_enter_context_regenerate() {
    let files: Vec<DiffFile> = vec![make_diff_file("a.rs"), make_diff_file("b.rs"), make_diff_file("c.rs")];
    let mut state = make_keybinding_state();

    handle_key(&mut state, Key::Char('a'), 40, 40, &files);
    assert_state_invariants(&state);

    handle_key(&mut state, Key::Tab, 40, 40, &files);
    assert_state_invariants(&state);

    handle_key(&mut state, Key::Enter, 40, 40, &files);
    assert_state_invariants(&state);

    let result = handle_key(&mut state, Key::Char('z'), 40, 40, &files);
    assert_state_invariants(&state);
    if matches!(result, KeyResult::ReGenerate) {
        re_render(&mut state, &files, false, 80);
    }
    assert_state_invariants(&state);
}

#[test]
fn sequence_du_in_both_context_modes_with_tree_focus_changes() {
    let mut state = make_mixed_content_state();
    let files: Vec<DiffFile> = vec![];

    state.full_context = false;
    handle_key(&mut state, Key::Char('d'), 40, 40, &files);
    assert_state_invariants(&state);
    handle_key(&mut state, Key::Char('u'), 40, 40, &files);
    assert_state_invariants(&state);

    state.set_tree_focused(true);
    let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
    state.tree_lines = tl;
    state.tree_visible_to_entry = tv;

    handle_key(&mut state, Key::Char('d'), 40, 40, &files);
    assert_state_invariants(&state);
    handle_key(&mut state, Key::Char('u'), 40, 40, &files);
    assert_state_invariants(&state);

    state.full_context = true;
    handle_key(&mut state, Key::Char('d'), 40, 40, &files);
    assert_state_invariants(&state);
    handle_key(&mut state, Key::Char('d'), 40, 40, &files);
    assert_state_invariants(&state);
    handle_key(&mut state, Key::Char('u'), 40, 40, &files);
    assert_state_invariants(&state);

    state.set_tree_focused(false);
    handle_key(&mut state, Key::Char('d'), 40, 40, &files);
    assert_state_invariants(&state);
}

#[test]
fn sequence_resize_rerender_in_search_and_visual_overlays() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, true);

    handle_key(&mut state, Key::Char('/'), 40, 40, &files);
    state.search_input = "first".to_string();
    state.search_cursor = 5;
    state.search_query = "first".to_string();
    state.search_matches = find_matches(&state.doc.lines, "first");
    state.current_match = find_nearest_match(&state.search_matches, state.top_line);
    state.mode = Mode::Search;
    assert_state_invariants(&state);

    re_render(&mut state, &files, false, 40);
    assert_state_invariants(&state);

    handle_key(&mut state, Key::Escape, 40, 40, &files);
    handle_key(&mut state, Key::Char('v'), 40, 40, &files);
    assert_state_invariants(&state);

    re_render(&mut state, &files, false, 60);
    assert_state_invariants(&state);

    handle_key(&mut state, Key::Escape, 40, 40, &files);
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
        Key::Char('d'),
        Key::Char('u'),
        Key::Char('g'),
        Key::Char('G'),
        Key::CtrlD,
        Key::CtrlU,
        Key::Tab,
        Key::Escape,
        Key::Enter,
        Key::Char('a'),
        Key::Char('z'),
        Key::Char('e'),
        Key::Char('v'),
        Key::Char('y'),
        Key::Char('h'),
        Key::Char('l'),
        Key::Char('1'),
        Key::Char('D'),
        Key::Char('U'),
    ];

    for step in 0..72 {
        let key_idx = (rng as usize) % keys.len();
        let key = keys[key_idx];
        rng = rng.wrapping_mul(1_103_515_245).wrapping_add(12_345);

        let ch = 24 + ((rng >> 16) as usize % 20);
        let rows = 40;
        let _ = handle_key(&mut state, key, ch, rows, &files);
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

#[test]
fn reducer_overlay_focus_transitions_produce_valid_state() {
    let files = vec![make_diff_file("a.rs"), make_diff_file("b.rs"), make_diff_file("c.rs")];
    let mut state = make_keybinding_state();

    let _ = handle_key(&mut state, Key::Tab, 40, 40, &files);
    assert!(state.tree_focused());
    assert_state_invariants(&state);

    let _ = handle_key(&mut state, Key::Escape, 40, 40, &files);
    assert!(!state.tree_focused());
    assert_state_invariants(&state);

    let _ = handle_key(&mut state, Key::Char('v'), 40, 40, &files);
    assert_eq!(state.mode, Mode::Visual);
    assert_state_invariants(&state);

    let _ = handle_key(&mut state, Key::Char('y'), 40, 40, &files);
    assert_eq!(state.mode, Mode::Normal);
    assert_state_invariants(&state);

    let _ = handle_key(&mut state, Key::Char('e'), 40, 40, &files);
    state.set_active_file(Some(1));
    let _ = handle_key(&mut state, Key::Char('e'), 40, 40, &files);
    assert!(!state.tree_visible);
    assert_state_invariants(&state);
}
