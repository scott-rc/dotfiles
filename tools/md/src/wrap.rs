use regex::Regex;
use std::sync::LazyLock;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

static ANSI_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*m").unwrap());
static BACKTICK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"((?:\x1b\[[0-9;]*m)*`(?:\x1b\[[0-9;]*m)*)$").unwrap());

/// Remove ANSI escape codes from a string.
pub fn strip_ansi(text: &str) -> String {
    ANSI_RE.replace_all(text, "").into_owned()
}

/// Get the visible length of a string, ignoring ANSI escape codes.
pub fn visible_length(text: &str) -> usize {
    strip_ansi(text).width()
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
    if results.len() >= 2 {
        let last_visible = strip_ansi(results.last().unwrap()).trim().to_string();
        if is_single_word(&last_visible) {
            let min_width = if width > 15 { width - 15 } else { 1 };
            for w in (min_width..width).rev() {
                let alt = wrap_line_greedy(line, w);
                let alt_last_visible = strip_ansi(alt.last().unwrap()).trim().to_string();
                if !is_single_word(&alt_last_visible) {
                    return alt;
                }
            }
        }
    }

    results
}

fn is_single_word(s: &str) -> bool {
    !s.is_empty() && !s.contains(char::is_whitespace)
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

    for seg in &segments {
        if ANSI_RE.is_match(seg) {
            // ANSI code: append without counting width
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
                        results.push(current_line.clone());
                        current_line.clear();
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
                        if !strip_ansi(&line_to_save).trim().is_empty() {
                            results.push(line_to_save);
                        }
                        current_line = format!("{captured}{word}");
                        current_width = 1 + word_len;
                        continue;
                    }
                }

                results.push(line_to_save);
                current_line.clear();
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

    for seg in segments {
        if seg.starts_with('\x1b') {
            current.push_str(seg);
            continue;
        }

        for c in seg.chars() {
            let cw = c.width().unwrap_or(0);
            if width + cw > max_width && width > 0 {
                rows.push(std::mem::take(&mut current));
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
