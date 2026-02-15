use crate::wrap::strip_ansi;
use regex::Regex;
use std::sync::LazyLock;

// GitHub Dark Default palette
const HEADING_BLUE: u32 = 0x79c0ff;
const FOREGROUND: u32 = 0xe6edf3;
const CODE_ORANGE: u32 = 0xffa657;
const QUOTE_GREEN: u32 = 0x7ee787;
const LINK_BLUE: u32 = 0x79c0ff;
const LIST_BLUE: u32 = 0x79c0ff;
const COMMENT_GRAY: u32 = 0x8b949e;

static ANSI_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*m").unwrap());

fn rgb24(s: &str, color: u32) -> String {
    let r = (color >> 16) & 0xff;
    let g = (color >> 8) & 0xff;
    let b = color & 0xff;
    format!("\x1b[38;2;{r};{g};{b}m{s}\x1b[39m")
}

fn bold(s: &str) -> String {
    format!("\x1b[1m{s}\x1b[22m")
}

fn italic(s: &str) -> String {
    format!("\x1b[3m{s}\x1b[23m")
}

fn underline(s: &str) -> String {
    format!("\x1b[4m{s}\x1b[24m")
}

/// Uppercase visible text while preserving ANSI codes.
fn ansi_upper_case(s: &str) -> String {
    let mut result = String::new();
    let mut last = 0;
    for m in ANSI_RE.find_iter(s) {
        if m.start() > last {
            result.push_str(&s[last..m.start()].to_uppercase());
        }
        result.push_str(m.as_str());
        last = m.end();
    }
    if last < s.len() {
        result.push_str(&s[last..].to_uppercase());
    }
    result
}

/// Style configuration. When `color` is false, all methods return unstyled text.
pub struct Style {
    pub color: bool,
}

impl Style {
    pub fn new(color: bool) -> Self {
        Self { color }
    }

    pub fn h1(&self, s: &str) -> String {
        if self.color {
            bold(&rgb24(&ansi_upper_case(s), HEADING_BLUE))
        } else {
            strip_ansi(s).to_uppercase()
        }
    }

    pub fn h2(&self, s: &str) -> String {
        if self.color {
            bold(&rgb24(s, HEADING_BLUE))
        } else {
            s.to_string()
        }
    }

    pub fn h3(&self, s: &str) -> String {
        if self.color {
            bold(&rgb24(s, HEADING_BLUE))
        } else {
            s.to_string()
        }
    }

    pub fn h4(&self, s: &str) -> String {
        if self.color {
            rgb24(s, HEADING_BLUE)
        } else {
            s.to_string()
        }
    }

    pub fn h5(&self, s: &str) -> String {
        if self.color {
            rgb24(s, HEADING_BLUE)
        } else {
            s.to_string()
        }
    }

    pub fn h6(&self, s: &str) -> String {
        if self.color {
            rgb24(s, HEADING_BLUE)
        } else {
            s.to_string()
        }
    }

    pub fn marker(&self, s: &str) -> String {
        if self.color {
            rgb24(s, COMMENT_GRAY)
        } else {
            s.to_string()
        }
    }

    pub fn list_marker(&self, s: &str) -> String {
        if self.color {
            rgb24(s, LIST_BLUE)
        } else {
            s.to_string()
        }
    }

    pub fn strong_style(&self, s: &str) -> String {
        if self.color {
            bold(&rgb24(s, FOREGROUND))
        } else {
            s.to_string()
        }
    }

    pub fn em_style(&self, s: &str) -> String {
        if self.color {
            italic(&rgb24(s, FOREGROUND))
        } else {
            s.to_string()
        }
    }

    pub fn code_span(&self, s: &str) -> String {
        if self.color {
            format!(
                "{}{}{}",
                rgb24("`", COMMENT_GRAY),
                rgb24(s, CODE_ORANGE),
                rgb24("`", COMMENT_GRAY)
            )
        } else {
            format!("`{s}`")
        }
    }

    pub fn code_language(&self, s: &str) -> String {
        if self.color {
            italic(&rgb24(s, COMMENT_GRAY))
        } else {
            s.to_string()
        }
    }

    pub fn link_text(&self, s: &str) -> String {
        if self.color {
            underline(&rgb24(s, FOREGROUND))
        } else {
            s.to_string()
        }
    }

    pub fn link_url(&self, s: &str) -> String {
        if self.color {
            italic(&underline(&rgb24(s, LINK_BLUE)))
        } else {
            s.to_string()
        }
    }

