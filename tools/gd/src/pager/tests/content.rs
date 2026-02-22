//! Characterization tests for `pager::content` — pure line-map helpers.

use super::common::{make_line_map, make_line_map_with_headers};
use crate::git::diff::LineKind;
use crate::pager::content::*;

#[cfg(test)]
mod is_content_line_tests {
    use super::*;

    #[test]
    fn empty_line_map() {
        let map = make_line_map(&[]);
        assert!(!is_content_line(&map, 0));
    }

    #[test]
    fn out_of_bounds() {
        let map = make_line_map(&[Some(LineKind::Context)]);
        assert!(!is_content_line(&map, 5));
    }

    #[test]
    fn context_line_is_content() {
        let map = make_line_map(&[Some(LineKind::Context)]);
        assert!(is_content_line(&map, 0));
    }

    #[test]
    fn added_line_is_content() {
        let map = make_line_map(&[Some(LineKind::Added)]);
        assert!(is_content_line(&map, 0));
    }

    #[test]
    fn deleted_line_is_content() {
        let map = make_line_map(&[Some(LineKind::Deleted)]);
        assert!(is_content_line(&map, 0));
    }

    #[test]
    fn header_line_is_not_content() {
        let map = make_line_map(&[None]);
        assert!(!is_content_line(&map, 0));
    }
}

#[cfg(test)]
mod next_content_line_tests {
    use super::*;

    #[test]
    fn empty_line_map_returns_from() {
        let map = make_line_map(&[]);
        assert_eq!(next_content_line(&map, 0, 0), 0);
    }

    #[test]
    fn single_content_at_zero() {
        let map = make_line_map(&[Some(LineKind::Context)]);
        assert_eq!(next_content_line(&map, 0, 0), 0);
    }

    #[test]
    fn all_none_returns_from() {
        let map = make_line_map(&[None, None, None, None, None, None]);
        assert_eq!(next_content_line(&map, 0, 5), 0);
    }

    #[test]
    fn headers_then_content() {
        let map = make_line_map_with_headers();
        // 0,1 are headers; 2 is Context
        assert_eq!(next_content_line(&map, 0, 8), 2);
    }

    #[test]
    fn from_middle_finds_next() {
        let map = make_line_map_with_headers();
        // 5,6,7 are None; 8 is Added
        assert_eq!(next_content_line(&map, 5, 8), 8);
    }

    #[test]
    fn from_past_last_content_returns_from() {
        let map = make_line_map_with_headers();
        // 7 is header, 8 is Added — from=7, max=8 → finds 8
        assert_eq!(next_content_line(&map, 7, 8), 8);
    }
}

#[cfg(test)]
mod prev_content_line_tests {
    use super::*;

    #[test]
    fn empty_line_map_returns_from() {
        let map = make_line_map(&[]);
        assert_eq!(prev_content_line(&map, 0, 0), 0);
    }

    #[test]
    fn all_non_content_returns_from() {
        let map = make_line_map(&[None, None, None, None, None, None]);
        assert_eq!(prev_content_line(&map, 5, 0), 5);
    }

    #[test]
    fn from_end_finds_last_content() {
        let map = make_line_map_with_headers();
        // 8 is Added — from=8, min=0 → 8
        assert_eq!(prev_content_line(&map, 8, 0), 8);
    }

    #[test]
    fn from_header_finds_previous_content() {
        let map = make_line_map_with_headers();
        // from=5 (None), prev content is 4 (Deleted)
        assert_eq!(prev_content_line(&map, 5, 0), 4);
    }

    #[test]
    fn from_one_no_content_before_returns_from() {
        let map = make_line_map_with_headers();
        // from=1 (None), min=0; idx 1 is None, idx 0 is None → returns from=1
        assert_eq!(prev_content_line(&map, 1, 0), 1);
    }
}

#[cfg(test)]
mod snap_to_content_tests {
    use super::*;

    #[test]
    fn no_content_lines_returns_prev_result() {
        let map = make_line_map(&[None, None, None]);
        // No content anywhere — forward fails, falls back to prev which also fails → returns pos
        assert_eq!(snap_to_content(&map, 1, 0, 2), 1);
    }

    #[test]
    fn pos_on_content_returns_pos() {
        let map = make_line_map_with_headers();
        // pos=3 is Added
        assert_eq!(snap_to_content(&map, 3, 0, 8), 3);
    }

    #[test]
    fn pos_between_content_snaps_forward() {
        let map = make_line_map_with_headers();
        // pos=5 (None); forward finds 8 (Added)
        assert_eq!(snap_to_content(&map, 5, 0, 8), 8);
    }

    #[test]
    fn pos_past_all_content_snaps_backward() {
        let map = make_line_map_with_headers();
        // range_end=4 so forward from 5 won't find anything within range
        // but snap_to_content uses next_content_line(map, 5, 4) which checks 5..=4 → returns 5
        // is_content_line(map, 5) → false, so falls to prev_content_line(map, 5, 0) → 4
        assert_eq!(snap_to_content(&map, 5, 0, 4), 4);
    }
}
