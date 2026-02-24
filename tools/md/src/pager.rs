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

use crate::wrap::wrap_line_for_display;

const REVERSE: &str = "\x1b[7m";
const NO_REVERSE: &str = "\x1b[27m";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const NO_DIM: &str = "\x1b[22m";
const STATUS_BG: &str = "\x1b[48;2;28;33;40m";
const STATUS_FG: &str = "\x1b[38;2;139;148;158m";

pub use tui::pager::Key;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Search,
    Help,
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
}

fn bar_visible(state: &PagerState) -> bool {
    state.mode == Mode::Search
        || state.mode == Mode::Help
        || !state.search_message.is_empty()
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
        "j/↓/Enter  Scroll down",
        "k/↑        Scroll up",
        "d/Space    Half page down",
        "u          Half page up",
        "g/Home     Top",
        "G/End      Bottom",
        "",
        "Search",
        "/          Search",
        "n          Next match",
        "N          Previous match",
        "",
        "Clipboard & Editor",
        "c          Copy path",
        "C          Copy absolute path",
        "y          Copy raw markdown",
        "e          Open in editor",
        "v          Open read-only",
        "p          Toggle plain/pretty",
        "r          Reload file",
        "",
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

    // Enable raw mode
    let _ = crossterm::terminal::enable_raw_mode();

    let mut state = PagerState {
        lines: content.lines().map(String::from).collect(),
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
                    // Exit raw mode & restore screen for editor
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

    // Cleanup
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

        while visual_row < ch && logical_idx < state.lines.len() {
            let mut line = state.lines[logical_idx].clone();
            if !state.search_query.is_empty() {
                line = highlight_search(&line, &state.search_query);
            }
            let wrapped = wrap_line_for_display(&line, cols as usize);
            for vline in wrapped {
                if visual_row >= ch {
                    break;
                }
                move_to(out, visual_row as u16, 0);
                let _ = write!(out, "{CLEAR_LINE}{vline}");
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
        };

        let mut on_rerender: Option<&mut dyn FnMut(bool, &str) -> String> = None;
        refresh_content(&mut state, &mut on_rerender);

        assert_eq!(state.lines, vec!["unchanged".to_string()]);
        assert_eq!(state.top_line, 0);
    }
}
