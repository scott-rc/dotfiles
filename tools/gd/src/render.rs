use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use similar::TextDiff;
use tui::highlight::{HighlightLines, SYNTAX_SET, THEME, highlight_line};

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
    /// Index of the hunk within the file this line belongs to, if from a hunk.
    pub hunk_idx: Option<usize>,
}

// ---------------------------------------------------------------------------
// Phase 1 output: width-independent styled content
// ---------------------------------------------------------------------------

/// Phase 1 output for a single source diff line. Contains the syntax-highlighted,
/// diff-colored, word-highlighted content string — but no wrapping or gutters.
#[derive(Debug, Clone)]
pub(crate) struct StyledLine {
    /// Syntax + diff colors + word highlights applied, no wrapping/gutter.
    pub(crate) styled_content: String,
    /// Original text (for search, word-range mapping, no-color path).
    pub(crate) raw_content: String,
    pub(crate) line_kind: LineKind,
    pub(crate) old_lineno: Option<u32>,
    pub(crate) new_lineno: Option<u32>,
    pub(crate) hunk_idx: usize,
}

/// Phase 1 output for a file.
#[derive(Debug, Clone)]
pub struct StyledFile {
    pub(crate) path: String,
    pub(crate) status: FileStatus,
    pub(crate) file_idx: usize,
    pub(crate) lines: Vec<StyledLine>,
}

// ---------------------------------------------------------------------------
// Phase 2 output: width-dependent layout
// ---------------------------------------------------------------------------

pub struct LayoutOutput {
    pub(crate) display_lines: Vec<String>,
    pub(crate) raw_texts: Vec<String>,
    pub(crate) line_map: Vec<LineInfo>,
    pub(crate) file_starts: Vec<usize>,
    pub(crate) hunk_starts: Vec<usize>,
}

// ---------------------------------------------------------------------------
// Combined render output
// ---------------------------------------------------------------------------

pub struct RenderOutput {
    /// Phase 1 (width-independent, reusable across relayouts).
    pub(crate) styled_files: Vec<StyledFile>,
    /// Phase 2 display lines.
    pub(crate) display_lines: Vec<String>,
    /// Phase 2 raw texts for search.
    pub(crate) raw_texts: Vec<String>,
    pub line_map: Vec<LineInfo>,
    pub file_starts: Vec<usize>,
    pub hunk_starts: Vec<usize>,
}

impl RenderOutput {
    pub fn lines(&self) -> &[String] {
        &self.display_lines
    }

