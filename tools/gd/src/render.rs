use std::sync::Arc;

use similar::TextDiff;
use tui::highlight::{
    HighlightLines, SYNTAX_SET, SyntaxReference, SyntectSyntaxSet, SyntectTheme, THEME,
    highlight_line,
};

use crate::git::diff::{DiffFile, DiffHunk, FileStatus, LineKind};
use crate::style;

/// Per-rendered-line metadata for the pager.
#[derive(Debug, Clone)]
pub struct LineInfo {
    pub file_idx: usize,
    pub path: Arc<str>,
    /// Source line number in the new file (for editor jump), if applicable.
    pub new_lineno: Option<u32>,
    /// Source line number in the old file (for deleted lines), if applicable.
    pub old_lineno: Option<u32>,
    /// Diff status of this line (Added/Deleted/Context), if from a hunk.
    pub line_kind: Option<LineKind>,
}

pub struct RenderOutput {
    pub lines: Vec<String>,
    pub line_map: Vec<LineInfo>,
    pub file_starts: Vec<usize>,
    pub hunk_starts: Vec<usize>,
}

struct HunkRenderContext<'a> {
    file_idx: usize,
    path: Arc<str>,
    syntax: &'a SyntaxReference,
    ss: &'a SyntectSyntaxSet,
    theme: &'a SyntectTheme,
    color: bool,
    width: usize,
}

/// Render a single file and return its lines, line_map, and local hunk_start offsets.
fn render_file(
    file_idx: usize,
    file: &DiffFile,
    width: usize,
    color: bool,
) -> (Vec<String>, Vec<LineInfo>, Vec<usize>) {
    let path = file.path();
    let arc_path: Arc<str> = Arc::from(path);
    let status_label = match file.status {
        FileStatus::Modified => "Modified",
        FileStatus::Added => "Added",
        FileStatus::Deleted => "Deleted",
        FileStatus::Renamed => "Renamed",
        FileStatus::Untracked => "Untracked",
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

    let cap = file.hunks.iter().map(|h| h.lines.len()).sum::<usize>() + 1;
    let mut lines = Vec::with_capacity(cap);
    let mut line_map = Vec::with_capacity(cap);
    let mut hunk_starts = Vec::new();

    lines.push(header);
    line_map.push(LineInfo {
        file_idx,
        path: Arc::clone(&arc_path),
        new_lineno: None,
        old_lineno: None,
        line_kind: None,
    });

    // Syntax highlighter for this file's extension
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(
            std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or(""),
        )
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
    let render_ctx = HunkRenderContext {
        file_idx,
        path: Arc::clone(&arc_path),
        syntax,
        ss: &SYNTAX_SET,
        theme: &THEME,
        color,
        width,
    };

    for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
        // Separator between hunks (not before the first)
        if hunk_idx > 0 {
            lines.push(style::hunk_separator(width, color));
            line_map.push(LineInfo {
                file_idx,
                path: Arc::clone(&arc_path),
                new_lineno: None,
                old_lineno: None,
                line_kind: None,
            });
        }

        hunk_starts.push(lines.len());

        // Render diff lines with word-level highlights
        render_hunk_lines(hunk, &render_ctx, &mut lines, &mut line_map);
    }

    (lines, line_map, hunk_starts)
}

