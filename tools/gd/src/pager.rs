use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use crate::ansi::{split_ansi, strip_ansi};
use crate::git::diff::{DiffFile, FileStatus};
use crate::render::{self, LineInfo, RenderOutput};
use crate::style;

const ALT_SCREEN_ON: &str = "\x1b[?1049h";
const ALT_SCREEN_OFF: &str = "\x1b[?1049l";
const CURSOR_HIDE: &str = "\x1b[?25l";
const CURSOR_SHOW: &str = "\x1b[?25h";
const CLEAR_LINE: &str = "\x1b[2K";

#[derive(Debug, Clone, PartialEq)]
enum Key {
    Char(char),
    Tab,
    Enter,
    Escape,
    Backspace,
    CtrlC,
    CtrlD,
    CtrlH,
    CtrlL,
    CtrlU,
    Up,
    Down,
    Left,
    Right,
    AltLeft,
    AltRight,
    AltBackspace,
    PageUp,
    PageDown,
    Home,
    End,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Normal,
    Search,
    Help,
    Visual,
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
}

fn crossterm_to_key(key_event: crossterm::event::KeyEvent) -> Key {
    use crossterm::event::{KeyCode, KeyModifiers};

    let mods = key_event.modifiers;
    match key_event.code {
        KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlC,
        KeyCode::Char('d') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlD,
        KeyCode::Char('h') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlH,
        KeyCode::Char('l') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlL,
        KeyCode::Char('u') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlU,
        KeyCode::Char('b') | KeyCode::Left if mods.contains(KeyModifiers::ALT) => Key::AltLeft,
        KeyCode::Char('f') | KeyCode::Right if mods.contains(KeyModifiers::ALT) => Key::AltRight,
        KeyCode::Backspace if mods.contains(KeyModifiers::ALT) => Key::AltBackspace,
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::Tab => Key::Tab,
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Escape,
        KeyCode::Backspace => Key::Backspace,
        _ => Key::Unknown,
    }
}

fn diff_area_width(cols: u16, tree_width: usize, tree_visible: bool) -> usize {
    if tree_visible {
        (cols as usize).saturating_sub(tree_width + 1)
    } else {
        cols as usize
    }
}

fn max_scroll(line_count: usize, content_height: usize) -> usize {
    if line_count > content_height {
        line_count - content_height + content_height / 2
    } else {
        0
    }
}

const SCROLLOFF: usize = 8;

fn enforce_scrolloff(state: &mut PagerState, content_height: usize) {
    let max_top = max_scroll(state.lines.len(), content_height);
    let max_cursor = state.lines.len().saturating_sub(1);
    state.cursor_line = state.cursor_line.min(max_cursor);
    if state.cursor_line < state.top_line + SCROLLOFF {
        state.top_line = state.cursor_line.saturating_sub(SCROLLOFF);
    }
    if state.cursor_line + SCROLLOFF >= state.top_line + content_height {
        state.top_line = (state.cursor_line + SCROLLOFF + 1).saturating_sub(content_height);
    }
    state.top_line = state.top_line.min(max_top);
}

fn find_matches(lines: &[String], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return Vec::new();
    }
    let lower = query.to_lowercase();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| strip_ansi(line).to_lowercase().contains(&lower))
        .map(|(i, _)| i)
        .collect()
}

fn find_nearest_match(matches: &[usize], top_line: usize) -> isize {
    if matches.is_empty() {
        return -1;
    }
    for (i, &m) in matches.iter().enumerate() {
        if m >= top_line {
            return i as isize;
        }
    }
    matches.len() as isize - 1
}

