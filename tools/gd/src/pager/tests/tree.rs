//! Tree building and rendering tests.

use crate::git::diff::FileStatus;
use insta::assert_debug_snapshot;
use insta::assert_snapshot;

use super::super::tree::{
    build_tree_entries, build_tree_lines, compute_connector_prefix, compute_tree_width,
    TreeEntry,
};
use crate::git::diff::DiffFile;
use super::common::{entry, entry_with_status, make_diff_file, strip};

#[test]
fn test_compute_connector_prefix_flat() {
    let entries = [
        entry("a.rs", 0, Some(0)),
        entry("b.rs", 0, Some(1)),
        entry("c.rs", 0, Some(2)),
    ];
    let refs: Vec<&TreeEntry> = entries.iter().collect();
    assert_eq!(compute_connector_prefix(&refs, 0), "├── ");
    assert_eq!(compute_connector_prefix(&refs, 1), "├── ");
    assert_eq!(compute_connector_prefix(&refs, 2), "└── ");
}

#[test]
fn test_compute_connector_prefix_nested() {
    let entries = [
        entry("src", 0, None),
        entry("a.rs", 1, Some(0)),
        entry("b.rs", 1, Some(1)),
        entry("README.md", 0, Some(2)),
    ];
    let refs: Vec<&TreeEntry> = entries.iter().collect();
    assert_eq!(compute_connector_prefix(&refs, 0), "├── ");
    assert_eq!(compute_connector_prefix(&refs, 1), "│   ├── ");
    assert_eq!(compute_connector_prefix(&refs, 2), "│   └── ");
    assert_eq!(compute_connector_prefix(&refs, 3), "└── ");
}

#[test]
fn test_build_tree_lines_no_header() {
    let entries = vec![entry("a.rs", 0, Some(0)), entry("b.rs", 0, Some(1))];
    let width = compute_tree_width(&entries);
    let (lines, _mapping) = build_tree_lines(&entries, 0, width);
    let first = crate::ansi::strip_ansi(&lines[0]);
    assert!(!first.contains("CHANGED FILES"), "header should be removed");
}

#[test]
fn test_tree_cursor_line_continuous_background() {
    let entries = vec![entry("a.rs", 0, Some(0)), entry("b.rs", 0, Some(1))];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let cursor_line = &lines[0];
    let forbidden = format!(
        "{} {}",
        crate::style::RESET,
        crate::style::BG_TREE_CURSOR
    );
    assert!(
        !cursor_line.contains(&forbidden),
        "cursor line has a background gap between icon and label:\n{cursor_line}"
    );
}

#[test]
fn test_build_tree_entries_flat_files() {
    let files = vec![make_diff_file("a.rs"), make_diff_file("b.rs")];
    let entries = build_tree_entries(&files);
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|e| e.depth == 0));
    assert!(entries.iter().all(|e| e.file_idx.is_some()));
}

#[test]
fn test_build_tree_entries_nested() {
    let files = vec![make_diff_file("src/a.rs"), make_diff_file("src/b.rs")];
    let entries = build_tree_entries(&files);
    let dir_entry = entries.iter().find(|e| e.file_idx.is_none());
    assert!(dir_entry.is_some(), "should have a directory entry");
    assert_eq!(dir_entry.unwrap().label, "src");
    let file_entries: Vec<_> = entries.iter().filter(|e| e.file_idx.is_some()).collect();
    assert_eq!(file_entries.len(), 2);
    assert!(file_entries.iter().all(|e| e.depth == 1));
}

#[test]
fn test_build_tree_entries_single_child_collapse() {
    let files = vec![make_diff_file("src/lib/foo.rs")];
    let entries = build_tree_entries(&files);
    let dir_entry = entries.iter().find(|e| e.file_idx.is_none());
    assert!(dir_entry.is_some());
    assert_eq!(dir_entry.unwrap().label, "src/lib", "single-child dirs should collapse");
}

#[test]
fn test_compute_tree_width_empty() {
    assert_eq!(compute_tree_width(&[]), 0);
}

#[test]
fn test_compute_tree_width_capped_at_40() {
    let long_label = "a".repeat(60);
    let entries = vec![TreeEntry {
        label: long_label,
        depth: 0,
        file_idx: Some(0),
        status: Some(FileStatus::Modified),
        collapsed: false,
    }];
    let width = compute_tree_width(&entries);
    assert_eq!(width, 40, "tree width should be capped at 40");
}

#[test]
fn test_tree_status_symbol_modified() {
    let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Modified)];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped = crate::ansi::strip_ansi(&lines[0]);
    assert!(stripped.contains("M "), "modified entry should contain 'M ': {stripped:?}");
}

