use std::io::Write;

use tui::pager::{move_to, CLEAR_LINE};

use crate::git::diff::LineKind;
use crate::render::LineInfo;
use crate::style;

use super::state::{visible_range, PagerState};
use super::types::Mode;
use super::text::clamp_cursor_to_boundary;

pub(crate) fn diff_area_width(
    cols: u16,
    tree_width: usize,
    tree_visible: bool,
    scrollbar: bool,
) -> usize {
    let w = if tree_visible {
        (cols as usize).saturating_sub(tree_width + 1)
    } else {
        cols as usize
    };
    if scrollbar { w.saturating_sub(1) } else { w }
}

pub(crate) fn render_scrollbar_cell(
    row: usize,
    content_height: usize,
    vis_start: usize,
    vis_end: usize,
    top: usize,
    line_map: &[LineInfo],
) -> String {
    let range = vis_end - vis_start;
    if range == 0 {
        return format!("{} {}", style::BG_SCROLLBAR_TRACK, style::RESET);
    }

    let line_start = (vis_start + (row * range) / content_height).min(line_map.len());
    let line_end = (vis_start + ((row + 1) * range) / content_height).min(line_map.len());

    let mut change: Option<LineKind> = None;
    for li in &line_map[line_start..line_end] {
        if let Some(LineKind::Added | LineKind::Deleted) = li.line_kind {
            change = li.line_kind;
            break;
        }
    }

    let thumb_start = (top.saturating_sub(vis_start)) * content_height / range;
    let thumb_end = (thumb_start + content_height * content_height / range).max(thumb_start + 1);
    let in_thumb = row >= thumb_start && row < thumb_end;

    let bg = if in_thumb { style::BG_SCROLLBAR_THUMB } else { style::BG_SCROLLBAR_TRACK };

    match change {
        Some(LineKind::Added) => format!("{bg}{}\u{2590}{}", style::FG_ADDED_MARKER, style::RESET),
        Some(LineKind::Deleted) => format!("{bg}{}\u{2590}{}", style::FG_DELETED_MARKER, style::RESET),
        _ => format!("{bg} {}", style::RESET),
    }
}

const SCROLLOFF: usize = 8;

