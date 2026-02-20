use regex::Regex;
use std::sync::LazyLock;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

static ANSI_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*m").unwrap());
static BACKTICK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"((?:\x1b\[[0-9;]*m)*`(?:\x1b\[[0-9;]*m)*)$").unwrap());

#[derive(Clone, Default)]
struct AnsiState {
    bold: bool,
    italic: bool,
    underline: bool,
    fg: Option<String>,
}

impl AnsiState {
    fn update(&mut self, code: &str) {
        let inner = &code[2..code.len() - 1];
        if inner.is_empty() {
            return;
        }
        let params: Vec<&str> = inner.split(';').collect();
        let mut i = 0;
        while i < params.len() {
            match params[i] {
                "0" => *self = Self::default(),
                "1" => self.bold = true,
                "22" => self.bold = false,
                "3" => self.italic = true,
                "23" => self.italic = false,
                "4" => self.underline = true,
                "24" => self.underline = false,
                "38" if i + 4 < params.len() && params[i + 1] == "2" => {
                    self.fg = Some(format!(
                        "\x1b[38;2;{};{};{}m",
                        params[i + 2],
                        params[i + 3],
                        params[i + 4]
                    ));
                    i += 4;
                }
                "39" => self.fg = None,
                _ => {}
            }
            i += 1;
        }
    }

    fn is_active(&self) -> bool {
        self.bold || self.italic || self.underline || self.fg.is_some()
    }

    fn to_codes(&self) -> String {
        let mut s = String::new();
        if self.bold {
            s.push_str("\x1b[1m");
        }
        if self.italic {
            s.push_str("\x1b[3m");
        }
        if self.underline {
            s.push_str("\x1b[4m");
        }
        if let Some(ref fg) = self.fg {
            s.push_str(fg);
        }
        s
    }

    fn from_line(line: &str) -> Self {
        let mut state = Self::default();
        for m in ANSI_RE.find_iter(line) {
            state.update(m.as_str());
        }
        state
    }
}

/// Remove ANSI escape codes from a string.
pub fn strip_ansi(text: &str) -> String {
    if !text.contains('\x1b') {
        return text.to_string();
    }
    ANSI_RE.replace_all(text, "").into_owned()
}

/// Get the visible length of a string, ignoring ANSI escape codes.
pub fn visible_length(text: &str) -> usize {
    if !text.contains('\x1b') {
        return text.width();
    }
    let mut width = 0;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            // Skip ANSI escape: \x1b[...m
            i += 1;
            while i < bytes.len() && bytes[i] != b'm' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            continue;
        }
        // Decode UTF-8 char and sum its display width
        let start = i;
        if bytes[i] < 0x80 {
            width += 1; // ASCII char is always width 1
            i += 1;
        } else {
            // Multi-byte UTF-8: determine length from leading byte
            let len = if bytes[i] & 0xE0 == 0xC0 {
                2
            } else if bytes[i] & 0xF0 == 0xE0 {
                3
            } else {
                4
            };
            let end = (start + len).min(bytes.len());
            if let Ok(s) = std::str::from_utf8(&bytes[start..end]) {
                if let Some(c) = s.chars().next() {
                    width += c.width().unwrap_or(0);
                }
            }
            i = end;
        }
    }
    width
}

/// Split a string into alternating plain-text and ANSI-code segments.
pub fn split_ansi(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut last_index = 0;

    for m in ANSI_RE.find_iter(text) {
        if m.start() > last_index {
            parts.push(&text[last_index..m.start()]);
        }
        parts.push(m.as_str());
        last_index = m.end();
    }

    if last_index < text.len() {
        parts.push(&text[last_index..]);
    }

    parts
}

/// Word-wrap text to a given width, optionally prepending an indent to each line.
/// ANSI escape codes are not counted toward visible width.
pub fn word_wrap(text: &str, width: usize, indent: &str) -> String {
    let indent_width = visible_length(indent);
    if width <= indent_width {
        return text.to_string();
    }
    let available = width - indent_width;

    let lines: Vec<&str> = text.split('\n').collect();
    let mut result: Vec<String> = Vec::new();

    for line in lines {
        if visible_length(line) == 0 {
            result.push(indent.to_string());
            continue;
        }

        let wrapped = wrap_line(line, available);
        for w in wrapped {
            result.push(format!("{indent}{w}"));
        }
    }

    result.join("\n")
}

