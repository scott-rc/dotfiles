use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use regex::Regex;
use std::sync::LazyLock;

use tui::pager::{
    ALT_SCREEN_OFF, ALT_SCREEN_ON, CLEAR_LINE, CURSOR_HIDE, CURSOR_SHOW, copy_to_clipboard,
    get_term_size, move_to,
};
use tui::search::{
    find_matches, find_nearest_match, highlight_search, max_scroll, word_boundary_left,
    word_boundary_right,
};

use unicode_width::UnicodeWidthChar;

use crate::wrap::{split_ansi, wrap_line_for_display};

/// Matches OSC 8 hyperlink open sequences: \x1b]8;;URL\x07
static OSC8_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1b\]8;;([^\x07]*)\x07").unwrap());

const REVERSE: &str = "\x1b[7m";
const NO_REVERSE: &str = "\x1b[27m";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const NO_DIM: &str = "\x1b[22m";
const STATUS_BG: &str = "\x1b[48;2;28;33;40m";
const STATUS_FG: &str = "\x1b[38;2;139;148;158m";
const LINK_HIGHLIGHT_BG: &str = "\x1b[48;2;22;30;48m";

pub use tui::pager::Key;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Search,
    Help,
}

/// A link extracted from rendered output.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkInfo {
    pub line: usize,
    pub url: String,
}

/// Saved state when navigating into a linked file.
pub struct StackEntry {
    pub lines: Vec<String>,
    pub top_line: usize,
    pub file_path: Option<String>,
    pub raw_content: Option<String>,
    pub links: Vec<LinkInfo>,
    pub focused_link: isize,
}

pub struct PagerState {
    pub lines: Vec<String>,
    pub top_line: usize,
    pub is_plain: bool,
    pub search_query: String,
    pub search_matches: Vec<usize>,
    pub current_match: isize,
    pub mode: Mode,
    pub search_input: String,
    pub search_cursor: usize,
    pub search_message: String,
    pub file_path: Option<String>,
    pub raw_content: Option<String>,
    pub links: Vec<LinkInfo>,
    pub focused_link: isize,
    pub file_stack: Vec<StackEntry>,
}

/// Extract links from rendered lines by scanning for OSC 8 sequences.
/// Returns one `LinkInfo` per unique URL per line (deduped within a line).
pub fn extract_links(lines: &[String]) -> Vec<LinkInfo> {
    let mut result = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let mut seen = std::collections::HashSet::new();
        for cap in OSC8_RE.captures_iter(line) {
            let url = cap[1].to_string();
            if !url.is_empty() && seen.insert(url.clone()) {
                result.push(LinkInfo { line: i, url });
            }
        }
    }
    result
}

/// Resolve a link destination relative to the directory of the current file.
/// Returns `None` if the URL is not a local file path.
pub fn resolve_link_path(url: &str, current_file: &str) -> Option<String> {
    // Skip absolute URLs (http(s), mailto, etc.)
    if url.contains("://") || url.starts_with("mailto:") {
        return None;
    }
    let link_path = Path::new(url);
    if link_path.is_absolute() {
        return Some(url.to_string());
    }
    // Resolve relative to current file's directory
    let base = Path::new(current_file).parent()?;
    let resolved = base.join(link_path);
    Some(resolved.to_string_lossy().into_owned())
}

/// Returns true if the URL looks like a local markdown file.
pub fn is_md_link(url: &str) -> bool {
    let path = url.split('#').next().unwrap_or(url);
    let path = path.split('?').next().unwrap_or(path);
    let lower = path.to_lowercase();
    lower.ends_with(".md") || lower.ends_with(".mdx") || lower.ends_with(".markdown")
}

/// Find the OSC 8 hyperlink URL at a given visible column position in a rendered line.
pub fn link_url_at_col(line: &str, col: usize) -> Option<String> {
    let segments = split_ansi(line);
    let mut visible_col: usize = 0;
    let mut active_url: Option<String> = None;

    for seg in segments {
        if seg.starts_with("\x1b]8;") {
            // OSC 8 hyperlink open/close
            if seg == "\x1b]8;;\x07" {
                active_url = None;
            } else if let Some(url) = seg.strip_prefix("\x1b]8;;") {
                let url = url.strip_suffix('\x07').unwrap_or(url);
                if url.is_empty() {
                    active_url = None;
                } else {
                    active_url = Some(url.to_string());
                }
            }
            continue;
        }
        if seg.starts_with('\x1b') {
            // SGR sequence — skip
            continue;
        }
        // Visible text
        for c in seg.chars() {
            let cw = c.width().unwrap_or(0);
            if cw > 0 && col >= visible_col && col < visible_col + cw {
                return active_url.clone();
            }
            visible_col += cw;
        }
    }
    None
}

/// Map a screen row to a (logical line index, sub-row within that wrapped line).
pub fn screen_row_to_logical(
    lines: &[String],
    top_line: usize,
    row: usize,
    cols: usize,
) -> Option<(usize, usize)> {
    let mut visual_row: usize = 0;
    let mut idx = top_line;

    while idx < lines.len() {
        let wrapped = wrap_line_for_display(&lines[idx], cols);
        let count = wrapped.len();
        if row < visual_row + count {
            return Some((idx, row - visual_row));
        }
        visual_row += count;
        idx += 1;
    }
    None
}

