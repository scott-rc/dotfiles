// Pure line-map helpers: no PagerState, avoids circular deps.

use crate::render::LineInfo;

/// Returns true if the line at `idx` is a content line (Added, Deleted, or Context).
pub(crate) fn is_content_line(line_map: &[LineInfo], idx: usize) -> bool {
    line_map.get(idx).is_some_and(|li| li.line_kind.is_some())
}

/// Scan forward from `from` to find the next content line, clamped to `max`.
pub(crate) fn next_content_line(line_map: &[LineInfo], from: usize, max: usize) -> usize {
    let mut i = from;
    while i <= max {
        if is_content_line(line_map, i) {
            return i;
        }
        i += 1;
    }
    from
}

/// Scan backward from `from` to find the previous content line, clamped to `min`.
pub(crate) fn prev_content_line(line_map: &[LineInfo], from: usize, min: usize) -> usize {
    let mut i = from;
    loop {
        if is_content_line(line_map, i) {
            return i;
        }
        if i <= min {
            break;
        }
        i -= 1;
    }
    from
}

/// Snap `pos` to the nearest content line within `[range_start, range_end]`.
pub(crate) fn snap_to_content(
    line_map: &[LineInfo],
    pos: usize,
    range_start: usize,
    range_end: usize,
) -> usize {
    let fwd = next_content_line(line_map, pos, range_end);
    if is_content_line(line_map, fwd) {
        return fwd;
    }
    prev_content_line(line_map, pos, range_start)
}
