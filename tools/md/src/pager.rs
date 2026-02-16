use std::io::{self, Write};
use std::process::{Command, Stdio};

use crossterm::event::{self, Event, KeyEventKind};

use crate::wrap::{split_ansi, strip_ansi, wrap_line_for_display};

// ANSI constants
const ALT_SCREEN_ON: &str = "\x1b[?1049h";
const ALT_SCREEN_OFF: &str = "\x1b[?1049l";
const CURSOR_HIDE: &str = "\x1b[?25l";
const CURSOR_SHOW: &str = "\x1b[?25h";
const CLEAR_LINE: &str = "\x1b[2K";
const REVERSE: &str = "\x1b[7m";
const NO_REVERSE: &str = "\x1b[27m";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const NO_DIM: &str = "\x1b[22m";

#[derive(Debug, Clone, PartialEq)]
pub enum Key {
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
pub enum Mode {
    Normal,
    Search,
}

pub struct PagerState {
    pub lines: Vec<String>,
    pub top_line: usize,
    pub search_query: String,
    pub search_matches: Vec<usize>,
    pub current_match: isize,
    pub mode: Mode,
    pub search_input: String,
    pub search_cursor: usize,
    pub search_message: String,
    pub file_path: Option<String>,
    pub raw_content: Option<String>,
}

pub struct StatusBarInput {
    pub mode: Mode,
    pub search_input: String,
    pub search_cursor: usize,
    pub search_message: String,
    pub search_query: String,
    pub search_matches: Vec<usize>,
    pub current_match: isize,
    pub top_line: usize,
    pub line_count: usize,
    pub content_height: usize,
    pub file_path: Option<String>,
}

pub fn crossterm_key_to_key(key_event: crossterm::event::KeyEvent) -> Key {
    use crossterm::event::{KeyCode, KeyModifiers};

    let mods = key_event.modifiers;
    match key_event.code {
        // Ctrl combos
        KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlC,
        KeyCode::Char('d') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlD,
        KeyCode::Char('u') if mods.contains(KeyModifiers::CONTROL) => Key::CtrlU,
        // Alt combos
        KeyCode::Char('b') | KeyCode::Left if mods.contains(KeyModifiers::ALT) => Key::AltLeft,
        KeyCode::Char('f') | KeyCode::Right if mods.contains(KeyModifiers::ALT) => Key::AltRight,
        KeyCode::Backspace if mods.contains(KeyModifiers::ALT) => Key::AltBackspace,
        // Plain chars
        KeyCode::Char(c) => Key::Char(c),
        // Nav keys
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        // Special keys
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Escape,
        KeyCode::Backspace => Key::Backspace,
        _ => Key::Unknown,
    }
}

pub fn parse_key(buf: &[u8]) -> Key {
    if buf.is_empty() {
        return Key::Unknown;
    }

    match buf[0] {
        0x1b => {
            if buf.len() == 1 {
                return Key::Escape;
            }
            match buf[1] {
                0x5b => {
                    // CSI sequence
                    if buf.len() < 3 {
                        return Key::Unknown;
                    }
                    match buf[2] {
                        0x41 => Key::Up,    // A
                        0x42 => Key::Down,  // B
                        0x43 => Key::Right, // C
                        0x44 => Key::Left,  // D
                        0x48 => Key::Home,  // H
                        0x46 => Key::End,   // F
                        0x35 if buf.len() >= 4 && buf[3] == 0x7e => Key::PageUp,
                        0x36 if buf.len() >= 4 && buf[3] == 0x7e => Key::PageDown,
                        0x31 if buf.len() >= 6 && buf[3] == 0x3b && buf[4] == 0x33 => {
                            match buf[5] {
                                0x43 => Key::AltRight,
                                0x44 => Key::AltLeft,
                                _ => Key::Unknown,
                            }
                        }
                        _ => Key::Unknown,
                    }
                }
                0x62 => Key::AltLeft,      // ESC b
                0x66 => Key::AltRight,     // ESC f
                0x7f => Key::AltBackspace, // ESC DEL
                _ => Key::Unknown,
            }
        }
        0x03 => Key::CtrlC,
        0x04 => Key::CtrlD,
        0x0d => Key::Enter,
        0x15 => Key::CtrlU,
        0x7f => Key::Backspace,
        b @ 0x20..=0x7e => Key::Char(b as char),
        _ => Key::Unknown,
    }
}

pub fn highlight_search(line: &str, query: &str) -> String {
    if query.is_empty() {
        return line.to_string();
    }

    let stripped = strip_ansi(line);
    let lower_stripped = stripped.to_lowercase();
    let lower_query = query.to_lowercase();

    // Find all match positions in the stripped text
    let mut match_ranges: Vec<(usize, usize)> = Vec::new();
    let mut start = 0;
    while let Some(pos) = lower_stripped[start..].find(&lower_query) {
        let abs_pos = start + pos;
        match_ranges.push((abs_pos, abs_pos + query.len()));
        start = abs_pos + 1;
    }

    if match_ranges.is_empty() {
        return line.to_string();
    }

    // Build position map: visible char index → byte index in original string
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
    // Sentinel for end-of-string insertions
    vis_to_orig.push(orig_pos);

    // Collect insertion points (reverse order to preserve indices)
    let mut insertions: Vec<(usize, &str)> = Vec::new();
    for (mstart, mend) in &match_ranges {
        let orig_start = vis_to_orig[*mstart];
        let orig_end = vis_to_orig[*mend];
        insertions.push((orig_end, NO_REVERSE));
        insertions.push((orig_start, REVERSE));
    }
    insertions.sort_by(|a, b| b.0.cmp(&a.0));

    let mut result = line.to_string();
    for (pos, code) in insertions {
        result.insert_str(pos, code);
    }

    result
}

pub fn word_boundary_left(text: &str, cursor: usize) -> usize {
    let bytes = text.as_bytes();
    let mut pos = cursor;
    // Skip spaces left
    while pos > 0 && bytes[pos - 1] == b' ' {
        pos -= 1;
    }
    // Skip non-spaces left
    while pos > 0 && bytes[pos - 1] != b' ' {
        pos -= 1;
    }
    pos
}

pub fn word_boundary_right(text: &str, cursor: usize) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut pos = cursor;
    // Skip non-spaces right
    while pos < len && bytes[pos] != b' ' {
        pos += 1;
    }
    // Skip spaces right
    while pos < len && bytes[pos] == b' ' {
        pos += 1;
    }
    pos
}

