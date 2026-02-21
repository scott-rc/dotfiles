use std::io::{self, Write};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use tui::pager::{
    ALT_SCREEN_OFF, ALT_SCREEN_ON, CLEAR_LINE, CURSOR_HIDE, CURSOR_SHOW, copy_to_clipboard,
    get_term_size, move_to,
};
use tui::search::{
    find_matches, find_nearest_match, highlight_search, max_scroll, word_boundary_left,
    word_boundary_right,
};

use crate::git::diff::{DiffFile, FileStatus};
use crate::render::{self, LineInfo, RenderOutput};
use crate::style;

use tui::pager::Key;

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Normal,
    Search,
    Help,
    Visual,
}

pub struct DiffContext {
    pub repo: std::path::PathBuf,
    pub source: crate::git::DiffSource,
    pub no_untracked: bool,
}

struct PagerState {
    lines: Vec<String>,
    line_map: Vec<LineInfo>,
    file_starts: Vec<usize>,
    hunk_starts: Vec<usize>,
    top_line: usize,
    cursor_line: usize,
    cursor_visible: bool,
    visual_anchor: usize,
    search_query: String,
    search_matches: Vec<usize>,
    current_match: isize,
    mode: Mode,
    search_input: String,
    search_cursor: usize,
    status_message: String,
    /// Pending bracket key for two-key sequences like ]c, [c, ]f, [f
    pending_bracket: Option<char>,
    tree_visible: bool,
    tree_focused: bool,
    tree_cursor: usize,
    tree_width: usize,
    tree_scroll: usize,
    tree_lines: Vec<String>,
    tree_entries: Vec<TreeEntry>,
    /// When `Some(idx)`, diff panel shows only file `idx`; `None` = all-files view
    active_file: Option<usize>,
    top_padding: usize,
    full_context: bool,
}

use tui::pager::crossterm_to_key;

fn diff_area_width(cols: u16, tree_width: usize, tree_visible: bool) -> usize {
    if tree_visible {
        (cols as usize).saturating_sub(tree_width + 1)
    } else {
        cols as usize
    }
}

fn apply_top_padding(state: &mut PagerState, content_height: usize) {
    // Strip existing padding
    if state.top_padding > 0 {
        let old = state.top_padding;
        state.lines.drain(..old);
        state.line_map.drain(..old);
        for v in &mut state.file_starts {
            *v -= old;
        }
        for v in &mut state.hunk_starts {
            *v -= old;
        }
        for v in &mut state.search_matches {
            *v -= old;
        }
        state.top_line -= old;
    }

    // Apply new padding
    let padding = content_height / 2;
    state
        .lines
        .splice(0..0, std::iter::repeat_with(String::new).take(padding));
    state.line_map.splice(
        0..0,
        std::iter::repeat_with(|| LineInfo {
            file_idx: 0,
            path: String::new(),
            new_lineno: None,
            old_lineno: None,
        })
        .take(padding),
    );

    // Shift indices forward
    for v in &mut state.file_starts {
        *v += padding;
    }
    for v in &mut state.hunk_starts {
        *v += padding;
    }
    for v in &mut state.search_matches {
        *v += padding;
    }
    state.top_line += padding;
    state.top_padding = padding;
}

const SCROLLOFF: usize = 8;

fn enforce_scrolloff(state: &mut PagerState, content_height: usize) {
    let (range_start, range_end) = visible_range(state);
    let range_lines = range_end - range_start;
    let max_top = range_start + range_lines.saturating_sub(content_height);
    let max_cursor = range_end.saturating_sub(1);
    state.cursor_line = state.cursor_line.clamp(range_start, max_cursor);
    if state.cursor_line < state.top_line + SCROLLOFF {
        state.top_line = state.cursor_line.saturating_sub(SCROLLOFF).max(range_start);
    }
    if state.cursor_line + SCROLLOFF >= state.top_line + content_height {
        state.top_line = (state.cursor_line + SCROLLOFF + 1).saturating_sub(content_height);
    }
    state.top_line = state.top_line.clamp(range_start, max_top);
}

/// Return the `(start, end)` line range for the active file, or the full
/// document range when no file is selected.
fn visible_range(state: &PagerState) -> (usize, usize) {
    match state.active_file {
        Some(idx) => {
            let start = state.file_starts[idx];
            let end = state
                .file_starts
                .get(idx + 1)
                .copied()
                .unwrap_or(state.lines.len());
            (start, end)
        }
        None => (0, state.lines.len()),
    }
}

/// Apply visual selection background tint to a rendered line.
/// Prepends BG_VISUAL so the tint shows on gutter/context areas;
/// diff bg codes (BG_ADDED/BG_DELETED) naturally override in content.
fn highlight_visual_line(line: &str, width: usize) -> String {
    let vis_w = crate::ansi::visible_width(line);
    let target = width.saturating_sub(1);
    let pad = target.saturating_sub(vis_w);
    format!(
        "{}{line}{}{}",
        style::BG_VISUAL,
        " ".repeat(pad),
        style::RESET,
    )
}

/// Resolve start/end line numbers from a visual selection range.
/// Prefers new_lineno, falls back to old_lineno for deleted lines.
fn resolve_lineno(line_map: &[LineInfo], lo: usize, hi: usize) -> (Option<u32>, Option<u32>) {
    let pick = |info: &LineInfo| info.new_lineno.or(info.old_lineno);
    let start = (lo..=hi).find_map(|i| line_map.get(i).and_then(pick));
    let end = (lo..=hi).rev().find_map(|i| line_map.get(i).and_then(pick));
    (start, end)
}

fn format_copy_ref(path: &str, start: Option<u32>, end: Option<u32>) -> String {
    match (start, end) {
        (Some(s), Some(e)) if s == e => format!("{path}:{s}"),
        (Some(s), Some(e)) => format!("{path}:{s}-{e}"),
        (Some(s), None) => format!("{path}:{s}"),
        _ => path.to_string(),
    }
}