    #[allow(dead_code)]
    pub fn get(&self, idx: usize) -> &str {
        &self.display_lines[idx]
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.display_lines.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.display_lines.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Phase 1: style (expensive, width-independent, parallelized across files)
// ---------------------------------------------------------------------------

/// Syntax-highlight + diff-color + word-highlight a single line's content.
/// Returns the styled ANSI string without wrapping or gutters.
fn style_line_content(
    content: &str,
    line_kind: LineKind,
    word_ranges: &[(usize, usize)],
    color: bool,
    hl_state: &mut HighlightLines,
    nl_buf: &mut String,
) -> String {
    if !color {
        return content.to_string();
    }

    nl_buf.clear();
    nl_buf.push_str(content);
    nl_buf.push('\n');
    let syntax_colored = highlight_line(nl_buf, hl_state, &SYNTAX_SET, style::SOFT_RESET);

    let (line_bg, word_bg) = match line_kind {
        LineKind::Added => (style::BG_ADDED, style::BG_ADDED_WORD),
        LineKind::Deleted => (style::BG_DELETED, style::BG_DELETED_WORD),
        LineKind::Context => ("", ""),
    };

    let is_changed = line_kind != LineKind::Context;
    apply_diff_colors(&syntax_colored, content, line_bg, word_bg, word_ranges, is_changed)
}

/// Compute word-highlight byte ranges for every line in a hunk.
/// Returns a map from hunk-internal line index to highlight ranges.
fn compute_hunk_word_ranges(hunk: &DiffHunk) -> HashMap<usize, Vec<(usize, usize)>> {
    let blocks = find_change_blocks(hunk);
    let mut line_ranges = HashMap::new();
    for block in &blocks {
        let (del_hl, add_hl) = word_highlights(hunk, block);
        for (idx, ranges) in block.deleted.iter().zip(del_hl) {
            if !ranges.is_empty() {
                line_ranges.insert(*idx, ranges);
            }
        }
        for (idx, ranges) in block.added.iter().zip(add_hl) {
            if !ranges.is_empty() {
                line_ranges.insert(*idx, ranges);
            }
        }
    }
    line_ranges
}

/// Phase 1: style a single file (sequential — highlighter state is per-file).
fn style_file(file_idx: usize, file: &DiffFile, color: bool) -> StyledFile {
    let path = file.path().to_string();
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
    let mut hl_state = HighlightLines::new(syntax, &THEME);
    let mut nl_buf = String::new();

    let total_lines: usize = file.hunks.iter().map(|h| h.lines.len()).sum();
    let mut styled_lines = Vec::with_capacity(total_lines);

    for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
        let word_ranges_map = compute_hunk_word_ranges(hunk);

        for (i, diff_line) in hunk.lines.iter().enumerate() {
            let word_ranges = word_ranges_map
                .get(&i)
                .map_or(&[][..], Vec::as_slice);
            let styled_content = style_line_content(
                &diff_line.content,
                diff_line.kind,
                word_ranges,
                color,
                &mut hl_state,
                &mut nl_buf,
            );
            styled_lines.push(StyledLine {
                styled_content,
                raw_content: diff_line.content.clone(),
                line_kind: diff_line.kind,
                old_lineno: diff_line.old_lineno,
                new_lineno: diff_line.new_lineno,
                hunk_idx,
            });
        }
    }

    StyledFile {
        path,
        status: file.status,
        file_idx,
        lines: styled_lines,
    }
}

/// Phase 1: style all files in parallel.
///
/// Uses an atomic work queue so each thread grabs the next unprocessed file,
/// avoiding load imbalance when file sizes vary widely (e.g. a lock file with
/// 10k lines next to small config files). Files are processed largest-first
/// so the biggest job starts immediately while smaller files fill remaining cores.
pub(crate) fn style_files(files: &[DiffFile], color: bool) -> Vec<StyledFile> {
    let thread_count = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .min(files.len())
        .max(1);

    // No-color path skips syntect (the expensive part), so threading overhead
    // would dominate. Only parallelize with color.
    if thread_count <= 1 || !color {
        return files
            .iter()
            .enumerate()
            .map(|(i, f)| style_file(i, f, color))
            .collect();
    }

    // Sort file indices by line count descending so the largest file starts first.
    let mut order: Vec<usize> = (0..files.len()).collect();
    order.sort_unstable_by(|&a, &b| {
        let lines_a: usize = files[a].hunks.iter().map(|h| h.lines.len()).sum();
        let lines_b: usize = files[b].hunks.iter().map(|h| h.lines.len()).sum();
        lines_b.cmp(&lines_a)
    });

    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<StyledFile>>> =
        (0..files.len()).map(|_| Mutex::new(None)).collect();

    std::thread::scope(|s| {
        for _ in 0..thread_count {
            s.spawn(|| {
                loop {
                    let idx = next.fetch_add(1, Ordering::Relaxed);
                    if idx >= files.len() {
                        break;
                    }
                    let file_idx = order[idx];
                    let styled = style_file(file_idx, &files[file_idx], color);
                    *slots[file_idx].lock().unwrap() = Some(styled);
                }
            });
        }
    });

    slots
        .into_iter()
        .map(|slot| slot.into_inner().unwrap().unwrap())
        .collect()
}

// ---------------------------------------------------------------------------
// Phase 2: layout (cheap, width-dependent)
// ---------------------------------------------------------------------------

fn status_label(status: FileStatus) -> &'static str {
    match status {
        FileStatus::Modified => "Modified",
        FileStatus::Added => "Added",
        FileStatus::Deleted => "Deleted",
        FileStatus::Renamed => "Renamed",
        FileStatus::Untracked => "Untracked",
    }
}