pub fn map_scroll_position(old_top: usize, old_count: usize, new_count: usize) -> usize {
    if old_count <= 1 || new_count <= 1 {
        return 0;
    }
    let ratio = old_top as f64 / (old_count - 1) as f64;
    (ratio * (new_count - 1) as f64).round() as usize
}

pub fn find_nearest_match(matches: &[usize], top_line: usize) -> isize {
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

pub fn find_matches(lines: &[String], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return Vec::new();
    }
    let lower_query = query.to_lowercase();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| strip_ansi(line).to_lowercase().contains(&lower_query))
        .map(|(i, _)| i)
        .collect()
}

pub fn map_to_source_line(top_line: usize, rendered_line_count: usize, raw_content: &str) -> usize {
    let source_count = raw_content.lines().count();
    if rendered_line_count == 0 {
        return 1;
    }
    let ratio = top_line as f64 / rendered_line_count as f64;
    (ratio * source_count as f64).round() as usize + 1
}

pub fn format_status_bar(input: &StatusBarInput, cols: usize) -> String {
    // Search mode
    if input.mode == Mode::Search {
        let before = &input.search_input[..input.search_cursor];
        let after = &input.search_input[input.search_cursor..];

        let cursor_char = if input.search_cursor < input.search_input.len() {
            let c = after.chars().next().unwrap();
            let rest = &after[c.len_utf8()..];
            format!("{NO_REVERSE}{c}{REVERSE}{rest}")
        } else {
            format!("{NO_REVERSE}\u{2588}{REVERSE}")
        };

        let content = format!("/{before}{cursor_char}");
        // "/" + input chars + block cursor (only if cursor is at end, adding a new char)
        let visible_len = if input.search_cursor < input.search_input.len() {
            1 + input.search_input.len()
        } else {
            1 + input.search_input.len() + 1
        };
        let padding = if cols > visible_len {
            " ".repeat(cols - visible_len)
        } else {
            String::new()
        };
        return format!("{content}{padding}");
    }

    // Search message mode
    if !input.search_message.is_empty() {
        let msg = &input.search_message;
        let padding = if cols > msg.len() {
            " ".repeat(cols - msg.len())
        } else {
            String::new()
        };
        return format!("{msg}{padding}");
    }

    // Normal mode: build right side first
    let end_line = (input.top_line + input.content_height).min(input.line_count);
    let range = format!("{}-{}/{}", input.top_line + 1, end_line, input.line_count);

    let position = if input.top_line == 0 {
        "TOP".to_string()
    } else if end_line >= input.line_count {
        "END".to_string()
    } else {
        let pct = (end_line as f64 / input.line_count as f64 * 100.0).round() as usize;
        format!("{pct}%")
    };

    let right = format!("{DIM}{range}{NO_DIM} {position}");
    let right_visible_len = range.len() + 1 + position.len();

    // Build left side
    let left = if input.search_query.is_empty() {
        input
            .file_path
            .as_ref()
            .map(|p| p.rsplit('/').next().unwrap_or(p).to_string())
            .unwrap_or_default()
    } else if input.current_match >= 0 {
        format!(
            "/{} ({}/{})",
            input.search_query,
            input.current_match + 1,
            input.search_matches.len()
        )
    } else {
        format!("/{}", input.search_query)
    };

    let left_visible_len = left.len();
    let total_visible = left_visible_len + right_visible_len;

    if total_visible >= cols {
        // Right side only
        let padding = if cols > right_visible_len {
            " ".repeat(cols - right_visible_len)
        } else {
            String::new()
        };
        format!("{padding}{right}")
    } else {
        let gap = cols - total_visible;
        format!("{left}{}{right}", " ".repeat(gap))
    }
}

