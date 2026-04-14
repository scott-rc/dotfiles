use tui::highlight::{HighlightLines, SYNTAX_SET, THEME, highlight_line_html};

use crate::git::diff::{DiffFile, LineKind};
use crate::pager::tree::{TreeEntry, build_tree_entries};
use crate::render::compute_hunk_word_ranges;
use crate::style;

use super::protocol::{
    ServerMessage, WebDiffFile, WebDiffHunk, WebDiffLine, WebLineKind, WebTreeEntry,
};

/// Build the full server message from parsed diff files.
pub(crate) fn build_diff_data(files: &[DiffFile]) -> ServerMessage {
    let web_files: Vec<WebDiffFile> = files.iter().map(render_file).collect();
    let tree_entries = build_tree_entries(files);
    let web_tree = tree_entries.iter().map(render_tree_entry).collect();
    ServerMessage::DiffData {
        files: web_files,
        tree: web_tree,
    }
}

fn render_file(file: &DiffFile) -> WebDiffFile {
    let path = file.path().to_string();
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
    let mut hl_state = HighlightLines::new(syntax, &THEME);

    let hunks: Vec<WebDiffHunk> = file
        .hunks
        .iter()
        .map(|hunk| {
            let word_ranges_map = compute_hunk_word_ranges(hunk);

            let lines: Vec<WebDiffLine> = hunk
                .lines
                .iter()
                .enumerate()
                .map(|(i, diff_line)| {
                    let word_ranges = word_ranges_map.get(&i).map_or(&[][..], Vec::as_slice);
                    let syntax_html =
                        highlight_line_html(&diff_line.content, &mut hl_state, &SYNTAX_SET);
                    let content_html = apply_word_highlights_html(
                        &syntax_html,
                        &diff_line.content,
                        word_ranges,
                        diff_line.kind,
                    );

                    WebDiffLine {
                        kind: WebLineKind::from(diff_line.kind),
                        content_html,
                        raw_content: diff_line.content.clone(),
                        old_lineno: diff_line.old_lineno,
                        new_lineno: diff_line.new_lineno,
                        line_idx: i,
                    }
                })
                .collect();

            WebDiffHunk {
                old_start: hunk.old_start,
                new_start: hunk.new_start,
                lines,
            }
        })
        .collect();

    WebDiffFile {
        path,
        old_path: file.old_path.clone(),
        status: file.status.into(),
        hunks,
    }
}

/// Apply word-level highlight spans to HTML-escaped, syntax-highlighted content.
///
/// Word ranges are byte offsets into `raw` (the original uncolored text).
/// We navigate through the HTML (skipping tags) to insert `<mark class="wd-add">`
/// or `<mark class="wd-del">` at the correct visible-character positions.
fn apply_word_highlights_html(
    syntax_html: &str,
    raw: &str,
    word_ranges: &[(usize, usize)],
    kind: LineKind,
) -> String {
    if word_ranges.is_empty() || kind == LineKind::Context {
        return syntax_html.to_string();
    }

    let mark_class = match kind {
        LineKind::Added => "wd-add",
        LineKind::Deleted => "wd-del",
        LineKind::Context => return syntax_html.to_string(),
    };

    // Build a set of raw byte offsets that should be highlighted.
    // We track per-byte whether it's in a highlight range.
    let raw_len = raw.len();
    let mut highlighted = vec![false; raw_len];
    for &(start, end) in word_ranges {
        for i in start..end.min(raw_len) {
            highlighted[i] = true;
        }
    }

    // Walk through the syntax HTML and raw content in parallel.
    // When we encounter text (outside HTML tags), we match it to raw bytes
    // and insert mark tags at highlight boundaries.
    let mut result = String::with_capacity(syntax_html.len() + word_ranges.len() * 40);
    let mut raw_byte_idx = 0;
    let mut in_mark = false;
    let html_bytes = syntax_html.as_bytes();
    let mut i = 0;

    while i < html_bytes.len() {
        if html_bytes[i] == b'<' {
            // HTML tag — copy verbatim until closing >
            if in_mark {
                result.push_str("</mark>");
                in_mark = false;
            }
            let tag_start = i;
            while i < html_bytes.len() && html_bytes[i] != b'>' {
                i += 1;
            }
            if i < html_bytes.len() {
                i += 1; // skip >
            }
            result.push_str(&syntax_html[tag_start..i]);

            // Re-enter mark if we're still in a highlighted region
            if raw_byte_idx < raw_len && highlighted[raw_byte_idx] {
                result.push_str(&format!("<mark class=\"{mark_class}\">"));
                in_mark = true;
            }
        } else {
            // Text content — match against raw bytes
            // Decode the HTML entity or plain char
            let (html_text, html_advance) = decode_html_char(syntax_html, i);
            let raw_advance = html_text.len_utf8();

            let should_highlight = raw_byte_idx < raw_len && highlighted[raw_byte_idx];

            if should_highlight && !in_mark {
                result.push_str(&format!("<mark class=\"{mark_class}\">"));
                in_mark = true;
            } else if !should_highlight && in_mark {
                result.push_str("</mark>");
                in_mark = false;
            }

            result.push_str(&syntax_html[i..i + html_advance]);
            raw_byte_idx += raw_advance;
            i += html_advance;
            continue;
        }
    }

    if in_mark {
        result.push_str("</mark>");
    }

    result
}

