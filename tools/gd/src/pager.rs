use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use crate::ansi::{split_ansi, strip_ansi, wrap_line_for_display};
use crate::render::{LineInfo, RenderOutput};
use crate::style;

const ALT_SCREEN_ON: &str = "\x1b[?1049h";
const ALT_SCREEN_OFF: &str = "\x1b[?1049l";
const CURSOR_HIDE: &str = "\x1b[?25l";
const CURSOR_SHOW: &str = "\x1b[?25h";
const CLEAR_LINE: &str = "\x1b[2K";

#[derive(Debug, Clone, PartialEq)]
enum Key {
    Char(char),
    Enter,
    Escape,
    Backspace,
    CtrlC,
    CtrlD,
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
}

struct PagerState {
    lines: Vec<String>,
    line_map: Vec<LineInfo>,
    file_starts: Vec<usize>,
    hunk_starts: Vec<usize>,
    top_line: usize,
    search_query: String,
    search_matches: Vec<usize>,
    current_match: isize,
    mode: Mode,
    search_input: String,
    search_cursor: usize,
    search_message: String,
    /// Pending bracket key for two-key sequences like ]c, [c, ]f, [f
    pending_bracket: Option<char>,
}

fn crossterm_to_key(key_event: crossterm::event::KeyEvent) -> Key {
    use crossterm::event::{KeyCode, KeyModifiers};

    let mods = key_event.modifiers;
    match key_event.code {
        KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlC,
        KeyCode::Char('d') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlD,
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
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Escape,
        KeyCode::Backspace => Key::Backspace,
        _ => Key::Unknown,
    }
}

fn max_scroll(line_count: usize, content_height: usize) -> usize {
    if line_count > content_height {
        line_count - content_height + content_height / 2
    } else {
        0
    }
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
                state.search_message = format!("Pattern not found: {query}");
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
        "d/Space    Half page down",
        "u          Half page up",
        "g/Home     Top",
        "G/End      Bottom",
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
    let top = state.top_line.min(state.line_map.len() - 1);
    let info = &state.line_map[top];
    let file_idx = info.file_idx + 1;
    let total = state.file_starts.len();
    let name = info.path.rsplit('/').next().unwrap_or(&info.path);
    format!("{name} ({file_idx}/{total})")
}

