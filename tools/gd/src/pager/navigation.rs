use crate::render::LineInfo;

use super::content::next_content_line;
use super::state::{PagerState, visible_range};
use super::tree::file_idx_to_entry_idx;

pub(crate) fn jump_next(targets: &[usize], top_line: usize) -> Option<usize> {
    targets.iter().find(|&&t| t > top_line).copied()
}

pub(crate) fn jump_prev(targets: &[usize], top_line: usize) -> Option<usize> {
    targets.iter().rev().find(|&&t| t < top_line).copied()
}

fn targets_in_range(targets: &[usize], range_start: usize, range_end: usize) -> Vec<usize> {
    targets
        .iter()
        .filter(|&&s| s >= range_start && s < range_end)
        .copied()
        .collect()
}

pub(crate) fn change_group_starts(
    line_map: &[LineInfo],
    range_start: usize,
    range_end: usize,
) -> Vec<usize> {
    use crate::git::diff::LineKind;

    let end = range_end.min(line_map.len());
    let mut starts = Vec::new();
    for i in range_start..end {
        let is_change = matches!(
            line_map[i].line_kind,
            Some(LineKind::Added | LineKind::Deleted)
        );
        let prev_is_change = i > range_start
            && matches!(
                line_map[i - 1].line_kind,
                Some(LineKind::Added | LineKind::Deleted)
            );
        if is_change && !prev_is_change {
            starts.push(i);
        }
    }
    starts
}

fn du_nav_targets(state: &PagerState) -> Vec<usize> {
    let (range_start, range_end) = visible_range(state);
    change_group_starts(&state.doc.line_map, range_start, range_end)
}

pub(crate) struct NavDuResult {
    pub cursor_line: usize,
    pub status_message: String,
    pub moved: bool,
}

#[allow(dead_code)]
pub(crate) struct NavDUResult {
    pub cursor_line: usize,
    pub top_line: usize,
    pub status_message: String,
    pub moved: bool,
}

pub(crate) fn nav_status_message(
    label: &str,
    cursor: usize,
    starts: &[usize],
    line_map: &[LineInfo],
) -> String {
    if starts.is_empty() {
        return String::new();
    }
    let idx = starts.partition_point(|&s| s <= cursor).saturating_sub(1);
    let path = line_map.get(cursor).map_or("", |li| &li.path);
    format!("{label} {}/{} \u{00b7} {}", idx + 1, starts.len(), path)
}

fn file_status_message(cursor: usize, file_starts: &[usize], line_map: &[LineInfo]) -> String {
    nav_status_message("File", cursor, file_starts, line_map)
}

pub(crate) fn bottom_padding(state: &PagerState, content_height: usize) -> usize {
    if state.active_file().is_some() {
        content_height / 2
    } else {
        0
    }
}

pub(crate) fn viewport_bounds(
    state: &PagerState,
    content_height: usize,
) -> (usize, usize, usize, usize) {
    let (range_start, range_end) = visible_range(state);
    let max_line = range_end.saturating_sub(1);
    let pad = bottom_padding(state, content_height);
    let max_top = (range_end + pad).saturating_sub(content_height).max(range_start);
    (range_start, range_end, max_line, max_top)
}

pub(crate) fn recenter_top_line(
    cursor_line: usize,
    content_height: usize,
    range_start: usize,
    max_top: usize,
) -> usize {
    cursor_line
        .saturating_sub(content_height / 2)
        .max(range_start)
        .min(max_top)
}

pub(crate) fn nav_du_down(state: &PagerState) -> NavDuResult {
    if state.doc.line_map.is_empty() {
        return NavDuResult {
            cursor_line: state.cursor_line,
            status_message: String::new(),
            moved: false,
        };
    }
    let anchor = state.cursor_line;
    let max_line = state.doc.line_map.len().saturating_sub(1);
    let hunks = du_nav_targets(state);
    if let Some(target) = jump_next(&hunks, anchor) {
        let cursor = next_content_line(&state.doc.line_map, target, max_line);
        let status = nav_status_message(
            if state.full_context { "Change" } else { "Hunk" },
            cursor,
            &hunks,
            &state.doc.line_map,
        );
        NavDuResult {
            cursor_line: cursor,
            status_message: status,
            moved: true,
        }
    } else {
        NavDuResult {
            cursor_line: state.cursor_line,
            status_message: String::new(),
            moved: false,
        }
    }
}

