use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

pub fn highlight_code(code: &str, lang: Option<&str>, color: bool) -> String {
    if !color {
        return code.to_string();
    }

    let Some(lang) = lang else {
        return code.to_string();
    };

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let syntax = match ss.find_syntax_by_token(lang) {
        Some(s) => s,
        None => return code.to_string(),
    };

    let mut highlighter = syntect::easy::HighlightLines::new(syntax, theme);
    let mut result = String::new();

    for line in syntect::util::LinesWithEndings::from(code) {
        let regions = match highlighter.highlight_line(line, &ss) {
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