pub fn render(files: &[DiffFile], width: usize, color: bool) -> RenderOutput {
    let thread_count = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .min(files.len())
        .max(1);

    // No-color render skips syntect highlighting (the expensive part),
    // so threading overhead would dominate. Only parallelize with color.
    if thread_count <= 1 || !color {
        // Sequential path: no threading overhead
        let cap: usize = files
            .iter()
            .map(|f| f.hunks.iter().map(|h| h.lines.len()).sum::<usize>())
            .sum::<usize>()
            + files.len();
        let mut all_lines = Vec::with_capacity(cap);
        let mut all_line_map = Vec::with_capacity(cap);
        let mut file_starts = Vec::new();
        let mut hunk_starts = Vec::new();

        for (file_idx, file) in files.iter().enumerate() {
            let offset = all_lines.len();
            file_starts.push(offset);
            let (fl, fm, fh) = render_file(file_idx, file, width, color);
            for hs in &fh {
                hunk_starts.push(hs + offset);
            }
            all_lines.extend(fl);
            all_line_map.extend(fm);
        }

        return RenderOutput {
            lines: all_lines,
            line_map: all_line_map,
            file_starts,
            hunk_starts,
        };
    }

    // Parallel path: render files across threads, then merge in order
    let file_results: Vec<(Vec<String>, Vec<LineInfo>, Vec<usize>)> =
        std::thread::scope(|s| {
            let chunk_size = files.len().div_ceil(thread_count);
            let handles: Vec<_> = files
                .chunks(chunk_size)
                .enumerate()
                .map(|(chunk_idx, chunk)| {
                    let base_file_idx = chunk_idx * chunk_size;
                    s.spawn(move || {
                        let mut results = Vec::with_capacity(chunk.len());
                        for (i, file) in chunk.iter().enumerate() {
                            results.push(render_file(base_file_idx + i, file, width, color));
                        }
                        results
                    })
                })
                .collect();

            let mut all_results = Vec::with_capacity(files.len());
            for handle in handles {
                all_results.extend(handle.join().unwrap());
            }
            all_results
        });

    // Merge results in file order
    let total_lines: usize = file_results.iter().map(|(l, _, _)| l.len()).sum();
    let mut all_lines = Vec::with_capacity(total_lines);
    let mut all_line_map = Vec::with_capacity(total_lines);
    let mut file_starts = Vec::with_capacity(files.len());
    let mut hunk_starts = Vec::new();

    for (fl, fm, fh) in file_results {
        let offset = all_lines.len();
        file_starts.push(offset);
        for hs in &fh {
            hunk_starts.push(hs + offset);
        }
        all_lines.extend(fl);
        all_line_map.extend(fm);
    }

    RenderOutput {
        lines: all_lines,
        line_map: all_line_map,
        file_starts,
        hunk_starts,
    }
}

/// Group consecutive added/deleted lines into change blocks for word-level diffing.
pub struct ChangeBlock {
    pub deleted: Vec<usize>,
    pub added: Vec<usize>,
}

