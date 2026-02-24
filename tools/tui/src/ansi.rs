use regex::Regex;
use std::sync::LazyLock;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Matches ANSI SGR escape sequences (e.g., `\x1b[0m`, `\x1b[38;2;100;200;50m`)
/// and OSC 8 hyperlink sequences (e.g., `\x1b]8;;url\x07`).
pub static ANSI_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*m|\x1b\][^\x07]*\x07").unwrap());

/// Tracks active ANSI styling state across text segments.
///
/// Supports bold, dim, italic, underline, 24-bit foreground, and 24-bit background.
#[derive(Clone, Default)]
pub struct AnsiState {
    attrs: u8,
    pub fg: Option<String>,
    pub bg: Option<String>,
}

const ATTR_BOLD: u8 = 1 << 0;
const ATTR_DIM: u8 = 1 << 1;
const ATTR_ITALIC: u8 = 1 << 2;
const ATTR_UNDERLINE: u8 = 1 << 3;

impl AnsiState {
    fn set_attr(&mut self, attr: u8, enabled: bool) {
        if enabled {
            self.attrs |= attr;
        } else {
            self.attrs &= !attr;
        }
    }

    fn has_attr(&self, attr: u8) -> bool {
        self.attrs & attr != 0
    }

    pub fn is_bold(&self) -> bool {
        self.has_attr(ATTR_BOLD)
    }

    pub fn is_dim(&self) -> bool {
        self.has_attr(ATTR_DIM)
    }

    pub fn is_italic(&self) -> bool {
        self.has_attr(ATTR_ITALIC)
    }

    pub fn is_underline(&self) -> bool {
        self.has_attr(ATTR_UNDERLINE)
    }

    /// Update state from a single ANSI escape sequence (e.g., `\x1b[1m`).
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
                "1" => self.set_attr(ATTR_BOLD, true),
                "2" => self.set_attr(ATTR_DIM, true),
                "3" => self.set_attr(ATTR_ITALIC, true),
                "4" => self.set_attr(ATTR_UNDERLINE, true),
                "22" => {
                    self.set_attr(ATTR_BOLD, false);
                    self.set_attr(ATTR_DIM, false);
                }
                "23" => self.set_attr(ATTR_ITALIC, false),
                "24" => self.set_attr(ATTR_UNDERLINE, false),
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

    /// Returns `true` if any styling is currently active.
    pub fn is_active(&self) -> bool {
        self.attrs != 0 || self.fg.is_some() || self.bg.is_some()
    }