/// Wrap a single line (no embedded newlines) to fit within `width` visible characters.
/// Applies widow prevention: if the last line would contain a single word,
/// retries with narrower widths to pull a second word onto the last line.
fn wrap_line(line: &str, width: usize) -> Vec<String> {
    let results = wrap_line_greedy(line, width);

    // Widow prevention: avoid a single word on the last line
    if results.len() >= 2 && !has_multiple_visible_words(results.last().unwrap()) {
        let min_width = if width > 15 { width - 15 } else { 1 };
        for w in (min_width..width).rev() {
            let alt = wrap_line_greedy(line, w);
            if has_multiple_visible_words(alt.last().unwrap()) {
                return alt;
            }
        }
    }

    results
}

/// Check if a string has multiple visible words (skipping ANSI sequences),
/// without allocating. Returns false for empty/whitespace-only strings.
fn has_multiple_visible_words(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut word_count = 0;
    let mut in_word = false;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            i += 1;
            while i < bytes.len() && bytes[i] != b'm' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b' ' || bytes[i] == b'\t' {
            in_word = false;
        } else if !in_word {
            in_word = true;
            word_count += 1;
            if word_count >= 2 {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Greedy line wrapping: fit as many words as possible on each line.
pub fn wrap_line_greedy(line: &str, width: usize) -> Vec<String> {
    if visible_length(line) <= width {
        return vec![line.to_string()];
    }

    let segments = split_ansi(line);
    let mut results: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let mut current_width: usize = 0;
    let mut state = AnsiState::default();

    for seg in &segments {
        if ANSI_RE.is_match(seg) {
            // ANSI code: append without counting width
            state.update(seg);
            current_line.push_str(seg);
            continue;
        }

        // Plain text: split into words (preserving spaces as separate tokens)
        let words = split_with_spaces(seg);
        for word in &words {
            if word.is_empty() {
                continue;
            }

            let word_len = word.len();

            // If this word alone exceeds width, force-break it
            if word_len > width && current_width == 0 {
                let mut i = 0;
                while i < word.len() {
                    if i > 0 {
                        if state.is_active() {
                            current_line.push_str("\x1b[0m");
                        }
                        results.push(current_line.clone());
                        current_line = state.to_codes();
                        current_width = 0;
                    }
                    let end = (i + width).min(word.len());
                    let chunk = &word[i..end];
                    current_line.push_str(chunk);
                    current_width += chunk.len();
                    i = end;
                }
                continue;
            }

            // If adding this word would exceed width, wrap
            if current_width + word_len > width && current_width > 0 {
                let mut line_to_save = current_line.trim_end().to_string();

                // Don't leave a dangling opening backtick at end of line.
                let vis = strip_ansi(&line_to_save);
                let bt_count = vis.matches('`').count();
                if bt_count % 2 == 1 && vis.trim_end().ends_with('`') {
                    // Try to find and remove trailing backtick (possibly wrapped in ANSI codes)
                    if let Some(m) = BACKTICK_RE.find(&line_to_save) {
                        let captured = m.as_str().to_string();
                        line_to_save = line_to_save[..m.start()].trim_end().to_string();
                        let save_state = AnsiState::from_line(&line_to_save);
                        if !strip_ansi(&line_to_save).trim().is_empty() {
                            if save_state.is_active() {
                                line_to_save.push_str("\x1b[0m");
                            }
                            results.push(line_to_save);
                        }
                        current_line = format!("{}{captured}{word}", save_state.to_codes());
                        current_width = 1 + word_len;
                        continue;
                    }
                }

                // Don't orphan a closing bracket at the start of a new line.
                if is_closing_bracket_word(word) {
                    if let Some(split_pos) = last_visible_space_pos(&line_to_save) {
                        let pulled = line_to_save[split_pos..].to_string();
                        line_to_save = line_to_save[..split_pos].trim_end().to_string();
                        if !strip_ansi(&line_to_save).trim().is_empty() {
                            let save_state = AnsiState::from_line(&line_to_save);
                            if save_state.is_active() {
                                line_to_save.push_str("\x1b[0m");
                            }
                            results.push(line_to_save);
                        }
                        let pulled_trimmed = pulled.trim_start();
                        current_line =
                            format!("{}{pulled_trimmed}{word}", state.to_codes());
                        current_width = visible_length(pulled_trimmed) + word_len;
                        continue;
                    }
                }

                if state.is_active() {
                    line_to_save.push_str("\x1b[0m");
                }
                results.push(line_to_save);
                current_line = state.to_codes();
                current_width = 0;

                // Skip leading spaces at the start of a new line
                if word.trim().is_empty() {
                    continue;
                }
            }

            current_line.push_str(word);
            current_width += word_len;
        }
    }

    if !current_line.is_empty() {
        results.push(current_line);
    }

    results
}

/// Check if a word consists entirely of closing brackets optionally followed by punctuation.
fn is_closing_bracket_word(s: &str) -> bool {
    !s.is_empty()
        && s.starts_with(|c: char| c == ')' || c == ']')
        && s.chars()
            .all(|c| matches!(c, ')' | ']' | '.' | ',' | ';' | ':'))
}

/// Find the byte position of the last visible space in a string that may contain ANSI codes.
fn last_visible_space_pos(s: &str) -> Option<usize> {
    let mut last_space = None;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            i += 1;
            while i < bytes.len() && bytes[i] != b'm' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b' ' {
            last_space = Some(i);
        }
        i += 1;
    }
    last_space
}

/// Split a string by spaces, preserving spaces as separate elements.
/// e.g. `"hello  world"` -> `["hello", "  ", "world"]`
fn split_with_spaces(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut last = 0;
    let mut in_space = false;

    for (i, c) in s.char_indices() {
        let is_space = c == ' ';
        if i == 0 {
            in_space = is_space;
            continue;
        }
        if is_space != in_space {
            if i > last {
                parts.push(&s[last..i]);
            }
            last = i;
            in_space = is_space;
        }
    }
    if last < s.len() {
        parts.push(&s[last..]);
    }

    parts
}

/// Break a single line at character boundaries to fit within max_width visible columns.
/// Preserves ANSI codes without counting them toward width.
pub fn wrap_line_for_display(line: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![line.to_string()];
    }

    let segments = split_ansi(line);
    let mut rows: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut width: usize = 0;
    let mut state = AnsiState::default();

    for seg in segments {
        if seg.starts_with('\x1b') {
            state.update(seg);
            current.push_str(seg);
            continue;
        }

        for c in seg.chars() {
            let cw = c.width().unwrap_or(0);
            if width + cw > max_width && width > 0 {
                if state.is_active() {
                    current.push_str("\x1b[0m");
                }
                rows.push(std::mem::take(&mut current));
                current = state.to_codes();
                width = 0;
            }
            current.push(c);
            width += cw;
        }
    }

    if !current.is_empty() || rows.is_empty() {
        rows.push(current);
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Fixtures {
        #[serde(rename = "wordWrap")]
        word_wrap: Vec<WordWrapCase>,
        #[serde(rename = "stripAnsi")]
        strip_ansi_cases: Vec<StripAnsiCase>,
        #[serde(rename = "visibleLength")]
        visible_length_cases: Vec<VisibleLengthCase>,
    }

    #[derive(Deserialize)]
    struct WordWrapCase {
        name: String,
        input: String,
        params: WordWrapParams,
        expected: String,
    }

    #[derive(Deserialize)]
    struct WordWrapParams {
        width: usize,
        indent: Option<String>,
    }

    #[derive(Deserialize)]
    struct StripAnsiCase {
        name: String,
        input: String,
        expected: String,
    }

    #[derive(Deserialize)]
    struct VisibleLengthCase {
        name: String,
        input: String,
        expected: usize,
    }

    fn load_fixtures() -> Fixtures {
        let json = include_str!("../fixtures/wrapping/word-wrap.json");
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_strip_ansi_fixtures() {
        let fixtures = load_fixtures();
        for case in &fixtures.strip_ansi_cases {
            assert_eq!(
                strip_ansi(&case.input),
                case.expected,
                "stripAnsi: {}",
                case.name
            );
        }
    }

    #[test]
    fn test_visible_length_fixtures() {
        let fixtures = load_fixtures();
        for case in &fixtures.visible_length_cases {
            assert_eq!(
                visible_length(&case.input),
                case.expected,
                "visibleLength: {}",
                case.name
            );
        }
    }

    #[test]
    fn test_word_wrap_fixtures() {
        let fixtures = load_fixtures();
        for case in &fixtures.word_wrap {
            let indent = case.params.indent.as_deref().unwrap_or("");
            let result = word_wrap(&case.input, case.params.width, indent);
            assert_eq!(result, case.expected, "wordWrap: {}", case.name);
        }
    }

    #[test]
    fn test_widow_prevention() {
        let result = word_wrap("the quick brown fox jumps over the lazy dog", 20, "");
        let lines: Vec<&str> = result.split('\n').collect();
        let last_words: Vec<&str> = lines.last().unwrap().trim().split_whitespace().collect();
        assert!(
            last_words.len() >= 2,
            "Last line should have >= 2 words, got: {:?}",
            last_words
        );
    }

    #[test]
    fn test_backtick_aware_breaking() {
        let gray = "\x1b[38;2;139;148;158m";
        let orange = "\x1b[38;2;255;166;87m";
        let reset = "\x1b[39m";
        let code_span = format!("{gray}`{reset}{orange}code{reset}{gray}`{reset}");
        let text = format!("some text {code_span} end");
        let result = word_wrap(&text, 12, "");
        let lines: Vec<&str> = result.split('\n').collect();
        let first_visible = strip_ansi(lines[0]).trim_end().to_string();
        assert!(
            !first_visible.ends_with('`'),
            "First line should not end with backtick, got: {first_visible:?}"
        );
        assert_eq!(first_visible, "some text");
    }

    #[test]
    fn test_dangling_closing_bracket() {
        let gray = "\x1b[38;2;139;148;158m";
        let orange = "\x1b[38;2;255;166;87m";
        let reset = "\x1b[39m";
        let shell_span = format!("{gray}`{reset}{orange}$SHELL{reset}{gray}`{reset}");

        // Short text where `)` ends up alone on the last line at width 18:
        // visible = "text (via `$SHELL`)" = 19 chars, so `)` overflows
        let short = format!("text (via {shell_span})");
        let result = word_wrap(&short, 18, "");
        let lines: Vec<&str> = result.split('\n').collect();
        for (i, line) in lines.iter().enumerate() {
            let visible = strip_ansi(line).trim().to_string();
            assert!(
                visible != ")" && visible != ")," && visible != "]",
                "Short text, Line {i} has only a closing bracket: {visible:?}\nFull output:\n{}",
                lines.iter().enumerate().map(|(j, l)| format!("  [{j}] {:?} (vis: {:?})", l, strip_ansi(l))).collect::<Vec<_>>().join("\n")
            );
        }

        // Longer text with multiple code spans across a range of widths
        let find_span = format!("{gray}`{reset}{orange}find{reset}{gray}`{reset}");
        let fzf_span = format!("{gray}`{reset}{orange}fzf{reset}{gray}`{reset}");
        let text = format!("uses {find_span} + {fzf_span} (via {shell_span}) to pick a file");
        for w in 30..50 {
            let result = word_wrap(&text, w, "");
            let lines: Vec<&str> = result.split('\n').collect();
            for (i, line) in lines.iter().enumerate() {
                let visible = strip_ansi(line).trim().to_string();
                assert!(
                    visible != ")" && visible != ")," && visible != "]",
                    "Width {w}, Line {i} has only a closing bracket: {visible:?}\nFull output:\n{}",
                    lines.iter().enumerate().map(|(j, l)| format!("  [{j}] {:?} (vis: {:?})", l, strip_ansi(l))).collect::<Vec<_>>().join("\n")
                );
            }
        }
    }

    #[test]
    fn test_split_ansi() {
        let input = "hello\x1b[1mworld\x1b[0m!";
        let parts = split_ansi(input);
        assert_eq!(parts, vec!["hello", "\x1b[1m", "world", "\x1b[0m", "!"]);
    }

    #[test]
    fn test_split_ansi_no_codes() {
        let parts = split_ansi("plain text");
        assert_eq!(parts, vec!["plain text"]);
    }

    #[test]
    fn test_split_with_spaces() {
        let parts = split_with_spaces("hello  world");
        assert_eq!(parts, vec!["hello", "  ", "world"]);
    }

    // ---- wrap_line_for_display ----
    #[derive(Deserialize)]
    struct WrapLineCase {
        name: String,
        input: String,
        params: WrapLineParams,
        expected: Vec<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct WrapLineParams {
        max_width: usize,
    }

    #[test]
    fn test_wrap_line_for_display() {
        let json = include_str!("../fixtures/pager/wrap-line-for-display.json");
        let cases: Vec<WrapLineCase> = serde_json::from_str(json).unwrap();
        for case in &cases {
            let result = wrap_line_for_display(&case.input, case.params.max_width);
            assert_eq!(
                result, case.expected,
                "wrap_line_for_display: {}",
                case.name
            );
        }
    }
}