/// Wrap a styled content string and assemble with gutter, marker, padding.
fn layout_line(
    styled_content: &str,
    line_kind: LineKind,
    old_lineno: Option<u32>,
    new_lineno: Option<u32>,
    width: usize,
    color: bool,
) -> Vec<String> {
    let (marker, line_bg, marker_color) = match line_kind {
        LineKind::Added => ("+", style::BG_ADDED, style::FG_ADDED_MARKER),
        LineKind::Deleted => ("-", style::BG_DELETED, style::FG_DELETED_MARKER),
        LineKind::Context => (" ", "", ""),
    };

    let gutter = if color {
        style::gutter(old_lineno, new_lineno)
    } else {
        let old = old_lineno.map_or(String::new(), |n| format!("{n}"));
        let new = new_lineno.map_or(String::new(), |n| format!("{n}"));
        format!("{old:>4} |{new:>4} |")
    };

    let marker_styled = if color && !marker_color.is_empty() {
        format!("{}{marker}{}", marker_color, style::SOFT_RESET)
    } else {
        marker.to_string()
    };

    let is_changed = line_kind != LineKind::Context;
    let avail = width.saturating_sub(style::GUTTER_WIDTH + 2);
    let cont_gutter = style::continuation_gutter(color);
    let wrap_marker = style::wrap_marker(color);

    let wrappable = if color && is_changed {
        format!("{line_bg}{styled_content}")
    } else {
        styled_content.to_string()
    };
    let wrapped = crate::ansi::wrap_line_for_display(&wrappable, avail);

    let mut output = Vec::with_capacity(wrapped.len());
    for (seg_idx, seg) in wrapped.iter().enumerate() {
        let content_part = seg.trim_end_matches(style::RESET);
        let seg_width = crate::ansi::visible_width(content_part);
        let pad_len = avail.saturating_sub(seg_width);
        let padding = if color && is_changed && pad_len > 0 {
            " ".repeat(pad_len)
        } else {
            String::new()
        };

        let line = if seg_idx == 0 {
            if color && is_changed {
                format!(
                    "{gutter}{line_bg}{marker_styled}{content_part}{padding}{}",
                    style::RESET
                )
            } else {
                format!("{gutter}{marker_styled}{content_part}")
            }
        } else if color && is_changed {
            format!(
                "{cont_gutter}{line_bg}{wrap_marker}{content_part}{padding}{}",
                style::RESET
            )
        } else {
            format!("{cont_gutter}{wrap_marker}{content_part}")
        };
        output.push(line);
    }

    output
}

