use std::fmt::Write;
use std::sync::LazyLock;

use syntect::highlighting::{FontStyle, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

// Re-export syntect types so consumers don't need a direct syntect dependency.
pub use syntect::easy::HighlightLines;
pub use syntect::highlighting::Theme as SyntectTheme;
pub use syntect::parsing::{SyntaxReference, SyntaxSet as SyntectSyntaxSet};

pub static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_newlines);
pub static THEME: LazyLock<Theme> = LazyLock::new(|| {
    let theme_bytes = include_bytes!("../themes/github-dark.tmTheme");
    ThemeSet::load_from_reader(&mut std::io::Cursor::new(theme_bytes)).unwrap()
});

/// Syntax-highlight a single line using syntect, returning ANSI-colored text.
///
/// The caller-supplied `reset` code is appended after each token. Use `"\x1b[0m"`
/// for a full reset (md) or `"\x1b[22;23;39m"` for a soft reset that preserves
/// background colors (gd).
pub fn highlight_line(
    line: &str,
    hl: &mut syntect::easy::HighlightLines,
    ss: &SyntaxSet,
    reset: &str,
) -> String {
    let regions = hl.highlight_line(line, ss).unwrap_or_default();
    let mut out = String::new();

    for (style, text) in &regions {
        let text = text.trim_end_matches('\n');
        if text.is_empty() {
            continue;
        }

        let fg = style.foreground;
        if style.font_style.contains(FontStyle::BOLD) {
            out.push_str("\x1b[1m");
        }
        if style.font_style.contains(FontStyle::ITALIC) {
            out.push_str("\x1b[3m");
        }
        let _ = write!(out, "\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b);
        out.push_str(text);
        out.push_str(reset);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_set_loads() {
        assert!(SYNTAX_SET.find_syntax_by_extension("rs").is_some());
    }

    #[test]
    fn test_theme_loads() {
        assert!(THEME.settings.foreground.is_some());
    }

    #[test]
    fn test_highlight_line_produces_ansi() {
        let syntax = SYNTAX_SET.find_syntax_by_extension("rs").unwrap();
        let mut hl = syntect::easy::HighlightLines::new(syntax, &THEME);
        let result = highlight_line("let x = 42;\n", &mut hl, &SYNTAX_SET, "\x1b[0m");
        assert!(
            result.contains("\x1b["),
            "expected ANSI codes, got: {result}"
        );
    }

    #[test]
    fn test_highlight_line_with_soft_reset() {
        let syntax = SYNTAX_SET.find_syntax_by_extension("rs").unwrap();
        let mut hl = syntect::easy::HighlightLines::new(syntax, &THEME);
        let result = highlight_line("let x = 42;\n", &mut hl, &SYNTAX_SET, "\x1b[22;23;39m");
        assert!(
            result.contains("\x1b[22;23;39m"),
            "expected soft reset in output, got: {result}"
        );
        assert!(
            !result.contains("\x1b[0m"),
            "should not contain full reset when using soft reset, got: {result}"
        );
    }
}