#[test]
fn test_tree_status_symbol_added() {
    let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Added)];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped = crate::ansi::strip_ansi(&lines[0]);
    assert!(stripped.contains("A "), "added entry should contain 'A ': {stripped:?}");
}

#[test]
fn test_tree_status_symbol_deleted() {
    let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Deleted)];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped = crate::ansi::strip_ansi(&lines[0]);
    assert!(stripped.contains("D "), "deleted entry should contain 'D ': {stripped:?}");
}

#[test]
fn test_tree_status_symbol_renamed() {
    let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Renamed)];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped = crate::ansi::strip_ansi(&lines[0]);
    assert!(stripped.contains("R "), "renamed entry should contain 'R ': {stripped:?}");
}

#[test]
fn test_tree_status_symbol_untracked() {
    let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Untracked)];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped = crate::ansi::strip_ansi(&lines[0]);
    assert!(stripped.contains("? "), "untracked entry should contain '? ': {stripped:?}");
}

#[test]
fn test_tree_status_symbol_directory() {
    let entries = vec![TreeEntry {
        label: "src".to_string(),
        depth: 0,
        file_idx: None,
        status: None,
        collapsed: false,
    }];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped = crate::ansi::strip_ansi(&lines[0]);
    assert!(!stripped.contains("M "), "directory must not show M: {stripped:?}");
    assert!(!stripped.contains("A "), "directory must not show A: {stripped:?}");
    assert!(!stripped.contains("D "), "directory must not show D: {stripped:?}");
    assert!(!stripped.contains("R "), "directory must not show R: {stripped:?}");
    assert!(!stripped.contains("? "), "directory must not show ?: {stripped:?}");
}

#[test]
fn test_compute_tree_width_includes_status_chars() {
    let entry_with = entry_with_status("foo.rs", 0, 0, FileStatus::Modified);
    let entry_without = TreeEntry {
        label: "foo.rs".to_string(),
        depth: 0,
        file_idx: Some(0),
        status: None,
        collapsed: false,
    };
    let width_with = compute_tree_width(&[entry_with]);
    let width_without = compute_tree_width(&[entry_without]);
    assert_eq!(
        width_with,
        width_without + 2,
        "status symbol adds 2 columns (char + space)"
    );
}

#[test]
fn snapshot_tree_entries_flat() {
    let files = vec![make_diff_file("a.rs"), make_diff_file("b.rs"), make_diff_file("c.rs")];
    assert_debug_snapshot!(build_tree_entries(&files));
}

#[test]
fn snapshot_tree_entries_nested() {
    let files = vec![
        make_diff_file("src/lib.rs"),
        make_diff_file("src/main.rs"),
        make_diff_file("README.md"),
    ];
    assert_debug_snapshot!(build_tree_entries(&files));
}

#[test]
fn snapshot_tree_entries_single_child_collapse() {
    let files = vec![
        make_diff_file("src/lib/foo.rs"),
        make_diff_file("src/lib/bar.rs"),
        make_diff_file("tests/integration.rs"),
    ];
    assert_debug_snapshot!(build_tree_entries(&files));
}

#[test]
fn snapshot_tree_entries_with_status() {
    let files = vec![
        DiffFile {
            old_path: Some("a.rs".into()),
            new_path: Some("a.rs".into()),
            status: FileStatus::Modified,
            hunks: vec![],
        },
        DiffFile {
            old_path: None,
            new_path: Some("b.rs".into()),
            status: FileStatus::Added,
            hunks: vec![],
        },
        DiffFile {
            old_path: Some("c.rs".into()),
            new_path: None,
            status: FileStatus::Deleted,
            hunks: vec![],
        },
        DiffFile {
            old_path: Some("d.rs".into()),
            new_path: Some("e.rs".into()),
            status: FileStatus::Renamed,
            hunks: vec![],
        },
        DiffFile {
            old_path: None,
            new_path: Some("f.rs".into()),
            status: FileStatus::Untracked,
            hunks: vec![],
        },
    ];
    assert_debug_snapshot!(build_tree_entries(&files));
}

#[test]
fn snapshot_tree_lines_flat() {
    let entries = vec![
        entry("a.rs", 0, Some(0)),
        entry("b.rs", 0, Some(1)),
        entry("c.rs", 0, Some(2)),
    ];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped: Vec<String> = lines.iter().map(|l| strip(l)).collect();
    assert_snapshot!(stripped.join("\n"));
}

#[test]
fn snapshot_tree_lines_nested() {
    let files = vec![
        make_diff_file("src/lib.rs"),
        make_diff_file("src/main.rs"),
        make_diff_file("README.md"),
    ];
    let entries = build_tree_entries(&files);
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped: Vec<String> = lines.iter().map(|l| strip(l)).collect();
    assert_snapshot!(stripped.join("\n"));
}