/// Phase 2: layout styled files at a specific width.
pub(crate) fn layout(styled_files: &[StyledFile], width: usize, color: bool) -> LayoutOutput {
    let total_source_lines: usize = styled_files.iter().map(|f| f.lines.len()).sum();
    let cap = total_source_lines + styled_files.len() * 2;

    let mut display_lines = Vec::with_capacity(cap);
    let mut raw_texts = Vec::with_capacity(cap);
    let mut line_map = Vec::with_capacity(cap);
    let mut file_starts = Vec::with_capacity(styled_files.len());
    let mut hunk_starts = Vec::new();

    for sf in styled_files {
        file_starts.push(display_lines.len());
        let arc_path: Arc<str> = Arc::from(sf.path.as_str());
        let label = status_label(sf.status);

        // File header
        let header = if color {
            style::file_header(&sf.path, label, width)
        } else {
            let label_str = format!(" {} ({}) ", sf.path, label);
            let bar_len = width.saturating_sub(2 + label_str.len());
            format!(
                "{}{}{}",
                "\u{2500}".repeat(2),
                label_str,
                "\u{2500}".repeat(bar_len)
            )
        };
        display_lines.push(header);
        raw_texts.push(format!("{} ({label})", sf.path));
        line_map.push(LineInfo {
            file_idx: sf.file_idx,
            path: Arc::clone(&arc_path),
            new_lineno: None,
            old_lineno: None,
            line_kind: None,
            hunk_idx: None,
        });

        let mut prev_hunk_idx: Option<usize> = None;
        for sl in &sf.lines {
            // Hunk separator (between hunks, not before the first)
            if Some(sl.hunk_idx) != prev_hunk_idx {
                if prev_hunk_idx.is_some() {
                    display_lines.push(style::hunk_separator(width, color));
                    raw_texts.push(String::new());
                    line_map.push(LineInfo {
                        file_idx: sf.file_idx,
                        path: Arc::clone(&arc_path),
                        new_lineno: None,
                        old_lineno: None,
                        line_kind: None,
                        hunk_idx: None,
                    });
                }
                hunk_starts.push(display_lines.len());
                prev_hunk_idx = Some(sl.hunk_idx);
            }

            let segments = layout_line(
                &sl.styled_content,
                sl.line_kind,
                sl.old_lineno,
                sl.new_lineno,
                width,
                color,
            );

            for seg in segments {
                display_lines.push(seg);
                raw_texts.push(sl.raw_content.clone());
                line_map.push(LineInfo {
                    file_idx: sf.file_idx,
                    path: Arc::clone(&arc_path),
                    new_lineno: sl.new_lineno,
                    old_lineno: sl.old_lineno,
                    line_kind: Some(sl.line_kind),
                    hunk_idx: Some(sl.hunk_idx),
                });
            }
        }
    }

    LayoutOutput {
        display_lines,
        raw_texts,
        line_map,
        file_starts,
        hunk_starts,
    }
}

// ---------------------------------------------------------------------------
// Top-level render entry point
// ---------------------------------------------------------------------------

pub fn render(files: &[DiffFile], width: usize, color: bool) -> RenderOutput {
    let t0 = std::time::Instant::now();
    let styled_files = style_files(files, color);
    crate::debug::trace("render", "style_files done", t0);
    let lo = layout(&styled_files, width, color);
    crate::debug::trace("render", "layout done", t0);
    RenderOutput {
        styled_files,
        display_lines: lo.display_lines,
        raw_texts: lo.raw_texts,
        line_map: lo.line_map,
        file_starts: lo.file_starts,
        hunk_starts: lo.hunk_starts,
    }
}

/// Relayout existing styled files at a new width. Skips the expensive Phase 1.
pub(crate) fn relayout(styled_files: Vec<StyledFile>, width: usize, color: bool) -> RenderOutput {
    let lo = layout(&styled_files, width, color);
    RenderOutput {
        styled_files,
        display_lines: lo.display_lines,
        raw_texts: lo.raw_texts,
        line_map: lo.line_map,
        file_starts: lo.file_starts,
        hunk_starts: lo.hunk_starts,
    }
}

// ---------------------------------------------------------------------------
// Word-level diff utilities (unchanged)
// ---------------------------------------------------------------------------

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

