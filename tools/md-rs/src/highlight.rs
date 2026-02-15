use std::sync::LazyLock;

use syntect::highlighting::{FontStyle, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME: LazyLock<Theme> = LazyLock::new(|| {
    let theme_bytes = include_bytes!("../themes/github-dark.tmTheme");
    ThemeSet::load_from_reader(&mut std::io::Cursor::new(theme_bytes)).unwrap()
});

pub fn highlight_code(code: &str, lang: Option<&str>, color: bool) -> String {
    if !color {
        return code.to_string();
    }

    let Some(lang) = lang else {
        return code.to_string();
    };

    let theme = &*THEME;

    let syntax = match SYNTAX_SET.find_syntax_by_token(lang) {
        Some(s) => s,
        None => return code.to_string(),
    };

    let mut highlighter = syntect::easy::HighlightLines::new(syntax, theme);
    let mut result = String::new();

    for line in syntect::util::LinesWithEndings::from(code) {
        let regions = match highlighter.highlight_line(line, &SYNTAX_SET) {
            Ok(r) => r,
            Err(_) => return code.to_string(),
        };
        for (style, text) in regions {
            let mut codes = Vec::new();

            // Foreground color
            let fg = style.foreground;
            codes.push(format!("38;2;{};{};{}", fg.r, fg.g, fg.b));

            if style.font_style.contains(FontStyle::BOLD) {
                codes.push("1".to_string());
            }
            if style.font_style.contains(FontStyle::ITALIC) {
                codes.push("3".to_string());
            }
            if style.font_style.contains(FontStyle::UNDERLINE) {
                codes.push("4".to_string());
            }

            result.push_str(&format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text));
        }
    }

    result
}