pub(crate) fn enforce_scrolloff(state: &mut PagerState, content_height: usize) {
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

pub(crate) fn viewport_bounds(
    state: &PagerState,
    content_height: usize,
) -> (usize, usize, usize, usize) {
    super::navigation::viewport_bounds(state, content_height)
}

pub(crate) const TOOLTIP_HEIGHT: usize = 2;

pub(crate) fn bar_visible(state: &PagerState) -> bool {
    matches!(state.mode, Mode::Search) || !state.status_message.is_empty() || state.tooltip_visible
}

pub(crate) fn content_height(rows: u16, state: &PagerState) -> usize {
    let mut h = rows as usize;
    if bar_visible(state) {
        h = h.saturating_sub(1);
    }
    if state.tooltip_visible {
        h = h.saturating_sub(TOOLTIP_HEIGHT);
    }
    h
}

pub(crate) fn format_tooltip_lines(cols: usize) -> Vec<String> {
    let raw = [
        "j/k scroll  d/u page  g/G top/bot  z center  ]/[ hunk  }/{ file",
        "s single  o context  l tree  / search  n/N match  m mark  y yank  e edit  q quit",
    ];
    raw.iter()
        .map(|line| {
            let vis = line.chars().count();
            let pad = cols.saturating_sub(vis + 1);
            format!(" {}{line}{}{}", style::DIM, " ".repeat(pad), style::NO_DIM)
        })
        .collect()
}

pub(crate) fn format_status_bar(state: &PagerState, content_height: usize, cols: usize) -> String {
    if state.mode == Mode::Search {
        let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
        let before = &state.search_input[..cursor];
        let after = &state.search_input[cursor..];
        let cursor_char = if cursor < state.search_input.len() {
            let c = after.chars().next().unwrap();
            let rest = &after[c.len_utf8()..];
            format!("{}{c}{}{}{rest}", style::RESET, style::STATUS_BG, style::STATUS_FG)
        } else {
            format!("{}\u{2588}{}{}", style::RESET, style::STATUS_BG, style::STATUS_FG)
        };
        let content = format!("/{before}{cursor_char}");
        let vis_len = if cursor < state.search_input.len() {
            1 + state.search_input.chars().count()
        } else {
            1 + state.search_input.chars().count() + 1
        };
        let pad = " ".repeat(cols.saturating_sub(vis_len));
        return format!("{content}{pad}");
    }

    if !state.status_message.is_empty() {
        let msg = &state.status_message;
        let pad = " ".repeat(cols.saturating_sub(msg.len()));
        return format!("{msg}{pad}");
    }

    // Position indicator
    use tui::search::max_scroll;
    let line_count = state.doc.line_count();
    let max_top = max_scroll(line_count, content_height);
    let top = state.top_line.min(max_top);
    let end = (top + content_height).min(line_count);
    let position = if top == 0 {
        "TOP".to_string()
    } else if end >= line_count {
        "END".to_string()
    } else {
        format!("{}%", (end as f64 / line_count as f64 * 100.0).round() as usize)
    };
    let right = format!("{}{}-{}/{}{} {}", style::DIM, top + 1, end, line_count, style::NO_DIM, position);
    let right_vis = format!("{}-{}/{} {}", top + 1, end, line_count, position).len();

    let left = if let Some(idx) = state.active_file() {
        let path = state.doc.line_map.get(state.cursor_line).map_or("", |li| li.path.as_str());
        format!("Single: {path} (file {}/{})", idx + 1, state.doc.file_count())
    } else if state.mark_line.is_some() {
        "Mark set".to_string()
    } else {
        String::new()
    };
    let left_vis = left.len();

    let total_vis = left_vis + right_vis;
    if total_vis >= cols {
        let pad = " ".repeat(cols.saturating_sub(right_vis));
        return format!("{pad}{right}");
    }
    let gap = cols - total_vis;
    format!("{left}{}{right}", " ".repeat(gap))
}

pub(crate) fn resolve_lineno(
    line_map: &[LineInfo],
    lo: usize,
    hi: usize,
) -> (Option<u32>, Option<u32>) {
    let new_start = (lo..=hi).find_map(|i| line_map.get(i).and_then(|li| li.new_lineno));
    let new_end = (lo..=hi).rev().find_map(|i| line_map.get(i).and_then(|li| li.new_lineno));
    if new_start.is_some() && new_end.is_some() {
        return (new_start, new_end);
    }
    let old_start = (lo..=hi).find_map(|i| line_map.get(i).and_then(|li| li.old_lineno));
    let old_end = (lo..=hi).rev().find_map(|i| line_map.get(i).and_then(|li| li.old_lineno));
    (old_start, old_end)
}

pub(crate) fn format_copy_ref(path: &str, start: Option<u32>, end: Option<u32>) -> String {
    match (start, end) {
        (Some(s), Some(e)) if s == e => format!("{path}:{s}"),
        (Some(s), Some(e)) => format!("{path}:{s}-{e}"),
        (Some(s), None) => format!("{path}:{s}"),
        _ => path.to_string(),
    }
}

pub(crate) fn render_content_area(
    out: &mut impl Write,
    state: &PagerState,
    cols: u16,
    content_rows: u16,
) {
    use tui::search::highlight_search;

    let content_height = content_rows as usize;
    let (vis_start, vis_end, _, max_top) = viewport_bounds(state, content_height);
    let top = state.top_line.clamp(vis_start, max_top);

    let diff_w = diff_area_width(cols, state.tree_width, state.tree_visible, state.full_context);
    let show_scrollbar = state.full_context;

    for row in 0..content_height {
        move_to(out, row as u16, 0);
        let idx = top + row;
        if idx >= vis_start && idx < vis_end {
            let mut line = state.doc.lines[idx].clone();
            if !state.search_query.is_empty() {
                line = highlight_search(&line, &state.search_query);
            }
            // Mark highlight
            if let Some(mark) = state.mark_line {
                let lo = mark.min(state.cursor_line);
                let hi = mark.max(state.cursor_line);
                if idx >= lo && idx <= hi && idx != state.cursor_line {
                    let vis_w = crate::ansi::visible_width(&line);
                    let pad = diff_w.saturating_sub(vis_w);
                    line = format!("{}{line}{}{}", style::BG_VISUAL, " ".repeat(pad), style::RESET);
                }
            }
            if idx == state.cursor_line {
                let vis_w = crate::ansi::visible_width(&line);
                let pad = diff_w.saturating_sub(vis_w);
                line = format!("{}{line}{}{}", style::BG_CURSOR, " ".repeat(pad), style::RESET);
            }
            let _ = write!(out, "{CLEAR_LINE}{line}");
        } else {
            let _ = write!(out, "{CLEAR_LINE}");
        }

        if show_scrollbar {
            let cell = render_scrollbar_cell(row, content_height, vis_start, vis_end, top, &state.doc.line_map);
            let _ = write!(out, "{}\x1b[{}G{cell}", style::RESET, diff_w + 1);
        }

        if state.tree_visible {
            let tree_col = diff_w + 1 + usize::from(show_scrollbar);
            let _ = write!(
                out,
                "{}\x1b[{}G\x1b[K{}│{}",
                style::RESET, tree_col, style::FG_SEP, style::RESET,
            );
            if let Some(tree_line) = state.tree_lines.get(state.tree_scroll + row) {
                let _ = write!(out, "{tree_line}");
            }
            let _ = write!(out, "{}", style::RESET);
        }
    }
}

fn render_tooltip_bar(out: &mut impl Write, state: &PagerState, cols: u16, start_row: u16) {
    if !state.tooltip_visible {
        return;
    }
    let lines = format_tooltip_lines(cols as usize);
    for (i, line) in lines.iter().enumerate() {
        move_to(out, start_row + i as u16, 0);
        let _ = write!(out, "{CLEAR_LINE}{line}");
    }
}

fn render_status_bar(out: &mut impl Write, state: &PagerState, cols: u16, row: u16) {
    let content_height = row as usize;
    move_to(out, row, 0);
    let _ = write!(out, "{CLEAR_LINE}");
    let status = format_status_bar(state, content_height, cols as usize);
    let _ = write!(
        out,
        "{}{}{}{}{}",
        style::RESET, style::STATUS_BG, style::STATUS_FG, status, style::RESET
    );
}

pub(crate) fn render_screen(out: &mut impl Write, state: &PagerState, cols: u16, rows: u16) {
    let ch = content_height(rows, state);
    let ch16 = u16::try_from(ch).unwrap_or(u16::MAX);
    render_content_area(out, state, cols, ch16);

    let mut next_row = ch16;

    if state.tooltip_visible {
        render_tooltip_bar(out, state, cols, next_row);
        next_row += TOOLTIP_HEIGHT as u16;
    }

    if bar_visible(state) {
        render_status_bar(out, state, cols, next_row);
    } else {
        move_to(out, rows - 1, 0);
        let _ = write!(out, "{CLEAR_LINE}");
    }
    let _ = out.flush();
}
