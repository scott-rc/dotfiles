use tui::highlight::{highlight_line, SYNTAX_SET, THEME};

pub fn highlight_code(code: &str, lang: Option<&str>, color: bool) -> String {
    if !color {
        return code.to_string();
    }

    let Some(lang) = lang else {
        return code.to_string();
    };

    let Some(syntax) = SYNTAX_SET.find_syntax_by_token(lang) else {
        return code.to_string();
    };

    let mut highlighter = syntect::easy::HighlightLines::new(syntax, &THEME);
    let mut result = String::new();

    for line in syntect::util::LinesWithEndings::from(code) {
        result.push_str(&highlight_line(line, &mut highlighter, &SYNTAX_SET, "\x1b[0m"));
        if line.ends_with('\n') {
            result.push('\n');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wrap::strip_ansi;

    #[test]
    fn known_language_produces_ansi() {
        let result = highlight_code("const x = 1;", Some("js"), true);
        assert!(
            result.contains("\x1b["),
            "expected ANSI codes, got: {result}"
        );
    }

    #[test]
    fn unknown_language_returns_plain() {
        let result = highlight_code("hello", Some("not-a-language"), true);
        assert_eq!(result, "hello");
    }

    #[test]
    fn no_language_returns_plain() {
        let result = highlight_code("hello", None, true);
        assert_eq!(result, "hello");
    }

    #[test]
    fn no_color_returns_plain() {
        let result = highlight_code("const x = 1;", Some("js"), false);
        assert_eq!(result, "const x = 1;");
    }

    #[test]
    fn strip_ansi_preserves_original() {
        let code = "const x = 1;";
        let highlighted = highlight_code(code, Some("js"), true);
        assert_eq!(strip_ansi(&highlighted), code);
    }

    #[test]
    fn empty_input() {
        let result = highlight_code("", Some("js"), true);
        assert_eq!(result, "");
    }

    #[test]
    fn multiline_preserved() {
        let code = "const x = 1;\nconst y = 2;\nconst z = 3;";
        let result = highlight_code(code, Some("js"), true);
        let input_lines = code.split('\n').count();
        let output_lines = result.split('\n').count();
        assert_eq!(input_lines, output_lines, "line count mismatch");
    }

    #[test]
    fn rust_syntax_highlighted() {
        let result = highlight_code("fn main() {}", Some("rust"), true);
        assert!(
            result.contains("\x1b["),
            "Rust code should produce ANSI output, got: {result}"
        );
        assert_eq!(strip_ansi(&result), "fn main() {}");
    }

    #[test]
    fn python_syntax_highlighted() {
        let result = highlight_code("def foo():", Some("python"), true);
        assert!(
            result.contains("\x1b["),
            "Python code should produce ANSI output, got: {result}"
        );
        assert_eq!(strip_ansi(&result), "def foo():");
    }
}