/// Token product threshold for word-level diffs. Blocks larger than this
/// skip word-level highlighting (entire lines are highlighted instead).
/// Patience diff is O(n log n) for matching unique elements but falls back
/// to Myers O(n*d) for non-unique regions, so very large blocks still cost.
const WORD_DIFF_TOKEN_LIMIT: usize = 10_000;

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

    // Skip word-level diff for very large blocks — highlight entire lines instead
    if old_tokens.len() * new_tokens.len() > WORD_DIFF_TOKEN_LIMIT {
        return (
            vec![Vec::new(); block.deleted.len()],
            vec![Vec::new(); block.added.len()],
        );
    }

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
        assert_snapshot!(strip(output.lines()));
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
        assert_snapshot!(strip(output.lines()));
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
        assert_snapshot!(strip(output.lines()));
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
        assert_snapshot!(strip(output.lines()));
    }

    #[test]
    fn snapshot_untracked_file() {
        let file = diff::DiffFile::from_content("new.rs", "hello\nworld\n");
        let output = render(&[file], 80, false);
        assert_snapshot!(strip(output.lines()));
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
        assert_eq!(output.lines().len(), 5);
        assert_eq!(output.file_starts.len(), 1);
        assert_eq!(output.hunk_starts.len(), 1);
        assert_eq!(output.file_starts[0], 0);
        assert_eq!(output.hunk_starts[0], 1); // after file header, at first content line

        // Check the added line has a + marker
        let added_line = &output.lines()[2]; // header, ctx, added
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
            !output.lines().iter().any(std::string::String::is_empty),
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
        let added = &output.lines()[2];
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
            output.lines().len() > 3,
            "long line should wrap into multiple output lines, got {}",
            output.lines().len()
        );

        // Continuation lines should have blank gutter with separators
        let all = output.lines();
        let cont_lines: Vec<_> = all.iter().skip(3).collect();
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
        assert_eq!(output.lines().len(), output.line_map.len());
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
        let header = &output.lines()[0];
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

        for line in output.lines() {
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
        let all_lines = output.lines();
        let first_added = all_lines
            .iter()
            .position(|l| crate::ansi::strip_ansi(l).contains('+'))
            .expect("should find added line");
        let cont_lines: Vec<_> = all_lines
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

    #[test]
    fn render_deleted_file_shows_status_label() {
        let raw = "\
diff --git a/old.txt b/old.txt
deleted file mode 100644
--- a/old.txt
+++ /dev/null
@@ -1,2 +0,0 @@
-line1
-line2
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let header = crate::ansi::strip_ansi(&output.lines()[0]);
        assert!(
            header.contains("Deleted"),
            "file header should contain 'Deleted': {header:?}"
        );
        assert_eq!(files[0].status, FileStatus::Deleted);
        // Deleted lines should be present
        let deleted_count = output
            .line_map
            .iter()
            .filter(|li| li.line_kind == Some(LineKind::Deleted))
            .count();
        assert_eq!(deleted_count, 2, "should have 2 deleted lines");
    }

    #[test]
    fn render_renamed_file_shows_status_label() {
        let raw = "\
diff --git a/old.txt b/new.txt
similarity index 100%
rename from old.txt
rename to new.txt
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let header = crate::ansi::strip_ansi(&output.lines()[0]);
        assert!(
            header.contains("Renamed"),
            "file header should contain 'Renamed': {header:?}"
        );
        assert_eq!(files[0].status, FileStatus::Renamed);
        // Pure rename has no hunks — just the header line
        assert_eq!(output.lines().len(), 1);
    }

    #[test]
    fn render_data_len_matches_lines_multi_file() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,2 +1,3 @@
 ctx
+added
 more
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1,1 +1,2 @@
 keep
+new
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_eq!(
            output.display_lines.len(),
            output.lines().len(),
            "display_lines length should match lines() length"
        );
    }

    #[test]
    fn wrap_segment_mismatch_fewer_segments_no_panic() {
        let long_content = "x".repeat(120);
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
        let lines = output.lines();
        assert_eq!(
            lines.len(),
            output.line_map.len(),
            "lines().len() must equal line_map.len()"
        );
    }

    #[test]
    fn render_all_header_only_file_no_hunks() {
        let raw = "\
diff --git a/old.txt b/new.txt
similarity index 100%
rename from old.txt
rename to new.txt
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_eq!(output.lines().len(), 1, "header-only file should produce 1 line");
    }

    #[test]
    fn word_highlights_appear_in_rendered_output() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,2 +1,2 @@
 context