fn handle_search_key(state: &mut PagerState, key: &Key) {
    match key {
        Key::Char(c) => {
            state.search_input.insert(state.search_cursor, *c);
            state.search_cursor += 1;
        }
        Key::Backspace => {
            if state.search_cursor > 0 {
                state.search_cursor -= 1;
                state.search_input.remove(state.search_cursor);
            }
            if state.search_input.is_empty() {
                state.mode = Mode::Normal;
            }
        }
        Key::AltBackspace => {
            let new_pos = word_boundary_left(&state.search_input, state.search_cursor);
            state.search_input = format!(
                "{}{}",
                &state.search_input[..new_pos],
                &state.search_input[state.search_cursor..]
            );
            state.search_cursor = new_pos;
            if state.search_input.is_empty() {
                state.mode = Mode::Normal;
            }
        }
        Key::CtrlU => {
            if state.search_cursor > 0 {
                state.search_input = state.search_input[state.search_cursor..].to_string();
                state.search_cursor = 0;
            }
            if state.search_input.is_empty() {
                state.mode = Mode::Normal;
            }
        }
        Key::Left => {
            if state.search_cursor > 0 {
                state.search_cursor -= 1;
            }
        }
        Key::Right => {
            if state.search_cursor < state.search_input.len() {
                state.search_cursor += 1;
            }
        }
        Key::AltLeft => {
            state.search_cursor = word_boundary_left(&state.search_input, state.search_cursor);
        }
        Key::AltRight => {
            state.search_cursor = word_boundary_right(&state.search_input, state.search_cursor);
        }
        Key::Enter => {
            let query = std::mem::take(&mut state.search_input);
            state.search_cursor = 0;
            state.mode = Mode::Normal;

            let matches = find_matches(&state.lines, &query);
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
        Key::Escape | Key::CtrlC => {
            state.search_input.clear();
            state.search_cursor = 0;
            state.mode = Mode::Normal;
        }
        _ => {}
    }
}

fn open_in_editor(path: &str, line: Option<u32>) {
    tui::pager::open_in_editor(path, line, false);
}

fn format_help_lines(cols: usize, rows: usize) -> Vec<String> {
    let content_height = rows.saturating_sub(1);
    let help = [
        "Navigation",
        "j/\u{2193}/Enter  Scroll down",
        "k/\u{2191}        Scroll up",
        "d/Ctrl-D   Half page down",
        "u/Ctrl-U   Half page up",
        "g/Home     Top",
        "G/End      Bottom",
        "Space      Toggle cursor",
        "",
        "Diff Navigation",
        "]c         Next hunk",
        "[c         Previous hunk",
        "]f         Next file",
        "[f         Previous file",
        "o          Toggle full file",
        "",
        "Search",
        "/          Search",
        "n          Next match",
        "N          Previous match",
        "",
        "File Tree",
        "e          Toggle panel",
        "Tab        Focus panel",
        "Ctrl-L     Show + focus tree",
        "Ctrl-H     Return to diff",
        "",
        "Visual Mode",
        "v          Enter visual mode",
        "j/k        Extend selection",
        "y          Copy path:lines",
        "Esc        Cancel",
        "",
        "E          Open in editor",
        "q          Quit",
        "? / Esc    Close help",
    ];

    let mut lines = Vec::with_capacity(content_height);
    let top_pad = content_height.saturating_sub(help.len()) / 2;
    for _ in 0..top_pad {
        lines.push(" ".repeat(cols));
    }

    let max_w = help.iter().map(|h| h.chars().count()).max().unwrap_or(0);
    let left_pad = cols.saturating_sub(max_w) / 2;

    for &h in &help {
        if lines.len() >= content_height {
            break;
        }
        let vis_len = h.chars().count();
        if vis_len >= cols {
            lines.push(h.chars().take(cols).collect());
        } else {
            let right_pad = cols.saturating_sub(left_pad + vis_len);
            lines.push(format!(
                "{}{}{}",
                " ".repeat(left_pad),
                h,
                " ".repeat(right_pad)
            ));
        }
    }

    while lines.len() < content_height {
        lines.push(" ".repeat(cols));
    }

    lines
}

fn current_file_label(state: &PagerState) -> String {
    if state.line_map.is_empty() {
        return String::new();
    }
    if let Some(fi) = state.active_file {
        let total = state.file_starts.len();
        let pos = state.file_starts[fi].min(state.line_map.len() - 1);
        let info = &state.line_map[pos];
        let name = info.path.rsplit('/').next().unwrap_or(&info.path);
        return format!("{name} ({}/{total})", fi + 1);
    }
    let pos = state.cursor_line.min(state.line_map.len() - 1);
    let info = &state.line_map[pos];
    let file_idx = info.file_idx + 1;
    let total = state.file_starts.len();
    let name = info.path.rsplit('/').next().unwrap_or(&info.path);
    format!("{name} ({file_idx}/{total})")
}

fn format_status_bar(state: &PagerState, content_height: usize, cols: usize) -> String {
    if state.mode == Mode::Visual {
        let lo = state.visual_anchor.min(state.cursor_line);
        let hi = state.visual_anchor.max(state.cursor_line);
        let count = hi - lo + 1;
        let left = format!("-- VISUAL -- ({count} lines)");
        let left_len = left.len();
        let pad = cols.saturating_sub(left_len);
        return format!("{left}{}", " ".repeat(pad));
    }

    if state.mode == Mode::Help {
        let left = "? to close";
        let line_count = state.lines.len();
        let max_top = max_scroll(line_count, content_height);
        let top = state.top_line.min(max_top);
        let end = (top + content_height).min(line_count);
        let range = format!("{}-{}/{}", top + 1, end, line_count);
        let position = if top == 0 {
            "TOP".to_string()
        } else if end >= line_count {
            "END".to_string()
        } else {
            format!("{}%", (end as f64 / line_count as f64 * 100.0).round() as usize)
        };
        let right = format!("{}{}{} {}", style::DIM, range, style::NO_DIM, position);
        let right_vis = range.len() + 1 + position.len();
        let total_vis = left.len() + right_vis;
        if total_vis >= cols {
            let pad = " ".repeat(cols.saturating_sub(right_vis));
            return format!("{pad}{right}");
        }
        let gap = cols - total_vis;
        return format!("{left}{}{right}", " ".repeat(gap));
    }

    if state.mode == Mode::Search {
        let before = &state.search_input[..state.search_cursor];
        let after = &state.search_input[state.search_cursor..];
        let cursor_char = if state.search_cursor < state.search_input.len() {
            let c = after.chars().next().unwrap();
            let rest = &after[c.len_utf8()..];
            format!("{}{c}{}{rest}", style::NO_REVERSE, style::REVERSE)
        } else {
            format!("{}\u{2588}{}", style::NO_REVERSE, style::REVERSE)
        };
        let content = format!("/{before}{cursor_char}");
        let vis_len = if state.search_cursor < state.search_input.len() {
            1 + state.search_input.len()
        } else {
            1 + state.search_input.len() + 1
        };
        let pad = " ".repeat(cols.saturating_sub(vis_len));
        return format!("{content}{pad}");
    }

    if !state.status_message.is_empty() {
        let msg = &state.status_message;
        let pad = " ".repeat(cols.saturating_sub(msg.len()));
        return format!("{msg}{pad}");
    }

    // Normal mode
    let line_count = state.lines.len();
    let max_top = max_scroll(line_count, content_height);
    let top = state.top_line.min(max_top);
    let end = (top + content_height).min(line_count);
    let range = format!("{}-{}/{}", top + 1, end, line_count);
    let position = if top == 0 {
        "TOP".to_string()
    } else if end >= line_count {
        "END".to_string()
    } else {
        format!("{}%", (end as f64 / line_count as f64 * 100.0).round() as usize)
    };
    let right = format!("{}{}{} {}", style::DIM, range, style::NO_DIM, position);
    let right_vis = range.len() + 1 + position.len();

    let file_label = if !state.search_query.is_empty() {
        if state.current_match >= 0 {
            format!(
                "/{} ({}/{})",
                state.search_query,
                state.current_match + 1,
                state.search_matches.len()
            )
        } else {
            format!("/{}", state.search_query)
        }
    } else {
        current_file_label(state)
    };
    let left = if state.tree_focused {
        format!("TREE │ {file_label}")
    } else {
        file_label
    };

    let left_vis = left.len();
    let total_vis = left_vis + right_vis;
    if total_vis >= cols {
        let pad = " ".repeat(cols.saturating_sub(right_vis));
        format!("{pad}{right}")
    } else {
        let gap = cols - total_vis;
        format!("{left}{}{right}", " ".repeat(gap))
    }
}

fn render_content_area(out: &mut impl Write, state: &PagerState, cols: u16, content_rows: u16) {
    let content_height = content_rows as usize;
    let max_top = max_scroll(state.lines.len(), content_height);
    let top = state.top_line.min(max_top);

    if state.mode == Mode::Help {
        let help_lines = format_help_lines(cols as usize, (content_rows + 1) as usize);
        for (i, line) in help_lines.iter().enumerate() {
            move_to(out, i as u16, 0);
            let _ = write!(out, "{CLEAR_LINE}{}{line}{}", style::DIM, style::NO_DIM);
        }
        return;
    }

    let diff_w = diff_area_width(cols, state.tree_width, state.tree_visible);

    for row in 0..content_height {
        move_to(out, row as u16, 0);
        let idx = top + row;
        if idx < state.lines.len() {
            let mut line = state.lines[idx].clone();
            if !state.search_query.is_empty() {
                line = highlight_search(&line, &state.search_query);
            }
            if state.mode == Mode::Visual {
                let lo = state.visual_anchor.min(state.cursor_line);
                let hi = state.visual_anchor.max(state.cursor_line);
                if idx >= lo && idx <= hi {
                    line = highlight_visual_line(&line, diff_w);
                }
            }
            if state.cursor_visible && idx == state.cursor_line && state.mode != Mode::Visual {
                line = format!("{}{line}{}", style::UNDERLINE, style::NO_UNDERLINE);
            }
            let _ = write!(out, "{CLEAR_LINE}{line}");
        } else {
            let _ = write!(out, "{CLEAR_LINE}");
        }

        if state.tree_visible {
            let _ = write!(
                out,
                "{}\x1b[{}G\x1b[K{}│{}",
                style::RESET,
                diff_w + 1,
                style::FG_SEP,
                style::RESET,
            );
            if let Some(tree_line) = state.tree_lines.get(state.tree_scroll + row) {
                let _ = write!(out, "{tree_line}");
            }
            let _ = write!(out, "{}", style::RESET);
        }
    }
}

fn render_status_bar(out: &mut impl Write, state: &PagerState, cols: u16, content_rows: u16) {
    let content_height = content_rows as usize;
    move_to(out, content_rows, 0);
    let _ = write!(out, "{CLEAR_LINE}");
    let status = format_status_bar(state, content_height, cols as usize);
    let _ = write!(out, "{}{}{}{}", style::RESET, style::REVERSE, status, style::RESET);
}

fn render_screen(out: &mut impl Write, state: &PagerState, cols: u16, rows: u16) {
    let content_rows = rows.saturating_sub(1);
    render_content_area(out, state, cols, content_rows);
    render_status_bar(out, state, cols, content_rows);
    let _ = out.flush();
}

fn scroll_to_match(state: &mut PagerState, rows: u16) {
    if state.current_match < 0 || state.current_match as usize >= state.search_matches.len() {
        return;
    }
    let match_line = state.search_matches[state.current_match as usize];
    let content_height = rows.saturating_sub(1) as usize;
    if state.cursor_visible {
        state.cursor_line = match_line;
        enforce_scrolloff(state, content_height);
    } else {
        let target = match_line.saturating_sub(content_height / 3);
        let max_top = max_scroll(state.lines.len(), content_height);
        state.top_line = target.min(max_top);
        state.cursor_line = state.top_line;
    }
}

/// Jump to next entry in `targets` after current top_line.
fn jump_next(targets: &[usize], top_line: usize) -> Option<usize> {
    targets.iter().find(|&&t| t > top_line).copied()
}

/// Jump to previous entry in `targets` before current top_line.
fn jump_prev(targets: &[usize], top_line: usize) -> Option<usize> {
    targets.iter().rev().find(|&&t| t < top_line).copied()
}

/// Re-render the diff at a new width, preserving scroll position by anchoring
/// to the file_idx/new_lineno of the current top line.
fn re_render(
    state: &mut PagerState,
    files: &[DiffFile],
    color: bool,
    cols: u16,
    content_height: usize,
) {
    // Capture anchor from current top line (subtract top_padding to get un-padded position)
    let anchor = if !state.line_map.is_empty() {
        let unpadded_top = state.top_line.saturating_sub(state.top_padding);
        let top = unpadded_top.min(state.line_map.len() - 1);
        let info = &state.line_map[top];
        Some((info.file_idx, info.new_lineno))
    } else {
        None
    };

    let width = diff_area_width(cols, state.tree_width, state.tree_visible);
    let output = render::render(files, width, color);
    state.lines = output.lines;
    state.line_map = output.line_map;
    state.file_starts = output.file_starts;
    state.hunk_starts = output.hunk_starts;
    state.top_padding = 0;

    // Restore scroll position by finding the anchored line
    if let Some((file_idx, new_lineno)) = anchor {
        state.top_line = state
            .line_map
            .iter()
            .position(|li| li.file_idx == file_idx && li.new_lineno == new_lineno)
            .unwrap_or(0);
    }
    state.cursor_line = state.top_line;
    state.visual_anchor = state.cursor_line;

    // Re-run search against new lines
    if !state.search_query.is_empty() {
        state.search_matches = find_matches(&state.lines, &state.search_query);
        state.current_match = find_nearest_match(&state.search_matches, state.top_line);
    }

    // Rebuild tree entries and lines at current cursor
    if state.tree_visible {
        state.tree_entries = build_tree_entries(files);
        state.tree_width = compute_tree_width(&state.tree_entries);
        let cursor_entry_idx = file_idx_to_entry_idx(&state.tree_entries, state.tree_cursor);
        state.tree_lines = build_tree_lines(&state.tree_entries, cursor_entry_idx, state.tree_width);
    }

    apply_top_padding(state, content_height);
}

#[allow(dead_code)]
struct TreeEntry {
    label: String,
    depth: usize,
    file_idx: Option<usize>,
    status: Option<FileStatus>,
}

fn build_tree_entries(files: &[DiffFile]) -> Vec<TreeEntry> {
    // Sort files by path for grouping
    let mut indexed: Vec<(usize, &str)> = files.iter().enumerate().map(|(i, f)| (i, f.path())).collect();
    indexed.sort_by(|a, b| a.1.cmp(b.1));

    let mut entries: Vec<TreeEntry> = Vec::new();
    let mut prev_components: Vec<&str> = Vec::new();

    for (file_idx, path) in &indexed {
        let parts: Vec<&str> = path.split('/').collect();
        let dir_parts = &parts[..parts.len() - 1];
        let basename = parts[parts.len() - 1];

        // Find common prefix length with previous path's directory components
        let common = prev_components
            .iter()
            .zip(dir_parts.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // Emit new directory entries for components beyond the common prefix
        for (depth, &component) in dir_parts.iter().enumerate().skip(common) {
            entries.push(TreeEntry {
                label: component.to_string(),
                depth,
                file_idx: None,
                status: None,
            });
        }

        // Emit the file entry
        entries.push(TreeEntry {
            label: basename.to_string(),
            depth: dir_parts.len(),
            file_idx: Some(*file_idx),
            status: Some(files[*file_idx].status.clone()),
        });

        prev_components = dir_parts.to_vec();
    }

    // Post-process: collapse single-child directories
    let mut i = 0;
    while i + 1 < entries.len() {
        if entries[i].file_idx.is_none() && entries[i + 1].file_idx.is_none()
            && entries[i + 1].depth == entries[i].depth + 1
        {
            // Check that the parent has no other children (no sibling at same depth between i+1 and next entry at parent depth or lower)
            let parent_depth = entries[i].depth;
            let has_sibling = entries[i + 2..].iter().any(|e| e.depth <= parent_depth + 1 && e.depth > parent_depth)
                && entries[i + 2..].iter().take_while(|e| e.depth > parent_depth).any(|e| e.depth == parent_depth + 1);
            if !has_sibling {
                // Merge: join labels with '/'
                let child_label = entries[i + 1].label.clone();
                entries[i].label = format!("{}/{}", entries[i].label, child_label);
                // All subsequent entries that were children of the removed child dir need depth decremented
                let removed_depth = entries[i + 1].depth;
                entries.remove(i + 1);
                for e in &mut entries[i + 1..] {
                    if e.depth > removed_depth {
                        e.depth -= 1;
                    } else {
                        break;
                    }
                }
                continue; // Re-check in case of further collapsing
            }
        }
        i += 1;
    }

    entries
}

fn compute_tree_width(tree_entries: &[TreeEntry]) -> usize {
    let max_len = tree_entries
        .iter()
        .map(|e| (e.depth + 1) * 4 + 2 + e.label.len() + 2) // connectors + icon+space + label + padding
        .max()
        .unwrap_or(0);
    max_len.min(40)
}

fn file_idx_to_entry_idx(tree_entries: &[TreeEntry], file_idx: usize) -> usize {
    tree_entries
        .iter()
        .position(|e| e.file_idx == Some(file_idx))
        .unwrap_or(0)
}

/// Build the Unicode box-drawing prefix for a tree entry at `idx`.
/// Each depth level contributes 4 characters: either a continuation pipe
/// (`│   `) or blank (`    `), and the entry's own connector (`├── ` or `└── `).
fn compute_connector_prefix(entries: &[TreeEntry], idx: usize) -> String {
    let depth = entries[idx].depth;
    let mut prefix = String::new();

    // Ancestor columns: for each depth 0..depth-1, draw a continuation pipe
    // if any subsequent entry returns to that depth (meaning the branch continues)
    for d in 0..depth {
        let has_continuation = entries[idx + 1..].iter().any(|e| e.depth <= d);
        if has_continuation {
            prefix.push_str("│   ");
        } else {
            prefix.push_str("    ");
        }
    }

    // Entry's own connector: check if a sibling follows at the same depth
    let has_sibling_after = entries[idx + 1..]
        .iter()
        .take_while(|e| e.depth >= depth)
        .any(|e| e.depth == depth);
    if has_sibling_after {
        prefix.push_str("├── ");
    } else {
        prefix.push_str("└── ");
    }

    prefix
}

fn build_tree_lines(tree_entries: &[TreeEntry], cursor_entry_idx: usize, width: usize) -> Vec<String> {
    let mut lines = Vec::new();

    // Tree entries with box-drawing connectors
    for (i, entry) in tree_entries.iter().enumerate() {
        let prefix = compute_connector_prefix(tree_entries, i);
        let (icon, icon_color) = if entry.file_idx.is_some() {
            style::file_icon(&entry.label)
        } else {
            style::dir_icon()
        };

        // prefix is (depth+1)*4 chars, plus icon(1) + space(1) + label
        let vis_len = (entry.depth + 1) * 4 + 2 + entry.label.chars().count();
        let right_pad = width.saturating_sub(vis_len);
        let guide = style::FG_TREE_GUIDE;

        if i == cursor_entry_idx {
            let reset = style::RESET;
            let fg = style::FG_FILE_HEADER;
            let label = &entry.label;
            let rpad = " ".repeat(right_pad);
            let padded = format!("{guide}{prefix}{reset}{icon_color}{icon}{reset} {fg}{label}{rpad}");
            lines.push(format!("{reset}{fg}\x1b[7m{padded}\x1b[27m{reset}"));
        } else if entry.file_idx.is_some() {
            let reset = style::RESET;
            let fg = style::FG_TREE;
            let label = &entry.label;
            let rpad = " ".repeat(right_pad);
            lines.push(format!("{guide}{prefix}{reset}{icon_color}{icon}{reset} {fg}{label}{rpad}{reset}"));
        } else {
            let reset = style::RESET;
            let fg = style::FG_TREE_DIR;
            let label = &entry.label;
            let rpad = " ".repeat(right_pad);
            lines.push(format!("{guide}{prefix}{reset}{icon_color}{icon}{reset} {fg}{label}{rpad}{reset}"));
        }
    }

    lines
}

fn sync_tree_cursor(state: &mut PagerState, content_height: usize) {
    if !state.tree_visible || state.tree_focused {
        return;
    }
    let new_cursor = if let Some(fi) = state.active_file {
        fi
    } else {
        let anchor = if state.cursor_visible {
            state.cursor_line
        } else {
            state.top_line
        };
        state
            .line_map
            .get(anchor)
            .map(|li| li.file_idx)
            .unwrap_or(0)
    };
    let new_entry_idx = file_idx_to_entry_idx(&state.tree_entries, new_cursor);
    if new_entry_idx != state.tree_cursor {
        state.tree_cursor = new_entry_idx;
        state.tree_lines = build_tree_lines(&state.tree_entries, state.tree_cursor, state.tree_width);
        ensure_tree_cursor_visible(state, content_height);
    }
}

fn ensure_tree_cursor_visible(state: &mut PagerState, content_height: usize) {
    let offset = state.tree_cursor;
    if offset < state.tree_scroll {
        state.tree_scroll = offset;
    }
    if offset >= state.tree_scroll + content_height {
        state.tree_scroll = offset + 1 - content_height;
    }
}

fn regenerate_files(diff_ctx: &DiffContext, full_context: bool) -> Vec<DiffFile> {
    let diff_args = if full_context {
        diff_ctx.source.diff_args_full_context()
    } else {
        diff_ctx.source.diff_args()
    };
    let str_args: Vec<&str> = diff_args.iter().map(String::as_str).collect();
    let raw = crate::git::run(&diff_ctx.repo, &str_args).unwrap_or_default();
    let mut files = crate::git::diff::parse(&raw);

    if !diff_ctx.no_untracked && matches!(diff_ctx.source, crate::git::DiffSource::WorkingTree) {
        let max_size: u64 = 256 * 1024;
        for path in crate::git::untracked_files(&diff_ctx.repo) {
            let full = diff_ctx.repo.join(&path);
            let meta = match full.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if !meta.is_file() || meta.len() > max_size {
                continue;
            }
            let content = match std::fs::read(&full) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };
            if content.contains(&0) {
                continue;
            }
            let text = String::from_utf8_lossy(&content);
            files.push(DiffFile::from_content(&path, &text));
        }
    }

    files
}

pub fn run_pager(output: RenderOutput, files: Vec<DiffFile>, color: bool, diff_ctx: DiffContext) {
    let mut files = files;
    let mut stdout = io::BufWriter::new(io::stdout());

    let _ = write!(stdout, "{ALT_SCREEN_ON}{CURSOR_HIDE}");
    let _ = stdout.flush();
    let _ = crossterm::terminal::enable_raw_mode();

    let mut state = PagerState {
        lines: output.lines,
        line_map: output.line_map,
        file_starts: output.file_starts,
        hunk_starts: output.hunk_starts,
        top_line: 0,
        cursor_line: 0,
        cursor_visible: false,
        visual_anchor: 0,
        search_query: String::new(),
        search_matches: Vec::new(),
        current_match: -1,
        mode: Mode::Normal,
        search_input: String::new(),
        search_cursor: 0,
        status_message: String::new(),
        pending_bracket: None,
        tree_visible: false,
        tree_focused: false,
        tree_cursor: 0,
        tree_width: 0,
        tree_scroll: 0,
        tree_lines: vec![],
        tree_entries: Vec::new(),
        active_file: None,
        top_padding: 0,
        full_context: false,
    };

    // Initialize file tree panel
    state.tree_entries = build_tree_entries(&files);
    state.tree_width = compute_tree_width(&state.tree_entries);
    state.tree_lines = build_tree_lines(&state.tree_entries, 0, state.tree_width);
    state.tree_visible = false;

    let mut last_size = get_term_size();
    let content_height = last_size.1.saturating_sub(1) as usize;
    re_render(&mut state, &files, color, last_size.0, content_height);
    render_screen(&mut stdout, &state, last_size.0, last_size.1);

    loop {
        let ev = match event::poll(Duration::from_millis(50)) {
            Ok(true) => match event::read() {
                Ok(ev) => ev,
                Err(_) => break,
            },
            Ok(false) => {
                let current_size = get_term_size();
                if current_size != last_size {
                    last_size = current_size;
                    re_render(&mut state, &files, color, last_size.0, last_size.1.saturating_sub(1) as usize);
                    let ch = last_size.1.saturating_sub(1) as usize;
                    ensure_tree_cursor_visible(&mut state, ch);
                    render_screen(&mut stdout, &state, last_size.0, last_size.1);
                }
                continue;
            }
            Err(_) => break,
        };

        let key = match ev {
            Event::Resize(_, _) => {
                last_size = get_term_size();
                re_render(&mut state, &files, color, last_size.0, last_size.1.saturating_sub(1) as usize);
                let ch = last_size.1.saturating_sub(1) as usize;
                ensure_tree_cursor_visible(&mut state, ch);
                render_screen(&mut stdout, &state, last_size.0, last_size.1);
                continue;
            }
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                crossterm_to_key(key_event)
            }
            _ => continue,
        };

        if state.mode == Mode::Search {
            handle_search_key(&mut state, &key);
            if state.mode == Mode::Normal && state.current_match >= 0 {
                scroll_to_match(&mut state, last_size.1);
                let ch = last_size.1.saturating_sub(1) as usize;
                sync_tree_cursor(&mut state, ch);
            }
            render_screen(&mut stdout, &state, last_size.0, last_size.1);
            continue;
        }

        if state.mode == Mode::Help {
            state.mode = Mode::Normal;
            render_screen(&mut stdout, &state, last_size.0, last_size.1);
            continue;
        }

        if state.mode == Mode::Visual {
            let content_height = last_size.1.saturating_sub(1) as usize;
            match key {
                Key::Escape | Key::CtrlC => {
                    state.mode = Mode::Normal;
                    state.cursor_line = state.top_line;
                }
                Key::Char('j') | Key::Down => {
                    let next = state.cursor_line + 1;
                    let anchor_file = state
                        .line_map
                        .get(state.visual_anchor)
                        .map(|l| l.file_idx)
                        .unwrap_or(0);
                    let next_file = state
                        .line_map
                        .get(next)
                        .map(|l| l.file_idx)
                        .unwrap_or(usize::MAX);
                    if next < state.lines.len() && next_file == anchor_file {
                        state.cursor_line = next;
                        if state.cursor_line >= state.top_line + content_height {
                            state.top_line = state.cursor_line - content_height + 1;
                        }
                    }
                }
                Key::Char('k') | Key::Up => {
                    if state.cursor_line > 0 {
                        let prev = state.cursor_line - 1;
                        let anchor_file = state
                            .line_map
                            .get(state.visual_anchor)
                            .map(|l| l.file_idx)
                            .unwrap_or(0);
                        let prev_file = state
                            .line_map
                            .get(prev)
                            .map(|l| l.file_idx)
                            .unwrap_or(usize::MAX);
                        if prev_file == anchor_file {
                            state.cursor_line = prev;
                            if state.cursor_line < state.top_line {
                                state.top_line = state.cursor_line;
                            }
                        }
                    }
                }
                Key::Char('y') => {
                    let lo = state.visual_anchor.min(state.cursor_line);
                    let hi = state.visual_anchor.max(state.cursor_line);
                    let path = state
                        .line_map
                        .get(lo)
                        .map(|l| l.path.clone())
                        .unwrap_or_default();
                    let (start, end) = resolve_lineno(&state.line_map, lo, hi);
                    let text = format_copy_ref(&path, start, end);
                    let ok = copy_to_clipboard(&text);
                    state.status_message = if ok {
                        format!("Copied: {text}")
                    } else {
                        "Copy failed (pbcopy not available)".to_string()
                    };
                    state.mode = Mode::Normal;
                    state.cursor_line = state.top_line;
                }
                Key::Char('q') => break,
                _ => {}
            }
            render_screen(&mut stdout, &state, last_size.0, last_size.1);
            continue;
        }

        // Tree focus mode
        if state.tree_focused {
            match key {
                Key::Char('j') | Key::Down => {
                    let idx = state.tree_cursor;
                    if let Some(next) = state.tree_entries[idx + 1..]
                        .iter()
                        .position(|e| e.file_idx.is_some())
                        .map(|p| idx + 1 + p)
                    {
                        state.tree_cursor = next;
                        let fi = state.tree_entries[next].file_idx.unwrap();
                        state.active_file = Some(fi);
                        state.top_line = state.file_starts[fi];
                        state.cursor_line = state.top_line;
                        state.tree_lines =
                            build_tree_lines(&state.tree_entries, state.tree_cursor, state.tree_width);
                        let ch = last_size.1.saturating_sub(1) as usize;
                        ensure_tree_cursor_visible(&mut state, ch);
                    }
                }
                Key::Char('k') | Key::Up => {
                    let idx = state.tree_cursor;
                    if let Some(prev) = state.tree_entries[..idx]
                        .iter()
                        .rposition(|e| e.file_idx.is_some())
                    {
                        state.tree_cursor = prev;
                        let fi = state.tree_entries[prev].file_idx.unwrap();
                        state.active_file = Some(fi);
                        state.top_line = state.file_starts[fi];
                        state.cursor_line = state.top_line;
                        state.tree_lines =
                            build_tree_lines(&state.tree_entries, state.tree_cursor, state.tree_width);
                        let ch = last_size.1.saturating_sub(1) as usize;
                        ensure_tree_cursor_visible(&mut state, ch);
                    }
                }
                Key::Enter => {
                    if let Some(file_idx) = state.tree_entries.get(state.tree_cursor).and_then(|e| e.file_idx) {
                        state.active_file = Some(file_idx);
                        if let Some(&target) = state.file_starts.get(file_idx) {
                            state.top_line = target;
                            state.cursor_line = state.top_line;
                        }
                    }
                }
                Key::CtrlH | Key::Escape | Key::Tab => {
                    state.tree_focused = false;
                }
                Key::Char('e') => {
                    state.active_file = None;
                    state.tree_visible = false;
                    state.tree_focused = false;
                    re_render(&mut state, &files, color, last_size.0, last_size.1.saturating_sub(1) as usize);
                }
                Key::Char('q') | Key::CtrlC => break,
                _ => {}
            }
            render_screen(&mut stdout, &state, last_size.0, last_size.1);
            continue;
        }

        // Handle pending bracket sequences
        if let Some(bracket) = state.pending_bracket.take() {
            let rows = last_size.1;
            let content_height = rows.saturating_sub(1) as usize;
            let max_top = max_scroll(state.lines.len(), content_height);
            let anchor = if state.cursor_visible {
                state.cursor_line
            } else {
                state.top_line
            };
            let mut jump_target: Option<usize> = None;
            match (bracket, &key) {
                (']', Key::Char('c')) => {
                    if state.active_file.is_some() {
                        let (rs, re) = visible_range(&state);
                        let filtered: Vec<usize> = state.hunk_starts.iter().copied()
                            .filter(|&h| h >= rs && h < re).collect();
                        jump_target = jump_next(&filtered, anchor);
                    } else if let Some(t) = jump_next(&state.hunk_starts, anchor) {
                        jump_target = Some(t);
                    }
                }
                ('[', Key::Char('c')) => {
                    if state.active_file.is_some() {
                        let (rs, re) = visible_range(&state);
                        let filtered: Vec<usize> = state.hunk_starts.iter().copied()
                            .filter(|&h| h >= rs && h < re).collect();
                        jump_target = jump_prev(&filtered, anchor);
                    } else if let Some(t) = jump_prev(&state.hunk_starts, anchor) {
                        jump_target = Some(t);
                    }
                }
                (']', Key::Char('f')) => {
                    if let Some(idx) = state.active_file {
                        let next = (idx + 1).min(state.file_starts.len() - 1);
                        if next != idx {
                            state.active_file = Some(next);
                            state.top_line = state.file_starts[next];
                            state.cursor_line = state.top_line;
                            state.tree_cursor = file_idx_to_entry_idx(&state.tree_entries, next);
                            state.tree_lines = build_tree_lines(&state.tree_entries, state.tree_cursor, state.tree_width);
                            ensure_tree_cursor_visible(&mut state, content_height);
                        }
                    } else if let Some(t) = jump_next(&state.file_starts, anchor) {
                        jump_target = Some(t);
                    }
                }
                ('[', Key::Char('f')) => {
                    if let Some(idx) = state.active_file {
                        let prev = idx.saturating_sub(1);
                        if prev != idx {
                            state.active_file = Some(prev);
                            state.top_line = state.file_starts[prev];
                            state.cursor_line = state.top_line;
                            state.tree_cursor = file_idx_to_entry_idx(&state.tree_entries, prev);
                            state.tree_lines = build_tree_lines(&state.tree_entries, state.tree_cursor, state.tree_width);
                            ensure_tree_cursor_visible(&mut state, content_height);
                        }
                    } else if let Some(t) = jump_prev(&state.file_starts, anchor) {
                        jump_target = Some(t);
                    }
                }
                _ => {} // Unknown sequence — ignore
            }
            if let Some(target) = jump_target {
                if state.cursor_visible {
                    state.cursor_line = target;
                    state.top_line = target
                        .saturating_sub(content_height / 2)
                        .min(max_top);
                    enforce_scrolloff(&mut state, content_height);
                } else {
                    state.top_line = target
                        .saturating_sub(content_height / 2)
                        .min(max_top);
                    state.cursor_line = state.top_line;
                }
            }
            let ch = last_size.1.saturating_sub(1) as usize;
            sync_tree_cursor(&mut state, ch);
            render_screen(&mut stdout, &state, last_size.0, last_size.1);
            continue;
        }

        // Normal mode
        let rows = last_size.1;
        let content_height = rows.saturating_sub(1) as usize;
        let half_page = content_height / 2;
        let (range_start, range_end) = visible_range(&state);
        let range_lines = range_end - range_start;
        let max_top = if state.active_file.is_some() {
            range_start + range_lines.saturating_sub(content_height)
        } else {
            max_scroll(state.lines.len(), content_height)
        };
        let max_cursor = range_end.saturating_sub(1);

        state.status_message.clear();

        match key {
            Key::Char('q') | Key::CtrlC => break,
            Key::Char('j') | Key::Down | Key::Enter => {
                if state.cursor_visible {
                    state.cursor_line = (state.cursor_line + 1).min(max_cursor);
                } else {
                    state.top_line = (state.top_line + 1).min(max_top);
                }
            }
            Key::Char('k') | Key::Up => {
                if state.cursor_visible {
                    state.cursor_line =
                        state.cursor_line.saturating_sub(1).max(range_start);
                } else {
                    state.top_line =
                        state.top_line.saturating_sub(1).max(range_start);
                }
            }
            Key::Char('d') | Key::CtrlD | Key::PageDown => {
                if state.cursor_visible {
                    state.cursor_line =
                        (state.cursor_line + half_page).min(max_cursor);
                } else {
                    state.top_line = (state.top_line + half_page).min(max_top);
                }
            }
            Key::Char('u') | Key::CtrlU | Key::PageUp => {
                if state.cursor_visible {
                    state.cursor_line =
                        state.cursor_line.saturating_sub(half_page).max(range_start);
                } else {
                    state.top_line =
                        state.top_line.saturating_sub(half_page).max(range_start);
                }
            }
            Key::Char('g') | Key::Home => {
                if state.cursor_visible {
                    state.cursor_line = range_start;
                } else {
                    state.top_line = range_start;
                }
            }
            Key::Char('G') | Key::End => {
                if state.cursor_visible {
                    state.cursor_line = max_cursor;
                } else {
                    state.top_line = max_top;
                }
            }
            Key::Char(' ') => {
                state.cursor_visible = !state.cursor_visible;
                if state.cursor_visible {
                    state.cursor_line = (state.top_line + content_height / 2)
                        .min(state.lines.len().saturating_sub(1));
                }
            }
            Key::Char(c @ (']' | '[')) => {
                state.pending_bracket = Some(c);
                continue; // Don't render yet — wait for second key
            }
            Key::Char('/') => {
                state.mode = Mode::Search;
            }
            Key::Char('n') => {
                if !state.search_matches.is_empty() {
                    if state.active_file.is_some() {
                        let (rs, re) = visible_range(&state);
                        let filtered: Vec<usize> = state.search_matches.iter().copied()
                            .filter(|&m| m >= rs && m < re).collect();
                        if !filtered.is_empty() {
                            let cur_line = if state.current_match >= 0 {
                                state.search_matches[state.current_match as usize]
                            } else {
                                0
                            };
                            if let Some(pos) = filtered.iter().position(|&m| m > cur_line) {
                                let global = state.search_matches.iter().position(|&m| m == filtered[pos]).unwrap();
                                state.current_match = global as isize;
                            } else {
                                let global = state.search_matches.iter().position(|&m| m == filtered[0]).unwrap();
                                state.current_match = global as isize;
                            }
                            scroll_to_match(&mut state, rows);
                        }
                    } else {
                        state.current_match =
                            (state.current_match + 1) % state.search_matches.len() as isize;
                        scroll_to_match(&mut state, rows);
                    }
                }
            }
            Key::Char('N') => {
                if !state.search_matches.is_empty() {
                    if state.active_file.is_some() {
                        let (rs, re) = visible_range(&state);
                        let filtered: Vec<usize> = state.search_matches.iter().copied()
                            .filter(|&m| m >= rs && m < re).collect();
                        if !filtered.is_empty() {
                            let cur_line = if state.current_match >= 0 {
                                state.search_matches[state.current_match as usize]
                            } else {
                                usize::MAX
                            };
                            if let Some(pos) = filtered.iter().rposition(|&m| m < cur_line) {
                                let global = state.search_matches.iter().position(|&m| m == filtered[pos]).unwrap();
                                state.current_match = global as isize;
                            } else {
                                let last = *filtered.last().unwrap();
                                let global = state.search_matches.iter().position(|&m| m == last).unwrap();
                                state.current_match = global as isize;
                            }
                            scroll_to_match(&mut state, rows);
                        }
                    } else {
                        state.current_match = (state.current_match - 1
                            + state.search_matches.len() as isize)
                            % state.search_matches.len() as isize;
                        scroll_to_match(&mut state, rows);
                    }
                }
            }
            Key::Char('E') => {
                let pos = state.cursor_line.min(state.line_map.len().saturating_sub(1));
                if !state.line_map.is_empty() {
                    let info = &state.line_map[pos];
                    let path = info.path.clone();
                    let lineno = info.new_lineno;

                    let _ = crossterm::terminal::disable_raw_mode();
                    let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
                    let _ = stdout.flush();

                    open_in_editor(&path, lineno);

                    let _ = write!(stdout, "{ALT_SCREEN_ON}{CURSOR_HIDE}");
                    let _ = stdout.flush();
                    let _ = crossterm::terminal::enable_raw_mode();
                    last_size = get_term_size();
                }
            }
            Key::Char('e') => {
                if !state.tree_visible {
                    state.tree_visible = true;
                    let anchor = if state.cursor_visible {
                        state.cursor_line
                    } else {
                        state.top_line
                    };
                    let file_idx = state
                        .line_map
                        .get(anchor)
                        .map(|li| li.file_idx)
                        .unwrap_or(0);
                    state.active_file = Some(file_idx);
                    state.tree_entries = build_tree_entries(&files);
                    state.tree_width = compute_tree_width(&state.tree_entries);
                    state.tree_cursor = file_idx_to_entry_idx(&state.tree_entries, file_idx);
                    ensure_tree_cursor_visible(&mut state, content_height);
                    state.tree_focused = true;
                    re_render(&mut state, &files, color, last_size.0, content_height);
                } else if !state.tree_focused {
                    state.tree_focused = true;
                    re_render(&mut state, &files, color, last_size.0, content_height);
                }
            }
            Key::Tab => {
                if state.tree_visible {
                    state.tree_focused = true;
                    ensure_tree_cursor_visible(&mut state, content_height);
                }
            }
            Key::CtrlL => {
                if !state.tree_visible {
                    state.tree_visible = true;
                    let anchor = if state.cursor_visible {
                        state.cursor_line
                    } else {
                        state.top_line
                    };
                    let file_idx = state
                        .line_map
                        .get(anchor)
                        .map(|li| li.file_idx)
                        .unwrap_or(0);
                    state.active_file = Some(file_idx);
                    state.tree_entries = build_tree_entries(&files);
                    state.tree_width = compute_tree_width(&state.tree_entries);
                    state.tree_cursor = file_idx_to_entry_idx(&state.tree_entries, file_idx);
                    re_render(&mut state, &files, color, last_size.0, content_height);
                }
                state.tree_focused = true;
                ensure_tree_cursor_visible(&mut state, content_height);
            }
            Key::Char('v') => {
                state.mode = Mode::Visual;
                state.visual_anchor = state.cursor_line;
            }
            Key::Char('o') => {
                state.full_context = !state.full_context;
                files = regenerate_files(&diff_ctx, state.full_context);
                re_render(&mut state, &files, color, last_size.0, content_height);
                state.status_message = if state.full_context {
                    "Full file context".into()
                } else {
                    "Hunk context".into()
                };
            }
            Key::Char('?') => {
                state.mode = Mode::Help;
            }
            _ => {}
        }

        if state.cursor_visible {
            enforce_scrolloff(&mut state, content_height);
        } else {
            state.cursor_line = state.top_line;
        }
        sync_tree_cursor(&mut state, content_height);
        render_screen(&mut stdout, &state, last_size.0, last_size.1);
    }

    let _ = crossterm::terminal::disable_raw_mode();
    let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
    let _ = stdout.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(label: &str, depth: usize, file_idx: Option<usize>) -> TreeEntry {
        TreeEntry {
            label: label.to_string(),
            depth,
            file_idx,
            status: file_idx.map(|_| FileStatus::Modified),
        }
    }

    #[test]
    fn test_compute_connector_prefix_flat() {
        let entries = vec![
            entry("a.rs", 0, Some(0)),
            entry("b.rs", 0, Some(1)),
            entry("c.rs", 0, Some(2)),
        ];
        assert_eq!(compute_connector_prefix(&entries, 0), "├── ");
        assert_eq!(compute_connector_prefix(&entries, 1), "├── ");
        assert_eq!(compute_connector_prefix(&entries, 2), "└── ");
    }

    #[test]
    fn test_compute_connector_prefix_nested() {
        // src/
        //   a.rs
        //   b.rs
        // README.md
        let entries = vec![
            entry("src", 0, None),
            entry("a.rs", 1, Some(0)),
            entry("b.rs", 1, Some(1)),
            entry("README.md", 0, Some(2)),
        ];
        // src dir: has sibling README.md at depth 0 after it
        assert_eq!(compute_connector_prefix(&entries, 0), "├── ");
        // a.rs: parent (depth 0) continues, sibling b.rs at depth 1 follows
        assert_eq!(compute_connector_prefix(&entries, 1), "│   ├── ");
        // b.rs: parent (depth 0) continues, no more siblings at depth 1
        assert_eq!(compute_connector_prefix(&entries, 2), "│   └── ");
        // README.md: last root entry
        assert_eq!(compute_connector_prefix(&entries, 3), "└── ");
    }

    #[test]
    fn test_build_tree_lines_no_header() {
        let entries = vec![
            entry("a.rs", 0, Some(0)),
            entry("b.rs", 0, Some(1)),
        ];
        let width = compute_tree_width(&entries);
        let lines = build_tree_lines(&entries, 0, width);
        // First line should be the first tree entry, not a "CHANGED FILES" header
        let first = crate::ansi::strip_ansi(&lines[0]);
        assert!(!first.contains("CHANGED FILES"), "header should be removed");
    }
}
