//! Search input and status bar tests.

use super::super::rendering::format_status_bar;
use super::super::search::{
    cancel_search, handle_search_key, next_match_in_range, prev_match_in_range, scroll_to_match,
    submit_search,
};
use super::common::{make_keybinding_state, make_search_state};
use tui::pager::Key;

#[test]
fn test_search_left_right_accented() {
    let mut state = make_search_state("cafe\u{301}", "cafe\u{301}".len());
    handle_search_key(&mut state, Key::Left);
    assert_eq!(state.search_cursor, 4);
    handle_search_key(&mut state, Key::Left);
    assert_eq!(state.search_cursor, 3);
    handle_search_key(&mut state, Key::Right);
    assert_eq!(state.search_cursor, 4);
    handle_search_key(&mut state, Key::Right);
    assert_eq!(state.search_cursor, 6);
}

#[test]
fn test_search_backspace_accented() {
    let mut state = make_search_state("nai\u{308}ve", 5);
    handle_search_key(&mut state, Key::Backspace);
    assert_eq!(state.search_input, "naive");
    assert_eq!(state.search_cursor, 3);
}

#[test]
fn test_search_alt_backspace_multibyte() {
    let mut state = make_search_state("hello \u{4e16}\u{754c}", "hello \u{4e16}\u{754c}".len());
    handle_search_key(&mut state, Key::AltBackspace);
    assert_eq!(state.search_input, "hello ");
    assert_eq!(state.search_cursor, "hello ".len());
}

#[test]
fn test_search_ctrl_u_emoji() {
    let mut state = make_search_state("a\u{1f600}b", "a\u{1f600}b".len());
    handle_search_key(&mut state, Key::CtrlU);
    assert_eq!(state.search_input, "");
    assert_eq!(state.search_cursor, 0);
    assert_eq!(state.mode, super::super::types::Mode::Normal);
}

#[test]
fn test_format_status_bar_emoji() {
    let state = make_search_state("\u{1f50d}test", 4);
    let status = format_status_bar(&state, 10, 40);
    let stripped = crate::ansi::strip_ansi(&status);
    assert!(stripped.contains("/\u{1f50d}test"));
}

#[test]
fn test_format_status_bar_mid_char_no_panic() {
    let state = make_search_state("a\u{1f50d}", 2);
    let result = std::panic::catch_unwind(|| format_status_bar(&state, 10, 40));
    assert!(result.is_ok());
}

// --- next_match_in_range ---

#[test]
fn test_next_match_in_range_basic_forward() {
    let matches = vec![2, 5, 8];
    assert_eq!(next_match_in_range(&matches, 0, 0, 10), Some(1));
}

#[test]
fn test_next_match_in_range_wrap_around() {
    let matches = vec![2, 5, 8];
    assert_eq!(next_match_in_range(&matches, 2, 0, 10), Some(0));
}

#[test]
fn test_next_match_in_range_no_matches_in_range() {
    let matches = vec![1, 3];
    assert_eq!(next_match_in_range(&matches, 0, 5, 10), None);
}

#[test]
fn test_next_match_in_range_single_match() {
    let matches = vec![4];
    assert_eq!(next_match_in_range(&matches, 0, 0, 10), Some(0));
}

#[test]
fn test_next_match_in_range_current_before_range() {
    let matches = vec![2, 5, 8];
    assert_eq!(next_match_in_range(&matches, 0, 5, 10), Some(1));
}

#[test]
fn test_next_match_in_range_current_at_end_of_range() {
    let matches = vec![3, 7, 12];
    assert_eq!(next_match_in_range(&matches, 1, 0, 10), Some(0));
}

#[test]
fn test_next_match_in_range_negative_current() {
    let matches = vec![2, 5, 8];
    assert_eq!(next_match_in_range(&matches, -1, 0, 10), Some(0));
}

// --- prev_match_in_range ---

#[test]
fn test_prev_match_in_range_basic_backward() {
    let matches = vec![2, 5, 8];
    assert_eq!(prev_match_in_range(&matches, 2, 0, 10), Some(1));
}

#[test]
fn test_prev_match_in_range_wrap_around() {
    let matches = vec![2, 5, 8];
    assert_eq!(prev_match_in_range(&matches, 0, 0, 10), Some(2));
}

#[test]
fn test_prev_match_in_range_no_matches_in_range() {
    let matches = vec![1, 3];
    assert_eq!(prev_match_in_range(&matches, 0, 5, 10), None);
}

#[test]
fn test_prev_match_in_range_single_match() {
    let matches = vec![4];
    assert_eq!(prev_match_in_range(&matches, 0, 0, 10), Some(0));
}

#[test]
fn test_prev_match_in_range_current_after_range() {
    let matches = vec![2, 5, 12];
    assert_eq!(prev_match_in_range(&matches, 2, 0, 10), Some(1));
}

#[test]
fn test_prev_match_in_range_current_at_start_of_range() {
    let matches = vec![3, 7, 12];
    assert_eq!(prev_match_in_range(&matches, 0, 0, 10), Some(1));
}

// --- submit_search ---

#[test]
fn test_submit_search_with_matches() {
    let mut state = make_search_state("line", 4);
    submit_search(&mut state);
    assert_eq!(state.search_query, "line");
    assert!(!state.search_matches.is_empty());
    assert!(state.current_match >= 0);
    assert_eq!(state.mode, super::super::types::Mode::Normal);
}

#[test]
fn test_submit_search_no_matches() {
    let mut state = make_search_state("xyz999", 6);
    submit_search(&mut state);
    assert!(
        state.status_message.contains("Pattern not found"),
        "expected 'Pattern not found' in {:?}",
        state.status_message
    );
    assert_eq!(state.current_match, -1);
    assert!(state.search_matches.is_empty());
    assert_eq!(state.mode, super::super::types::Mode::Normal);
}

// --- cancel_search ---

#[test]
fn test_cancel_search() {
    let mut state = make_search_state("foo", 2);
    cancel_search(&mut state);
    assert!(state.search_input.is_empty());
    assert_eq!(state.search_cursor, 0);
    assert_eq!(state.mode, super::super::types::Mode::Normal);
}

// --- scroll_to_match ---

#[test]
fn test_scroll_to_match_valid() {
    let mut state = make_keybinding_state();
    state.search_matches = vec![10, 20, 30];
    state.current_match = 1;
    state.cursor_line = 0;
    scroll_to_match(&mut state, 24);
    assert_eq!(state.cursor_line, 20);
}

#[test]
fn test_scroll_to_match_negative_current_no_op() {
    let mut state = make_keybinding_state();
    state.search_matches = vec![5];
    state.current_match = -1;
    state.cursor_line = 3;
    scroll_to_match(&mut state, 24);
    assert_eq!(state.cursor_line, 3);
}

#[test]
fn test_scroll_to_match_out_of_bounds_no_op() {
    let mut state = make_keybinding_state();
    state.search_matches = vec![5];
    state.current_match = 5;
    state.cursor_line = 3;
    scroll_to_match(&mut state, 24);
    assert_eq!(state.cursor_line, 3);
}