pub fn find_change_blocks(hunk: &DiffHunk) -> Vec<ChangeBlock> {
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

/// Tokenize text into words, whitespace runs, and individual punctuation characters.
/// Finer than `from_words` (which groups by whitespace only), so punctuation like
/// a trailing comma doesn't cause an entire word to be treated as changed.
pub fn tokenize(s: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut chars = s.char_indices().peekable();

    while let Some(&(start, ch)) = chars.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            let mut end = start + ch.len_utf8();
            chars.next();
            while let Some(&(_, c)) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    end += c.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(&s[start..end]);
        } else if ch.is_whitespace() {
            let mut end = start + ch.len_utf8();
            chars.next();
            while let Some(&(_, c)) = chars.peek() {
                if c.is_whitespace() {
                    end += c.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(&s[start..end]);
        } else {
            chars.next();
            tokens.push(&s[start..start + ch.len_utf8()]);
        }
    }

    tokens
}

/// For a change block, compute per-line word highlight ranges.
/// Returns (deleted_highlights, added_highlights) — each a Vec<Vec<(start, end)>> per line.
pub fn word_highlights(
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

    let old_tokens = tokenize(&old_text);
    let new_tokens = tokenize(&new_text);
    let diff = TextDiff::from_slices(&old_tokens, &new_tokens);

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

    // Merge contiguous ranges produced by individual token changes
    for ranges in del_highlights.iter_mut().chain(add_highlights.iter_mut()) {
        merge_ranges(ranges);
    }

    (del_highlights, add_highlights)
}

/// Merge contiguous or overlapping highlight ranges in-place.
pub(crate) fn merge_ranges(ranges: &mut Vec<(usize, usize)>) {
    if ranges.len() <= 1 {
        return;
    }
    ranges.sort_unstable_by_key(|&(start, _)| start);
    let mut merged = Vec::with_capacity(ranges.len());
    let mut current = ranges[0];
    for &(start, end) in &ranges[1..] {
        if start <= current.1 {
            current.1 = current.1.max(end);
        } else {
            merged.push(current);
            current = (start, end);
        }
    }
    merged.push(current);
    *ranges = merged;
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

fn render_hunk_lines(
    hunk: &DiffHunk,
    ctx: &HunkRenderContext<'_>,
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
    let mut hl_state = HighlightLines::new(ctx.syntax, ctx.theme);
    let mut nl_buf = String::new();
    let cont_gutter = style::continuation_gutter(ctx.color);
    let wrap = style::wrap_marker(ctx.color);

    for (i, diff_line) in hunk.lines.iter().enumerate() {
        let gutter = if ctx.color {
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
            LineKind::Added => (
                "+",
                style::BG_ADDED,
                style::BG_ADDED_WORD,
                style::FG_ADDED_MARKER,
            ),
            LineKind::Deleted => (
                "-",
                style::BG_DELETED,
                style::BG_DELETED_WORD,
                style::FG_DELETED_MARKER,
            ),
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
        let styled_content = if ctx.color {
            nl_buf.clear();
            nl_buf.push_str(content);
            nl_buf.push('\n');
            let syntax_colored = highlight_line(
                &nl_buf,
                &mut hl_state,
                ctx.ss,
                style::SOFT_RESET,
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

        let marker_styled = if ctx.color && !marker_color.is_empty() {
            format!("{}{marker}{}", marker_color, style::SOFT_RESET)
        } else {
            marker.to_string()
        };

        let is_changed = diff_line.kind != LineKind::Context;
        let avail = ctx.width.saturating_sub(style::GUTTER_WIDTH + 2); // +1 marker, +1 wrap indicator

        // Pre-wrap: prepend line_bg so AnsiState tracks it during wrapping
        let wrappable = if ctx.color && is_changed {
            format!("{line_bg}{styled_content}")
        } else {
            styled_content // move instead of clone
        };
        let wrapped = crate::ansi::wrap_line_for_display(&wrappable, avail);

        for (seg_idx, seg) in wrapped.iter().enumerate() {
            // Strip trailing reset from wrapped segment (we add our own)
            let content_part = seg.trim_end_matches(style::RESET);

            let seg_width = crate::ansi::visible_width(content_part);
            let pad_len = avail.saturating_sub(seg_width);
            // Padding spaces inherit the current bg
            let padding = if ctx.color && is_changed && pad_len > 0 {
                " ".repeat(pad_len)
            } else {
                String::new()
            };

            let line = if seg_idx == 0 {
                if ctx.color && is_changed {
                    format!(
                        "{gutter}{line_bg}{marker_styled}{content_part}{padding}{}",
                        style::RESET
                    )
                } else {
                    format!("{gutter}{marker_styled}{content_part}")
                }
            } else if ctx.color && is_changed {
                format!(
                    "{cont_gutter}{line_bg}{wrap}{content_part}{padding}{}",
                    style::RESET
                )
            } else {
                format!("{cont_gutter}{wrap}{content_part}")
            };

            lines.push(line);
            line_map.push(LineInfo {
                file_idx: ctx.file_idx,
                path: Arc::clone(&ctx.path),
                new_lineno: diff_line.new_lineno,
                old_lineno: diff_line.old_lineno,
                line_kind: Some(diff_line.kind),
            });
        }
    }
}

/// Apply diff background colors and word-level highlights to syntax-colored text.
/// The `raw` parameter is the original uncolored content used for word range mapping.
pub fn apply_diff_colors(
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

    // Collect (byte_pos, priority, ansi_code) pairs for a forward pass.
    // Priority 0 = word_bg (start highlight), 1 = line_bg (end highlight).
    // At the same byte position, word_bg sorts before line_bg so a range
    // ending and starting at the same point produces: end-old, start-new.
    let mut markers: Vec<(usize, u8, &str)> = Vec::with_capacity(word_ranges.len() * 2);

    for &(start, end) in word_ranges {
        // Map raw byte offsets to visible char indices via binary search
        let vis_start = {
            let idx = raw_chars.partition_point(|&b| b < start);
            if idx < raw_chars.len() { idx } else { vis_to_byte.len().saturating_sub(1) }
        };
        let vis_end = {
            let idx = raw_chars.partition_point(|&b| b < end);
            if idx < raw_chars.len() { idx } else { vis_to_byte.len().saturating_sub(1) }
        };

        if vis_start < vis_to_byte.len() && vis_end <= vis_to_byte.len() {
            let byte_start = vis_to_byte[vis_start];
            let byte_end = vis_to_byte[vis_end.min(vis_to_byte.len() - 1)];
            markers.push((byte_start, 0, word_bg));
            markers.push((byte_end, 1, line_bg));
        }
    }

    markers.sort_unstable();

    // Single forward pass: emit syntax_colored slices interleaved with ANSI codes
    let total_codes_len: usize = markers.iter().map(|(_, _, code)| code.len()).sum();
    let mut result = String::with_capacity(syntax_colored.len() + total_codes_len);
    let mut src_pos: usize = 0;

    for &(pos, _, code) in &markers {
        if pos <= syntax_colored.len() && pos >= src_pos {
            result.push_str(&syntax_colored[src_pos..pos]);
            result.push_str(code);
            src_pos = pos;
        }
    }
    result.push_str(&syntax_colored[src_pos..]);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::diff;
    use crate::git::diff::DiffLine;
    use insta::assert_debug_snapshot;
    use insta::assert_snapshot;

    fn strip(lines: &[String]) -> String {
        lines
            .iter()
            .map(|l| crate::ansi::strip_ansi(l))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn snapshot_single_file() {
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
        assert_snapshot!(strip(&output.lines));
    }

    #[test]
    fn snapshot_multi_file() {
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
        assert_snapshot!(strip(&output.lines));
    }

    #[test]
    fn snapshot_multi_hunk() {
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
        assert_snapshot!(strip(&output.lines));
    }

    #[test]
    fn snapshot_wrapped_long_line() {
        let long_content = "x".repeat(100);
        let raw = format!(
            "\
diff --git a/foo.txt b/foo.txt
--- a/foo.txt
+++ b/foo.txt
@@ -1,1 +1,2 @@
 ctx
+{long_content}
"
        );
        let files = diff::parse(&raw);
        let output = render(&files, 40, false);
        assert_snapshot!(strip(&output.lines));
    }

    #[test]
    fn snapshot_untracked_file() {
        let file = diff::DiffFile::from_content("new.rs", "hello\nworld\n");
        let output = render(&[file], 80, false);
        assert_snapshot!(strip(&output.lines));
    }

    #[test]
    fn snapshot_line_map() {
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
        assert_debug_snapshot!(output.line_map);
    }

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

        // File header + 4 diff lines (no hunk header)
        assert_eq!(output.lines.len(), 5);
        assert_eq!(output.file_starts.len(), 1);
        assert_eq!(output.hunk_starts.len(), 1);
        assert_eq!(output.file_starts[0], 0);
        assert_eq!(output.hunk_starts[0], 1); // after file header, at first content line

        // Check the added line has a + marker
        let added_line = &output.lines[2]; // header, ctx, added
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
        assert_eq!(&*first_file_info.path, "a.txt");
        let second_file_info = &output.line_map[output.file_starts[1]];
        assert_eq!(&*second_file_info.path, "b.txt");
    }

    #[test]
    fn render_multi_file_with_headers_has_no_blank_separator_line() {
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
        assert!(
            !output.lines.iter().any(std::string::String::is_empty),
            "all-files mode should not insert blank lines between file headers"
        );
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
        assert_eq!(&*added_info.path, "f.rs");
    }

    #[test]
    fn colored_added_line_has_bg_spanning_content() {
        let raw = "\
diff --git a/foo.txt b/foo.txt
--- a/foo.txt
+++ b/foo.txt
@@ -1,1 +1,2 @@
 ctx
+added line
";
        let files = diff::parse(raw);
        let output = render(&files, 80, true);

        // Find the added line (after file header, context)
        let added = &output.lines[2];
        let stripped = crate::ansi::strip_ansi(added);
        assert!(stripped.contains('+'), "should have + marker");

        // The full RESET (\x1b[0m) should only appear at the very end of the line,
        // not between the marker and content or between syntax tokens
        let content_area = added.split(style::BG_ADDED).last().unwrap_or(added);
        let mid_section =
            &content_area[..content_area.rfind("\x1b[0m").unwrap_or(content_area.len())];
        assert!(
            !mid_section.contains("\x1b[0m"),
            "full RESET should not appear mid-line (kills bg): {mid_section:?}"
        );
    }

    #[test]
    fn syntax_highlight_uses_soft_reset() {
        let syntax = SYNTAX_SET.find_syntax_by_extension("rs").unwrap();
        let mut hl = HighlightLines::new(syntax, &THEME);

        let result = highlight_line("let x = 42;\n", &mut hl, &SYNTAX_SET, style::SOFT_RESET);
        // Should use SOFT_RESET (22;23;39) not full RESET (0m) between tokens
        assert!(
            !result.contains("\x1b[0m"),
            "highlight_line should use SOFT_RESET, not RESET: {result:?}"
        );
        assert!(
            result.contains("\x1b[22;23;39m"),
            "should contain SOFT_RESET: {result:?}"
        );
    }

    #[test]
    fn long_line_wraps_with_continuation_gutter() {
        let long_content = "x".repeat(100);
        let raw = format!(
            "\
diff --git a/foo.txt b/foo.txt
--- a/foo.txt
+++ b/foo.txt
@@ -1,1 +1,2 @@
 ctx
+{long_content}
"
        );
        let files = diff::parse(&raw);
        // Narrow width forces wrapping: gutter(12) + marker(1) + avail
        let width = 40;
        let output = render(&files, width, false);

        // Should produce more lines than the 3 logical lines (header, ctx, added)
        assert!(
            output.lines.len() > 3,
            "long line should wrap into multiple output lines, got {}",
            output.lines.len()
        );

        // Continuation lines should have blank gutter with separators
        let cont_lines: Vec<_> = output.lines.iter().skip(3).collect();
        assert!(!cont_lines.is_empty(), "should have continuation lines");
        for cont in &cont_lines {
            assert!(
                cont.contains('|'),
                "continuation line should have gutter separators: {cont:?}"
            );
        }
    }

    #[test]
    fn word_highlights_punctuation_boundary() {
        // Regression: from_words treats "application"]  vs "application"], as entirely
        // different tokens, highlighting the whole word instead of just the insertion.
        use crate::git::diff::{DiffHunk, DiffLine, LineKind};

        let hunk = DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![
                DiffLine {
                    kind: LineKind::Deleted,
                    content: r#"  "--app": { type: AppArg, alias: ["-a", "--application"] },"#.into(),
                    old_lineno: Some(1),
                    new_lineno: None,
                },
                DiffLine {
                    kind: LineKind::Added,
                    content: r#"  "--app": { type: AppArg, alias: ["-a", "--application"], description: "Select the application" },"#.into(),
                    old_lineno: None,
                    new_lineno: Some(1),
                },
            ],
        };
        let block = ChangeBlock {
            deleted: vec![0],
            added: vec![1],
        };
        let (del_hl, add_hl) = word_highlights(&hunk, &block);

        // Nothing was deleted — the old line should have no highlights
        assert!(
            del_hl[0].is_empty(),
            "deleted line should have no highlights, got: {:?}",
            del_hl[0]
        );

        // The insertion is `, description: "Select the application"`
        // which starts right after the `]` in the added line
        assert_eq!(
            add_hl[0].len(),
            1,
            "added line should have exactly one highlight range, got: {:?}",
            add_hl[0]
        );
        let (start, end) = add_hl[0][0];
        let added = &hunk.lines[1].content;
        let highlighted = &added[start..end];
        assert_eq!(
            highlighted, r#", description: "Select the application""#,
            "should highlight only the inserted portion"
        );
    }

    #[test]
    fn test_render_single_file_populates_file_starts() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,1 +1,2 @@
 ctx
+added
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_eq!(output.file_starts.len(), 1);
        assert_eq!(output.file_starts[0], 0);
    }

    #[test]
    fn test_render_two_files_file_starts_ordered() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,1 +1,2 @@
 ctx
+added
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1,1 +1,2 @@
 ctx
+added2
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_eq!(output.file_starts.len(), 2);
        assert!(
            output.file_starts[1] > 0,
            "second file should start after line 0"
        );
    }

    #[test]
    fn test_render_hunk_starts_present() {
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
        assert!(
            output.hunk_starts[0] < output.hunk_starts[1],
            "hunk starts should be ordered"
        );
    }

    #[test]
    fn test_render_line_map_length_matches_lines() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,2 +1,3 @@
 ctx
+added
 more
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_eq!(output.lines.len(), output.line_map.len());
    }

    #[test]
    fn test_render_content_lines_have_line_kind() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,2 +1,3 @@
 ctx
+added
-old
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        // File header (index 0) should have None line_kind
        assert!(
            output.line_map[0].line_kind.is_none(),
            "file header should have None line_kind"
        );
        // Content lines should have Some(_)
        let content_lines: Vec<_> = output
            .line_map
            .iter()
            .filter(|li| li.line_kind.is_some())
            .collect();
        assert!(
            !content_lines.is_empty(),
            "should have content lines with Some line_kind"
        );
        for li in &content_lines {
            assert!(
                matches!(
                    li.line_kind,
                    Some(LineKind::Added | LineKind::Deleted | LineKind::Context)
                ),
                "content lines should be Added, Deleted, or Context"
            );
        }
    }

    #[test]
    fn render_untracked_file_shows_status_label() {
        let file = diff::DiffFile::from_content("new.rs", "hello\n");
        let output = render(&[file], 80, false);
        let header = &output.lines[0];
        assert!(
            header.contains("Untracked"),
            "file header should contain 'Untracked': {header:?}"
        );
    }

    #[test]
    fn test_no_hunk_header_in_output() {
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

        for line in &output.lines {
            assert!(
                !line.contains("@@"),
                "rendered output should not contain '@@' hunk headers, but found: {line:?}"
            );
        }
    }

    #[test]
    fn test_hunk_starts_points_at_content_line() {
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

        assert!(
            !output.hunk_starts.is_empty(),
            "should have at least one hunk_start"
        );
        let hs = output.hunk_starts[0];
        // Must be past the file header
        assert!(
            hs > output.file_starts[0],
            "hunk_start should be past the file header: hunk_start={hs}, file_start={}",
            output.file_starts[0]
        );
        // Must point at a content line (line_kind is Some)
        assert!(
            output.line_map[hs].line_kind.is_some(),
            "hunk_starts[0]={hs} should point at a content line with Some(line_kind), got {:?}",
            output.line_map[hs].line_kind
        );
    }

    #[test]
    fn test_multi_hunk_starts_all_point_at_content() {
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

        assert_eq!(output.hunk_starts.len(), 2, "should have two hunks");
        for (i, &hs) in output.hunk_starts.iter().enumerate() {
            assert!(
                output.line_map[hs].line_kind.is_some(),
                "hunk_starts[{i}]={hs} should point at a content line with Some(line_kind), got {:?}",
                output.line_map[hs].line_kind
            );
        }
    }

    #[test]
    fn apply_diff_colors_is_changed_false() {
        let result = apply_diff_colors("foo", "foo", "", "", &[(0, 3)], false);
        assert_eq!(result, "foo");
    }

    #[test]
    fn apply_diff_colors_empty_word_ranges() {
        let result = apply_diff_colors("foo", "foo", "", "", &[], true);
        assert_eq!(result, "foo");
    }

    #[test]
    fn apply_diff_colors_simple_ascii_one_range() {
        let result = apply_diff_colors(
            "hello",
            "hello",
            style::BG_ADDED,
            style::BG_ADDED_WORD,
            &[(0, 5)],
            true,
        );
        assert!(
            result.contains(style::BG_ADDED_WORD),
            "expected word highlight in result: {result:?}"
        );
        assert!(
            crate::ansi::strip_ansi(&result).contains("hello"),
            "content should be preserved"
        );
    }

    #[test]
    fn apply_diff_colors_multi_byte_unicode() {
        let raw = "a\u{1F600}b";
        let result = apply_diff_colors(
            raw,
            raw,
            style::BG_ADDED,
            style::BG_ADDED_WORD,
            &[(0, raw.len())],
            true,
        );
        assert!(
            crate::ansi::strip_ansi(&result).contains("a\u{1F600}b"),
            "multi-byte content preserved: {result:?}"
        );
    }

    #[test]
    fn apply_diff_colors_two_adjacent_ranges() {
        let result = apply_diff_colors(
            "abcd",
            "abcd",
            style::BG_DELETED,
            style::BG_DELETED_WORD,
            &[(0, 2), (2, 4)],
            true,
        );
        let word_count = result.matches(style::BG_DELETED_WORD).count();
        assert!(
            word_count >= 2,
            "expected at least 2 word highlight insertions, got {word_count}: {result:?}"
        );
    }

    #[test]
    fn merge_ranges_overlapping() {
        let mut ranges = vec![(0, 5), (3, 8), (10, 12)];
        merge_ranges(&mut ranges);
        assert_eq!(ranges, vec![(0, 8), (10, 12)]);
    }

    #[test]
    fn merge_ranges_single_or_empty() {
        let mut single = vec![(1, 3)];
        merge_ranges(&mut single);
        assert_eq!(single, vec![(1, 3)]);

        let mut empty: Vec<(usize, usize)> = vec![];
        merge_ranges(&mut empty);
        assert!(empty.is_empty());
    }

    fn make_line(kind: LineKind) -> DiffLine {
        DiffLine {
            kind,
            content: String::new(),
            old_lineno: None,
            new_lineno: None,
        }
    }

    #[test]
    fn find_change_blocks_added_only() {
        let hunk = DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![
                make_line(LineKind::Context),
                make_line(LineKind::Added),
                make_line(LineKind::Added),
                make_line(LineKind::Context),
            ],
        };
        let blocks = find_change_blocks(&hunk);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].deleted.is_empty());
        assert_eq!(blocks[0].added, vec![1, 2]);
    }

    #[test]
    fn find_change_blocks_mixed() {
        let hunk = DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![
                make_line(LineKind::Deleted),
                make_line(LineKind::Deleted),
                make_line(LineKind::Added),
                make_line(LineKind::Added),
                make_line(LineKind::Context),
                make_line(LineKind::Added),
            ],
        };
        let blocks = find_change_blocks(&hunk);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].deleted, vec![0, 1]);
        assert_eq!(blocks[0].added, vec![2, 3]);
        assert!(blocks[1].deleted.is_empty());
        assert_eq!(blocks[1].added, vec![5]);
    }

    #[test]
    fn colored_wrap_has_bg_on_continuation() {
        let long_content = "x".repeat(100);
        let raw = format!(
            "\
diff --git a/foo.txt b/foo.txt
--- a/foo.txt
+++ b/foo.txt
@@ -1,1 +1,2 @@
 ctx
+{long_content}
"
        );
        let files = diff::parse(&raw);
        let width = 40;
        let output = render(&files, width, true);

        // Find continuation lines — they have blank gutter (no line numbers)
        // and contain 'x' from the long content
        let first_added = output
            .lines
            .iter()
            .position(|l| crate::ansi::strip_ansi(l).contains('+'))
            .expect("should find added line");
        let cont_lines: Vec<_> = output
            .lines
            .iter()
            .skip(first_added + 1)
            .filter(|l| {
                let s = crate::ansi::strip_ansi(l);
                // Continuation lines have blank gutter (starts with spaces before │)
                // and contain 'xx' (the repeated content)
                s.contains("xx") && s.trim_start().starts_with('\u{2502}')
            })
            .collect();

        assert!(!cont_lines.is_empty(), "should have continuation lines");
        for cont in &cont_lines {
            assert!(
                cont.contains(style::BG_ADDED),
                "continuation line should have added bg: {cont:?}"
            );
        }
    }

    #[test]
    fn render_parallel_matches_sequential() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,1 +1,2 @@
 ctx
+added
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1,1 +1,2 @@
 ctx
+added2
";
        let files = diff::parse(raw);
        let color_output = render(&files, 80, true);
        let plain_output = render(&files, 80, false);
        // Structural shape must match regardless of rendering path
        assert_eq!(color_output.file_starts, plain_output.file_starts);
        assert_eq!(color_output.hunk_starts, plain_output.hunk_starts);
        assert_eq!(color_output.line_map.len(), plain_output.line_map.len());
    }
}