pub fn render_status_bar(input: &StatusBarInput, cols: usize) -> String {
    format!("{RESET}{REVERSE}{}{RESET}", format_status_bar(input, cols))
}

pub fn handle_search_key(state: &mut PagerState, key: &Key) {
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

fn move_to(out: &mut impl Write, row: u16, col: u16) {
    let _ = write!(out, "\x1b[{};{}H", row + 1, col + 1);
}

fn get_term_size() -> (u16, u16) {
    crossterm::terminal::size().unwrap_or((80, 24))
}

fn copy_to_clipboard(text: &str) -> bool {
    let child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match child {
        Ok(mut proc) => {
            if let Some(ref mut stdin) = proc.stdin {
                let _ = stdin.write_all(text.as_bytes());
            }
            drop(proc.stdin.take());
            proc.wait().map(|s| s.success()).unwrap_or(false)
        }
        Err(_) => false,
    }
}

fn open_in_editor(file_path: &str, line: Option<usize>) {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
    let basename = editor.rsplit('/').next().unwrap_or(&editor);
    let is_vim = basename == "vim" || basename == "nvim";

    let mut args: Vec<String> = Vec::new();
    if is_vim {
        args.push("-R".to_string());
    }
    if is_vim && let Some(l) = line {
        args.push(format!("+{l}"));
    }
    args.push(file_path.to_string());

    let _ = Command::new(&editor)
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
}

pub fn run_pager(
    content: &str,
    file_path: Option<&str>,
    raw_content: Option<&str>,
    mut on_resize: Option<&mut dyn FnMut() -> String>,
) {
    let mut stdout = io::stdout();

    // Enter alternate screen, hide cursor
    let _ = write!(stdout, "{ALT_SCREEN_ON}{CURSOR_HIDE}");
    let _ = stdout.flush();

    // Enable raw mode
    let _ = crossterm::terminal::enable_raw_mode();

    let mut state = PagerState {
        lines: content.lines().map(String::from).collect(),
        top_line: 0,
        search_query: String::new(),
        search_matches: Vec::new(),
        current_match: -1,
        mode: Mode::Normal,
        search_input: String::new(),
        search_cursor: 0,
        search_message: String::new(),
        file_path: file_path.map(String::from),
        raw_content: raw_content.map(String::from),
    };

    render_screen(&mut stdout, &state);

    while let Ok(event) = event::read() {
        let key = match event {
            Event::Resize(_, _) => {
                if let Some(ref mut resize_fn) = on_resize {
                    let new_content = resize_fn();
                    let old_count = state.lines.len();
                    state.lines = new_content.lines().map(String::from).collect();
                    state.top_line =
                        map_scroll_position(state.top_line, old_count, state.lines.len());
                    if !state.search_query.is_empty() {
                        state.search_matches = find_matches(&state.lines, &state.search_query);
                    }
                }
                render_screen(&mut stdout, &state);
                continue;
            }
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                crossterm_key_to_key(key_event)
            }
            _ => continue,
        };

        if state.mode == Mode::Search {
            handle_search_key(&mut state, &key);
            if state.mode == Mode::Normal && state.current_match >= 0 {
                scroll_to_match(&mut state);
            }
            render_screen(&mut stdout, &state);
            continue;
        }

        // Normal mode
        let (_cols, rows) = get_term_size();
        let content_height = rows.saturating_sub(1) as usize;
        let half_page = content_height / 2;
        let max_top = state.lines.len().saturating_sub(content_height);

        let mut quit = false;

        match key {
            Key::Char('q') | Key::CtrlC => quit = true,
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
            Key::Char('/') => {
                state.mode = Mode::Search;
                state.search_message.clear();
            }
            Key::Char('n') => {
                if !state.search_matches.is_empty() {
                    state.current_match =
                        (state.current_match + 1) % state.search_matches.len() as isize;
                    scroll_to_match(&mut state);
                }
            }
            Key::Char('N') => {
                if !state.search_matches.is_empty() {
                    state.current_match = (state.current_match - 1
                        + state.search_matches.len() as isize)
                        % state.search_matches.len() as isize;
                    scroll_to_match(&mut state);
                }
            }
            Key::Char('c') => {
                if let Some(ref fp) = state.file_path
                    && copy_to_clipboard(fp)
                {
                    let name = fp.rsplit('/').next().unwrap_or(fp);
                    state.search_message = format!("Copied: {name}");
                }
            }
            Key::Char('C') => {
                if let Some(ref fp) = state.file_path
                    && let Ok(abs) = std::fs::canonicalize(fp)
                {
                    let abs_str = abs.to_string_lossy().to_string();
                    if copy_to_clipboard(&abs_str) {
                        state.search_message = format!("Copied: {abs_str}");
                    }
                }
            }
            Key::Char('y') => {
                if let Some(ref raw) = state.raw_content
                    && copy_to_clipboard(raw)
                {
                    state.search_message = "Copied raw markdown".to_string();
                }
            }
            Key::Char('v') => {
                if let Some(ref fp) = state.file_path {
                    let fp = fp.clone();
                    // Exit raw mode & restore screen for editor
                    let _ = crossterm::terminal::disable_raw_mode();
                    let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
                    let _ = stdout.flush();

                    let line = state
                        .raw_content
                        .as_ref()
                        .map(|raw| map_to_source_line(state.top_line, state.lines.len(), raw));
                    open_in_editor(&fp, line);

                    // Re-enter pager
                    let _ = write!(stdout, "{ALT_SCREEN_ON}{CURSOR_HIDE}");
                    let _ = stdout.flush();
                    let _ = crossterm::terminal::enable_raw_mode();
                }
            }
            _ => {}
        }

        if quit {
            break;
        }

        state.search_message.clear();
        render_screen(&mut stdout, &state);
    }

    // Cleanup
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
    let _ = stdout.flush();
}

