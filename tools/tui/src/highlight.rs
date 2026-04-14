use std::fmt::Write;
use std::sync::LazyLock;

use syntect::highlighting::{FontStyle, Theme, ThemeSet};
use syntect::parsing::{ScopeStack, SyntaxSet};

// Re-export syntect types so consumers don't need a direct syntect dependency.
pub use syntect::easy::HighlightLines;
pub use syntect::highlighting::{HighlightState, Theme as SyntectTheme};
pub use syntect::parsing::{ParseState, SyntaxReference, SyntaxSet as SyntectSyntaxSet};

pub static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_newlines);

pub static THEME_DARK: LazyLock<Theme> = LazyLock::new(|| {
    let theme_bytes = include_bytes!("../themes/github-dark.tmTheme");
    ThemeSet::load_from_reader(&mut std::io::Cursor::new(theme_bytes)).unwrap()
});

pub static THEME_LIGHT: LazyLock<Theme> = LazyLock::new(|| {
    let theme_bytes = include_bytes!("../themes/github-light.tmTheme");
    ThemeSet::load_from_reader(&mut std::io::Cursor::new(theme_bytes)).unwrap()
});

/// Backward-compatible alias for the dark theme.
pub static THEME: LazyLock<Theme> = LazyLock::new(|| {
    let theme_bytes = include_bytes!("../themes/github-dark.tmTheme");
    ThemeSet::load_from_reader(&mut std::io::Cursor::new(theme_bytes)).unwrap()
});

/// Theme variant for syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeVariant {
    #[default]
    Dark,
    Light,
}

/// Get the theme for the given variant.
pub fn theme(variant: ThemeVariant) -> &'static Theme {
    match variant {
        ThemeVariant::Dark => &THEME_DARK,
        ThemeVariant::Light => &THEME_LIGHT,
    }
}

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

/// Syntax-highlight a single line using syntect, returning HTML with inline styles.
///
/// Each token becomes `<span style="color:rgb(R,G,B);font-weight:bold">text</span>`.
/// Text content is HTML-escaped (`<`, `>`, `&`, `"`).
pub fn highlight_line_html(
    line: &str,
    hl: &mut syntect::easy::HighlightLines,
    ss: &SyntaxSet,
) -> String {
    let mut input = String::with_capacity(line.len() + 1);
    input.push_str(line);
    input.push('\n');
    let regions = hl.highlight_line(&input, ss).unwrap_or_default();
    let mut out = String::new();

    for (style, text) in &regions {
        let text = text.trim_end_matches('\n');
        if text.is_empty() {
            continue;
        }

        let fg = style.foreground;
        let bold = style.font_style.contains(FontStyle::BOLD);
        let italic = style.font_style.contains(FontStyle::ITALIC);

        let mut css = format!("color:rgb({},{},{})", fg.r, fg.g, fg.b);
        if bold {
            css.push_str(";font-weight:bold");
        }
        if italic {
            css.push_str(";font-style:italic");
        }

        let _ = write!(out, "<span style=\"{css}\">");
        html_escape_into(&mut out, text);
        out.push_str("</span>");
    }

    out
}

/// HTML-escape text into an existing buffer.
pub fn html_escape_into(buf: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => buf.push_str("&amp;"),
            '<' => buf.push_str("&lt;"),
            '>' => buf.push_str("&gt;"),
            '"' => buf.push_str("&quot;"),
            _ => buf.push(ch),
        }
    }
}