-old value
+new value
";
        let files = diff::parse(raw);
        let output = render(&files, 80, true);
        let lines = output.lines();

        let added_line = lines
            .iter()
            .find(|l| crate::ansi::strip_ansi(l).contains("+new"))
            .expect("should find the added line");
        assert!(
            added_line.contains(style::BG_ADDED_WORD),
            "added line should contain word-level highlight BG_ADDED_WORD: {added_line:?}"
        );

        let deleted_line = lines
            .iter()
            .find(|l| crate::ansi::strip_ansi(l).contains("-old"))
            .expect("should find the deleted line");
        assert!(
            deleted_line.contains(style::BG_DELETED_WORD),
            "deleted line should contain word-level highlight BG_DELETED_WORD: {deleted_line:?}"
        );
    }

    #[test]
    fn word_highlights_match_eager_computation() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,3 @@
 context
-let x = old_value;
+let x = new_value;
 more context
";
        let files = diff::parse(raw);
        let output = render(&files, 80, true);

        let hunk = &files[0].hunks[0];
        let blocks = find_change_blocks(hunk);
        assert_eq!(blocks.len(), 1, "should have exactly one change block");
        let (del_hl, add_hl) = word_highlights(hunk, &blocks[0]);

        assert!(!del_hl[0].is_empty(), "deleted line should have word highlights");
        assert!(!add_hl[0].is_empty(), "added line should have word highlights");

        let lines = output.lines();
        let deleted_line = lines
            .iter()
            .find(|l| crate::ansi::strip_ansi(l).contains("-let x = old_value"))
            .expect("should find deleted line");
        let added_line = lines
            .iter()
            .find(|l| crate::ansi::strip_ansi(l).contains("+let x = new_value"))
            .expect("should find added line");

        assert!(
            deleted_line.contains(style::BG_DELETED_WORD),
            "deleted line should have word highlight: {deleted_line:?}"
        );
        assert!(
            added_line.contains(style::BG_ADDED_WORD),
            "added line should have word highlight: {added_line:?}"
        );
    }

    #[test]
    fn colored_rendering_produces_ansi() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,3 @@
 context
-deleted line
+added line
";
        let files = diff::parse(raw);
        let output = render(&files, 80, true);

        let added_idx = output
            .line_map
            .iter()
            .position(|li| li.line_kind == Some(LineKind::Added))
            .expect("should have an added line");
        let deleted_idx = output
            .line_map
            .iter()
            .position(|li| li.line_kind == Some(LineKind::Deleted))
            .expect("should have a deleted line");

        let added_line = &output.lines()[added_idx];
        assert!(
            added_line.contains(style::BG_ADDED),
            "added line should contain BG_ADDED: {added_line:?}"
        );

        let deleted_line = &output.lines()[deleted_idx];
        assert!(
            deleted_line.contains(style::BG_DELETED),
            "deleted line should contain BG_DELETED: {deleted_line:?}"
        );
    }

    #[test]
    fn raw_texts_includes_structural_content() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,2 +1,3 @@
 ctx
+added
 more
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        assert_eq!(
            output.raw_texts.len(),
            output.lines().len(),
            "raw_texts length should match display lines length"
        );

        // Header slot (index 0) should contain the file path
        assert!(
            output.raw_texts[0].contains("foo.rs"),
            "header raw text should contain the file path: {:?}",
            output.raw_texts[0]
        );

        // Content slots should contain raw diff text
        let has_added = output.raw_texts.iter().any(|t| t == "added");
        assert!(has_added, "raw_texts should include 'added' content");

        let has_ctx = output.raw_texts.iter().any(|t| t == "ctx");
        assert!(has_ctx, "raw_texts should include 'ctx' content");
    }

    #[test]
    fn relayout_preserves_content_at_different_width() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,2 +1,3 @@
 ctx
+added
 more
";
        let files = diff::parse(raw);
        let output_80 = render(&files, 80, false);
        let output_120 = relayout(output_80.styled_files, 120, false);

        // Same number of source lines, potentially different display lines
        assert_eq!(output_120.file_starts.len(), 1);
        assert_eq!(output_120.hunk_starts.len(), 1);
        // Content should be the same
        let has_added = output_120.lines().iter().any(|l| l.contains("added"));
        assert!(has_added, "relayouted output should contain 'added'");
    }
}
