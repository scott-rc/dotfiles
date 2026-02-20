use regex::Regex;
use std::sync::LazyLock;
use unicode_width::UnicodeWidthChar;

static ANSI_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*m").unwrap());

#[derive(Clone, Default)]
pub struct AnsiState {
    bold: bool,
    italic: bool,
    dim: bool,
    fg: Option<String>,
    bg: Option<String>,
}

impl AnsiState {
    pub fn update(&mut self, code: &str) {
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
                "2" => self.dim = true,
                "3" => self.italic = true,
                "22" => {
                    self.bold = false;
                    self.dim = false;
                }
                "23" => self.italic = false,
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
                "48" if i + 4 < params.len() && params[i + 1] == "2" => {
                    self.bg = Some(format!(
                        "\x1b[48;2;{};{};{}m",
                        params[i + 2],
                        params[i + 3],
                        params[i + 4]
                    ));
                    i += 4;
                }
                "49" => self.bg = None,
                _ => {}
            }
            i += 1;
        }
    }

    pub fn is_active(&self) -> bool {
        self.bold || self.italic || self.dim || self.fg.is_some() || self.bg.is_some()
    }

    pub fn to_codes(&self) -> String {
        let mut s = String::new();
        if self.bold {
            s.push_str("\x1b[1m");
        }
        if self.dim {
            s.push_str("\x1b[2m");
        }
        if self.italic {
            s.push_str("\x1b[3m");
        }
        if let Some(ref fg) = self.fg {
            s.push_str(fg);
        }
        if let Some(ref bg) = self.bg {
            s.push_str(bg);
        }
        s
    }
}

/// Remove ANSI escape codes from a string.
pub fn strip_ansi(text: &str) -> String {
    if !text.contains('\x1b') {
        return text.to_string();
    }
    ANSI_RE.replace_all(text, "").into_owned()
}

/// Get the visible width of a string, ignoring ANSI escape codes.
pub fn visible_width(text: &str) -> usize {
    if !text.contains('\x1b') {
        return unicode_width::UnicodeWidthStr::width(text);
    }
    let mut width = 0;
    let bytes = text.as_bytes();
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
        let start = i;
        if bytes[i] < 0x80 {
            width += 1;
            i += 1;
        } else {
            let len = if bytes[i] & 0xE0 == 0xC0 {
                2
            } else if bytes[i] & 0xF0 == 0xE0 {
                3
            } else {
                4
            };
            let end = (start + len).min(bytes.len());
            if let Ok(s) = std::str::from_utf8(&bytes[start..end])
                && let Some(c) = s.chars().next()
            {
                width += c.width().unwrap_or(0);
            }
            i = end;
        }
    }
    width
}

/// Split a string into alternating plain-text and ANSI-code segments.
pub fn split_ansi(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut last = 0;
    for m in ANSI_RE.find_iter(text) {
        if m.start() > last {
            parts.push(&text[last..m.start()]);
        }
        parts.push(m.as_str());
        last = m.end();
    }
    if last < text.len() {
        parts.push(&text[last..]);
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

    #[test]
    fn strip_ansi_plain() {
        assert_eq!(strip_ansi("hello"), "hello");
    }

    #[test]
    fn strip_ansi_with_codes() {
        assert_eq!(strip_ansi("\x1b[1mhello\x1b[0m"), "hello");
        assert_eq!(
            strip_ansi("\x1b[38;2;100;200;50mtext\x1b[0m"),
            "text"
        );
    }

    #[test]
    fn visible_width_plain() {
        assert_eq!(visible_width("hello"), 5);
    }

    #[test]
    fn visible_width_with_ansi() {
        assert_eq!(visible_width("\x1b[1mhello\x1b[0m world"), 11);
    }

    #[test]
    fn split_ansi_segments() {
        let parts = split_ansi("hello\x1b[1mworld\x1b[0m!");
        assert_eq!(parts, vec!["hello", "\x1b[1m", "world", "\x1b[0m", "!"]);
    }

    #[test]
    fn wrap_short_line() {
        let result = wrap_line_for_display("short", 80);
        assert_eq!(result, vec!["short"]);
    }

    #[test]
    fn wrap_long_line() {
        let result = wrap_line_for_display("abcdefghij", 5);
        assert_eq!(result, vec!["abcde", "fghij"]);
    }
}