pub(crate) fn nav_du_up(state: &PagerState) -> NavDuResult {
    if state.doc.line_map.is_empty() {
        return NavDuResult {
            cursor_line: state.cursor_line,
            status_message: String::new(),
            moved: false,
        };
    }
    let anchor = state.cursor_line;
    let max_line = state.doc.line_map.len().saturating_sub(1);
    let hunks = du_nav_targets(state);
    if let Some(target) = jump_prev(&hunks, anchor) {
        let mut cursor = next_content_line(&state.doc.line_map, target, max_line);
        if cursor >= anchor {
            if let Some(target2) = jump_prev(&hunks, target) {
                cursor = next_content_line(&state.doc.line_map, target2, max_line);
            } else {
                cursor = anchor;
            }
        }
        let status = nav_status_message(
            if state.full_context { "Change" } else { "Hunk" },
            cursor,
            &hunks,
            &state.doc.line_map,
        );
        NavDuResult {
            cursor_line: cursor,
            status_message: status,
            moved: cursor != anchor,
        }
    } else {
        NavDuResult {
            cursor_line: state.cursor_line,
            status_message: String::new(),
            moved: false,
        }
    }
}

#[allow(non_snake_case)]
pub(crate) fn nav_D_down(state: &PagerState, content_height: usize) -> NavDUResult {
    let anchor = state.cursor_line;
    let (rs, re) = visible_range(state);
    let max_line = re.saturating_sub(1);
    let max_top = re.saturating_sub(content_height).max(rs);
    let files = targets_in_range(&state.doc.file_starts, rs, re);
    if let Some(target) = jump_next(&files, anchor) {
        let cursor = next_content_line(&state.doc.line_map, target, max_line);
        let top = cursor
            .saturating_sub(content_height / 2)
            .max(rs)
            .min(max_top);
        let status = file_status_message(cursor, &files, &state.doc.line_map);
        NavDUResult {
            cursor_line: cursor,
            top_line: top,
            status_message: status,
            moved: true,
        }
    } else {
        NavDUResult {
            cursor_line: state.cursor_line,
            top_line: state.top_line,
            status_message: String::new(),
            moved: false,
        }
    }
}

#[allow(non_snake_case)]
pub(crate) fn nav_U_up(state: &PagerState, content_height: usize) -> NavDUResult {
    let anchor = state.cursor_line;
    let (rs, re) = visible_range(state);
    let max_line = re.saturating_sub(1);
    let max_top = re.saturating_sub(content_height).max(rs);
    let files = targets_in_range(&state.doc.file_starts, rs, re);
    if let Some(target) = jump_prev(&files, anchor) {
        let mut cursor = next_content_line(&state.doc.line_map, target, max_line);
        if cursor >= anchor {
            if let Some(target2) = jump_prev(&files, target) {
                cursor = next_content_line(&state.doc.line_map, target2, max_line);
            } else {
                cursor = anchor;
            }
        }
        let top = cursor
            .saturating_sub(content_height / 2)
            .max(rs)
            .min(max_top);
        let status = file_status_message(cursor, &files, &state.doc.line_map);
        NavDUResult {
            cursor_line: cursor,
            top_line: top,
            status_message: status,
            moved: cursor != anchor,
        }
    } else {
        NavDUResult {
            cursor_line: state.cursor_line,
            top_line: state.top_line,
            status_message: String::new(),
            moved: false,
        }
    }
}

pub(crate) fn sync_tree_cursor(state: &mut PagerState, content_height: usize) {
    if !state.tree_visible || state.tree_entries.is_empty() {
        return;
    }
    let new_cursor = if let Some(fi) = state.active_file() {
        fi
    } else {
        state
            .doc
            .line_map
            .get(state.cursor_line)
            .map_or(0, |li| li.file_idx)
    };
    let mut new_entry_idx = file_idx_to_entry_idx(&state.tree_entries, new_cursor);
    if !state.tree_visible_to_entry.contains(&new_entry_idx) {
        let target_depth = state.tree_entry(new_entry_idx).map_or(0, |e| e.depth);
        new_entry_idx = state.tree_entries[..new_entry_idx]
            .iter()
            .rposition(|e| e.file_idx.is_none() && e.depth < target_depth)
            .unwrap_or(0);
    }
    if new_entry_idx != state.tree_cursor() {
        state.set_tree_cursor(new_entry_idx);
        state.rebuild_tree_lines();
        ensure_tree_cursor_visible(state, content_height);
    }
}

pub(crate) fn sync_active_file_to_cursor(state: &mut PagerState) {
    if state.active_file().is_none() {
        return;
    }
    if let Some(file_idx) = state
        .doc
        .line_map
        .get(state.cursor_line)
        .map(|li| li.file_idx)
    {
        state.set_active_file(Some(file_idx));
    }
}

pub(crate) fn ensure_tree_cursor_visible(state: &mut PagerState, content_height: usize) {
    let offset = state
        .tree_visible_to_entry
        .iter()
        .position(|&ei| ei == state.tree_cursor())
        .unwrap_or(0);
    if offset < state.tree_scroll {
        state.tree_scroll = offset;
    }
    if offset >= state.tree_scroll + content_height {
        state.tree_scroll = offset + 1 - content_height;
    }
}
