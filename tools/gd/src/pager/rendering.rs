use std::io::Write;

use tui::pager::{move_to, CLEAR_LINE};

use crate::git::diff::LineKind;
use crate::render::LineInfo;
use crate::style;

use super::keymap::keymap_help_lines;
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
    if scrollbar {
        w.saturating_sub(1)
    } else {
        w
    }
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

    let bg = if in_thumb {
        style::BG_SCROLLBAR_THUMB
    } else {
        style::BG_SCROLLBAR_TRACK
    };

    match change {
        Some(LineKind::Added) => {
            format!("{bg}{}\u{2590}{}", style::FG_ADDED_MARKER, style::RESET)
        }
        Some(LineKind::Deleted) => {
            format!("{bg}{}\u{2590}{}", style::FG_DELETED_MARKER, style::RESET)
        }
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

pub(crate) fn bar_visible(state: &PagerState) -> bool {
    matches!(state.mode, Mode::Search | Mode::Help | Mode::Visual)
        || !state.status_message.is_empty()
}

pub(crate) fn content_height(rows: u16, state: &PagerState) -> usize {
    if bar_visible(state) {
        rows.saturating_sub(1) as usize
    } else {
        rows as usize
    }
}

fn format_help_lines(cols: usize, content_height: usize) -> Vec<String> {
    let help = keymap_help_lines();

    let mut lines = Vec::with_capacity(content_height);
    let top_pad = content_height.saturating_sub(help.len()) / 2;
    for _ in 0..top_pad {
        lines.push(" ".repeat(cols));
    }

    let max_w = help.iter().map(|h| h.chars().count()).max().unwrap_or(0);
    let left_pad = cols.saturating_sub(max_w) / 2;

    for h in &help {
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

pub(crate) fn format_status_bar(state: &PagerState, content_height: usize, cols: usize) -> String {
    use tui::search::max_scroll;

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
        let line_count = state.doc.line_count();
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
        let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
        let before = &state.search_input[..cursor];
        let after = &state.search_input[cursor..];
        let cursor_char = if cursor < state.search_input.len() {
            let c = after.chars().next().unwrap();
            let rest = &after[c.len_utf8()..];
            format!("{}{c}{}{}{rest}", style::RESET, style::STATUS_BG, style::STATUS_FG)
        } else {
            format!(
                "{}\u{2588}{}{}",
                style::RESET,
                style::STATUS_BG,
                style::STATUS_FG
            )
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

    " ".repeat(cols)
}

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

    if state.mode == Mode::Help {
        let help_lines = format_help_lines(cols as usize, content_height);
        for (i, line) in help_lines.iter().enumerate() {
            move_to(out, i as u16, 0);
            let _ = write!(out, "{CLEAR_LINE}{}{line}{}", style::DIM, style::NO_DIM);
        }
        return;
    }

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
            if state.mode == Mode::Visual {
                let lo = state.visual_anchor.min(state.cursor_line);
                let hi = state.visual_anchor.max(state.cursor_line);
                if idx >= lo && idx <= hi {
                    line = highlight_visual_line(&line, diff_w);
                }
            }
            if idx == state.cursor_line && state.mode != Mode::Visual {
                let vis_w = crate::ansi::visible_width(&line);
                let pad = diff_w.saturating_sub(vis_w);
                line = format!(
                    "{}{line}{}{}",
                    style::BG_CURSOR,
                    " ".repeat(pad),
                    style::RESET
                );
            }
            let _ = write!(out, "{CLEAR_LINE}{line}");
        } else {
            let _ = write!(out, "{CLEAR_LINE}");
        }

        if show_scrollbar {
            let cell = render_scrollbar_cell(
                row,
                content_height,
                vis_start,
                vis_end,
                top,
                &state.doc.line_map,
            );
            let _ = write!(out, "{}\x1b[{}G{cell}", style::RESET, diff_w + 1);
        }

        if state.tree_visible {
            let tree_col = diff_w + 1 + usize::from(show_scrollbar);
            let sep_color = if state.tree_focused() {
                style::FG_SEP_ACTIVE
            } else {
                style::FG_SEP
            };
            let _ = write!(
                out,
                "{}\x1b[{}G\x1b[K{sep_color}│{}",
                style::RESET,
                tree_col,
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
    let _ = write!(
        out,
        "{}{}{}{}{}",
        style::RESET,
        style::STATUS_BG,
        style::STATUS_FG,
        status,
        style::RESET
    );
}

pub(crate) fn render_screen(out: &mut impl Write, state: &PagerState, cols: u16, rows: u16) {
    let ch = content_height(rows, state);
    render_content_area(out, state, cols, u16::try_from(ch).unwrap_or(u16::MAX));
    if bar_visible(state) {
        render_status_bar(out, state, cols, u16::try_from(ch).unwrap_or(u16::MAX));
    } else {
        move_to(out, rows - 1, 0);
        let _ = write!(out, "{CLEAR_LINE}");
    }
    let _ = out.flush();
}
