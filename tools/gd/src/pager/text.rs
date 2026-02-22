// Pure string/char boundary helpers to avoid circular deps between search and rendering.

pub(crate) fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut pos = pos.min(s.len());
    if pos == 0 {
        return 0;
    }
    pos -= 1;
    while pos > 0 && !s.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

pub(crate) fn next_char_boundary(s: &str, pos: usize) -> usize {
    let mut pos = pos.min(s.len());
    if pos == s.len() {
        return pos;
    }
    pos += 1;
    while pos < s.len() && !s.is_char_boundary(pos) {
        pos += 1;
    }
    pos
}

pub(crate) fn clamp_cursor_to_boundary(s: &str, cursor: usize) -> usize {
    let mut cursor = cursor.min(s.len());
    while cursor > 0 && !s.is_char_boundary(cursor) {
        cursor -= 1;
    }
    cursor
}