fn format_status_bar(state: &PagerState, content_height: usize, cols: usize) -> String {
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

    if !state.search_message.is_empty() {
        let msg = &state.search_message;
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

    let left = if !state.search_query.is_empty() {
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

fn render_screen(out: &mut impl Write, state: &PagerState, cols: u16, rows: u16) {
    let content_height = rows.saturating_sub(1) as usize;
    let max_top = max_scroll(state.lines.len(), content_height);
    let top = state.top_line.min(max_top);

    if state.mode == Mode::Help {
        let help_lines = format_help_lines(cols as usize, rows as usize);
        for (i, line) in help_lines.iter().enumerate() {
            move_to(out, i as u16, 0);
            let _ = write!(out, "{CLEAR_LINE}{}{line}{}", style::DIM, style::NO_DIM);
        }
    } else {
        let mut visual_row: usize = 0;
        let mut logical_idx = top;

        while visual_row < content_height && logical_idx < state.lines.len() {
            let mut line = state.lines[logical_idx].clone();
            if !state.search_query.is_empty() {
                line = highlight_search(&line, &state.search_query);
            }
            let wrapped = wrap_line_for_display(&line, cols as usize);
            for vline in wrapped {
                if visual_row >= content_height {
                    break;
                }
                move_to(out, visual_row as u16, 0);
                let _ = write!(out, "{CLEAR_LINE}{vline}");
                visual_row += 1;
            }
            logical_idx += 1;
        }

        while visual_row < content_height {
            move_to(out, visual_row as u16, 0);
            let _ = write!(out, "{CLEAR_LINE}");
            visual_row += 1;
        }
    }

    move_to(out, content_height as u16, 0);
    let _ = write!(out, "{CLEAR_LINE}");
    let status = format_status_bar(state, content_height, cols as usize);
    let _ = write!(out, "{}{}{}{}", style::RESET, style::REVERSE, status, style::RESET);
    let _ = out.flush();
}

fn scroll_to_match(state: &mut PagerState, rows: u16) {
    if state.current_match < 0 || state.current_match as usize >= state.search_matches.len() {
        return;
    }
    let match_line = state.search_matches[state.current_match as usize];
    let content_height = rows.saturating_sub(1) as usize;
    let target = match_line.saturating_sub(content_height / 3);
    let max_top = max_scroll(state.lines.len(), content_height);
    state.top_line = target.min(max_top);
}

/// Jump to next entry in `targets` after current top_line.
fn jump_next(targets: &[usize], top_line: usize) -> Option<usize> {
    targets.iter().find(|&&t| t > top_line).copied()
}

/// Jump to previous entry in `targets` before current top_line.
fn jump_prev(targets: &[usize], top_line: usize) -> Option<usize> {
    targets.iter().rev().find(|&&t| t < top_line).copied()
}

pub fn run_pager(output: RenderOutput) {
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
        search_query: String::new(),
        search_matches: Vec::new(),
        current_match: -1,
        mode: Mode::Normal,
        search_input: String::new(),
        search_cursor: 0,
        search_message: String::new(),
        pending_bracket: None,
    };

    let mut last_size = get_term_size();
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
                    render_screen(&mut stdout, &state, last_size.0, last_size.1);
                }
                continue;
            }
            Err(_) => break,
        };

        let key = match ev {
            Event::Resize(_, _) => {
                last_size = get_term_size();
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
            }
            render_screen(&mut stdout, &state, last_size.0, last_size.1);
            continue;
        }

        if state.mode == Mode::Help {
            state.mode = Mode::Normal;
            render_screen(&mut stdout, &state, last_size.0, last_size.1);
            continue;
        }

        // Handle pending bracket sequences
        if let Some(bracket) = state.pending_bracket.take() {
            let rows = last_size.1;
            let content_height = rows.saturating_sub(1) as usize;
            let max_top = max_scroll(state.lines.len(), content_height);
            match (bracket, &key) {
                (']', Key::Char('c')) => {
                    if let Some(target) = jump_next(&state.hunk_starts, state.top_line) {
                        state.top_line = target.min(max_top);
                    }
                }
                ('[', Key::Char('c')) => {
                    if let Some(target) = jump_prev(&state.hunk_starts, state.top_line) {
                        state.top_line = target.min(max_top);
                    }
                }
                (']', Key::Char('f')) => {
                    if let Some(target) = jump_next(&state.file_starts, state.top_line) {
                        state.top_line = target.min(max_top);
                    }
                }
                ('[', Key::Char('f')) => {
                    if let Some(target) = jump_prev(&state.file_starts, state.top_line) {
                        state.top_line = target.min(max_top);
                    }
                }
                _ => {} // Unknown sequence — ignore
            }
            render_screen(&mut stdout, &state, last_size.0, last_size.1);
            continue;
        }

        // Normal mode
        let rows = last_size.1;
        let content_height = rows.saturating_sub(1) as usize;
        let half_page = content_height / 2;
        let max_top = max_scroll(state.lines.len(), content_height);

        state.search_message.clear();

        match key {
            Key::Char('q') | Key::CtrlC => break,
            Key::Char('j') | Key::Down | Key::Enter => {
                state.top_line = (state.top_line + 1).min(max_top);
            }
            Key::Char('k') | Key::Up => {
                state.top_line = state.top_line.saturating_sub(1);
            }
            Key::Char('d' | ' ') | Key::CtrlD | Key::PageDown => {
                state.top_line = (state.top_line + half_page).min(max_top);
            }
            Key::Char('u') | Key::CtrlU | Key::PageUp => {
                state.top_line = state.top_line.saturating_sub(half_page);
            }
            Key::Char('g') | Key::Home => {
                state.top_line = 0;
            }
            Key::Char('G') | Key::End => {
                state.top_line = max_top;
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
                let top = state.top_line.min(state.line_map.len().saturating_sub(1));
                if !state.line_map.is_empty() {
                    let info = &state.line_map[top];
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
            Key::Char('?') => {
                state.mode = Mode::Help;
            }
            _ => {}
        }

        render_screen(&mut stdout, &state, last_size.0, last_size.1);
    }

    let _ = crossterm::terminal::disable_raw_mode();
    let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
    let _ = stdout.flush();
}
