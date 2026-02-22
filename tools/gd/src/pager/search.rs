use tui::pager::Key;
use tui::search::{find_matches, find_nearest_match};

use super::state::PagerState;
use super::text::{clamp_cursor_to_boundary, next_char_boundary, prev_char_boundary};
use super::types::Mode;

pub(crate) fn submit_search(state: &mut PagerState) {
    let query = std::mem::take(&mut state.search_input);
    state.search_cursor = 0;
    state.mode = Mode::Normal;

    let matches = find_matches(&state.doc.lines, &query);
    if matches.is_empty() {
        state.status_message = format!("Pattern not found: {query}");
        state.search_query = query;
        state.search_matches = Vec::new();
        state.current_match = -1;
    } else {
        let nearest = find_nearest_match(&matches, state.top_line);
        state.search_query = query;
        state.search_matches = matches;
        state.current_match = nearest;
    }
}

pub(crate) fn cancel_search(state: &mut PagerState) {
    state.search_input.clear();
    state.search_cursor = 0;
    state.mode = Mode::Normal;
}

pub(crate) fn handle_search_key(state: &mut PagerState, key: Key) {
    use tui::search::{word_boundary_left, word_boundary_right};

    match key {
        Key::Char(c) => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            state.search_input.insert(cursor, c);
            state.search_cursor = cursor + c.len_utf8();
        }
        Key::Backspace => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            if cursor > 0 {
                let remove_at = prev_char_boundary(&state.search_input, cursor);
                state.search_input.remove(remove_at);
                state.search_cursor = remove_at;
            } else {
                state.search_cursor = cursor;
            }
            if state.search_input.is_empty() {
                state.mode = Mode::Normal;
            }
        }
        Key::AltBackspace => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            let new_pos = clamp_cursor_to_boundary(
                &state.search_input,
                word_boundary_left(&state.search_input, cursor),
            );
            state.search_input.replace_range(new_pos..cursor, "");
            state.search_cursor = new_pos;
            if state.search_input.is_empty() {
                state.mode = Mode::Normal;
            }
        }
        Key::CtrlU => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            if cursor > 0 {
                state.search_input = String::from(&state.search_input[cursor..]);
                state.search_cursor = 0;
            } else {
                state.search_cursor = cursor;
            }
            if state.search_input.is_empty() {
                state.mode = Mode::Normal;
            }
        }
        Key::Left => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            state.search_cursor = prev_char_boundary(&state.search_input, cursor);
        }
        Key::Right => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            state.search_cursor = next_char_boundary(&state.search_input, cursor);
        }
        Key::AltLeft => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            state.search_cursor = clamp_cursor_to_boundary(
                &state.search_input,
                word_boundary_left(&state.search_input, cursor),
            );
        }
        Key::AltRight => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            state.search_cursor = clamp_cursor_to_boundary(
                &state.search_input,
                word_boundary_right(&state.search_input, cursor),
            );
        }
        _ => {}
    }
}

/// Find the next match in [range_start, range_end) after current_match.
/// When current_match is before first or at end, wraps to first in range.
/// Returns the global index into matches, or None if no matches in range.
pub(crate) fn next_match_in_range(
    matches: &[usize],
    current_match: isize,
    range_start: usize,
    range_end: usize,
) -> Option<isize> {
    let filtered: Vec<usize> = matches
        .iter()
        .copied()
        .filter(|&m| m >= range_start && m < range_end)
        .collect();
    if filtered.is_empty() {
        return None;
    }
    let cur_line = if current_match >= 0 && (current_match as usize) < matches.len() {
        matches[current_match as usize]
    } else {
        range_start.saturating_sub(1)
    };
    let next_line = filtered
        .iter()
        .find(|&&m| m > cur_line)
        .copied()
        .or_else(|| Some(filtered[0]));
    next_line.and_then(|line| {
        matches
            .iter()
            .position(|&m| m == line)
            .map(|idx| idx as isize)
    })
}

/// Find the previous match in [range_start, range_end) before current_match.
/// When current_match is after last or at start, wraps to last in range.
/// Returns the global index into matches, or None if no matches in range.
pub(crate) fn prev_match_in_range(
    matches: &[usize],
    current_match: isize,
    range_start: usize,
    range_end: usize,
) -> Option<isize> {
    let filtered: Vec<usize> = matches
        .iter()
        .copied()
        .filter(|&m| m >= range_start && m < range_end)
        .collect();
    if filtered.is_empty() {
        return None;
    }
    let cur_line = if current_match >= 0 && (current_match as usize) < matches.len() {
        matches[current_match as usize]
    } else {
        range_end
    };
    let prev_line = filtered
        .iter()
        .rev()
        .find(|&&m| m < cur_line)
        .copied()
        .or_else(|| filtered.last().copied());
    prev_line.and_then(|line| {
        matches
            .iter()
            .position(|&m| m == line)
            .map(|idx| idx as isize)
    })
}

pub(crate) fn scroll_to_match(
    state: &mut PagerState,
    content_height: usize,
) {
    let Ok(match_idx) = usize::try_from(state.current_match) else {
        return;
    };
    if match_idx >= state.search_matches.len() {
        return;
    }
    let match_line = state.search_matches[match_idx];
    state.cursor_line = match_line;
    super::rendering::enforce_scrolloff(state, content_height);
}
