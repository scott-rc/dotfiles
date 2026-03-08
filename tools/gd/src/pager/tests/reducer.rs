//! Reducer (handle_key) integration tests.

use crate::git::diff::{DiffFile, LineKind};
use insta::assert_debug_snapshot;
use tui::pager::Key;
use tui::search::{find_matches, find_nearest_match};

use super::super::reducer::handle_key;
use super::super::runtime::re_render;
use super::super::state::visible_range;
use super::super::types::{FocusPane, KeyResult, Mode};
use std::path::Path;

use super::common::{
    StateSnapshot, add_leading_context_before_hunk_changes, assert_state_invariants, entry,
    make_diff_file, make_keybinding_state, make_mixed_content_state, make_pager_state_from_files,
    make_staging_state, make_two_file_diff,
};

fn p() -> &'static Path {
    Path::new(".")
}

// ---- Navigation: j/k scroll ----

#[test]
fn key_j_next_content_line() {
    let mut state = make_keybinding_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_k_prev_content_line() {
    let mut state = make_keybinding_state();
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char('k'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_j_skips_headers() {
    let mut state = make_keybinding_state();
    state.cursor_line = 4;
    handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_g_jumps_to_first_content() {
    let mut state = make_keybinding_state();
    state.cursor_line = 15;
    handle_key(&mut state, Key::Char('g'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_G_jumps_to_last_content() {
    let mut state = make_keybinding_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('G'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Navigation: d/u half page ----

#[test]
fn key_d_half_page_down() {
    let mut state = make_keybinding_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('d'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_u_half_page_up() {
    let mut state = make_keybinding_state();
    state.cursor_line = 25;
    handle_key(&mut state, Key::Char('u'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Navigation: z center viewport ----

#[test]
fn key_z_centers_viewport() {
    let mut state = make_keybinding_state();
    state.cursor_line = 40;
    state.top_line = 0;
    handle_key(&mut state, Key::Char('z'), 20, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(
        state.top_line > 0,
        "z should center viewport around cursor, moving top_line from 0"
    );
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Diff nav: ]/[ hunk ----

#[test]
fn key_bracket_next_hunk_same_file() {
    let mut state = make_mixed_content_state();
    state.cursor_line = 8;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_prev_hunk_same_file() {
    let mut state = make_mixed_content_state();
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_prev_hunk_from_first_content_line() {
    let mut state = make_mixed_content_state();
    state.cursor_line = 16;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.cursor_line, 10);
}

#[test]
fn key_bracket_prev_hunk_single_file_retreats_to_prev_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.cursor_line = 36;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    // At the first hunk of file 1, [ should retreat to previous file
    assert_eq!(
        state.active_file(),
        Some(0),
        "[ at first hunk should retreat to prev file"
    );
    assert!(
        state.cursor_line < 30,
        "cursor should be in file 0 range, got {}",
        state.cursor_line
    );
}

#[test]
fn key_bracket_next_hunk_cross_file_boundary() {
    let mut state = make_mixed_content_state();
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_next_hunk_scrolloff_binding() {
    let mut state = make_mixed_content_state();
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char(']'), 15, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_next_hunk_at_last_is_noop() {
    let mut state = make_mixed_content_state();
    state.set_active_file(None);
    state.cursor_line = 76;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_prev_hunk_at_first_is_noop() {
    let mut state = make_mixed_content_state();
    state.set_active_file(None);
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_no_active_file_does_not_stick() {
    let mut state = make_mixed_content_state();
    state.set_active_file(None);
    state.cursor_line = 5;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Diff nav: }/{ file ----

#[test]
fn key_brace_next_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_prev_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('{'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_prev_file_no_active_stuck_cursor() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('{'), 50, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_next_file_no_active_file_does_not_stick() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 0;
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_next_file_at_last_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 66;
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_prev_file_at_first_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('{'), 50, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_next_file_single_file_switches() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
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
    handle_key(&mut state, Key::Char('{'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
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
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.active_file(), Some(2));
}

#[test]
fn key_brace_prev_file_single_file_first_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('{'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.active_file(), Some(0));
}

#[test]
fn key_brace_shows_file_status_message() {
    let mut state = make_mixed_content_state();
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_prev_shows_file_status_message() {
    let mut state = make_mixed_content_state();
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('{'), 50, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- Hunk nav in single-file mode ----

#[test]
fn key_bracket_single_file_advances_to_next_file_at_last_hunk() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    // At the last hunk of file 0 (hunks [5, 15]), ] should advance to next file
    assert_eq!(
        state.active_file(),
        Some(1),
        "] at last hunk should advance to next file"
    );
    assert!(
        state.cursor_line >= 30 && state.cursor_line < 60,
        "cursor should be in file 1 range, got {}",
        state.cursor_line
    );
}

#[test]
fn key_bracket_prev_single_file_retreats_to_prev_file_at_first_hunk() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(1));
    state.cursor_line = 36;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    // At first hunk of file 1, [ should retreat to previous file
    assert_eq!(
        state.active_file(),
        Some(0),
        "[ at first hunk should retreat to prev file"
    );
    assert!(
        state.cursor_line < 30,
        "cursor should be in file 0 range, got {}",
        state.cursor_line
    );
}

#[test]
fn key_bracket_single_file_last_file_at_last_hunk_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(2));
    state.cursor_line = 76;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    // At the last hunk of the last file, ] should be a no-op
    assert_eq!(state.active_file(), Some(2));
    assert_eq!(state.cursor_line, 76);
}

#[test]
fn key_bracket_single_file_first_file_at_first_hunk_is_noop() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    // At the first hunk of file 0, [ with no previous file should be a no-op
    assert_eq!(state.active_file(), Some(0));
    assert_eq!(state.cursor_line, 6);
}

#[test]
fn key_bracket_single_file_within_file_works() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.cursor_line, 10);
}

#[test]
fn key_bracket_single_file_clamps_top_line_to_active_file_range() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.cursor_line = 16;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    let (range_start, _range_end) = visible_range(&state);
    assert!(state.top_line >= range_start);
}

#[test]
fn key_bracket_next_advance_lands_on_first_change_group() {
    // When ] advances to the next file, cursor should land on the first
    // change group of that file, not the file header.
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    // Position at the last change group of file 0 (Deleted at 10-11)
    state.cursor_line = 10;
    // First ] stays within file 0 -- no more change groups after 10
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    // Should advance to file 1 and land on first change group (Added at 36)
    assert_eq!(
        state.active_file(),
        Some(1),
        "] should advance to next file"
    );
    assert_eq!(
        state.cursor_line, 36,
        "cursor should land on first change group of new file (Added at 36), not file header"
    );
}

#[test]
fn key_bracket_prev_retreat_lands_on_last_change_group() {
    // When [ retreats to the previous file, cursor should land on the last
    // change group of that file, not the file header.
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(1));
    // Position at the first change group of file 1 (Added at 36)
    state.cursor_line = 36;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    // Should retreat to file 0 and land on last change group (Deleted at 10)
    assert_eq!(
        state.active_file(),
        Some(0),
        "[ should retreat to prev file"
    );
    assert_eq!(
        state.cursor_line, 10,
        "cursor should land on last change group of prev file (Deleted at 10), not file header"
    );
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
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_state_invariants(&state);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_bracket_hunk_context_skips_leading_context_to_first_change() {
    let mut state = make_mixed_content_state();
    add_leading_context_before_hunk_changes(&mut state);
    state.full_context = false;
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.cursor_line, 8);
}

#[test]
fn key_bracket_prev_hunk_context_navigates_to_prev_change_group() {
    let mut state = make_mixed_content_state();
    add_leading_context_before_hunk_changes(&mut state);
    state.full_context = false;
    state.cursor_line = 17;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.cursor_line, 10);
}

#[test]
fn key_bracket_full_context_single_file_lands_on_change_group() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.cursor_line, 6);
}

#[test]
fn key_bracket_full_context_same_targets_as_hunk_mode() {
    // In mixed_content_state, file 0 has hunks at [5, 15].
    // Hunk 5 first change is at 6 (Added). Hunk 15 has no changes (all Context).
    // In both hunk mode and full context mode, ] from pos 1 should land on 6,
    // and ] again should behave the same way in both modes.
    let mut hunk_state = make_mixed_content_state();
    hunk_state.set_active_file(Some(0));
    hunk_state.full_context = false;
    hunk_state.cursor_line = 1;

    let mut ctx_state = make_mixed_content_state();
    ctx_state.set_active_file(Some(0));
    ctx_state.full_context = true;
    ctx_state.cursor_line = 1;

    // First ] should land on same position in both modes
    handle_key(&mut hunk_state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    handle_key(&mut ctx_state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(
        hunk_state.cursor_line, ctx_state.cursor_line,
        "first ] should land on same position: hunk={} full_ctx={}",
        hunk_state.cursor_line, ctx_state.cursor_line
    );

    // Second ] should also land on same position
    handle_key(&mut hunk_state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    handle_key(&mut ctx_state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(
        hunk_state.cursor_line, ctx_state.cursor_line,
        "second ] should land on same position: hunk={} full_ctx={}",
        hunk_state.cursor_line, ctx_state.cursor_line
    );
}

#[test]
fn key_bracket_full_context_advances_to_next_file() {
    // In full context mode + single file, ] at the last hunk should also advance
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    // Navigate to the last change in file 0
    state.cursor_line = 10; // Deleted group at 10-11
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    // If there are no more targets in file 0, should advance to file 1
    // (depends on mixed_content_state's layout for file 0)
    assert_state_invariants(&state);
}

#[test]
fn key_bracket_prev_full_context_single_file_at_first_change_is_noop() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 7;
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.cursor_line, 6);
}

#[test]
fn key_bracket_then_prev_round_trip_full_context_single_file() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 6;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    let after_next = state.cursor_line;
    assert!(
        after_next > 6,
        "] should move forward from 6, got {after_next}"
    );
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    let after_prev = state.cursor_line;
    assert_eq!(after_prev, 6, "[ should return to first change");
}

#[test]
fn key_bracket_full_context_all_context_file_advances_to_next_file() {
    // In full context mode, an all-Context file has no change groups to navigate.
    // ] advances to the next file.
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.full_context = true;
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- s toggle single file ----

#[test]
fn key_s_toggles_off_single_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_s_toggles_on_single_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(None);
    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_s_still_toggles_single_file() {
    let mut state = make_keybinding_state();
    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.active_file(), Some(0));
    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.active_file(), None);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn normal_s_toggle_on_lands_on_file_header() {
    let mut state = make_keybinding_state();
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    // Cursor lands on file header so ] (jump_next, strictly >) can find the first change group
    let file_start = state.file_start(state.active_file().unwrap()).unwrap();
    assert_eq!(
        state.cursor_line, file_start,
        "cursor should land on file header (start of file range), got {}",
        state.cursor_line
    );
}

// ---- o toggle full context ----

#[test]
fn key_o_toggles_full_context() {
    let mut state = make_keybinding_state();
    handle_key(&mut state, Key::Char('o'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.full_context);
}

#[test]
fn key_o_toggles_hunk_context() {
    let mut state = make_keybinding_state();
    state.full_context = true;
    handle_key(&mut state, Key::Char('o'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(!state.full_context);
}

#[test]
fn key_space_is_noop_for_full_context_toggle() {
    let mut state = make_keybinding_state();
    state.full_context = false;
    handle_key(&mut state, Key::Char(' '), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(!state.full_context);
}

#[test]
fn key_space_is_noop_for_context_toggle() {
    let mut state = make_keybinding_state();
    state.full_context = true;
    handle_key(&mut state, Key::Char(' '), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.full_context);
}

// ---- / search ----

#[test]
fn key_slash_enters_search() {
    let mut state = make_keybinding_state();
    handle_key(&mut state, Key::Char('/'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

// ---- n/N match navigation ----

#[test]
fn key_n_wraps_within_single_file() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![6, 36, 66];
    state.current_match = 0;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('n'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_N_wraps_within_single_file() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![6, 36, 66];
    state.current_match = 0;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('N'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_n_no_matches_in_active_file() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![36, 66];
    state.current_match = -1;
    state.set_active_file(Some(0));
    handle_key(&mut state, Key::Char('n'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_n_after_toggling_single_file_off_cycles_globally() {
    let mut state = make_mixed_content_state();
    state.search_matches = vec![6, 36, 66];
    state.current_match = 0;
    state.set_active_file(None);
    handle_key(&mut state, Key::Char('n'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn test_key_n_single_file_moves_to_next_match() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    state.search_matches = vec![5, 15];
    state.current_match = 0;
    handle_key(&mut state, Key::Char('n'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
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
    handle_key(&mut state, Key::Char('N'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.current_match, 0);
    assert_eq!(state.cursor_line, 5);
}

#[test]
fn test_key_n_empty_matches_noop() {
    let mut state = make_keybinding_state();
    state.search_matches = vec![];
    state.current_match = -1;
    handle_key(&mut state, Key::Char('n'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.current_match, -1);
}

#[test]
#[allow(non_snake_case)]
fn test_key_N_empty_matches_noop() {
    let mut state = make_keybinding_state();
    state.search_matches = vec![];
    state.current_match = -1;
    handle_key(&mut state, Key::Char('N'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
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
    handle_key(&mut state, Key::Char('l'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.tree_visible, "l should show tree");
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_l_toggles_tree_off() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Char('l'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
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
    handle_key(&mut state, Key::Char('l'), 40, 40, cols, &files, p(), &crate::git::DiffSource::WorkingTree);
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
    handle_key(&mut state, Key::Char('s'), 40, 40, cols, &files, p(), &crate::git::DiffSource::WorkingTree);
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
    handle_key(&mut state, Key::Char('l'), 40, 40, cols, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.tree_visible, "l should still toggle tree on");
    // Fallback tree_width must not make the diff unusable
    assert!(
        state.tree_width + MIN_DIFF_WIDTH < cols as usize,
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
    handle_key(&mut state, Key::Char('v'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.visual_anchor, Some(10));
    assert!(state.status_message.contains("VISUAL"));
}

#[test]
fn key_y_without_selection_shows_error() {
    let mut state = make_keybinding_state();
    state.visual_anchor = None;
    handle_key(&mut state, Key::Char('y'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.status_message.contains("No selection"));
}

#[test]
fn key_y_with_selection_clears_anchor() {
    let mut state = make_keybinding_state();
    state.visual_anchor = Some(5);
    state.cursor_line = 10;
    handle_key(&mut state, Key::Char('y'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.visual_anchor, None, "yank should clear visual anchor");
}

#[test]
fn key_esc_clears_visual_selection() {
    let mut state = make_keybinding_state();
    state.visual_anchor = Some(5);
    handle_key(&mut state, Key::Escape, 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.visual_anchor, None, "Esc should clear visual anchor");
}

#[test]
fn key_esc_without_selection_is_noop() {
    let mut state = make_keybinding_state();
    let cursor_before = state.cursor_line;
    handle_key(&mut state, Key::Escape, 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.cursor_line, cursor_before);
    assert_eq!(state.visual_anchor, None);
}

// ---- R reload ----

#[test]
#[allow(non_snake_case)]
fn key_R_returns_regenerate() {
    let mut state = make_keybinding_state();
    let result = handle_key(&mut state, Key::Char('R'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(
        matches!(result, KeyResult::ReGenerate),
        "R should return ReGenerate, got {result:?}"
    );
    assert!(state.status_message.contains("Reload"));
}

#[test]
#[allow(non_snake_case)]
fn key_R_preserves_visual_selection() {
    let mut state = make_keybinding_state();
    state.visual_anchor = Some(5);
    let result = handle_key(&mut state, Key::Char('R'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(matches!(result, KeyResult::ReGenerate));
    assert_eq!(
        state.visual_anchor,
        Some(5),
        "Reload should not clear visual selection"
    );
}

// ---- ? toggle tooltip ----

#[test]
fn key_question_toggles_tooltip() {
    let mut state = make_keybinding_state();
    assert!(!state.tooltip_visible);
    handle_key(&mut state, Key::Char('?'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.tooltip_visible, "? should show tooltip");
    handle_key(&mut state, Key::Char('?'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(!state.tooltip_visible, "? again should hide tooltip");
}

// ---- Single-file boundary tests ----

#[test]
fn key_g_single_file_lands_on_file_start() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(1));
    state.cursor_line = 50;
    handle_key(&mut state, Key::Char('g'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
#[allow(non_snake_case)]
fn key_G_single_file_lands_on_file_end() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.cursor_line = 1;
    handle_key(&mut state, Key::Char('G'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_d_single_file_clamps_to_file_end() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.cursor_line = 25;
    handle_key(&mut state, Key::Char('d'), 20, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_u_single_file_clamps_to_file_start() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(1));
    state.cursor_line = 32;
    handle_key(&mut state, Key::Char('u'), 20, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_j_at_last_content_line_of_single_file_is_noop() {
    let mut state = make_mixed_content_state();
    state.set_active_file(Some(0));
    state.cursor_line = 29;
    handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_debug_snapshot!(StateSnapshot::from(&state));
}

#[test]
fn key_brace_prev_no_active_file_at_file_boundary() {
    let mut state = make_mixed_content_state();
    state.set_active_file(None);
    state.cursor_line = 31;
    handle_key(&mut state, Key::Char('{'), 50, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
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

    handle_key(&mut state, Key::Char('s'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_state_invariants(&state);

    let result = handle_key(&mut state, Key::Char('o'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
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
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_state_invariants(&state);
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_state_invariants(&state);

    state.full_context = true;
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_state_invariants(&state);
    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_state_invariants(&state);
    handle_key(&mut state, Key::Char('['), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_state_invariants(&state);

    handle_key(&mut state, Key::Char(']'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_state_invariants(&state);
}

#[test]
fn sequence_resize_rerender_in_search() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, true);

    handle_key(&mut state, Key::Char('/'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    state.search_input = "first".to_string();
    state.search_cursor = 5;
    state.search_query = "first".to_string();
    state.search_matches = find_matches(&state.doc.lines, "first");
    state.current_match = find_nearest_match(&state.search_matches, state.top_line);
    state.mode = Mode::Search;
    assert_state_invariants(&state);

    re_render(&mut state, &files, false, 40);
    assert_state_invariants(&state);

    handle_key(&mut state, Key::Escape, 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
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
        Key::Char('R'),
    ];

    for step in 0..72 {
        let key_idx = (rng as usize) % keys.len();
        let key = keys[key_idx];
        rng = rng.wrapping_mul(1_103_515_245).wrapping_add(12_345);

        let ch = 24 + ((rng >> 16) as usize % 20);
        let rows = 40;
        let _ = handle_key(&mut state, key, ch, rows, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
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

// ---- Staging actions ----

#[test]
fn stage_line_working_tree_returns_apply_patch() {
    let (mut state, files) = make_staging_state();
    // Position cursor on a changed line (find an Added line)
    let added_line = state.doc.line_map.iter().position(|li| li.line_kind == Some(LineKind::Added)).unwrap();
    state.cursor_line = added_line;
    let result = handle_key(&mut state, Key::Char('a'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert!(
        matches!(result, KeyResult::ApplyPatch { cached: true, reverse: false, .. }),
        "stage line on WorkingTree should return ApplyPatch(cached=true, reverse=false), got {result:?}"
    );
}

#[test]
fn stage_hunk_working_tree_returns_apply_patch() {
    let (mut state, files) = make_staging_state();
    let added_line = state.doc.line_map.iter().position(|li| li.line_kind == Some(LineKind::Added)).unwrap();
    state.cursor_line = added_line;
    let result = handle_key(&mut state, Key::Char('A'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert!(
        matches!(result, KeyResult::ApplyPatch { cached: true, reverse: false, .. }),
        "stage hunk on WorkingTree should return ApplyPatch(cached=true, reverse=false), got {result:?}"
    );
}

#[test]
fn discard_line_working_tree_returns_apply_patch() {
    let (mut state, files) = make_staging_state();
    let added_line = state.doc.line_map.iter().position(|li| li.line_kind == Some(LineKind::Added)).unwrap();
    state.cursor_line = added_line;
    let result = handle_key(&mut state, Key::Char('x'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert!(
        matches!(result, KeyResult::ApplyPatch { cached: false, reverse: true, .. }),
        "discard line on WorkingTree should return ApplyPatch(cached=false, reverse=true), got {result:?}"
    );
}

#[test]
fn discard_hunk_working_tree_returns_apply_patch() {
    let (mut state, files) = make_staging_state();
    let added_line = state.doc.line_map.iter().position(|li| li.line_kind == Some(LineKind::Added)).unwrap();
    state.cursor_line = added_line;
    let result = handle_key(&mut state, Key::Char('X'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert!(
        matches!(result, KeyResult::ApplyPatch { cached: false, reverse: true, .. }),
        "discard hunk on WorkingTree should return ApplyPatch(cached=false, reverse=true), got {result:?}"
    );
}

#[test]
fn stage_line_commit_view_is_disabled() {
    let (mut state, files) = make_staging_state();
    let added_line = state.doc.line_map.iter().position(|li| li.line_kind == Some(LineKind::Added)).unwrap();
    state.cursor_line = added_line;
    let result = handle_key(&mut state, Key::Char('a'), 40, 40, 120, &files, p(), &crate::git::DiffSource::Commit("abc".into()));
    assert!(
        matches!(result, KeyResult::Continue),
        "stage in commit view should return Continue, got {result:?}"
    );
    assert!(state.status_message.contains("Cannot"), "should show Cannot message, got: {}", state.status_message);
}

#[test]
fn stage_line_range_view_is_disabled() {
    let (mut state, files) = make_staging_state();
    let added_line = state.doc.line_map.iter().position(|li| li.line_kind == Some(LineKind::Added)).unwrap();
    state.cursor_line = added_line;
    let result = handle_key(&mut state, Key::Char('a'), 40, 40, 120, &files, p(), &crate::git::DiffSource::Range("a".into(), "b".into()));
    assert!(
        matches!(result, KeyResult::Continue),
        "stage in range view should return Continue, got {result:?}"
    );
    assert!(state.status_message.contains("Cannot"), "should show Cannot message, got: {}", state.status_message);
}

#[test]
fn stage_line_with_visual_selection_stages_range() {
    let (mut state, files) = make_staging_state();
    // Find first and last content lines
    let first_content = state.doc.line_map.iter().position(|li| li.line_kind.is_some()).unwrap();
    let last_content = state.doc.line_map.iter().rposition(|li| li.line_kind.is_some()).unwrap();
    state.visual_anchor = Some(first_content);
    state.cursor_line = last_content;
    let result = handle_key(&mut state, Key::Char('a'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert!(
        matches!(result, KeyResult::ApplyPatch { cached: true, reverse: false, .. }),
        "stage line with visual selection should return ApplyPatch, got {result:?}"
    );
    assert_eq!(state.visual_anchor, None, "visual anchor should be cleared after staging");
}

// ---- Focus mode (t) ----

#[test]
fn focus_toggle_shows_tree_and_focuses_it() {
    let mut state = make_keybinding_state();
    state.tree_visible = false;
    state.focus = FocusPane::Diff;
    let files = vec![
        make_diff_file("a.rs"),
        make_diff_file("b.rs"),
        make_diff_file("c.rs"),
    ];
    handle_key(&mut state, Key::Char('t'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.tree_visible, "t should show tree when hidden");
    assert_eq!(state.focus, FocusPane::Tree, "t should focus tree");
}

#[test]
fn focus_toggle_switches_between_panes() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.focus = FocusPane::Diff;
    let files = vec![
        make_diff_file("a.rs"),
        make_diff_file("b.rs"),
        make_diff_file("c.rs"),
    ];
    // First t: focus tree
    handle_key(&mut state, Key::Char('t'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.focus, FocusPane::Tree);
    // Second t: back to diff
    handle_key(&mut state, Key::Char('t'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.focus, FocusPane::Diff, "second t should return focus to diff");
}

#[test]
fn escape_in_tree_focus_returns_to_diff() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    handle_key(&mut state, Key::Escape, 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.focus, FocusPane::Diff, "Escape should return focus to diff");
}

#[test]
fn j_in_tree_focus_moves_tree_cursor() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(0);
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.tree_cursor(), 1, "j in tree focus should advance tree cursor");
}

#[test]
fn k_in_tree_focus_moves_cursor_up() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(1);
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Char('k'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.tree_cursor(), 0, "k in tree focus should move tree cursor up");
}

#[test]
fn k_in_tree_focus_clamps_at_zero() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(0);
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Char('k'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.tree_cursor(), 0, "k at top should clamp at 0");
}

#[test]
fn j_in_diff_focus_scrolls_diff() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.focus = FocusPane::Diff;
    state.cursor_line = 1;
    let cursor_before = state.cursor_line;
    handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.cursor_line > cursor_before, "j in diff focus should scroll diff");
}

// ---- TreeEnter (context-sensitive Enter in tree) ----

#[test]
fn tree_enter_on_collapsed_directory_expands_it() {
    let mut state = make_keybinding_state();
    // Replace tree_entries with a directory + files structure
    state.tree_entries = vec![
        entry("src", 0, None),      // 0: directory
        entry("a.rs", 1, Some(0)),  // 1: file
        entry("b.rs", 1, Some(1)),  // 2: file
        entry("c.rs", 0, Some(2)),  // 3: file
    ];
    state.tree_entries[0].collapsed = true;
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(0);
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Enter, 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(!state.tree_entries[0].collapsed, "Enter on collapsed directory should expand it");
}

#[test]
fn tree_enter_on_expanded_directory_collapses_it() {
    let mut state = make_keybinding_state();
    state.tree_entries = vec![
        entry("src", 0, None),      // 0: directory
        entry("a.rs", 1, Some(0)),  // 1: file
        entry("b.rs", 1, Some(1)),  // 2: file
        entry("c.rs", 0, Some(2)),  // 3: file
    ];
    state.tree_entries[0].collapsed = false;
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(0);
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Enter, 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.tree_entries[0].collapsed, "Enter on expanded directory should collapse it");
}

#[test]
fn tree_enter_on_file_jumps_cursor_and_keeps_tree_focus() {
    let mut state = make_keybinding_state();
    // tree_entries: flat file list (default from make_keybinding_state has a.rs, b.rs, c.rs)
    // file_starts = [0, 30, 60], so file 1 starts at line 30
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    // Select tree entry for b.rs (index 1, file_idx=Some(1))
    state.set_tree_cursor(1);
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Enter, 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.focus, FocusPane::Tree, "Enter on file should keep focus on tree");
    // cursor_line should be at or near file_starts[1] = 30
    assert!(
        state.cursor_line >= 30 && state.cursor_line <= 31,
        "cursor should jump to file start (30), got {}",
        state.cursor_line
    );
}

#[test]
fn tree_space_on_file_jumps_cursor_and_keeps_tree_focus() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(1);
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Char(' '), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.focus, FocusPane::Tree, "Space on file should keep focus on tree");
    assert!(
        state.cursor_line >= 30 && state.cursor_line <= 31,
        "cursor should jump to file start (30), got {}",
        state.cursor_line
    );
}

#[test]
fn tree_enter_on_file_in_single_file_mode_switches_active_file() {
    let mut state = make_keybinding_state();
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    // Start in single-file mode on file 0
    state.set_active_file(Some(0));
    // Select tree entry for b.rs (index 1, file_idx=Some(1))
    state.set_tree_cursor(1);
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Enter, 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.focus, FocusPane::Tree, "Enter should keep focus on tree");
    assert_eq!(state.active_file(), Some(1), "active file should switch to file 1");
    assert!(
        state.cursor_line >= 30 && state.cursor_line <= 31,
        "cursor should be at file 1 start (30), got {}",
        state.cursor_line
    );
}

#[test]
fn enter_when_diff_focused_scrolls_down() {
    let mut state = make_keybinding_state();
    state.focus = FocusPane::Diff;
    state.cursor_line = 1;
    let cursor_before = state.cursor_line;
    handle_key(&mut state, Key::Enter, 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.cursor_line > cursor_before, "Enter in diff focus should scroll down");
}

// ---- za / zA collapse control ----

#[test]
fn za_on_expanded_directory_collapses_it() {
    let mut state = make_keybinding_state();
    state.tree_entries = vec![
        entry("src", 0, None),
        entry("a.rs", 1, Some(0)),
        entry("b.rs", 1, Some(1)),
        entry("c.rs", 0, Some(2)),
    ];
    state.tree_entries[0].collapsed = false;
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(0);
    state.rebuild_tree_lines();
    // Press z then a
    handle_key(&mut state, Key::Char('z'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    handle_key(&mut state, Key::Char('a'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.tree_entries[0].collapsed, "za on expanded directory should collapse it");
    assert!(state.collapsed_paths.contains("src"), "collapsed_paths should track 'src'");
}

#[test]
fn za_on_collapsed_directory_expands_it() {
    let mut state = make_keybinding_state();
    state.tree_entries = vec![
        entry("src", 0, None),
        entry("a.rs", 1, Some(0)),
        entry("b.rs", 1, Some(1)),
        entry("c.rs", 0, Some(2)),
    ];
    state.tree_entries[0].collapsed = true;
    state.collapsed_paths.insert("src".to_string());
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(0);
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Char('z'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    handle_key(&mut state, Key::Char('a'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(!state.tree_entries[0].collapsed, "za on collapsed directory should expand it");
    assert!(!state.collapsed_paths.contains("src"), "collapsed_paths should remove 'src'");
}

#[test]
fn za_collapses_directory_and_all_descendants() {
    let mut state = make_keybinding_state();
    state.tree_entries = vec![
        entry("src", 0, None),       // 0
        entry("lib", 1, None),       // 1
        entry("a.rs", 2, Some(0)),   // 2
        entry("bin", 1, None),       // 3
        entry("b.rs", 2, Some(1)),   // 4
        entry("c.rs", 0, Some(2)),   // 5
    ];
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(0);
    state.rebuild_tree_lines();
    // Press z then A (recursive)
    handle_key(&mut state, Key::Char('z'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    handle_key(&mut state, Key::Char('A'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(state.tree_entries[0].collapsed, "zA should collapse cursor dir");
    assert!(state.tree_entries[1].collapsed, "zA should collapse descendant dir 'lib'");
    assert!(state.tree_entries[3].collapsed, "zA should collapse descendant dir 'bin'");
}

#[test]
fn za_on_collapsed_expands_all_descendants() {
    let mut state = make_keybinding_state();
    state.tree_entries = vec![
        entry("src", 0, None),
        entry("lib", 1, None),
        entry("a.rs", 2, Some(0)),
        entry("bin", 1, None),
        entry("b.rs", 2, Some(1)),
        entry("c.rs", 0, Some(2)),
    ];
    // Collapse all directories
    state.tree_entries[0].collapsed = true;
    state.tree_entries[1].collapsed = true;
    state.tree_entries[3].collapsed = true;
    state.collapsed_paths.insert("src".to_string());
    state.collapsed_paths.insert("src/lib".to_string());
    state.collapsed_paths.insert("src/bin".to_string());
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(0);
    state.rebuild_tree_lines();
    // Press z then A (recursive expand)
    handle_key(&mut state, Key::Char('z'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    handle_key(&mut state, Key::Char('A'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(!state.tree_entries[0].collapsed, "zA should expand cursor dir");
    assert!(!state.tree_entries[1].collapsed, "zA should expand descendant 'lib'");
    assert!(!state.tree_entries[3].collapsed, "zA should expand descendant 'bin'");
}

#[test]
fn za_on_file_entry_is_noop() {
    let mut state = make_keybinding_state();
    state.tree_entries = vec![
        entry("src", 0, None),
        entry("a.rs", 1, Some(0)),
        entry("b.rs", 1, Some(1)),
        entry("c.rs", 0, Some(2)),
    ];
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(1); // file entry
    state.rebuild_tree_lines();
    let lines_before = state.tree_lines.len();
    handle_key(&mut state, Key::Char('z'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    handle_key(&mut state, Key::Char('a'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.tree_lines.len(), lines_before, "za on file entry should be a noop");
}

#[test]
fn z_followed_by_non_a_cancels_pending() {
    let mut state = make_keybinding_state();
    state.tree_entries = vec![
        entry("src", 0, None),
        entry("a.rs", 1, Some(0)),
        entry("b.rs", 1, Some(1)),
        entry("c.rs", 0, Some(2)),
    ];
    state.tree_entries[0].collapsed = false;
    state.tree_visible = true;
    state.focus = FocusPane::Tree;
    state.set_tree_cursor(0);
    state.rebuild_tree_lines();
    handle_key(&mut state, Key::Char('z'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    handle_key(&mut state, Key::Char('x'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert!(!state.tree_entries[0].collapsed, "zx should not toggle collapse");
    assert!(state.pending_tree_key.is_none(), "pending should be cleared after non-a key");
}

#[test]
fn collapsed_paths_survives_remap_after_document_swap() {
    use crate::pager::state::{capture_view_anchor, remap_after_document_swap, Document};
    use crate::render;

    let files = vec![
        make_diff_file("src/a.rs"),
        make_diff_file("src/b.rs"),
    ];
    let mut state = make_pager_state_from_files(&files, true);
    // Collapse the "src" directory and track it
    if let Some(dir_idx) = state.tree_entries.iter().position(|e| e.file_idx.is_none()) {
        state.tree_entries[dir_idx].collapsed = true;
        state.collapsed_paths.insert("src".to_string());
    }
    state.rebuild_tree_lines();
    let anchor = capture_view_anchor(&state);
    let output = render::render(&files, 80, false);
    let new_doc = Document::from_render_output(output);
    remap_after_document_swap(&mut state, anchor, new_doc, &files, 120);
    // The "src" directory should still be collapsed after remap
    if let Some(dir_idx) = state.tree_entries.iter().position(|e| e.file_idx.is_none()) {
        assert!(
            state.tree_entries[dir_idx].collapsed,
            "collapsed state should survive remap_after_document_swap"
        );
    } else {
        panic!("expected a directory entry after remap");
    }
}

#[test]
fn single_child_chain_defaults_to_collapsed() {
    let files = vec![make_diff_file("src/lib/foo.rs")];
    let entries = super::super::tree::build_tree_entries(&files);
    let chain_dir = entries.iter().find(|e| e.file_idx.is_none());
    assert!(chain_dir.is_some(), "should have a dir entry");
    assert!(
        chain_dir.unwrap().collapsed,
        "single-child chain directory should default to collapsed"
    );
}

// ---- Tree-driven file jumping via NextFile/PrevFile ----

use super::super::tree::TreeEntry;
use crate::git::diff::FileStatus;
use crate::render::LineInfo;
use super::super::state::PagerState;

/// Build a state with a collapsed directory hiding its children from tree_visible_to_entry.
fn make_tree_file_jump_state() -> PagerState {
    let tree_entries = vec![
        TreeEntry { label: "src".into(), depth: 0, file_idx: None, status: None, collapsed: true },
        TreeEntry { label: "a.rs".into(), depth: 1, file_idx: Some(0), status: Some(FileStatus::Modified), collapsed: false },
        TreeEntry { label: "b.rs".into(), depth: 1, file_idx: Some(1), status: Some(FileStatus::Modified), collapsed: false },
        TreeEntry { label: "README.md".into(), depth: 0, file_idx: Some(2), status: Some(FileStatus::Modified), collapsed: false },
    ];

    let line_map: Vec<LineInfo> = (0..30)
        .map(|i| LineInfo {
            file_idx: if i < 10 { 0 } else if i < 20 { 1 } else { 2 },
            path: if i < 10 { "src/a.rs" } else if i < 20 { "src/b.rs" } else { "README.md" }.into(),
            new_lineno: Some(i as u32 + 1),
            old_lineno: None,
            line_kind: Some(LineKind::Context),
            hunk_idx: None,
        })
        .collect();

    let mut state = PagerState::new(
        vec!["line".into(); 30],
        line_map,
        vec![0, 10, 20],
        vec![],
        tree_entries,
        120,
    );
    state.tree_visible = true;
    state.rebuild_tree_lines();
    state
}

#[test]
fn next_file_tree_visible_uses_nav_d_down() {
    let mut state = make_tree_file_jump_state();
    state.set_tree_cursor(0);
    state.rebuild_tree_lines();
    state.cursor_line = 0;

    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);

    // Uses nav_D_down, jumps to next file header (file_starts[1]=10)
    assert_eq!(state.cursor_line, 10, "cursor should jump to next file via nav_D_down");
}

#[test]
fn prev_file_tree_visible_uses_nav_u_up() {
    let mut state = make_tree_file_jump_state();
    state.set_tree_cursor(3);
    state.rebuild_tree_lines();
    state.cursor_line = 20;

    handle_key(&mut state, Key::Char('{'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);

    // Uses nav_U_up, jumps to previous file header (file_starts[1]=10)
    assert_eq!(state.cursor_line, 10, "cursor should jump to prev file via nav_U_up");
}

#[test]
fn next_file_tree_hidden_uses_nav_d_down() {
    let mut state = make_keybinding_state();
    state.tree_visible = false;
    state.cursor_line = 1;

    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);

    // Should use nav_D_down behavior, jumping to file_starts[1]=30
    assert!(state.cursor_line >= 30, "cursor should jump to second file area via nav_D_down");
}

#[test]
fn next_file_tree_visible_any_focus() {
    let mut state = make_tree_file_jump_state();
    state.focus = FocusPane::Diff;
    state.set_tree_cursor(0);
    state.rebuild_tree_lines();
    state.cursor_line = 0;

    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);

    // Uses nav_D_down regardless of tree visibility or focus
    assert_eq!(state.cursor_line, 10, "cursor should jump to next file via nav_D_down");
}

// ---- File position memory (cursor position per file) ----

/// Helper: build a two-file state in single-file mode with enough content lines
/// to allow scrolling. File 0 = lines 0..30, File 1 = lines 30..60.
fn make_single_file_state() -> PagerState {
    let mut state = make_keybinding_state();
    // Enter single file mode on file 0
    state.set_active_file(Some(0));
    // Position cursor somewhere in file 0 (not at the start)
    state.cursor_line = 1; // first content line
    state.top_line = 0;
    state
}

#[test]
fn file_position_restored_on_return() {
    let mut state = make_single_file_state();

    // Scroll down in file 0
    for _ in 0..10 {
        handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    }
    let file0_cursor = state.cursor_line;
    let file0_top = state.top_line;
    assert!(file0_cursor > 1, "should have scrolled down in file 0");

    // Switch to file 1
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.active_file(), Some(1), "should be on file 1");

    // Switch back to file 0
    handle_key(&mut state, Key::Char('{'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.active_file(), Some(0), "should be back on file 0");
    assert_eq!(
        state.cursor_line, file0_cursor,
        "cursor position should be restored for file 0"
    );
    assert_eq!(
        state.top_line, file0_top,
        "top_line should be restored for file 0"
    );
}

#[test]
fn file_position_round_trip() {
    let mut state = make_single_file_state();

    // Scroll in file 0
    for _ in 0..5 {
        handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    }
    let file0_cursor = state.cursor_line;

    // Go to file 1, scroll there too
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.active_file(), Some(1));
    for _ in 0..3 {
        handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    }
    let file1_cursor = state.cursor_line;

    // Go back to file 0
    handle_key(&mut state, Key::Char('{'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.cursor_line, file0_cursor, "file 0 position restored");

    // Go back to file 1
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.cursor_line, file1_cursor, "file 1 position restored");
}

#[test]
fn fresh_file_starts_at_header() {
    let mut state = make_single_file_state();
    // File 0 starts at line 0. On first visit, cursor should be at the file start.
    // The existing behavior places cursor at the file's start line.
    // Switch to file 1 (never visited)
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    let file1_start = state.file_start(1).unwrap();
    assert_eq!(
        state.cursor_line, file1_start,
        "first visit to file 1 should start at file header"
    );
}

#[test]
fn file_positions_cleared_on_document_swap() {
    use super::super::state::{capture_view_anchor, remap_after_document_swap};

    let mut state = make_single_file_state();

    // Scroll in file 0
    for _ in 0..5 {
        handle_key(&mut state, Key::Char('j'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    }
    let scrolled_cursor = state.cursor_line;
    assert!(scrolled_cursor > 1);

    // Switch to file 1 to save file 0's position
    handle_key(&mut state, Key::Char('}'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);

    // Simulate document swap (regenerate)
    let anchor = capture_view_anchor(&state);
    let new_doc = state.doc.clone();
    remap_after_document_swap(&mut state, anchor, new_doc, &[], 120);

    // Switch back to file 0 — position should NOT be restored (cache cleared)
    handle_key(&mut state, Key::Char('{'), 40, 40, 120, &[], p(), &crate::git::DiffSource::WorkingTree);
    let file0_start = state.file_start(0).unwrap();
    assert_eq!(
        state.cursor_line, file0_start,
        "after document swap, file 0 should start at header (cache cleared)"
    );
}

#[test]
fn focus_tree_syncs_cursor_to_current_file() {
    let mut state = make_keybinding_state();
    let files = vec![
        make_diff_file("a.rs"),
        make_diff_file("b.rs"),
        make_diff_file("c.rs"),
    ];

    // Open tree, then go back to diff
    handle_key(&mut state, Key::Super('e'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.focus, FocusPane::Tree);
    handle_key(&mut state, Key::Super('e'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.focus, FocusPane::Diff);

    // Move diff cursor into file 1 (starts at line 30)
    state.cursor_line = 31;

    // Re-focus tree — cursor should sync to file 1's tree entry (index 1)
    handle_key(&mut state, Key::Super('e'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.focus, FocusPane::Tree);
    assert_eq!(state.tree_cursor(), 1, "tree cursor should sync to file 1 when re-focusing");
}

#[test]
fn toggle_focus_syncs_cursor_to_current_file() {
    let mut state = make_keybinding_state();
    let files = vec![
        make_diff_file("a.rs"),
        make_diff_file("b.rs"),
        make_diff_file("c.rs"),
    ];

    // Open and focus tree with t
    handle_key(&mut state, Key::Char('t'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.focus, FocusPane::Tree);
    assert_eq!(state.tree_cursor(), 0);

    // Back to diff
    handle_key(&mut state, Key::Char('t'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.focus, FocusPane::Diff);

    // Move diff cursor into file 2 (starts at line 60)
    state.cursor_line = 61;

    // Re-focus tree — cursor should sync to file 2's tree entry (index 2)
    handle_key(&mut state, Key::Char('t'), 40, 40, 120, &files, p(), &crate::git::DiffSource::WorkingTree);
    assert_eq!(state.tree_cursor(), 2, "tree cursor should sync to file 2 when re-focusing via t");
}