fn bar_visible(state: &PagerState) -> bool {
    state.mode == Mode::Search
        || state.mode == Mode::Help
        || !state.search_message.is_empty()
        || state.focused_link >= 0
}

fn content_height(rows: u16, state: &PagerState) -> usize {
    if bar_visible(state) {
        rows.saturating_sub(1) as usize
    } else {
        rows as usize
    }
}

pub use tui::pager::crossterm_to_key as crossterm_key_to_key;

pub fn map_scroll_position(old_top: usize, old_count: usize, new_count: usize) -> usize {
    if old_count <= 1 || new_count <= 1 {
        return 0;
    }
    let ratio = old_top as f64 / (old_count - 1) as f64;
    (ratio * (new_count - 1) as f64).round() as usize
}

pub fn map_to_source_line(top_line: usize, rendered_line_count: usize, raw_content: &str) -> usize {
    let source_count = raw_content.lines().count();
    if rendered_line_count == 0 {
        return 1;
    }
    let ratio = top_line as f64 / rendered_line_count as f64;
    (ratio * source_count as f64).round() as usize + 1
}

pub fn format_help_lines(cols: usize, content_height: usize) -> Vec<String> {
    let help = [
        "Navigation",
        "j/↓        Scroll down",
        "k/↑        Scroll up",
        "d/Space    Half page down",
        "u          Half page up",
        "g/Home     Top",
        "G/End      Bottom",
        "",
        "Links",
        "Tab        Next link",
        "Shift-Tab  Previous link",
        "Enter      Follow link / scroll down",
        "Backspace  Go back",
        "",
        "Search",
        "/          Search",
        "n/N        Next/prev match",
        "",
        "c/C/y/e/v/p/r  Copy/edit/toggle",
        "q          Quit",
    ];

    let mut lines = Vec::with_capacity(content_height);

    // Vertically center: blank lines above, content, blank lines below
    let top_padding = if content_height > help.len() {
        (content_height - help.len()) / 2
    } else {
        0
    };

    for _ in 0..top_padding {
        lines.push(" ".repeat(cols));
    }

    let max_width = help.iter().map(|h| h.chars().count()).max().unwrap_or(0);
    let left_pad = cols.saturating_sub(max_width) / 2;

    for &h in &help {
        if lines.len() >= content_height {
            break;
        }
        let visible_len = h.chars().count();
        if visible_len >= cols {
            let truncated: String = h.chars().take(cols).collect();
            lines.push(truncated);
        } else {
            let right_pad = cols - left_pad - visible_len;
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

pub fn format_status_bar(state: &PagerState, cols: usize) -> String {
    // Help mode
    if state.mode == Mode::Help {
        let left = "? to close";
        let padding = cols.saturating_sub(left.len());
        return format!("{left}{}", " ".repeat(padding));
    }

    // Search mode
    if state.mode == Mode::Search {
        let before = &state.search_input[..state.search_cursor];
        let after = &state.search_input[state.search_cursor..];

        let cursor_char = if state.search_cursor < state.search_input.len() {
            let c = after.chars().next().unwrap();
            let rest = &after[c.len_utf8()..];
            format!("{REVERSE}{c}{NO_REVERSE}{rest}")
        } else {
            format!("{REVERSE}\u{2588}{NO_REVERSE}")
        };

        let content = format!("/{before}{cursor_char}");
        // "/" + input chars + block cursor (only if cursor is at end, adding a new char)
        let visible_len = if state.search_cursor < state.search_input.len() {
            1 + state.search_input.len()
        } else {
            1 + state.search_input.len() + 1
        };
        let padding = if cols > visible_len {
            " ".repeat(cols - visible_len)
        } else {
            String::new()
        };
        return format!("{content}{padding}");
    }

    // Search message mode
    if !state.search_message.is_empty() {
        let msg = &state.search_message;
        let padding = if cols > msg.len() {
            " ".repeat(cols - msg.len())
        } else {
            String::new()
        };
        return format!("{msg}{padding}");
    }

    // Focused link mode
    if state.focused_link >= 0 && (state.focused_link as usize) < state.links.len() {
        let link = &state.links[state.focused_link as usize];
        let idx = state.focused_link as usize + 1;
        let total = state.links.len();
        let prefix = if !state.file_stack.is_empty() {
            format!("[{idx}/{total}] {}", link.url)
        } else {
            format!("[{idx}/{total}] {}", link.url)
        };
        let back_hint = if !state.file_stack.is_empty() {
            " ← Backspace"
        } else {
            ""
        };
        let msg = format!("{prefix}{back_hint}");
        let padding = if cols > msg.len() {
            " ".repeat(cols - msg.len())
        } else {
            String::new()
        };
        return format!("{msg}{padding}");
    }

    String::new()
}

pub fn render_status_bar(state: &PagerState, cols: usize) -> String {
    format!(
        "{RESET}{STATUS_BG}{STATUS_FG}{}{RESET}",
        format_status_bar(state, cols)
    )
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

fn open_in_editor(file_path: &str, line: Option<usize>, read_only: bool) {
    tui::pager::open_in_editor(file_path, line.map(|l| l as u32), read_only);
}

/// Follow a link URL: open local .md files stacked in the pager, or open external URLs in browser.
fn follow_link(
    state: &mut PagerState,
    url: &str,
    on_rerender: &mut Option<&mut dyn FnMut(bool, &str) -> String>,
) {
    if is_md_link(url) {
        if let Some(ref fp) = state.file_path {
            if let Some(resolved) = resolve_link_path(url, fp) {
                match std::fs::read_to_string(&resolved) {
                    Ok(new_raw) => {
                        state.file_stack.push(StackEntry {
                            lines: std::mem::take(&mut state.lines),
                            top_line: state.top_line,
                            file_path: state.file_path.take(),
                            raw_content: state.raw_content.take(),
                            links: std::mem::take(&mut state.links),
                            focused_link: state.focused_link,
                        });
                        state.raw_content = Some(new_raw);
                        state.file_path = Some(resolved);
                        state.top_line = 0;
                        state.focused_link = -1;
                        state.search_query.clear();
                        state.search_matches.clear();
                        state.current_match = -1;
                        refresh_content(state, on_rerender);
                    }
                    Err(e) => {
                        state.search_message = format!("Cannot open: {e}");
                    }
                }
            }
        }
    } else {
        let _ = std::process::Command::new("open").arg(url).spawn();
        state.search_message = format!("Opened: {url}");
    }
}

fn refresh_content(
    state: &mut PagerState,
    on_rerender: &mut Option<&mut dyn FnMut(bool, &str) -> String>,
) {
    let Some(rerender_fn) = on_rerender.as_mut() else {
        return;
    };
    let Some(raw) = state.raw_content.take() else {
        return;
    };
    let new_content = rerender_fn(state.is_plain, &raw);
    let old_count = state.lines.len();
    state.lines = new_content.lines().map(String::from).collect();
    state.top_line = map_scroll_position(state.top_line, old_count, state.lines.len());
    if !state.search_query.is_empty() {
        state.search_matches = find_matches(&state.lines, &state.search_query);
    }
    state.links = extract_links(&state.lines);
    state.focused_link = -1;
    state.raw_content = Some(raw);
}

pub fn run_pager(
    content: &str,
    file_path: Option<&str>,
    raw_content: Option<&str>,
    plain: bool,
    mut on_rerender: Option<&mut dyn FnMut(bool, &str) -> String>,
) {
    let mut stdout = io::BufWriter::new(io::stdout());

    // Enter alternate screen, hide cursor
    let _ = write!(stdout, "{ALT_SCREEN_ON}{CURSOR_HIDE}");
    let _ = stdout.flush();

    // Enable raw mode and mouse tracking
    let _ = crossterm::terminal::enable_raw_mode();
    let _ = write!(stdout, "\x1b[?1000h\x1b[?1006h");
    let _ = stdout.flush();

    let lines: Vec<String> = content.lines().map(String::from).collect();
    let links = extract_links(&lines);
    let mut state = PagerState {
        lines,
        top_line: 0,
        is_plain: plain,
        search_query: String::new(),
        search_matches: Vec::new(),
        current_match: -1,
        mode: Mode::Normal,
        search_input: String::new(),
        search_cursor: 0,
        search_message: String::new(),
        file_path: file_path.map(String::from),
        raw_content: raw_content.map(String::from),
        links,
        focused_link: -1,
        file_stack: Vec::new(),
    };

    let mut last_size = get_term_size();
    let mut pending_rerender = false;
    render_screen(&mut stdout, &state, last_size.0, last_size.1);

    loop {
        let event = match event::poll(Duration::from_millis(50)) {
            Ok(true) => match event::read() {
                Ok(ev) => ev,
                Err(_) => break,
            },
            Ok(false) => {
                // Poll timeout — check for size changes (Zellij may not send SIGWINCH)
                let current_size = get_term_size();
                if current_size != last_size || pending_rerender {
                    last_size = current_size;
                    pending_rerender = false;
                    refresh_content(&mut state, &mut on_rerender);
                    render_screen(&mut stdout, &state, last_size.0, last_size.1);
                }
                continue;
            }
            Err(_) => break,
        };

        // Handle mouse events before key conversion
        if let Event::Mouse(mouse) = &event {
            use crossterm::event::{MouseButton, MouseEventKind};
            match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    if state.mode != Mode::Search && state.mode != Mode::Help {
                        let cols = last_size.0;
                        let row = mouse.row as usize;
                        let col = mouse.column as usize;
                        if let Some((logical, sub_row)) =
                            screen_row_to_logical(&state.lines, state.top_line, row, cols as usize)
                        {
                            let wrapped =
                                wrap_line_for_display(&state.lines[logical], cols as usize);
                            if sub_row < wrapped.len() {
                                if let Some(url) = link_url_at_col(&wrapped[sub_row], col) {
                                    state.search_message.clear();
                                    follow_link(&mut state, &url, &mut on_rerender);
                                    render_screen(&mut stdout, &state, last_size.0, last_size.1);
                                }
                            }
                        }
                    }
                    continue;
                }
                MouseEventKind::ScrollUp => {
                    state.focused_link = -1;
                    state.top_line = state.top_line.saturating_sub(3);
                    render_screen(&mut stdout, &state, last_size.0, last_size.1);
                    continue;
                }
                MouseEventKind::ScrollDown => {
                    let ch = content_height(last_size.1, &state);
                    let max_top = max_scroll(state.lines.len(), ch);
                    state.focused_link = -1;
                    state.top_line = (state.top_line + 3).min(max_top);
                    render_screen(&mut stdout, &state, last_size.0, last_size.1);
                    continue;
                }
                _ => continue,
            }
        }

        let key = match event {
            Event::Resize(_, _) => {
                last_size = get_term_size();
                refresh_content(&mut state, &mut on_rerender);
                render_screen(&mut stdout, &state, last_size.0, last_size.1);
                pending_rerender = true;
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
                let ch = content_height(last_size.1, &state);
                scroll_to_match(&mut state, ch);
            }
            render_screen(&mut stdout, &state, last_size.0, last_size.1);
            continue;
        }

        if state.mode == Mode::Help {
            state.mode = Mode::Normal;
            render_screen(&mut stdout, &state, last_size.0, last_size.1);
            continue;
        }

        // Normal mode
        let rows = last_size.1;
        let ch = content_height(rows, &state);
        let half_page = ch / 2;
        let max_top = max_scroll(state.lines.len(), ch);

        state.search_message.clear();

        match key {
            Key::Char('q') | Key::CtrlC => break,
            Key::Tab => {
                // Cycle to next link
                if !state.links.is_empty() {
                    state.focused_link =
                        (state.focused_link + 1) % state.links.len() as isize;
                    let link_line = state.links[state.focused_link as usize].line;
                    // Scroll to show the link, centered in viewport
                    let target = link_line.saturating_sub(ch / 3);
                    state.top_line = target.min(max_top);
                }
            }
            Key::BackTab => {
                // Cycle to previous link
                if !state.links.is_empty() {
                    if state.focused_link <= 0 {
                        state.focused_link = state.links.len() as isize - 1;
                    } else {
                        state.focused_link -= 1;
                    }
                    let link_line = state.links[state.focused_link as usize].line;
                    let target = link_line.saturating_sub(ch / 3);
                    state.top_line = target.min(max_top);
                }
            }
            Key::Enter => {
                if state.focused_link >= 0 && (state.focused_link as usize) < state.links.len() {
                    let url = state.links[state.focused_link as usize].url.clone();
                    follow_link(&mut state, &url, &mut on_rerender);
                } else {
                    // No link focused: scroll down
                    state.top_line = (state.top_line + 1).min(max_top);
                }
            }
            Key::Backspace => {
                if let Some(entry) = state.file_stack.pop() {
                    state.lines = entry.lines;
                    state.top_line = entry.top_line;
                    state.file_path = entry.file_path;
                    state.raw_content = entry.raw_content;
                    state.links = entry.links;
                    state.focused_link = entry.focused_link;
                    state.search_query.clear();
                    state.search_matches.clear();
                    state.current_match = -1;
                }
            }
            Key::Char('j') | Key::Down => {
                state.focused_link = -1;
                state.top_line = (state.top_line + 1).min(max_top);
            }
            Key::Char('k') | Key::Up => {
                state.focused_link = -1;
                state.top_line = state.top_line.saturating_sub(1);
            }
            Key::Char('d' | ' ') | Key::CtrlD | Key::PageDown => {
                state.focused_link = -1;
                state.top_line = (state.top_line + half_page).min(max_top);
            }
            Key::Char('u') | Key::CtrlU | Key::PageUp => {
                state.focused_link = -1;
                state.top_line = state.top_line.saturating_sub(half_page);
            }
            Key::Char('g') | Key::Home => {
                state.focused_link = -1;
                state.top_line = 0;
            }
            Key::Char('G') | Key::End => {
                state.focused_link = -1;
                state.top_line = max_top;
            }
            Key::Char('/') => {
                state.mode = Mode::Search;
            }
            Key::Char('n') => {
                if !state.search_matches.is_empty() {
                    state.current_match =
                        (state.current_match + 1) % state.search_matches.len() as isize;
                    scroll_to_match(&mut state, ch);
                }
            }
            Key::Char('N') => {
                if !state.search_matches.is_empty() {
                    state.current_match = (state.current_match - 1
                        + state.search_matches.len() as isize)
                        % state.search_matches.len() as isize;
                    scroll_to_match(&mut state, ch);
                }
            }
            Key::Char('c') => {
                if let Some(ref fp) = state.file_path
                    && copy_to_clipboard(fp)
                {
                    state.search_message = format!("Copied: {fp}");
                }
            }
            Key::Char('?') => {
                state.mode = Mode::Help;
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
            Key::Char('p') => {
                state.is_plain = !state.is_plain;
                refresh_content(&mut state, &mut on_rerender);
                let label = if state.is_plain { "plain" } else { "pretty" };
                state.search_message = format!("Switched to {label} mode");
            }
            Key::Char('r') => {
                if let Some(ref fp) = state.file_path {
                    match std::fs::read_to_string(fp) {
                        Ok(new_raw) => {
                            state.raw_content = Some(new_raw);
                            refresh_content(&mut state, &mut on_rerender);
                            state.search_message = "Reloaded".to_string();
                        }
                        Err(e) => {
                            state.search_message = format!("Reload failed: {e}");
                        }
                    }
                }
            }
            Key::Char('v') | Key::Char('e') => {
                if let Some(ref fp) = state.file_path {
                    let fp = fp.clone();
                    let read_only = key == Key::Char('v');
                    // Exit raw mode, mouse tracking & restore screen for editor
                    let _ = write!(stdout, "\x1b[?1000l\x1b[?1006l");
                    let _ = crossterm::terminal::disable_raw_mode();
                    let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
                    let _ = stdout.flush();

                    let line = state
                        .raw_content
                        .as_ref()
                        .map(|raw| map_to_source_line(state.top_line, state.lines.len(), raw));
                    open_in_editor(&fp, line, read_only);

                    // Re-enter pager
                    let _ = write!(stdout, "{ALT_SCREEN_ON}{CURSOR_HIDE}");
                    let _ = stdout.flush();
                    let _ = crossterm::terminal::enable_raw_mode();
                    let _ = write!(stdout, "\x1b[?1000h\x1b[?1006h");
                    let _ = stdout.flush();
                    last_size = get_term_size();
                    if let Ok(new_raw) = std::fs::read_to_string(&fp) {
                        state.raw_content = Some(new_raw);
                    }
                    refresh_content(&mut state, &mut on_rerender);
                }
            }
            _ => {}
        }

        render_screen(&mut stdout, &state, last_size.0, last_size.1);
    }

    // Cleanup: disable mouse tracking, raw mode, restore screen
    let _ = write!(stdout, "\x1b[?1000l\x1b[?1006l");
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
    let _ = stdout.flush();
}

fn scroll_to_match(state: &mut PagerState, content_height: usize) {
    if state.current_match < 0 || state.current_match as usize >= state.search_matches.len() {
        return;
    }
    let match_line = state.search_matches[state.current_match as usize];
    let target = match_line.saturating_sub(content_height / 3);
    let max_top = max_scroll(state.lines.len(), content_height);
    state.top_line = target.min(max_top);
}

fn render_screen(out: &mut impl Write, state: &PagerState, cols: u16, rows: u16) {
    let ch = content_height(rows, state);

    // Clamp top_line
    let max_top = max_scroll(state.lines.len(), ch);
    let top = state.top_line.min(max_top);

    if state.mode == Mode::Help {
        let help_lines = format_help_lines(cols as usize, ch);
        for (i, line) in help_lines.iter().enumerate() {
            move_to(out, i as u16, 0);
            let _ = write!(out, "{CLEAR_LINE}{DIM}{line}{NO_DIM}");
        }
    } else {
        let mut visual_row: usize = 0;
        let mut logical_idx = top;

        let focused_line = if state.focused_link >= 0
            && (state.focused_link as usize) < state.links.len()
        {
            Some(state.links[state.focused_link as usize].line)
        } else {
            None
        };

        while visual_row < ch && logical_idx < state.lines.len() {
            let mut line = state.lines[logical_idx].clone();
            if !state.search_query.is_empty() {
                line = highlight_search(&line, &state.search_query);
            }
            let wrapped = wrap_line_for_display(&line, cols as usize);
            let is_focused = focused_line == Some(logical_idx);
            for vline in wrapped {
                if visual_row >= ch {
                    break;
                }
                move_to(out, visual_row as u16, 0);
                if is_focused {
                    let _ = write!(out, "{LINK_HIGHLIGHT_BG}{CLEAR_LINE}{vline}{RESET}");
                } else {
                    let _ = write!(out, "{CLEAR_LINE}{vline}");
                }
                visual_row += 1;
            }
            logical_idx += 1;
        }

        // Clear remaining rows
        while visual_row < ch {
            move_to(out, visual_row as u16, 0);
            let _ = write!(out, "{CLEAR_LINE}");
            visual_row += 1;
        }
    }

    if bar_visible(state) {
        move_to(out, ch as u16, 0);
        let _ = write!(out, "{CLEAR_LINE}");
        let status = render_status_bar(state, cols as usize);
        let _ = write!(out, "{status}");
    }
    let _ = out.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct KeyJson {
        r#type: String,
        char: Option<String>,
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
        file_path: Option<String>,
    }

    #[test]
    fn test_format_status_bar() {
        let json = include_str!("../fixtures/pager/format-status-bar.json");
        let cases: Vec<FormatStatusBarCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let state = PagerState {
                lines: vec![String::new(); case.input.state.line_count],
                top_line: case.input.state.top_line,
                search_query: case.input.state.search_query.clone(),
                search_matches: case.input.state.search_matches.clone(),
                current_match: case.input.state.current_match,
                mode: match case.input.state.mode.as_str() {
                    "search" => Mode::Search,
                    "help" => Mode::Help,
                    _ => Mode::Normal,
                },
                search_input: case.input.state.search_input.clone(),
                search_cursor: case.input.state.search_cursor,
                search_message: case.input.state.search_message.clone(),
                file_path: case.input.state.file_path.clone(),
                is_plain: false,
                raw_content: None,
                links: Vec::new(),
                focused_link: -1,
                file_stack: Vec::new(),
            };
            let result = format_status_bar(&state, case.input.cols);
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

    // ---- format_help_lines ----
    #[derive(Deserialize)]
    struct FormatHelpLinesCase {
        name: String,
        input: FormatHelpLinesInput,
        expected: FormatHelpLinesExpected,
    }

    #[derive(Deserialize)]
    struct FormatHelpLinesInput {
        cols: usize,
        rows: usize,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FormatHelpLinesExpected {
        line_count: usize,
        contains_lines: Vec<String>,
        all_line_width: usize,
        #[serde(default)]
        same_leading_spaces: Vec<String>,
    }

    #[test]
    fn test_format_help_lines() {
        let json = include_str!("../fixtures/pager/format-help-lines.json");
        let cases: Vec<FormatHelpLinesCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let lines = format_help_lines(case.input.cols, case.input.rows - 1);
            assert_eq!(
                lines.len(),
                case.expected.line_count,
                "format_help_lines line count: {}",
                case.name
            );
            for expected_content in &case.expected.contains_lines {
                assert!(
                    lines.iter().any(|l| l.contains(expected_content.as_str())),
                    "format_help_lines missing '{}': {}",
                    expected_content,
                    case.name
                );
            }
            for (i, line) in lines.iter().enumerate() {
                assert_eq!(
                    line.chars().count(),
                    case.expected.all_line_width,
                    "format_help_lines line {} width: {}",
                    i,
                    case.name
                );
            }
            if case.expected.same_leading_spaces.len() >= 2 {
                let pads: Vec<usize> = case
                    .expected
                    .same_leading_spaces
                    .iter()
                    .map(|needle| {
                        let line = lines
                            .iter()
                            .find(|l| l.contains(needle.as_str()))
                            .unwrap_or_else(|| {
                                panic!("same_leading_spaces needle '{}' not found: {}", needle, case.name)
                            });
                        line.chars().take_while(|c| *c == ' ').count()
                    })
                    .collect();
                for pad in &pads[1..] {
                    assert_eq!(
                        *pad,
                        pads[0],
                        "format_help_lines leading spaces differ: {}",
                        case.name
                    );
                }
            }
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
                is_plain: false,
                search_query: case.state.search_query.clone(),
                search_matches: case.state.search_matches.clone(),
                current_match: case.state.current_match,
                mode: match case.state.mode.as_str() {
                    "search" => Mode::Search,
                    "help" => Mode::Help,
                    _ => Mode::Normal,
                },
                search_input: case.state.search_input.clone(),
                search_cursor: case.state.search_cursor,
                search_message: case.state.search_message.clone(),
                file_path: None,
                raw_content: None,
                links: Vec::new(),
                focused_link: -1,
                file_stack: Vec::new(),
            };

            let key = json_to_key(&case.key);
            handle_search_key(&mut state, &key);

            if let Some(ref expected_mode) = case.expected.mode {
                let actual_mode = match state.mode {
                    Mode::Normal => "normal",
                    Mode::Search => "search",
                    Mode::Help => "help",
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

    // ---- max_scroll ----

    #[test]
    fn test_max_scroll_content_fits() {
        assert_eq!(max_scroll(10, 24), 0);
    }

    #[test]
    fn test_max_scroll_exact_fit() {
        assert_eq!(max_scroll(24, 24), 0);
    }

    #[test]
    fn test_max_scroll_barely_exceeds() {
        // 25 - 24 + 12 = 13
        assert_eq!(max_scroll(25, 24), 13);
    }

    #[test]
    fn test_max_scroll_exceeds() {
        // 50 - 24 + 12 = 38
        assert_eq!(max_scroll(50, 24), 38);
    }

    #[test]
    fn test_max_scroll_no_lines() {
        assert_eq!(max_scroll(0, 24), 0);
    }

    #[test]
    fn test_max_scroll_odd_content_height() {
        // 50 - 25 + 12 = 37
        assert_eq!(max_scroll(50, 25), 37);
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
            crossterm_key_to_key(make_key(KeyCode::Insert, KeyModifiers::NONE)),
            Key::Unknown
        );
    }

    // ---- refresh_content ----

    #[test]
    fn test_refresh_content_updates_content_and_scroll() {
        let mut state = PagerState {
            lines: vec!["line1".into(), "line2".into(), "line3".into(), "line4".into()],
            top_line: 2,
            is_plain: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match: -1,
            mode: Mode::Normal,
            search_input: String::new(),
            search_cursor: 0,
            search_message: String::new(),
            file_path: None,
            raw_content: Some("dummy".to_string()),
            links: Vec::new(),
            focused_link: -1,
            file_stack: Vec::new(),
        };

        let mut callback = |_: bool, _: &str| "a\nb\nc\nd\ne\nf\ng\nh".to_string();
        let mut on_rerender: Option<&mut dyn FnMut(bool, &str) -> String> = Some(&mut callback);
        refresh_content(&mut state, &mut on_rerender);

        assert_eq!(state.lines.len(), 8);
        // top_line=2, old_count=4, new_count=8 → ratio 2/3 * 7 ≈ 5
        assert_eq!(state.top_line, 5);
    }

    #[test]
    fn test_refresh_content_updates_search_matches() {
        let mut state = PagerState {
            lines: vec!["hello".into(), "world".into()],
            top_line: 0,
            is_plain: false,
            search_query: "foo".to_string(),
            search_matches: Vec::new(),
            current_match: -1,
            mode: Mode::Normal,
            search_input: String::new(),
            search_cursor: 0,
            search_message: String::new(),
            file_path: None,
            raw_content: Some("dummy".to_string()),
            links: Vec::new(),
            focused_link: -1,
            file_stack: Vec::new(),
        };

        let mut callback = |_: bool, _: &str| "no match\nfoo bar\nanother\nfoo baz".to_string();
        let mut on_rerender: Option<&mut dyn FnMut(bool, &str) -> String> = Some(&mut callback);
        refresh_content(&mut state, &mut on_rerender);

        assert_eq!(state.search_matches, vec![1, 3]);
    }

    #[test]
    fn test_refresh_content_no_callback() {
        let mut state = PagerState {
            lines: vec!["unchanged".into()],
            top_line: 0,
            is_plain: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match: -1,
            mode: Mode::Normal,
            search_input: String::new(),
            search_cursor: 0,
            search_message: String::new(),
            file_path: None,
            raw_content: None,
            links: Vec::new(),
            focused_link: -1,
            file_stack: Vec::new(),
        };

        let mut on_rerender: Option<&mut dyn FnMut(bool, &str) -> String> = None;
        refresh_content(&mut state, &mut on_rerender);

        assert_eq!(state.lines, vec!["unchanged".to_string()]);
        assert_eq!(state.top_line, 0);
    }

    // ---- extract_links ----

    #[test]
    fn test_extract_links_finds_osc8() {
        let lines = vec![
            "plain text".to_string(),
            "click \x1b]8;;https://example.com\x07here\x1b]8;;\x07 text".to_string(),
            "no links".to_string(),
            "\x1b]8;;./other.md\x07other\x1b]8;;\x07".to_string(),
        ];
        let links = extract_links(&lines);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], LinkInfo { line: 1, url: "https://example.com".to_string() });
        assert_eq!(links[1], LinkInfo { line: 3, url: "./other.md".to_string() });
    }

    #[test]
    fn test_extract_links_empty() {
        let lines = vec!["no links".to_string(), "at all".to_string()];
        assert!(extract_links(&lines).is_empty());
    }

    #[test]
    fn test_extract_links_dedupes_same_url_on_line() {
        let lines = vec![
            "\x1b]8;;https://x.com\x07a\x1b]8;;\x07 \x1b]8;;https://x.com\x07b\x1b]8;;\x07".to_string(),
        ];
        let links = extract_links(&lines);
        assert_eq!(links.len(), 1, "same URL on same line should dedup");
    }

    #[test]
    fn test_extract_links_multiple_urls_on_line() {
        let lines = vec![
            "\x1b]8;;https://a.com\x07a\x1b]8;;\x07 \x1b]8;;https://b.com\x07b\x1b]8;;\x07".to_string(),
        ];
        let links = extract_links(&lines);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].url, "https://a.com");
        assert_eq!(links[1].url, "https://b.com");
    }

    // ---- resolve_link_path ----

    #[test]
    fn test_resolve_link_path_relative() {
        let result = resolve_link_path("./other.md", "/docs/README.md");
        assert_eq!(result, Some("/docs/./other.md".to_string()));
    }

    #[test]
    fn test_resolve_link_path_parent_dir() {
        let result = resolve_link_path("../guide.md", "/docs/api/README.md");
        assert_eq!(result, Some("/docs/api/../guide.md".to_string()));
    }

    #[test]
    fn test_resolve_link_path_absolute_url() {
        let result = resolve_link_path("https://example.com", "/docs/README.md");
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_link_path_mailto() {
        let result = resolve_link_path("mailto:foo@bar.com", "/docs/README.md");
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_link_path_absolute_path() {
        let result = resolve_link_path("/tmp/test.md", "/docs/README.md");
        assert_eq!(result, Some("/tmp/test.md".to_string()));
    }

    // ---- link_url_at_col ----

    #[test]
    fn test_link_url_at_col_plain_text() {
        assert_eq!(link_url_at_col("hello world", 3), None);
    }

    #[test]
    fn test_link_url_at_col_within_link() {
        let line = "click \x1b]8;;https://example.com\x07here\x1b]8;;\x07 text";
        assert_eq!(
            link_url_at_col(line, 7), // 'h' of "here"
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_link_url_at_col_before_link() {
        let line = "click \x1b]8;;https://example.com\x07here\x1b]8;;\x07 text";
        assert_eq!(link_url_at_col(line, 3), None); // 'c' of "click"
    }

    #[test]
    fn test_link_url_at_col_after_link() {
        let line = "click \x1b]8;;https://example.com\x07here\x1b]8;;\x07 text";
        assert_eq!(link_url_at_col(line, 11), None); // 't' of " text"
    }

    #[test]
    fn test_link_url_at_col_first_col_of_link() {
        let line = "ab\x1b]8;;https://x.com\x07cd\x1b]8;;\x07";
        assert_eq!(
            link_url_at_col(line, 2), // 'c'
            Some("https://x.com".to_string())
        );
    }

    #[test]
    fn test_link_url_at_col_last_col_of_link() {
        let line = "ab\x1b]8;;https://x.com\x07cd\x1b]8;;\x07ef";
        assert_eq!(
            link_url_at_col(line, 3), // 'd'
            Some("https://x.com".to_string())
        );
    }

    #[test]
    fn test_link_url_at_col_two_links() {
        let line = "\x1b]8;;https://a.com\x07aa\x1b]8;;\x07 \x1b]8;;https://b.com\x07bb\x1b]8;;\x07";
        assert_eq!(
            link_url_at_col(line, 0),
            Some("https://a.com".to_string())
        );
        assert_eq!(
            link_url_at_col(line, 1),
            Some("https://a.com".to_string())
        );
        assert_eq!(link_url_at_col(line, 2), None); // space between
        assert_eq!(
            link_url_at_col(line, 3),
            Some("https://b.com".to_string())
        );
        assert_eq!(
            link_url_at_col(line, 4),
            Some("https://b.com".to_string())
        );
    }

    #[test]
    fn test_link_url_at_col_with_sgr_inside() {
        // SGR bold inside the OSC 8 region
        let line = "\x1b]8;;https://x.com\x07\x1b[1mlink\x1b[22m\x1b]8;;\x07";
        assert_eq!(
            link_url_at_col(line, 0),
            Some("https://x.com".to_string())
        );
        assert_eq!(
            link_url_at_col(line, 3),
            Some("https://x.com".to_string())
        );
    }

    // ---- screen_row_to_logical ----

    #[test]
    fn test_screen_row_to_logical_no_wrap() {
        let lines = vec!["aaa".into(), "bbb".into(), "ccc".into()];
        assert_eq!(screen_row_to_logical(&lines, 0, 0, 80), Some((0, 0)));
        assert_eq!(screen_row_to_logical(&lines, 0, 1, 80), Some((1, 0)));
        assert_eq!(screen_row_to_logical(&lines, 0, 2, 80), Some((2, 0)));
    }

    #[test]
    fn test_screen_row_to_logical_scrolled() {
        let lines: Vec<String> = (0..10).map(|i| format!("line {i}")).collect();
        assert_eq!(screen_row_to_logical(&lines, 5, 0, 80), Some((5, 0)));
        assert_eq!(screen_row_to_logical(&lines, 5, 1, 80), Some((6, 0)));
    }

    #[test]
    fn test_screen_row_to_logical_with_wrap() {
        // "abcdefghij" at width 5 wraps into 2 visual rows
        let lines = vec!["abcdefghij".into(), "short".into()];
        assert_eq!(screen_row_to_logical(&lines, 0, 0, 5), Some((0, 0)));
        assert_eq!(screen_row_to_logical(&lines, 0, 1, 5), Some((0, 1)));
        assert_eq!(screen_row_to_logical(&lines, 0, 2, 5), Some((1, 0)));
    }

    #[test]
    fn test_screen_row_to_logical_beyond_content() {
        let lines = vec!["a".into(), "b".into()];
        assert_eq!(screen_row_to_logical(&lines, 0, 5, 80), None);
    }

    #[test]
    fn test_screen_row_to_logical_empty() {
        let lines: Vec<String> = vec![];
        assert_eq!(screen_row_to_logical(&lines, 0, 0, 80), None);
    }

    // ---- is_md_link ----

    #[test]
    fn test_is_md_link() {
        assert!(is_md_link("other.md"));
        assert!(is_md_link("./docs/guide.md"));
        assert!(is_md_link("CHANGELOG.mdx"));
        assert!(is_md_link("notes.markdown"));
        assert!(is_md_link("file.md#section"));
        assert!(!is_md_link("https://example.com"));
        assert!(!is_md_link("image.png"));
        assert!(!is_md_link("script.js"));
    }
}
