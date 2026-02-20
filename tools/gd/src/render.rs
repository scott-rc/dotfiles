use std::fmt::Write as _;

use similar::TextDiff;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

use crate::git::diff::{DiffFile, DiffHunk, FileStatus, LineKind};
use crate::style;

/// Per-rendered-line metadata for the pager.
#[derive(Debug, Clone)]
pub struct LineInfo {
    pub file_idx: usize,
    pub path: String,
    /// Source line number in the new file (for editor jump), if applicable.
    pub new_lineno: Option<u32>,
}

pub struct RenderOutput {
    pub lines: Vec<String>,
    pub line_map: Vec<LineInfo>,
    pub file_starts: Vec<usize>,
    pub hunk_starts: Vec<usize>,
}

pub fn render(files: &[DiffFile], width: usize, color: bool) -> RenderOutput {
    let ss = SyntaxSet::load_defaults_newlines();
    let theme_bytes = include_bytes!("../themes/github-dark.tmTheme");
    let theme = ThemeSet::load_from_reader(&mut std::io::Cursor::new(theme_bytes))
        .unwrap_or_else(|_| ThemeSet::load_defaults().themes["base16-ocean.dark"].clone());

    let mut lines = Vec::new();
    let mut line_map = Vec::new();
    let mut file_starts = Vec::new();
    let mut hunk_starts = Vec::new();

    for (file_idx, file) in files.iter().enumerate() {
        file_starts.push(lines.len());
        let path = file.path();
        let status_label = match file.status {
            FileStatus::Modified => "Modified",
            FileStatus::Added => "Added",
            FileStatus::Deleted => "Deleted",
            FileStatus::Renamed => "Renamed",
        };

        // File header
        let header = if color {
            style::file_header(path, status_label, width)
        } else {
            let label = format!(" {path} ({status_label}) ");
            let bar_len = width.saturating_sub(2 + label.len());
            format!(
                "{}{}{}",
                "\u{2500}".repeat(2),
                label,
                "\u{2500}".repeat(bar_len)
            )
        };
        lines.push(header);
        line_map.push(LineInfo {
            file_idx,
            path: path.to_string(),
            new_lineno: None,
        });

        // Syntax highlighter for this file's extension
        let syntax = ss
            .find_syntax_by_extension(
                std::path::Path::new(path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or(""),
            )
            .unwrap_or_else(|| ss.find_syntax_plain_text());
        let theme_ref = &theme;

        for hunk in &file.hunks {
            hunk_starts.push(lines.len());

            // Hunk header (@@ ... @@)
            let hunk_text = format!(
                "@@ -{},{} +{},{} @@",
                hunk.old_start,
                hunk.lines.iter().filter(|l| l.kind != LineKind::Added).count(),
                hunk.new_start,
                hunk.lines.iter().filter(|l| l.kind != LineKind::Deleted).count(),
            );
            let hunk_line = if color {
                style::hunk_header(&hunk_text)
            } else {
                hunk_text
            };
            lines.push(hunk_line);
            line_map.push(LineInfo {
                file_idx,
                path: path.to_string(),
                new_lineno: None,
            });

            // Render diff lines with word-level highlights
            render_hunk_lines(
                hunk,
                file_idx,
                path,
                syntax,
                &ss,
                theme_ref,
                color,
                width,
                &mut lines,
                &mut line_map,
            );
        }

        // Blank line between files
        if file_idx + 1 < files.len() {
            lines.push(String::new());
            line_map.push(LineInfo {
                file_idx,
                path: path.to_string(),
                new_lineno: None,
            });
        }
    }

    RenderOutput {
        lines,
        line_map,
        file_starts,
        hunk_starts,
    }
}

/// Group consecutive added/deleted lines into change blocks for word-level diffing.
struct ChangeBlock {
    deleted: Vec<usize>,
    added: Vec<usize>,
}

fn find_change_blocks(hunk: &DiffHunk) -> Vec<ChangeBlock> {
    let mut blocks = Vec::new();
    let mut i = 0;
    let hunk_lines = &hunk.lines;

    while i < hunk_lines.len() {
        if hunk_lines[i].kind == LineKind::Deleted {
            let mut deleted = Vec::new();
            while i < hunk_lines.len() && hunk_lines[i].kind == LineKind::Deleted {
                deleted.push(i);
                i += 1;
            }
            let mut added = Vec::new();
            while i < hunk_lines.len() && hunk_lines[i].kind == LineKind::Added {
                added.push(i);
                i += 1;
            }
            if !added.is_empty() {
                blocks.push(ChangeBlock { deleted, added });
            } else {
                // Pure deletions — no word-level diff
                blocks.push(ChangeBlock {
                    deleted,
                    added: Vec::new(),
                });
            }
        } else if hunk_lines[i].kind == LineKind::Added {
            let mut added = Vec::new();
            while i < hunk_lines.len() && hunk_lines[i].kind == LineKind::Added {
                added.push(i);
                i += 1;
            }
            blocks.push(ChangeBlock {
                deleted: Vec::new(),
                added,
            });
        } else {
            i += 1;
        }
    }

    blocks
}

/// For a change block, compute per-line word highlight ranges.
/// Returns (deleted_highlights, added_highlights) — each a Vec<Vec<(start, end)>> per line.
fn word_highlights(
    hunk: &DiffHunk,
    block: &ChangeBlock,
) -> (Vec<Vec<(usize, usize)>>, Vec<Vec<(usize, usize)>>) {
    if block.deleted.is_empty() || block.added.is_empty() {
        return (
            vec![Vec::new(); block.deleted.len()],
            vec![Vec::new(); block.added.len()],
        );
    }

    let old_text: String = block
        .deleted
        .iter()
        .map(|&i| hunk.lines[i].content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let new_text: String = block
        .added
        .iter()
        .map(|&i| hunk.lines[i].content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let diff = TextDiff::from_words(&old_text, &new_text);

    let mut del_highlights: Vec<Vec<(usize, usize)>> = vec![Vec::new(); block.deleted.len()];
    let mut add_highlights: Vec<Vec<(usize, usize)>> = vec![Vec::new(); block.added.len()];

    // Track positions in old/new text to map back to per-line ranges
    let mut old_pos = 0;
    let mut new_pos = 0;

    for change in diff.iter_all_changes() {
        let value = change.value();
        match change.tag() {
            similar::ChangeTag::Equal => {
                old_pos += value.len();
                new_pos += value.len();
            }
            similar::ChangeTag::Delete => {
                add_ranges_to_lines(
                    old_pos,
                    value.len(),
                    &block.deleted,
                    hunk,
                    &mut del_highlights,
                );
                old_pos += value.len();
            }
            similar::ChangeTag::Insert => {
                add_ranges_to_lines(
                    new_pos,
                    value.len(),
                    &block.added,
                    hunk,
                    &mut add_highlights,
                );
                new_pos += value.len();
            }
        }
    }

    (del_highlights, add_highlights)
}

/// Map a range in a concatenated multi-line string back to per-line highlight ranges.
fn add_ranges_to_lines(
    start: usize,
    len: usize,
    line_indices: &[usize],
    hunk: &DiffHunk,
    highlights: &mut [Vec<(usize, usize)>],
) {
    let end = start + len;
    let mut offset = 0;

    for (idx, &line_idx) in line_indices.iter().enumerate() {
        let line_len = hunk.lines[line_idx].content.len();
        let line_end = offset + line_len;

        // The "\n" between joined lines
        let sep_end = if idx + 1 < line_indices.len() {
            line_end + 1
        } else {
            line_end
        };

        if start < sep_end && end > offset {
            let local_start = start.saturating_sub(offset);
            let local_end = end.min(line_end) - offset;
            if local_start < local_end {
                highlights[idx].push((local_start, local_end));
            }
        }

        offset = line_end + 1; // +1 for "\n"
    }
}

#[allow(clippy::too_many_arguments)]
fn render_hunk_lines(
    hunk: &DiffHunk,
    file_idx: usize,
    path: &str,
    syntax: &syntect::parsing::SyntaxReference,
    ss: &SyntaxSet,
    theme: &syntect::highlighting::Theme,
    color: bool,
    width: usize,
    lines: &mut Vec<String>,
    line_map: &mut Vec<LineInfo>,
) {
    let blocks = find_change_blocks(hunk);

    // Build a map of line_index → (highlight_ranges) for deleted and added lines
    let mut del_word_hl: Vec<Vec<(usize, usize)>> = vec![Vec::new(); hunk.lines.len()];
    let mut add_word_hl: Vec<Vec<(usize, usize)>> = vec![Vec::new(); hunk.lines.len()];

    for block in &blocks {
        let (del_hl, add_hl) = word_highlights(hunk, block);
        for (idx, ranges) in block.deleted.iter().zip(del_hl) {
            del_word_hl[*idx] = ranges;
        }
        for (idx, ranges) in block.added.iter().zip(add_hl) {
            add_word_hl[*idx] = ranges;
        }
    }

    // Syntax highlighter state (applied in order for best results)
    let mut hl_state = syntect::easy::HighlightLines::new(syntax, theme);

    for (i, diff_line) in hunk.lines.iter().enumerate() {
        let gutter = if color {
            style::gutter(diff_line.old_lineno, diff_line.new_lineno)
        } else {
            let old = diff_line
                .old_lineno
                .map_or(String::new(), |n| format!("{n}"));
            let new = diff_line
                .new_lineno
                .map_or(String::new(), |n| format!("{n}"));
            format!("{old:>4} |{new:>4} |")
        };

        let (marker, line_bg, word_bg, marker_color) = match diff_line.kind {
            LineKind::Added => ("+", style::BG_ADDED, style::BG_ADDED_WORD, style::FG_ADDED_MARKER),
            LineKind::Deleted => ("-", style::BG_DELETED, style::BG_DELETED_WORD, style::FG_DELETED_MARKER),
            LineKind::Context => (" ", "", "", ""),
        };

        let empty = Vec::new();
        let word_ranges = match diff_line.kind {
            LineKind::Deleted => &del_word_hl[i],
            LineKind::Added => &add_word_hl[i],
            LineKind::Context => &empty,
        };

        let content = &diff_line.content;

        // Build the content portion with syntax + diff coloring
        let styled_content = if color {
            let syntax_colored = syntax_highlight_line(
                &format!("{content}\n"),
                &mut hl_state,
                ss,
            );
            apply_diff_colors(
                &syntax_colored,
                content,
                line_bg,
                word_bg,
                word_ranges,
                diff_line.kind != LineKind::Context,
            )
        } else {
            content.clone()
        };

        // Pad to terminal width for full-line background
        let content_width = crate::ansi::visible_width(&styled_content);
        let avail = width.saturating_sub(style::GUTTER_WIDTH + 1); // +1 for marker
        let padding = if color && diff_line.kind != LineKind::Context && content_width < avail {
            " ".repeat(avail - content_width)
        } else {
            String::new()
        };

        let reset = style::RESET;
        let marker_styled = if color && !marker_color.is_empty() {
            format!("{marker_color}{marker}{reset}")
        } else {
            marker.to_string()
        };

        let line = if color && diff_line.kind != LineKind::Context {
            format!(
                "{gutter}{line_bg}{marker_styled}{styled_content}{padding}{reset}"
            )
        } else {
            format!("{gutter}{marker_styled}{styled_content}")
        };

        lines.push(line);
        line_map.push(LineInfo {
            file_idx,
            path: path.to_string(),
            new_lineno: diff_line.new_lineno,
        });
    }
}

/// Syntax-highlight a single line using syntect, returning ANSI-colored text.
fn syntax_highlight_line(
    line: &str,
    hl: &mut syntect::easy::HighlightLines,
    ss: &SyntaxSet,
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
        out.push_str(style::RESET);
    }

    out
}

/// Apply diff background colors and word-level highlights to syntax-colored text.
/// The `raw` parameter is the original uncolored content used for word range mapping.
fn apply_diff_colors(
    syntax_colored: &str,
    raw: &str,
    line_bg: &str,
    word_bg: &str,
    word_ranges: &[(usize, usize)],
    is_changed: bool,
) -> String {
    if !is_changed || word_ranges.is_empty() {
        return syntax_colored.to_string();
    }

    // Build a mapping from visible char index → byte position in syntax_colored
    let segments = crate::ansi::split_ansi(syntax_colored);
    let mut vis_to_byte: Vec<usize> = Vec::new();
    let mut byte_pos = 0;

    for seg in &segments {
        if seg.starts_with('\x1b') {
            byte_pos += seg.len();
        } else {
            for (i, _) in seg.char_indices() {
                vis_to_byte.push(byte_pos + i);
            }
            byte_pos += seg.len();
        }
    }
    vis_to_byte.push(byte_pos); // sentinel

    // Also build visible char index for the raw content
    let raw_chars: Vec<usize> = raw.char_indices().map(|(i, _)| i).collect();

    let mut result = syntax_colored.to_string();
    let mut insertions: Vec<(usize, String)> = Vec::new();

    for &(start, end) in word_ranges {
        // Map raw byte offsets to visible char indices
        let vis_start = raw_chars
            .iter()
            .position(|&b| b >= start)
            .unwrap_or(vis_to_byte.len().saturating_sub(1));
        let vis_end = raw_chars
            .iter()
            .position(|&b| b >= end)
            .unwrap_or(vis_to_byte.len().saturating_sub(1));

        if vis_start < vis_to_byte.len() && vis_end <= vis_to_byte.len() {
            let byte_start = vis_to_byte[vis_start];
            let byte_end = vis_to_byte[vis_end.min(vis_to_byte.len() - 1)];
            let reset = style::RESET;
            insertions.push((byte_end, format!("{reset}{line_bg}")));
            insertions.push((byte_start, word_bg.to_string()));
        }
    }

    // Apply insertions in reverse order
    insertions.sort_by(|a, b| b.0.cmp(&a.0));
    for (pos, code) in insertions {
        if pos <= result.len() {
            result.insert_str(pos, &code);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::diff;

    #[test]
    fn render_produces_lines_for_simple_diff() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        // File header + hunk header + 4 diff lines
        assert_eq!(output.lines.len(), 6);
        assert_eq!(output.file_starts.len(), 1);
        assert_eq!(output.hunk_starts.len(), 1);
        assert_eq!(output.file_starts[0], 0);
        assert_eq!(output.hunk_starts[0], 1); // after file header

        // Check the added line has a + marker
        let added_line = &output.lines[3]; // header, hunk header, ctx, added
        assert!(added_line.contains('+'));
        assert!(added_line.contains("added"));
    }

    #[test]
    fn render_multi_file() {
        let raw = "\
diff --git a/a.txt b/a.txt
--- a/a.txt
+++ b/a.txt
@@ -1,1 +1,2 @@
 first
+second
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1,2 +1,1 @@
 keep
-remove
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        assert_eq!(output.file_starts.len(), 2);
        assert_eq!(output.hunk_starts.len(), 2);

        // line_map should reference correct file indices
        let first_file_info = &output.line_map[output.file_starts[0]];
        assert_eq!(first_file_info.path, "a.txt");
        let second_file_info = &output.line_map[output.file_starts[1]];
        assert_eq!(second_file_info.path, "b.txt");
    }

    #[test]
    fn render_multi_hunk() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added1
 line2
 line3
@@ -10,3 +11,4 @@
 line10
+added2
 line11
 line12
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        assert_eq!(output.hunk_starts.len(), 2);
    }

    #[test]
    fn line_map_tracks_new_lineno() {
        let raw = "\
diff --git a/f.rs b/f.rs
--- a/f.rs
+++ b/f.rs
@@ -1,2 +1,3 @@
 ctx
+added
 more
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        // Find the added line's info
        let added_info = output
            .line_map
            .iter()
            .find(|li| li.new_lineno == Some(2))
            .expect("should find new_lineno=2");
        assert_eq!(added_info.path, "f.rs");
    }
}
