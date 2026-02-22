//! Search input and status bar tests.

use super::super::rendering::format_status_bar;
use super::super::search::handle_search_key;
use super::common::make_search_state;
use tui::pager::Key;

#[test]
fn test_search_left_right_accented() {
    let mut state = make_search_state("cafe\u{301}", "cafe\u{301}".len());
    handle_search_key(&mut state, &Key::Left);
    assert_eq!(state.search_cursor, 4);
    handle_search_key(&mut state, &Key::Left);
    assert_eq!(state.search_cursor, 3);
    handle_search_key(&mut state, &Key::Right);
    assert_eq!(state.search_cursor, 4);
    handle_search_key(&mut state, &Key::Right);
    assert_eq!(state.search_cursor, 6);
}

#[test]
fn test_search_backspace_accented() {
    let mut state = make_search_state("nai\u{308}ve", 5);
    handle_search_key(&mut state, &Key::Backspace);
    assert_eq!(state.search_input, "naive");
    assert_eq!(state.search_cursor, 3);
}

#[test]
fn test_search_alt_backspace_multibyte() {
    let mut state = make_search_state("hello \u{4e16}\u{754c}", "hello \u{4e16}\u{754c}".len());
    handle_search_key(&mut state, &Key::AltBackspace);
    assert_eq!(state.search_input, "hello ");
    assert_eq!(state.search_cursor, "hello ".len());
}

#[test]
fn test_search_ctrl_u_emoji() {
    let mut state = make_search_state("a\u{1f600}b", "a\u{1f600}b".len());
    handle_search_key(&mut state, &Key::CtrlU);
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
