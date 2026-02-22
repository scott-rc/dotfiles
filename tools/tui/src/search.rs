use crate::ansi::{split_ansi, strip_ansi};

const REVERSE: &str = "\x1b[7m";
const NO_REVERSE: &str = "\x1b[27m";

/// Highlight all occurrences of `query` in `line` with reverse video,
/// preserving existing ANSI codes. Case-insensitive matching.
pub fn highlight_search(line: &str, query: &str) -> String {
    if query.is_empty() {
        return line.to_string();
    }

    let stripped = strip_ansi(line);
    let lower_stripped = stripped.to_lowercase();
    let lower_query = query.to_lowercase();

    let mut match_ranges: Vec<(usize, usize)> = Vec::new();
    let mut start = 0;
    while let Some(pos) = lower_stripped[start..].find(&lower_query) {
        let abs = start + pos;
        match_ranges.push((abs, abs + query.len()));
        start = abs + 1;
    }

    if match_ranges.is_empty() {
        return line.to_string();
    }

    // Build position map: visible char index -> byte index in original string
    let mut vis_to_orig: Vec<usize> = Vec::new();
    let segments = split_ansi(line);
    let mut orig_pos = 0;
    for seg in &segments {
        if seg.starts_with('\x1b') {
            orig_pos += seg.len();
        } else {
            for (i, _) in seg.char_indices() {
                vis_to_orig.push(orig_pos + i);
            }
            orig_pos += seg.len();
        }
    }
    // Sentinel for end-of-string insertions
    vis_to_orig.push(orig_pos);

    let mut insertions: Vec<(usize, &str)> = Vec::new();
    for (mstart, mend) in &match_ranges {
        if *mend <= vis_to_orig.len() && *mstart < vis_to_orig.len() {
            let orig_start = vis_to_orig[*mstart];
            let orig_end = vis_to_orig[*mend];
            insertions.push((orig_end, NO_REVERSE));
            insertions.push((orig_start, REVERSE));
        }
    }
    insertions.sort_by(|a, b| b.0.cmp(&a.0));

    let mut result = line.to_string();
    for (pos, code) in insertions {
        if pos <= result.len() {
            result.insert_str(pos, code);
        }
    }

    result
}

/// Find all line indices containing `query` (case-insensitive, ANSI-stripped).
pub fn find_matches(lines: &[String], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return Vec::new();
    }
    let lower = query.to_lowercase();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| strip_ansi(line).to_lowercase().contains(&lower))
        .map(|(i, _)| i)
        .collect()
}

/// Find the index into `matches` of the nearest match at or after `top_line`.
/// Returns -1 if `matches` is empty, or the last index if all matches are before `top_line`.
pub fn find_nearest_match(matches: &[usize], top_line: usize) -> isize {
    if matches.is_empty() {
        return -1;
    }
    for (i, &m) in matches.iter().enumerate() {
        if m >= top_line {
            return isize::try_from(i).unwrap_or(isize::MAX);
        }
    }
    isize::try_from(matches.len().saturating_sub(1)).unwrap_or(isize::MAX)
}

/// Move cursor left to the previous word boundary.
/// Skips spaces left, then skips non-spaces left.
pub fn word_boundary_left(text: &str, cursor: usize) -> usize {
    let bytes = text.as_bytes();
    let mut pos = cursor;
    // Skip spaces left
    while pos > 0 && bytes[pos - 1] == b' ' {
        pos -= 1;
    }
    // Skip non-spaces left
    while pos > 0 && bytes[pos - 1] != b' ' {
        pos -= 1;
    }
    pos
}

/// Move cursor right to the next word boundary.
/// Skips non-spaces right, then skips spaces right.
pub fn word_boundary_right(text: &str, cursor: usize) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut pos = cursor;
    // Skip non-spaces right
    while pos < len && bytes[pos] != b' ' {
        pos += 1;
    }
    // Skip spaces right
    while pos < len && bytes[pos] == b' ' {
        pos += 1;
    }
    pos
}