fn scroll_to_match(state: &mut PagerState) {
    if state.current_match < 0 || state.current_match as usize >= state.search_matches.len() {
        return;
    }
    let match_line = state.search_matches[state.current_match as usize];
    let (_, rows) = get_term_size();
    let content_height = rows.saturating_sub(1) as usize;
    let target = match_line.saturating_sub(content_height / 3);
    let max_top = state.lines.len().saturating_sub(content_height);
    state.top_line = target.min(max_top);
}

fn render_screen(out: &mut impl Write, state: &PagerState) {
    let (cols, rows) = get_term_size();
    let content_height = rows.saturating_sub(1) as usize;

    // Clamp top_line
    let max_top = state.lines.len().saturating_sub(content_height);
    let top = state.top_line.min(max_top);

    move_to(out, 0, 0);

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
            let _ = write!(out, "{CLEAR_LINE}{vline}");
            if visual_row < content_height - 1 {
                let _ = write!(out, "\r\n");
            }
            visual_row += 1;
        }
        logical_idx += 1;
    }

    // Clear remaining rows
    while visual_row < content_height {
        let _ = write!(out, "{CLEAR_LINE}");
        if visual_row < content_height - 1 {
            let _ = write!(out, "\r\n");
        }
        visual_row += 1;
    }

    let _ = write!(out, "\r\n{CLEAR_LINE}");

    let status_input = StatusBarInput {
        mode: state.mode.clone(),
        search_input: state.search_input.clone(),
        search_cursor: state.search_cursor,
        search_message: state.search_message.clone(),
        search_query: state.search_query.clone(),
        search_matches: state.search_matches.clone(),
        current_match: state.current_match,
        top_line: top,
        line_count: state.lines.len(),
        content_height,
        file_path: state.file_path.clone(),
    };

    let status = render_status_bar(&status_input, cols as usize);
    let _ = write!(out, "{status}");
    let _ = out.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    // ---- parse_key ----
    #[derive(Deserialize)]
    struct ParseKeyCase {
        name: String,
        input: Vec<u8>,
        expected: KeyJson,
    }

    #[derive(Deserialize)]
    struct KeyJson {
        r#type: String,
        char: Option<String>,
    }

    fn key_matches(key: &Key, json: &KeyJson) -> bool {
        match key {
            Key::Char(c) => json.r#type == "char" && json.char.as_deref() == Some(&c.to_string()),
            Key::Enter => json.r#type == "enter",
            Key::Escape => json.r#type == "escape",
            Key::Backspace => json.r#type == "backspace",
            Key::CtrlC => json.r#type == "ctrl-c",
            Key::CtrlD => json.r#type == "ctrl-d",
            Key::CtrlU => json.r#type == "ctrl-u",
            Key::Up => json.r#type == "up",
            Key::Down => json.r#type == "down",
            Key::Left => json.r#type == "left",
            Key::Right => json.r#type == "right",
            Key::AltLeft => json.r#type == "alt-left",
            Key::AltRight => json.r#type == "alt-right",
            Key::AltBackspace => json.r#type == "alt-backspace",
            Key::PageUp => json.r#type == "pageup",
            Key::PageDown => json.r#type == "pagedown",
            Key::Home => json.r#type == "home",
            Key::End => json.r#type == "end",
            Key::Unknown => json.r#type == "unknown",
        }
    }

    #[test]
    fn test_parse_key() {
        let json = include_str!("../fixtures/pager/parse-key.json");
        let cases: Vec<ParseKeyCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let key = parse_key(&case.input);
            assert!(
                key_matches(&key, &case.expected),
                "parse_key: {} — got {:?}, expected type={} char={:?}",
                case.name,
                key,
                case.expected.r#type,
                case.expected.char
            );
        }
    }

    // ---- highlight_search ----
    #[derive(Deserialize)]
    struct HighlightSearchCase {
        name: String,
        input: String,
        params: HighlightSearchParams,
        expected: String,
    }

    #[derive(Deserialize)]
    struct HighlightSearchParams {
        query: String,
    }

    #[test]
    fn test_highlight_search() {
        let json = include_str!("../fixtures/pager/highlight-search.json");
        let cases: Vec<HighlightSearchCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let result = highlight_search(&case.input, &case.params.query);
            assert_eq!(result, case.expected, "highlight_search: {}", case.name);
        }
    }

    // ---- find_matches ----
    #[derive(Deserialize)]
    struct FindMatchesCase {
        name: String,
        input: FindMatchesInput,
        expected: Vec<usize>,
    }

    #[derive(Deserialize)]
    struct FindMatchesInput {
        lines: Vec<String>,
        query: String,
    }

    #[test]
    fn test_find_matches() {
        let json = include_str!("../fixtures/pager/find-matches.json");
        let cases: Vec<FindMatchesCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let result = find_matches(&case.input.lines, &case.input.query);
            assert_eq!(result, case.expected, "find_matches: {}", case.name);
        }
    }

    // ---- map_to_source_line ----
    #[derive(Deserialize)]
    struct MapToSourceLineCase {
        name: String,
        input: MapToSourceLineInput,
        expected: usize,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct MapToSourceLineInput {
        top_line: usize,
        rendered_line_count: usize,
        raw_content: String,
    }

    #[test]
    fn test_map_to_source_line() {
        let json = include_str!("../fixtures/pager/map-to-source-line.json");
        let cases: Vec<MapToSourceLineCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let result = map_to_source_line(
                case.input.top_line,
                case.input.rendered_line_count,
                &case.input.raw_content,
            );
            assert_eq!(result, case.expected, "map_to_source_line: {}", case.name);
        }
    }

    // ---- map_scroll_position ----
    #[derive(Deserialize)]
    struct MapScrollPositionCase {
        name: String,
        input: MapScrollPositionInput,
        expected: usize,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct MapScrollPositionInput {
        old_top_line: usize,
        old_line_count: usize,
        new_line_count: usize,
    }

    #[test]
    fn test_map_scroll_position() {
        let json = include_str!("../fixtures/pager/map-scroll-position.json");
        let cases: Vec<MapScrollPositionCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let result = map_scroll_position(
                case.input.old_top_line,
                case.input.old_line_count,
                case.input.new_line_count,
            );
            assert_eq!(result, case.expected, "map_scroll_position: {}", case.name);
        }
    }

    // ---- find_nearest_match ----
    #[derive(Deserialize)]
    struct FindNearestMatchCase {
        name: String,
        input: FindNearestMatchInput,
        expected: isize,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FindNearestMatchInput {
        matches: Vec<usize>,
        top_line: usize,
    }

    #[test]
    fn test_find_nearest_match() {
        let json = include_str!("../fixtures/pager/find-nearest-match.json");
        let cases: Vec<FindNearestMatchCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let result = find_nearest_match(&case.input.matches, case.input.top_line);
            assert_eq!(result, case.expected, "find_nearest_match: {}", case.name);
        }
    }

    // ---- format_status_bar ----
    #[derive(Deserialize)]
    struct FormatStatusBarCase {
        name: String,
        input: FormatStatusBarFixtureInput,
        expected: String,
    }

    #[derive(Deserialize)]
    struct FormatStatusBarFixtureInput {
        state: StatusBarFixture,
        cols: usize,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct StatusBarFixture {
        mode: String,
        search_input: String,
        search_cursor: usize,
        search_message: String,
        search_query: String,
        search_matches: Vec<usize>,
        current_match: isize,
        top_line: usize,
        line_count: usize,
        content_height: usize,
        file_path: Option<String>,
    }

    #[test]
    fn test_format_status_bar() {
        let json = include_str!("../fixtures/pager/format-status-bar.json");
        let cases: Vec<FormatStatusBarCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let input = StatusBarInput {
                mode: if case.input.state.mode == "search" {
                    Mode::Search
                } else {
                    Mode::Normal
                },
                search_input: case.input.state.search_input.clone(),
                search_cursor: case.input.state.search_cursor,
                search_message: case.input.state.search_message.clone(),
                search_query: case.input.state.search_query.clone(),
                search_matches: case.input.state.search_matches.clone(),
                current_match: case.input.state.current_match,
                top_line: case.input.state.top_line,
                line_count: case.input.state.line_count,
                content_height: case.input.state.content_height,
                file_path: case.input.state.file_path.clone(),
            };
            let result = format_status_bar(&input, case.input.cols);
            assert_eq!(
                result,
                case.expected,
                "format_status_bar: {} — result bytes: {:?}, expected bytes: {:?}",
                case.name,
                result.as_bytes(),
                case.expected.as_bytes()
            );
        }
    }

    // ---- word_boundary ----
    #[derive(Deserialize)]
    struct WordBoundaryFixtures {
        left: Vec<WordBoundaryCase>,
        right: Vec<WordBoundaryCase>,
    }

    #[derive(Deserialize)]
    struct WordBoundaryCase {
        name: String,
        input: WordBoundaryInput,
        expected: usize,
    }

    #[derive(Deserialize)]
    struct WordBoundaryInput {
        text: String,
        cursor: usize,
    }

    #[test]
    fn test_word_boundary_left() {
        let json = include_str!("../fixtures/pager/word-boundary.json");
        let fixtures: WordBoundaryFixtures = serde_json::from_str(json).unwrap();
        for case in &fixtures.left {
            let result = word_boundary_left(&case.input.text, case.input.cursor);
            assert_eq!(result, case.expected, "word_boundary_left: {}", case.name);
        }
    }

    #[test]
    fn test_word_boundary_right() {
        let json = include_str!("../fixtures/pager/word-boundary.json");
        let fixtures: WordBoundaryFixtures = serde_json::from_str(json).unwrap();
        for case in &fixtures.right {
            let result = word_boundary_right(&case.input.text, case.input.cursor);
            assert_eq!(result, case.expected, "word_boundary_right: {}", case.name);
        }
    }

    // ---- handle_search_key ----
    #[derive(Deserialize)]
    struct HandleSearchKeyCase {
        name: String,
        state: HandleSearchKeyState,
        key: KeyJson,
        expected: HandleSearchKeyExpected,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct HandleSearchKeyState {
        lines: Vec<String>,
        top_line: usize,
        search_query: String,
        search_matches: Vec<usize>,
        current_match: isize,
        mode: String,
        search_input: String,
        search_cursor: usize,
        search_message: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct HandleSearchKeyExpected {
        mode: Option<String>,
        search_input: Option<String>,
        search_cursor: Option<usize>,
        search_query: Option<String>,
        search_matches: Option<Vec<usize>>,
        current_match: Option<isize>,
        search_message: Option<String>,
    }

    fn json_to_key(json: &KeyJson) -> Key {
        match json.r#type.as_str() {
            "char" => Key::Char(json.char.as_ref().unwrap().chars().next().unwrap()),
            "enter" => Key::Enter,
            "escape" => Key::Escape,
            "backspace" => Key::Backspace,
            "ctrl-c" => Key::CtrlC,
            "ctrl-d" => Key::CtrlD,
            "ctrl-u" => Key::CtrlU,
            "up" => Key::Up,
            "down" => Key::Down,
            "left" => Key::Left,
            "right" => Key::Right,
            "alt-left" => Key::AltLeft,
            "alt-right" => Key::AltRight,
            "alt-backspace" => Key::AltBackspace,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            "home" => Key::Home,
            "end" => Key::End,
            _ => Key::Unknown,
        }
    }

    #[test]
    fn test_handle_search_key() {
        let json = include_str!("../fixtures/pager/handle-search-key.json");
        let cases: Vec<HandleSearchKeyCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let mut state = PagerState {
                lines: case.state.lines.clone(),
                top_line: case.state.top_line,
                search_query: case.state.search_query.clone(),
                search_matches: case.state.search_matches.clone(),
                current_match: case.state.current_match,
                mode: if case.state.mode == "search" {
                    Mode::Search
                } else {
                    Mode::Normal
                },
                search_input: case.state.search_input.clone(),
                search_cursor: case.state.search_cursor,
                search_message: case.state.search_message.clone(),
                file_path: None,
                raw_content: None,
            };

            let key = json_to_key(&case.key);
            handle_search_key(&mut state, &key);

            if let Some(ref expected_mode) = case.expected.mode {
                let actual_mode = match state.mode {
                    Mode::Normal => "normal",
                    Mode::Search => "search",
                };
                assert_eq!(
                    actual_mode, expected_mode,
                    "handle_search_key mode: {}",
                    case.name
                );
            }
            if let Some(ref expected_input) = case.expected.search_input {
                assert_eq!(
                    &state.search_input, expected_input,
                    "handle_search_key searchInput: {}",
                    case.name
                );
            }
            if let Some(expected_cursor) = case.expected.search_cursor {
                assert_eq!(
                    state.search_cursor, expected_cursor,
                    "handle_search_key searchCursor: {}",
                    case.name
                );
            }
            if let Some(ref expected_query) = case.expected.search_query {
                assert_eq!(
                    &state.search_query, expected_query,
                    "handle_search_key searchQuery: {}",
                    case.name
                );
            }
            if let Some(ref expected_matches) = case.expected.search_matches {
                assert_eq!(
                    &state.search_matches, expected_matches,
                    "handle_search_key searchMatches: {}",
                    case.name
                );
            }
            if let Some(expected_match) = case.expected.current_match {
                assert_eq!(
                    state.current_match, expected_match,
                    "handle_search_key currentMatch: {}",
                    case.name
                );
            }
            if let Some(ref expected_msg) = case.expected.search_message {
                assert_eq!(
                    &state.search_message, expected_msg,
                    "handle_search_key searchMessage: {}",
                    case.name
                );
            }
        }
    }

    // ---- crossterm_key_to_key ----
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_crossterm_key_to_key_ctrl_combos() {
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Key::CtrlC
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Char('d'), KeyModifiers::CONTROL)),
            Key::CtrlD
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Char('u'), KeyModifiers::CONTROL)),
            Key::CtrlU
        );
    }

    #[test]
    fn test_crossterm_key_to_key_alt_combos() {
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Char('b'), KeyModifiers::ALT)),
            Key::AltLeft
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Char('f'), KeyModifiers::ALT)),
            Key::AltRight
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Left, KeyModifiers::ALT)),
            Key::AltLeft
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Right, KeyModifiers::ALT)),
            Key::AltRight
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Backspace, KeyModifiers::ALT)),
            Key::AltBackspace
        );
    }

    #[test]
    fn test_crossterm_key_to_key_plain_chars() {
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Char('q'), KeyModifiers::NONE)),
            Key::Char('q')
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Char('G'), KeyModifiers::SHIFT)),
            Key::Char('G')
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Char(' '), KeyModifiers::NONE)),
            Key::Char(' ')
        );
    }

    #[test]
    fn test_crossterm_key_to_key_nav_keys() {
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Up, KeyModifiers::NONE)),
            Key::Up
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Down, KeyModifiers::NONE)),
            Key::Down
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Left, KeyModifiers::NONE)),
            Key::Left
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Right, KeyModifiers::NONE)),
            Key::Right
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::PageUp, KeyModifiers::NONE)),
            Key::PageUp
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::PageDown, KeyModifiers::NONE)),
            Key::PageDown
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Home, KeyModifiers::NONE)),
            Key::Home
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::End, KeyModifiers::NONE)),
            Key::End
        );
    }

    #[test]
    fn test_crossterm_key_to_key_special_keys() {
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Enter, KeyModifiers::NONE)),
            Key::Enter
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Esc, KeyModifiers::NONE)),
            Key::Escape
        );
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Backspace, KeyModifiers::NONE)),
            Key::Backspace
        );
    }

    #[test]
    fn test_crossterm_key_to_key_unknown() {
        assert_eq!(
            crossterm_key_to_key(make_key(KeyCode::Tab, KeyModifiers::NONE)),
            Key::Unknown
        );
    }
}
