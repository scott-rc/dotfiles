use crate::pager::state::{default_full_context, default_view_scope};
use crate::pager::reducer::handle_key;
use crate::pager::types::{FileIx, ViewScope};
use std::path::Path;
use tui::pager::Key;

mod full_context {
    use super::*;

    #[test]
    fn single_file_one_hunk() {
        assert!(default_full_context(1, 1));
    }

    #[test]
    fn single_file_three_hunks() {
        assert!(default_full_context(1, 3));
    }

    #[test]
    fn single_file_four_hunks() {
        assert!(!default_full_context(1, 4));
    }

    #[test]
    fn two_files_one_hunk_each() {
        assert!(!default_full_context(2, 2));
    }

    #[test]
    fn zero_files() {
        assert!(!default_full_context(0, 0));
    }

    #[test]
    fn single_file_zero_hunks() {
        assert!(default_full_context(1, 0));
    }
}

mod view_scope {
    use super::*;

    #[test]
    fn few_files_short_output_returns_all() {
        assert_eq!(default_view_scope(3, 50, 40), ViewScope::AllFiles);
    }

    #[test]
    fn six_files_returns_single() {
        assert_eq!(
            default_view_scope(6, 10, 40),
            ViewScope::SingleFile(FileIx(0))
        );
    }

    #[test]
    fn two_files_large_output_returns_single() {
        // 2 files, 200 lines, 40 rows -> 200 > 40 * 3 = 120
        assert_eq!(
            default_view_scope(2, 200, 40),
            ViewScope::SingleFile(FileIx(0))
        );
    }

    #[test]
    fn two_files_moderate_output_returns_all() {
        // 2 files, 100 lines, 40 rows -> 100 < 40 * 3 = 120
        assert_eq!(default_view_scope(2, 100, 40), ViewScope::AllFiles);
    }

    #[test]
    fn one_file_huge_output_returns_all() {
        // Single file always returns AllFiles regardless of size
        assert_eq!(default_view_scope(1, 10000, 40), ViewScope::AllFiles);
    }

    #[test]
    fn five_files_exactly_returns_all_when_short() {
        assert_eq!(default_view_scope(5, 50, 40), ViewScope::AllFiles);
    }

    #[test]
    fn five_files_long_output_returns_single() {
        // 5 files, 200 lines, 40 rows -> 200 > 40 * 3 = 120
        assert_eq!(
            default_view_scope(5, 200, 40),
            ViewScope::SingleFile(FileIx(0))
        );
    }

    #[test]
    fn boundary_exactly_three_times_terminal_returns_all() {
        // 2 files, 120 lines, 40 rows -> 120 == 40 * 3, not strictly greater
        assert_eq!(default_view_scope(2, 120, 40), ViewScope::AllFiles);
    }

    #[test]
    fn boundary_one_over_three_times_terminal_returns_single() {
        // 2 files, 121 lines, 40 rows -> 121 > 120
        assert_eq!(
            default_view_scope(2, 121, 40),
            ViewScope::SingleFile(FileIx(0))
        );
    }
}

mod user_override_flags {
    use super::*;
    use super::super::common::make_keybinding_state;

    fn p() -> &'static Path {
        Path::new(".")
    }

    #[test]
    fn toggle_single_file_sets_user_flag() {
        let mut state = make_keybinding_state();
        assert!(!state.view_scope_user_set);
        handle_key(&mut state, Key::Char('s'), 40, 40, 120, &[], p());
        assert!(state.view_scope_user_set);
    }

    #[test]
    fn toggle_full_context_sets_user_flag() {
        let mut state = make_keybinding_state();
        assert!(!state.full_context_user_set);
        handle_key(&mut state, Key::Char('o'), 40, 40, 120, &[], p());
        assert!(state.full_context_user_set);
    }

    #[test]
    fn flags_start_false() {
        let state = make_keybinding_state();
        assert!(!state.view_scope_user_set);
        assert!(!state.full_context_user_set);
    }
}