/// Map a syntect scope stack to a semantic CSS class name.
///
/// This enables theme-switching via CSS variables instead of inline RGB values.
pub fn scope_to_class(scope: &ScopeStack) -> &'static str {
    let Some(last_scope) = scope.as_slice().last() else {
        return "syn-default";
    };
    let scope_str = last_scope.build_string();

    // Order matters: more specific matches first
    if scope_str.starts_with("comment") {
        return "syn-comment";
    }
    if scope_str.starts_with("string") {
        return "syn-string";
    }
    if scope_str.starts_with("keyword.operator") {
        return "syn-operator";
    }
    if scope_str.starts_with("keyword") {
        return "syn-keyword";
    }
    if scope_str.starts_with("storage") {
        return "syn-keyword";
    }
    if scope_str.starts_with("constant.numeric") {
        return "syn-number";
    }
    if scope_str.starts_with("constant.character") {
        return "syn-constant";
    }
    if scope_str.starts_with("constant") {
        return "syn-constant";
    }
    if scope_str.starts_with("entity.name.function")
        || scope_str.starts_with("variable.function")
        || scope_str.starts_with("support.function")
        || scope_str.starts_with("meta.function-call")
    {
        return "syn-function";
    }
    if scope_str.starts_with("entity.name.type")
        || scope_str.starts_with("entity.name.class")
        || scope_str.starts_with("support.type")
        || scope_str.starts_with("support.class")
    {
        return "syn-type";
    }
    if scope_str.starts_with("entity.name.tag") {
        return "syn-tag";
    }
    if scope_str.starts_with("entity.other.attribute-name") {
        return "syn-attribute";
    }
    if scope_str.starts_with("variable") {
        return "syn-variable";
    }
    if scope_str.starts_with("punctuation") {
        return "syn-punctuation";
    }
    "syn-default"
}

/// Syntax-highlight a single line using syntect, returning HTML with CSS classes.
///
/// Each token becomes `<span class="syn-keyword">text</span>`.
/// Text content is HTML-escaped (`<`, `>`, `&`, `"`).
/// This enables instant theme switching via CSS variables.
pub fn highlight_line_html_classes(
    line: &str,
    parse_state: &mut ParseState,
    ss: &SyntaxSet,
) -> String {
    let ops = parse_state.parse_line(line, ss).unwrap_or_default();
    let mut out = String::with_capacity(line.len() * 2);
    let mut scope_stack = ScopeStack::new();
    let mut pos = 0;

    for (end, op) in ops {
        let text = &line[pos..end];
        if !text.is_empty() {
            let class = scope_to_class(&scope_stack);
            let _ = write!(out, "<span class=\"{class}\">");
            html_escape_into(&mut out, text);
            out.push_str("</span>");
        }
        scope_stack.apply(&op).ok();
        pos = end;
    }

    // Handle any remaining text after the last operation
    if pos < line.len() {
        let text = &line[pos..];
        let class = scope_to_class(&scope_stack);
        let _ = write!(out, "<span class=\"{class}\">");
        html_escape_into(&mut out, text);
        out.push_str("</span>");
    }

    out
}

/// HTML-escape a string, returning a new `String`.
pub fn html_escape(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    html_escape_into(&mut buf, s);
    buf
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
    fn test_dual_themes_load() {
        assert!(THEME_DARK.settings.foreground.is_some());
        assert!(THEME_LIGHT.settings.foreground.is_some());
        // Themes should have different backgrounds
        let dark_bg = THEME_DARK.settings.background.unwrap();
        let light_bg = THEME_LIGHT.settings.background.unwrap();
        assert_ne!(dark_bg, light_bg);
    }

    #[test]
    fn test_theme_variant_selection() {
        let dark = theme(ThemeVariant::Dark);
        let light = theme(ThemeVariant::Light);
        assert_eq!(dark.settings.background, THEME_DARK.settings.background);
        assert_eq!(light.settings.background, THEME_LIGHT.settings.background);
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

    #[test]
    fn test_highlight_line_html_classes_produces_classes() {
        let syntax = SYNTAX_SET.find_syntax_by_extension("rs").unwrap();
        let mut parse_state = ParseState::new(syntax);
        let result = highlight_line_html_classes("let x = 42;", &mut parse_state, &SYNTAX_SET);
        // Should contain CSS class spans
        assert!(
            result.contains("class=\"syn-"),
            "expected CSS classes, got: {result}"
        );
        // Should NOT contain inline styles
        assert!(
            !result.contains("style="),
            "should not have inline styles, got: {result}"
        );
    }

    #[test]
    fn test_highlight_line_html_classes_escapes_html() {
        let syntax = SYNTAX_SET.find_syntax_by_extension("rs").unwrap();
        let mut parse_state = ParseState::new(syntax);
        let result =
            highlight_line_html_classes("let x = \"<script>\";", &mut parse_state, &SYNTAX_SET);
        assert!(
            result.contains("&lt;script&gt;"),
            "should escape HTML, got: {result}"
        );
    }
}