/// Calculate the maximum scroll position for content that overflows the viewport.
/// Returns 0 when content fits within `content_height`.
pub fn max_scroll(line_count: usize, content_height: usize) -> usize {
    if line_count > content_height {
        line_count - content_height + content_height / 2
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- highlight_search ----

    #[test]
    fn test_highlight_search_empty_query() {
        let input = "hello world";
        assert_eq!(highlight_search(input, ""), input);
    }

    #[test]
    fn test_highlight_search_no_match() {
        let input = "hello world";
        assert_eq!(highlight_search(input, "xyz"), input);
    }

    #[test]
    fn test_highlight_search_single_match() {
        let result = highlight_search("hello world", "world");
        assert_eq!(result, "hello \x1b[7mworld\x1b[27m");
    }

    #[test]
    fn test_highlight_search_case_insensitive() {
        let result = highlight_search("hello foo bar", "FOO");
        assert_eq!(result, "hello \x1b[7mfoo\x1b[27m bar");
    }

    // ---- find_matches ----

    #[test]
    fn test_find_matches_empty_query() {
        let lines = vec!["hello".into(), "world".into()];
        assert_eq!(find_matches(&lines, ""), Vec::<usize>::new());
    }

    #[test]
    fn test_find_matches_some_hits() {
        let lines = vec![
            "apple pie".into(),
            "banana split".into(),
            "apple sauce".into(),
        ];
        assert_eq!(find_matches(&lines, "apple"), vec![0, 2]);
    }

    #[test]
    fn test_find_matches_with_ansi() {
        let lines = vec![
            "\x1b[31mred apple\x1b[0m".into(),
            "plain banana".into(),
            "\x1b[32mgreen apple\x1b[0m".into(),
        ];
        assert_eq!(find_matches(&lines, "apple"), vec![0, 2]);
    }

    // ---- find_nearest_match ----

    #[test]
    fn test_find_nearest_match_empty() {
        assert_eq!(find_nearest_match(&[], 5), -1);
    }

    #[test]
    fn test_find_nearest_match_at_top() {
        let matches = vec![3, 7, 12];
        assert_eq!(find_nearest_match(&matches, 5), 1); // 7 is at index 1
    }

    #[test]
    fn test_find_nearest_match_past_end() {
        let matches = vec![3, 7, 12];
        assert_eq!(find_nearest_match(&matches, 20), 2); // last index
    }

    // ---- word_boundary ----

    #[test]
    fn test_word_boundary_left_from_end() {
        // "hello world" cursor at 11 (end) -> should go to 6
        assert_eq!(word_boundary_left("hello world", 11), 6);
    }

    #[test]
    fn test_word_boundary_left_at_start() {
        assert_eq!(word_boundary_left("hello world", 0), 0);
    }

    #[test]
    fn test_word_boundary_right_from_start() {
        // "hello world" cursor at 0 -> skip "hello" then space -> 6
        assert_eq!(word_boundary_right("hello world", 0), 6);
    }

    #[test]
    fn test_word_boundary_right_at_end() {
        let text = "hello world";
        assert_eq!(word_boundary_right(text, text.len()), text.len());
    }

    // ---- max_scroll ----

    #[test]
    fn test_max_scroll_content_fits() {
        assert_eq!(max_scroll(10, 24), 0);
    }

    #[test]
    fn test_max_scroll_exact_fit() {
        assert_eq!(max_scroll(24, 24), 0);
    }

    #[test]
    fn test_max_scroll_barely_exceeds() {
        // 25 - 24 + 12 = 13
        assert_eq!(max_scroll(25, 24), 13);
    }

    #[test]
    fn test_max_scroll_exceeds() {
        // 50 - 24 + 12 = 38
        assert_eq!(max_scroll(50, 24), 38);
    }

    #[test]
    fn test_max_scroll_no_lines() {
        assert_eq!(max_scroll(0, 24), 0);
    }

    #[test]
    fn test_max_scroll_odd_content_height() {
        // 50 - 25 + 12 = 37
        assert_eq!(max_scroll(50, 25), 37);
    }
}
