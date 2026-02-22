//! Characterization tests for `pager::types` — newtype index validation.

use crate::pager::types::{FileIx, LineIx, TreeEntryIx};

#[cfg(test)]
mod file_ix_tests {
    use super::*;

    #[test]
    fn valid_zero_index() {
        let ix = FileIx::new(0, 5);
        assert!(ix.is_some());
        assert_eq!(ix.unwrap().get(), 0);
    }

    #[test]
    fn valid_at_boundary() {
        let ix = FileIx::new(4, 5);
        assert!(ix.is_some());
        assert_eq!(ix.unwrap().get(), 4);
    }

    #[test]
    fn out_of_bounds() {
        assert!(FileIx::new(5, 5).is_none());
    }

    #[test]
    fn zero_count() {
        assert!(FileIx::new(0, 0).is_none());
    }
}

#[cfg(test)]
mod line_ix_tests {
    use super::*;

    #[test]
    fn valid_zero_index() {
        let ix = LineIx::new(0, 5);
        assert!(ix.is_some());
        assert_eq!(ix.unwrap().get(), 0);
    }

    #[test]
    fn valid_at_boundary() {
        let ix = LineIx::new(4, 5);
        assert!(ix.is_some());
        assert_eq!(ix.unwrap().get(), 4);
    }

    #[test]
    fn out_of_bounds() {
        assert!(LineIx::new(5, 5).is_none());
    }

    #[test]
    fn zero_count() {
        assert!(LineIx::new(0, 0).is_none());
    }
}

#[cfg(test)]
mod tree_entry_ix_tests {
    use super::*;

    #[test]
    fn valid_zero_index() {
        let ix = TreeEntryIx::new(0, 5);
        assert!(ix.is_some());
        assert_eq!(ix.unwrap().get(), 0);
    }

    #[test]
    fn valid_at_boundary() {
        let ix = TreeEntryIx::new(4, 5);
        assert!(ix.is_some());
        assert_eq!(ix.unwrap().get(), 4);
    }

    #[test]
    fn out_of_bounds() {
        assert!(TreeEntryIx::new(5, 5).is_none());
    }

    #[test]
    fn zero_count() {
        assert!(TreeEntryIx::new(0, 0).is_none());
    }
}