    pub fn blockquote_text(&self, s: &str) -> String {
        if self.color {
            rgb24(s, QUOTE_GREEN)
        } else {
            s.to_string()
        }
    }

    pub fn hr_style(&self, s: &str) -> String {
        if self.color {
            rgb24(s, COMMENT_GRAY)
        } else {
            s.to_string()
        }
    }

    pub fn frontmatter_key(&self, s: &str) -> String {
        if self.color {
            rgb24(s, COMMENT_GRAY)
        } else {
            s.to_string()
        }
    }

    pub fn frontmatter_value(&self, s: &str) -> String {
        if self.color {
            rgb24(s, FOREGROUND)
        } else {
            s.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wrap::strip_ansi;
    use std::collections::HashMap;

    fn load_palette() -> HashMap<String, String> {
        let json = include_str!("../fixtures/style/palette.json");
        serde_json::from_str(json).unwrap()
    }

    fn parse_hex(s: &str) -> u32 {
        u32::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
    }

    fn rgb24_sequence(hex: u32) -> String {
        let r = (hex >> 16) & 0xff;
        let g = (hex >> 8) & 0xff;
        let b = hex & 0xff;
        format!("\x1b[38;2;{r};{g};{b}m")
    }

    #[test]
    fn test_h1_bold_blue_uppercase() {
        let palette = load_palette();
        let blue = parse_hex(&palette["HEADING_BLUE"]);
        let style = Style::new(true);
        let result = style.h1("hello");
        assert!(
            result.contains(&rgb24_sequence(blue)),
            "h1 should contain HEADING_BLUE"
        );
        assert!(result.contains("\x1b[1m"), "h1 should contain bold");
        assert_eq!(strip_ansi(&result), "HELLO");
    }

    #[test]
    fn test_h1_preserves_ansi_while_uppercasing() {
        let style = Style::new(true);
        let inner = "\x1b[31mred\x1b[0m text";
        let result = style.h1(inner);
        assert_eq!(strip_ansi(&result), "RED TEXT");
    }

    #[test]
    fn test_h2_bold_blue_no_uppercase() {
        let palette = load_palette();
        let blue = parse_hex(&palette["HEADING_BLUE"]);
        let style = Style::new(true);
        let result = style.h2("hello");
        assert!(result.contains(&rgb24_sequence(blue)));
        assert!(result.contains("\x1b[1m"));
        assert_eq!(strip_ansi(&result), "hello");
    }

    #[test]
    fn test_h3_bold_blue() {
        let palette = load_palette();
        let blue = parse_hex(&palette["HEADING_BLUE"]);
        let style = Style::new(true);
        let result = style.h3("sub");
        assert!(result.contains(&rgb24_sequence(blue)));
        assert!(result.contains("\x1b[1m"));
    }

    #[test]
    fn test_h4_blue_no_bold() {
        let palette = load_palette();
        let blue = parse_hex(&palette["HEADING_BLUE"]);
        let style = Style::new(true);
        let result = style.h4("sub");
        assert!(result.contains(&rgb24_sequence(blue)));
        assert!(!result.contains("\x1b[1m"));
    }

    #[test]
    fn test_h5_blue_no_bold() {
        let palette = load_palette();
        let blue = parse_hex(&palette["HEADING_BLUE"]);
        let style = Style::new(true);
        let result = style.h5("sub");
        assert!(result.contains(&rgb24_sequence(blue)));
        assert!(!result.contains("\x1b[1m"));
    }

    #[test]
    fn test_h6_blue_no_bold() {
        let palette = load_palette();
        let blue = parse_hex(&palette["HEADING_BLUE"]);
        let style = Style::new(true);
        let result = style.h6("sub");
        assert!(result.contains(&rgb24_sequence(blue)));
        assert!(!result.contains("\x1b[1m"));
    }

    #[test]
    fn test_marker_comment_gray() {
        let palette = load_palette();
        let gray = parse_hex(&palette["COMMENT_GRAY"]);
        let style = Style::new(true);
        let result = style.marker("#");
        assert!(result.contains(&rgb24_sequence(gray)));
        assert_eq!(strip_ansi(&result), "#");
    }

    #[test]
    fn test_list_marker_blue() {
        let palette = load_palette();
        let blue = parse_hex(&palette["LIST_BLUE"]);
        let style = Style::new(true);
        let result = style.list_marker("-");
        assert!(result.contains(&rgb24_sequence(blue)));
        assert_eq!(strip_ansi(&result), "-");
    }

    #[test]
    fn test_strong_style_bold_foreground() {
        let palette = load_palette();
        let fg = parse_hex(&palette["FOREGROUND"]);
        let style = Style::new(true);
        let result = style.strong_style("bold");
        assert!(result.contains(&rgb24_sequence(fg)));
        assert!(result.contains("\x1b[1m"));
        assert_eq!(strip_ansi(&result), "bold");
    }

    #[test]
    fn test_em_style_italic_foreground() {
        let palette = load_palette();
        let fg = parse_hex(&palette["FOREGROUND"]);
        let style = Style::new(true);
        let result = style.em_style("italic");
        assert!(result.contains(&rgb24_sequence(fg)));
        assert!(result.contains("\x1b[3m"));
        assert_eq!(strip_ansi(&result), "italic");
    }

    #[test]
    fn test_code_span_gray_backticks_orange_content() {
        let palette = load_palette();
        let gray = parse_hex(&palette["COMMENT_GRAY"]);
        let orange = parse_hex(&palette["CODE_ORANGE"]);
        let style = Style::new(true);
        let result = style.code_span("foo");
        assert!(result.contains(&rgb24_sequence(gray)));
        assert!(result.contains(&rgb24_sequence(orange)));
        assert_eq!(strip_ansi(&result), "`foo`");
    }

    #[test]
    fn test_code_language_italic_gray() {
        let palette = load_palette();
        let gray = parse_hex(&palette["COMMENT_GRAY"]);
        let style = Style::new(true);
        let result = style.code_language("typescript");
        assert!(result.contains(&rgb24_sequence(gray)));
        assert!(result.contains("\x1b[3m"));
        assert_eq!(strip_ansi(&result), "typescript");
    }

    #[test]
    fn test_link_text_underline_foreground() {
        let palette = load_palette();
        let fg = parse_hex(&palette["FOREGROUND"]);
        let style = Style::new(true);
        let result = style.link_text("example");
        assert!(result.contains(&rgb24_sequence(fg)));
        assert!(result.contains("\x1b[4m"));
        assert_eq!(strip_ansi(&result), "example");
    }

    #[test]
    fn test_link_url_italic_underline_blue() {
        let palette = load_palette();
        let blue = parse_hex(&palette["LINK_BLUE"]);
        let style = Style::new(true);
        let result = style.link_url("https://example.com");
        assert!(result.contains(&rgb24_sequence(blue)));
        assert!(result.contains("\x1b[3m"));
        assert!(result.contains("\x1b[4m"));
        assert_eq!(strip_ansi(&result), "https://example.com");
    }

    #[test]
    fn test_blockquote_text_green() {
        let palette = load_palette();
        let green = parse_hex(&palette["QUOTE_GREEN"]);
        let style = Style::new(true);
        let result = style.blockquote_text("quoted");
        assert!(result.contains(&rgb24_sequence(green)));
        assert_eq!(strip_ansi(&result), "quoted");
    }

    #[test]
    fn test_hr_style_gray() {
        let palette = load_palette();
        let gray = parse_hex(&palette["COMMENT_GRAY"]);
        let style = Style::new(true);
        let result = style.hr_style("---");
        assert!(result.contains(&rgb24_sequence(gray)));
        assert_eq!(strip_ansi(&result), "---");
    }

    #[test]
    fn test_frontmatter_key_gray() {
        let palette = load_palette();
        let gray = parse_hex(&palette["COMMENT_GRAY"]);
        let style = Style::new(true);
        let result = style.frontmatter_key("title");
        assert!(result.contains(&rgb24_sequence(gray)));
        assert_eq!(strip_ansi(&result), "title");
    }

    #[test]
    fn test_frontmatter_value_foreground() {
        let palette = load_palette();
        let fg = parse_hex(&palette["FOREGROUND"]);
        let style = Style::new(true);
        let result = style.frontmatter_value("My Doc");
        assert!(result.contains(&rgb24_sequence(fg)));
        assert_eq!(strip_ansi(&result), "My Doc");
    }

    #[test]
    fn test_no_color_mode() {
        let style = Style::new(false);
        assert_eq!(style.h1("hello"), "HELLO");
        assert_eq!(style.h2("hello"), "hello");
        assert_eq!(style.marker("#"), "#");
        assert_eq!(style.code_span("foo"), "`foo`");
    }
}
