//! Characterization tests for `pager::text` — char boundary helpers.

use crate::pager::text::*;

#[cfg(test)]
mod prev_char_boundary_tests {
    use super::*;

    #[test]
    fn empty_string_pos_zero() {
        assert_eq!(prev_char_boundary("", 0), 0);
    }

    #[test]
    fn single_ascii_pos_zero() {
        assert_eq!(prev_char_boundary("a", 0), 0);
    }

    #[test]
    fn single_ascii_pos_one() {
        assert_eq!(prev_char_boundary("a", 1), 0);
    }

    #[test]
    fn two_ascii_pos_two() {
        assert_eq!(prev_char_boundary("ab", 2), 1);
    }

    #[test]
    fn two_byte_char_interior_pos() {
        let s = "\u{00E9}"; // é — 2 bytes (0xC3 0xA9)
        assert_eq!(prev_char_boundary(s, 1), 0);
    }

    #[test]
    fn two_byte_char_pos_zero() {
        let s = "\u{00E9}";
        assert_eq!(prev_char_boundary(s, 0), 0);
    }

    #[test]
    fn pos_beyond_len_clamps() {
        let s = "\u{00E9}"; // 2 bytes
        assert_eq!(prev_char_boundary(s, 10), 0);
    }

    #[test]
    fn three_byte_char_interior() {
        let s = "\u{2603}"; // ☃ — 3 bytes
        assert_eq!(prev_char_boundary(s, 2), 0);
    }

    #[test]
    fn four_byte_char_interior() {
        let s = "\u{1F600}"; // 😀 — 4 bytes
        assert_eq!(prev_char_boundary(s, 2), 0);
    }
}

#[cfg(test)]
mod next_char_boundary_tests {
    use super::*;

    #[test]
    fn empty_string_pos_zero() {
        assert_eq!(next_char_boundary("", 0), 0);
    }

    #[test]
    fn single_ascii_pos_zero() {
        assert_eq!(next_char_boundary("a", 0), 1);
    }

    #[test]
    fn single_ascii_pos_at_end() {
        assert_eq!(next_char_boundary("a", 1), 1);
    }

    #[test]
    fn two_byte_char_interior_pos() {
        let s = "\u{00E9}"; // 2 bytes
        assert_eq!(next_char_boundary(s, 1), 2);
    }

    #[test]
    fn two_byte_char_pos_at_end() {
        let s = "\u{00E9}";
        assert_eq!(next_char_boundary(s, 2), 2);
    }

    #[test]
    fn pos_beyond_len_clamps() {
        let s = "\u{00E9}"; // 2 bytes
        assert_eq!(next_char_boundary(s, 10), 2);
    }

    #[test]
    fn three_byte_char_from_start() {
        let s = "\u{2603}"; // ☃ — 3 bytes
        assert_eq!(next_char_boundary(s, 0), 3);
    }

    #[test]
    fn four_byte_char_from_interior() {
        let s = "\u{1F600}"; // 😀 — 4 bytes
        assert_eq!(next_char_boundary(s, 1), 4);
    }
}

#[cfg(test)]
mod clamp_cursor_to_boundary_tests {
    use super::*;

    #[test]
    fn empty_string_cursor_zero() {
        assert_eq!(clamp_cursor_to_boundary("", 0), 0);
    }

    #[test]
    fn single_ascii_cursor_zero() {
        assert_eq!(clamp_cursor_to_boundary("a", 0), 0);
    }

    #[test]
    fn single_ascii_cursor_one() {
        assert_eq!(clamp_cursor_to_boundary("a", 1), 1);
    }

    #[test]
    fn mid_two_byte_char_clamps_before() {
        let s = "\u{00E9}"; // 2 bytes
        assert_eq!(clamp_cursor_to_boundary(s, 1), 0);
    }

    #[test]
    fn cursor_beyond_len_clamps_to_last_boundary() {
        let s = "\u{00E9}"; // 2 bytes
        assert_eq!(clamp_cursor_to_boundary(s, 10), 2);
    }

    #[test]
    fn four_byte_char_mid_clamps_to_start() {
        let s = "\u{1F600}"; // 😀 — 4 bytes
        assert_eq!(clamp_cursor_to_boundary(s, 2), 0);
    }

    #[test]
    fn ascii_after_multibyte_cursor_valid() {
        let s = "\u{00E9}a"; // 2-byte + 1-byte = 3 bytes total
        assert_eq!(clamp_cursor_to_boundary(s, 2), 2);
    }
}