/// Decode one character from HTML text, handling entities.
/// Returns (decoded char, bytes consumed in html string).
fn decode_html_char(html: &str, pos: usize) -> (char, usize) {
    let s = &html[pos..];
    if s.starts_with("&amp;") {
        ('&', 5)
    } else if s.starts_with("&lt;") {
        ('<', 4)
    } else if s.starts_with("&gt;") {
        ('>', 4)
    } else if s.starts_with("&quot;") {
        ('"', 6)
    } else {
        let ch = s.chars().next().unwrap_or(' ');
        (ch, ch.len_utf8())
    }
}

fn render_tree_entry(entry: &TreeEntry) -> WebTreeEntry {
    let is_dir = entry.file_idx.is_none();
    let (icon, ansi_color) = if is_dir {
        style::dir_icon(entry.collapsed)
    } else {
        style::file_icon(&entry.label)
    };
    let icon_color = ansi_to_css(ansi_color);

    WebTreeEntry {
        label: entry.label.clone(),
        depth: entry.depth,
        file_idx: entry.file_idx,
        status: entry.status.map(Into::into),
        is_dir,
        collapsed: entry.collapsed,
        icon: icon.to_string(),
        icon_color,
    }
}

/// Convert ANSI RGB escape sequence to CSS rgb() format.
/// Input: "\x1b[38;2;R;G;Bm" (foreground color)
/// Output: "rgb(R, G, B)"
fn ansi_to_css(ansi: &str) -> String {
    // Parse \x1b[38;2;R;G;Bm format
    if ansi.starts_with("\x1b[38;2;") && ansi.ends_with('m') {
        let inner = &ansi[7..ansi.len() - 1]; // Strip "\x1b[38;2;" and "m"
        let parts: Vec<&str> = inner.split(';').collect();
        if parts.len() == 3 {
            return format!("rgb({}, {}, {})", parts[0], parts[1], parts[2]);
        }
    }
    // Fallback: no color (will use default text color)
    String::new()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_to_css_rgb() {
        let ansi = "\x1b[38;2;222;135;53m";
        let css = ansi_to_css(ansi);
        assert_eq!(css, "rgb(222, 135, 53)");
    }

    #[test]
    fn test_ansi_to_css_empty_for_invalid() {
        assert_eq!(ansi_to_css(""), "");
        assert_eq!(ansi_to_css("not ansi"), "");
        assert_eq!(ansi_to_css("\x1b[38;2;"), "");
    }

    #[test]
    fn test_render_tree_entry_uses_nerd_font_icons() {
        let entry = TreeEntry {
            label: "main.rs".to_string(),
            depth: 0,
            file_idx: Some(0),
            status: None,
            collapsed: false,
        };
        let web_entry = render_tree_entry(&entry);
        // Rust icon is U+E7A8
        assert_eq!(web_entry.icon, "\u{e7a8}");
        assert_eq!(web_entry.icon_color, "rgb(222, 135, 53)");
    }

    #[test]
    fn test_render_tree_entry_dir_icons() {
        let collapsed = TreeEntry {
            label: "src".to_string(),
            depth: 0,
            file_idx: None,
            status: None,
            collapsed: true,
        };
        let expanded = TreeEntry {
            label: "src".to_string(),
            depth: 0,
            file_idx: None,
            status: None,
            collapsed: false,
        };
        let web_collapsed = render_tree_entry(&collapsed);
        let web_expanded = render_tree_entry(&expanded);
        // Dir icons should differ based on collapsed state
        assert_ne!(web_collapsed.icon, web_expanded.icon);
    }
}