    /// Emit ANSI codes to reproduce the current state.
    ///
    /// Order: bold(1), dim(2), italic(3), underline(4), fg, bg.
    pub fn to_codes(&self) -> String {
        let mut s = String::new();
        if self.has_attr(ATTR_BOLD) {
            s.push_str("\x1b[1m");
        }
        if self.has_attr(ATTR_DIM) {
            s.push_str("\x1b[2m");
        }
        if self.has_attr(ATTR_ITALIC) {
            s.push_str("\x1b[3m");
        }
        if self.has_attr(ATTR_UNDERLINE) {
            s.push_str("\x1b[4m");
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
        return text.width();
    }
    let mut width = 0;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            i += 1;
            if i < bytes.len() && bytes[i] == b']' {
                // OSC sequence: skip to BEL (\x07)
                while i < bytes.len() && bytes[i] != 0x07 {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
            } else {
                // SGR sequence: skip to 'm'
                while i < bytes.len() && bytes[i] != b'm' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
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

/// Break a single line at character boundaries to fit within `max_width` visible columns.
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
    let mut hyperlink: Option<String> = None; // current OSC 8 URL

    for seg in segments {
        if seg.starts_with("\x1b]8;") {
            // OSC 8 hyperlink open/close
            if seg == "\x1b]8;;\x07" {
                hyperlink = None;
            } else {
                hyperlink = Some(seg.to_string());
            }
            current.push_str(seg);
            continue;
        }
        if seg.starts_with('\x1b') {
            state.update(seg);
            current.push_str(seg);
            continue;
        }

        for c in seg.chars() {
            let cw = c.width().unwrap_or(0);
            if width + cw > max_width && width > 0 {
                // Close hyperlink and SGR before wrapping
                if hyperlink.is_some() {
                    current.push_str("\x1b]8;;\x07");
                }
                if state.is_active() {
                    current.push_str("\x1b[0m");
                }
                rows.push(std::mem::take(&mut current));
                // Re-apply SGR and hyperlink on new line
                current = state.to_codes();
                if let Some(ref link) = hyperlink {
                    current.push_str(link);
                }
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
        assert_eq!(strip_ansi("\x1b[38;2;100;200;50mtext\x1b[0m"), "text");
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
    fn split_ansi_no_codes() {
        let parts = split_ansi("plain text");
        assert_eq!(parts, vec!["plain text"]);
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

    #[test]
    fn soft_reset_preserves_bg() {
        let mut state = AnsiState::default();
        // Set bg
        state.update("\x1b[48;2;22;39;27m");
        assert!(state.bg.is_some());
        // Apply SOFT_RESET (22;23;39 = no-bold/dim, no-italic, default-fg)
        state.update("\x1b[22;23;39m");
        assert!(state.bg.is_some(), "SOFT_RESET must preserve bg");
        assert!(!state.is_bold());
        assert!(!state.is_italic());
        assert!(state.fg.is_none());
    }

    #[test]
    fn full_reset_clears_bg() {
        let mut state = AnsiState::default();
        state.update("\x1b[48;2;22;39;27m");
        assert!(state.bg.is_some());
        state.update("\x1b[0m");
        assert!(state.bg.is_none(), "full RESET must clear bg");
    }

    #[test]
    fn wrap_preserves_bg_on_continuation() {
        // Content with bg set, long enough to wrap
        let bg = "\x1b[48;2;22;39;27m";
        let line = format!("{bg}abcdefghij");
        let result = wrap_line_for_display(&line, 5);
        assert_eq!(result.len(), 2);
        // Second row should re-apply the bg
        assert!(
            result[1].contains("48;2;22;39;27"),
            "continuation row must re-apply bg: {:?}",
            result[1]
        );
    }

    // ── OSC 8 hyperlink tests ──────────────────────────────

    #[test]
    fn strip_ansi_removes_osc8() {
        let input = "\x1b]8;;https://example.com\x07click here\x1b]8;;\x07";
        assert_eq!(strip_ansi(input), "click here");
    }

    #[test]
    fn strip_ansi_removes_osc8_with_surrounding_text() {
        let input = "before \x1b]8;;https://x.com\x07link\x1b]8;;\x07 after";
        assert_eq!(strip_ansi(input), "before link after");
    }

    #[test]
    fn visible_width_ignores_osc8() {
        let plain = "click here";
        let with_osc = "\x1b]8;;https://example.com\x07click here\x1b]8;;\x07";
        assert_eq!(visible_width(with_osc), visible_width(plain));
    }

    #[test]
    fn visible_width_osc8_with_sgr() {
        // OSC 8 + bold SGR around text
        let input = "\x1b]8;;https://x.com\x07\x1b[1mtext\x1b[22m\x1b]8;;\x07";
        assert_eq!(visible_width(input), 4);
    }

    #[test]
    fn split_ansi_separates_osc8() {
        let input = "before\x1b]8;;https://x.com\x07link\x1b]8;;\x07after";
        let parts = split_ansi(input);
        assert_eq!(
            parts,
            vec![
                "before",
                "\x1b]8;;https://x.com\x07",
                "link",
                "\x1b]8;;\x07",
                "after",
            ]
        );
    }

    #[test]
    fn wrap_line_for_display_preserves_osc8_no_wrap() {
        let input = "\x1b]8;;https://x.com\x07link\x1b]8;;\x07";
        let result = wrap_line_for_display(input, 80);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("\x1b]8;;https://x.com\x07"));
        assert!(result[0].contains("\x1b]8;;\x07"));
    }

    #[test]
    fn wrap_line_for_display_osc8_across_wrap() {
        // "abcdefghij" is 10 chars, wrap at 5 should split into 2 rows
        // Each row should have the hyperlink open/close
        let input = "\x1b]8;;https://x.com\x07abcdefghij\x1b]8;;\x07";
        let result = wrap_line_for_display(input, 5);
        assert_eq!(result.len(), 2);
        // Both rows should have the OSC 8 open and close
        for (i, row) in result.iter().enumerate() {
            assert!(
                row.contains("\x1b]8;;https://x.com\x07"),
                "row {i} should have OSC 8 open: {row:?}"
            );
            assert!(
                row.contains("\x1b]8;;\x07"),
                "row {i} should have OSC 8 close: {row:?}"
            );
        }
    }

    #[test]
    fn underline_state() {
        let mut state = AnsiState::default();
        // Set underline
        state.update("\x1b[4m");
        assert!(state.is_underline(), "underline should be set after code 4");
        assert!(state.to_codes().contains("\x1b[4m"), "to_codes should include underline");

        // Reset underline
        state.update("\x1b[24m");
        assert!(!state.is_underline(), "underline should be cleared after code 24");
        assert!(!state.to_codes().contains("\x1b[4m"), "to_codes should not include underline after reset");
    }
}