fn highlight_search(line: &str, query: &str) -> String {
    if query.is_empty() {
        return line.to_string();
    }

    let stripped = strip_ansi(line);
    let lower_stripped = stripped.to_lowercase();
    let lower_query = query.to_lowercase();

    let mut match_ranges: Vec<(usize, usize)> = Vec::new();
    let mut start = 0;
    while let Some(pos) = lower_stripped[start..].find(&lower_query) {
        let abs = start + pos;
        match_ranges.push((abs, abs + query.len()));
        start = abs + 1;
    }

    if match_ranges.is_empty() {
        return line.to_string();
    }

    let mut vis_to_orig: Vec<usize> = Vec::new();
    let segments = split_ansi(line);
    let mut orig_pos = 0;
    for seg in &segments {
        if seg.starts_with('\x1b') {
            orig_pos += seg.len();
        } else {
            for (i, _) in seg.char_indices() {
                vis_to_orig.push(orig_pos + i);
            }
            orig_pos += seg.len();
        }
    }
    vis_to_orig.push(orig_pos);

    let mut insertions: Vec<(usize, &str)> = Vec::new();
    for (mstart, mend) in &match_ranges {
        if *mend <= vis_to_orig.len() && *mstart < vis_to_orig.len() {
            let orig_start = vis_to_orig[*mstart];
            let orig_end = vis_to_orig[*mend];
            insertions.push((orig_end, style::NO_REVERSE));
            insertions.push((orig_start, style::REVERSE));
        }
    }
    insertions.sort_by(|a, b| b.0.cmp(&a.0));

    let mut result = line.to_string();
    for (pos, code) in insertions {
        if pos <= result.len() {
            result.insert_str(pos, code);
        }
    }

    result
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

fn copy_to_clipboard(text: &str) -> bool {
    use std::io::Write as _;
    let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() else {
        return false;
    };
    let Some(mut stdin) = child.stdin.take() else {
        return false;
    };
    if stdin.write_all(text.as_bytes()).is_err() {
        return false;
    }
    drop(stdin);
    child.wait().is_ok()
}

fn word_boundary_left(text: &str, cursor: usize) -> usize {
    let bytes = text.as_bytes();
    let mut pos = cursor;
    while pos > 0 && bytes[pos - 1] == b' ' {
        pos -= 1;
    }
    while pos > 0 && bytes[pos - 1] != b' ' {
        pos -= 1;
    }
    pos
}

fn word_boundary_right(text: &str, cursor: usize) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut pos = cursor;
    while pos < len && bytes[pos] != b' ' {
        pos += 1;
    }
    while pos < len && bytes[pos] == b' ' {
        pos += 1;
    }
    pos
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

fn get_term_size() -> (u16, u16) {
    crossterm::terminal::size().unwrap_or((80, 24))
}

fn move_to(out: &mut impl Write, row: u16, col: u16) {
    let _ = write!(out, "\x1b[{};{}H", row + 1, col + 1);
}

fn open_in_editor(path: &str, line: Option<u32>) {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
    let basename = editor.rsplit('/').next().unwrap_or(&editor);
    let is_vim = basename == "vim" || basename == "nvim";

    let mut args: Vec<String> = Vec::new();
    if is_vim && let Some(l) = line {
        args.push(format!("+{l}"));
    }
    args.push(path.to_string());

    let _ = Command::new(&editor)
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
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
        "",
        "Search",
        "/          Search",
        "n          Next match",
        "N          Previous match",
        "",
        "File Tree",
        "l          Toggle panel",
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
        "e          Open in editor",
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
fn re_render(state: &mut PagerState, files: &[DiffFile], color: bool, cols: u16) {
    // Capture anchor from current top line
    let anchor = if !state.line_map.is_empty() {
        let top = state.top_line.min(state.line_map.len() - 1);
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

    // Rebuild tree lines at current cursor
    if state.tree_visible {
        state.tree_lines = build_tree_lines(files, state.tree_cursor, state.tree_width);
    }
}

fn file_status_icon(status: &FileStatus) -> &'static str {
    match status {
        FileStatus::Added => "+",
        FileStatus::Deleted => "-",
        FileStatus::Modified => "~",
        FileStatus::Renamed => "→",
        FileStatus::Untracked => "?",
    }
}

fn compute_tree_width(files: &[DiffFile]) -> usize {
    let max_len = files.iter().map(|f| f.path().len()).max().unwrap_or(0);
    (max_len + 4).min(30)
}

fn build_tree_lines(files: &[DiffFile], cursor: usize, width: usize) -> Vec<String> {
    let mut lines = Vec::new();

    // Header
    let header = "CHANGED FILES";
    let hdr_len = header.len().min(width);
    let pad = width.saturating_sub(hdr_len);
    lines.push(format!("\x1b[1m{}{}\x1b[22m", &header[..hdr_len], " ".repeat(pad)));

    // Spacer
    lines.push(" ".repeat(width));

    // File entries
    for (i, file) in files.iter().enumerate() {
        let icon = file_status_icon(&file.status);
        let path = file.path();
        let label = format!("{icon} {path}");
        let max_label = width.saturating_sub(2);
        let truncated: String = label.chars().take(max_label).collect();
        let vis = truncated.chars().count();
        let content_len = 1 + vis; // leading space
        let right_pad = width.saturating_sub(content_len);
        let padded = format!(" {truncated}{}", " ".repeat(right_pad));

        if i == cursor {
            lines.push(format!(
                "{}\x1b[7m{padded}\x1b[27m{}",
                style::FG_FILE_HEADER,
                style::RESET,
            ));
        } else {
            lines.push(format!(
                "{}{padded}{}",
                style::FG_TREE,
                style::RESET,
            ));
        }
    }

    lines
}

fn sync_tree_cursor(state: &mut PagerState, files: &[DiffFile], content_height: usize) {
    if !state.tree_visible || state.tree_focused {
        return;
    }
    let anchor = if state.cursor_visible {
        state.cursor_line
    } else {
        state.top_line
    };
    let new_cursor = state
        .line_map
        .get(anchor)
        .map(|li| li.file_idx)
        .unwrap_or(0);
    if new_cursor != state.tree_cursor {
        state.tree_cursor = new_cursor;
        state.tree_lines = build_tree_lines(files, state.tree_cursor, state.tree_width);
        ensure_tree_cursor_visible(state, content_height);
    }
}

fn ensure_tree_cursor_visible(state: &mut PagerState, content_height: usize) {
    let offset = state.tree_cursor + 2; // +2 for header + spacer in tree_lines
    if offset < state.tree_scroll + 2 {
        state.tree_scroll = offset.saturating_sub(2);
    }
    if offset >= state.tree_scroll + content_height {
        state.tree_scroll = offset + 1 - content_height;
    }
}

pub fn run_pager(output: RenderOutput, files: &[DiffFile], color: bool) {
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
    };

    // Initialize file tree panel
    state.tree_width = compute_tree_width(files);
    state.tree_lines = build_tree_lines(files, 0, state.tree_width);
    state.tree_visible = false;

    let mut last_size = get_term_size();
    re_render(&mut state, files, color, last_size.0);
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
                    re_render(&mut state, files, color, last_size.0);
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
                re_render(&mut state, files, color, last_size.0);
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
                sync_tree_cursor(&mut state, files, ch);
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
                    if state.tree_cursor + 1 < files.len() {
                        state.tree_cursor += 1;
                        state.tree_lines =
                            build_tree_lines(files, state.tree_cursor, state.tree_width);
                        let ch = last_size.1.saturating_sub(1) as usize;
                        ensure_tree_cursor_visible(&mut state, ch);
                    }
                }
                Key::Char('k') | Key::Up => {
                    if state.tree_cursor > 0 {
                        state.tree_cursor -= 1;
                        state.tree_lines =
                            build_tree_lines(files, state.tree_cursor, state.tree_width);
                        let ch = last_size.1.saturating_sub(1) as usize;
                        ensure_tree_cursor_visible(&mut state, ch);
                    }
                }
                Key::Enter => {
                    if let Some(&target) = state.file_starts.get(state.tree_cursor) {
                        let content_height = last_size.1.saturating_sub(1) as usize;
                        let max_top = max_scroll(state.lines.len(), content_height);
                        state.top_line = target.min(max_top);
                        state.cursor_line = state.top_line;
                    }
                }
                Key::CtrlH | Key::Escape | Key::Tab => {
                    state.tree_focused = false;
                }
                Key::Char('l') => {
                    state.tree_visible = false;
                    state.tree_focused = false;
                    re_render(&mut state, files, color, last_size.0);
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
            match (bracket, &key) {
                (']', Key::Char('c')) => {
                    if let Some(target) = jump_next(&state.hunk_starts, anchor) {
                        if state.cursor_visible {
                            state.cursor_line = target;
                        } else {
                            state.top_line = target.min(max_top);
                        }
                    }
                }
                ('[', Key::Char('c')) => {
                    if let Some(target) = jump_prev(&state.hunk_starts, anchor) {
                        if state.cursor_visible {
                            state.cursor_line = target;
                        } else {
                            state.top_line = target.min(max_top);
                        }
                    }
                }
                (']', Key::Char('f')) => {
                    if let Some(target) = jump_next(&state.file_starts, anchor) {
                        if state.cursor_visible {
                            state.cursor_line = target;
                        } else {
                            state.top_line = target.min(max_top);
                        }
                    }
                }
                ('[', Key::Char('f')) => {
                    if let Some(target) = jump_prev(&state.file_starts, anchor) {
                        if state.cursor_visible {
                            state.cursor_line = target;
                        } else {
                            state.top_line = target.min(max_top);
                        }
                    }
                }
                _ => {} // Unknown sequence — ignore
            }
            if state.cursor_visible {
                enforce_scrolloff(&mut state, content_height);
            } else {
                state.cursor_line = state.top_line;
            }
            let ch = last_size.1.saturating_sub(1) as usize;
            sync_tree_cursor(&mut state, files, ch);
            render_screen(&mut stdout, &state, last_size.0, last_size.1);
            continue;
        }

        // Normal mode
        let rows = last_size.1;
        let content_height = rows.saturating_sub(1) as usize;
        let half_page = content_height / 2;
        let max_top = max_scroll(state.lines.len(), content_height);

        state.status_message.clear();

        match key {
            Key::Char('q') | Key::CtrlC => break,
            Key::Char('j') | Key::Down | Key::Enter => {
                if state.cursor_visible {
                    state.cursor_line = (state.cursor_line + 1)
                        .min(state.lines.len().saturating_sub(1));
                } else {
                    state.top_line = (state.top_line + 1).min(max_top);
                }
            }
            Key::Char('k') | Key::Up => {
                if state.cursor_visible {
                    state.cursor_line = state.cursor_line.saturating_sub(1);
                } else {
                    state.top_line = state.top_line.saturating_sub(1);
                }
            }
            Key::Char('d') | Key::CtrlD | Key::PageDown => {
                if state.cursor_visible {
                    state.cursor_line = (state.cursor_line + half_page)
                        .min(state.lines.len().saturating_sub(1));
                } else {
                    state.top_line = (state.top_line + half_page).min(max_top);
                }
            }
            Key::Char('u') | Key::CtrlU | Key::PageUp => {
                if state.cursor_visible {
                    state.cursor_line = state.cursor_line.saturating_sub(half_page);
                } else {
                    state.top_line = state.top_line.saturating_sub(half_page);
                }
            }
            Key::Char('g') | Key::Home => {
                if state.cursor_visible {
                    state.cursor_line = 0;
                } else {
                    state.top_line = 0;
                }
            }
            Key::Char('G') | Key::End => {
                if state.cursor_visible {
                    state.cursor_line = state.lines.len().saturating_sub(1);
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
                    state.current_match =
                        (state.current_match + 1) % state.search_matches.len() as isize;
                    scroll_to_match(&mut state, rows);
                }
            }
            Key::Char('N') => {
                if !state.search_matches.is_empty() {
                    state.current_match = (state.current_match - 1
                        + state.search_matches.len() as isize)
                        % state.search_matches.len() as isize;
                    scroll_to_match(&mut state, rows);
                }
            }
            Key::Char('e') => {
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
            Key::Char('l') => {
                state.tree_visible = !state.tree_visible;
                if state.tree_visible {
                    let anchor = if state.cursor_visible {
                        state.cursor_line
                    } else {
                        state.top_line
                    };
                    state.tree_cursor = state
                        .line_map
                        .get(anchor)
                        .map(|li| li.file_idx)
                        .unwrap_or(0);
                    ensure_tree_cursor_visible(&mut state, content_height);
                }
                re_render(&mut state, files, color, last_size.0);
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
                    state.tree_cursor = state
                        .line_map
                        .get(anchor)
                        .map(|li| li.file_idx)
                        .unwrap_or(0);
                    re_render(&mut state, files, color, last_size.0);
                }
                state.tree_focused = true;
                ensure_tree_cursor_visible(&mut state, content_height);
            }
            Key::Char('v') => {
                state.mode = Mode::Visual;
                state.visual_anchor = state.cursor_line;
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
        sync_tree_cursor(&mut state, files, content_height);
        render_screen(&mut stdout, &state, last_size.0, last_size.1);
    }

    let _ = crossterm::terminal::disable_raw_mode();
    let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
    let _ = stdout.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_status_icon_untracked() {
        assert_eq!(file_status_icon(&FileStatus::Untracked), "?");
    }
}
