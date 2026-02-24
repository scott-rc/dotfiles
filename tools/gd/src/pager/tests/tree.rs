//! Tree building and rendering tests.

use crate::git::diff::FileStatus;
use insta::assert_debug_snapshot;
use insta::assert_snapshot;

use super::super::rendering::diff_area_width;
use super::super::tree::{
    TreeEntry, build_tree_entries, build_tree_lines, compute_connector_prefix, compute_tree_width,
    resolve_tree_layout, truncate_label,
};
use super::common::{entry, entry_with_status, make_diff_file, strip};
use crate::git::diff::DiffFile;

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
    let forbidden = format!("{} {}", crate::style::RESET, crate::style::BG_TREE_CURSOR);
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
    assert_eq!(
        dir_entry.unwrap().label,
        "src/lib",
        "single-child dirs should collapse"
    );
}

#[test]
fn test_compute_tree_width_empty() {
    assert_eq!(compute_tree_width(&[]), 0);
}

#[test]
fn test_compute_tree_width_returns_raw_content_width() {
    let long_label = "a".repeat(60);
    let entries = vec![TreeEntry {
        label: long_label.clone(),
        depth: 0,
        file_idx: Some(0),
        status: Some(FileStatus::Modified),
        collapsed: false,
    }];
    let width = compute_tree_width(&entries);
    // raw formula: (depth + 1) * 4 + 2 + status_extra + label.len() + 2
    // = (0 + 1) * 4 + 2 + 2 + 60 + 2 = 70
    let expected = (0 + 1) * 4 + 2 + 2 + long_label.len() + 2;
    assert_eq!(width, expected, "tree width should equal raw content width without capping");
}

#[test]
fn test_resolve_tree_layout_hides_when_no_dirs_and_few_files() {
    let result = resolve_tree_layout(30, 120, false, 3);
    assert_eq!(result, None, "should hide tree when no dirs and <4 files");
}

#[test]
fn test_resolve_tree_layout_shows_when_has_directories() {
    let result = resolve_tree_layout(30, 120, true, 2);
    assert_eq!(result, Some(30), "should show tree at content width when has directories");
}

#[test]
fn test_resolve_tree_layout_shows_when_many_files() {
    let result = resolve_tree_layout(30, 120, false, 4);
    assert_eq!(result, Some(30), "should show tree at content width when >=4 files");
}

#[test]
fn test_resolve_tree_layout_clamps_to_available_space() {
    // terminal_cols=110, MIN_DIFF_WIDTH=80 => allocated = 110 - 80 - 1 = 29
    let result = resolve_tree_layout(50, 110, true, 5);
    assert_eq!(result, Some(29), "should clamp to available space");
}

#[test]
fn test_resolve_tree_layout_terminal_cols_zero() {
    let result = resolve_tree_layout(30, 0, true, 5);
    assert_eq!(result, None, "should return None when terminal_cols is 0");
}

#[test]
fn test_resolve_tree_layout_content_width_zero() {
    let result = resolve_tree_layout(0, 120, true, 5);
    assert_eq!(result, None, "should return None when content_width is 0");
}

#[test]
fn test_resolve_tree_layout_exact_min_tree_width_boundary() {
    use super::super::tree::{MIN_DIFF_WIDTH, MIN_TREE_WIDTH};
    // terminal_cols such that allocated == MIN_TREE_WIDTH exactly
    let terminal_cols = MIN_TREE_WIDTH + MIN_DIFF_WIDTH + 1;
    let result = resolve_tree_layout(100, terminal_cols, true, 5);
    assert_eq!(
        result,
        Some(MIN_TREE_WIDTH),
        "should return Some(MIN_TREE_WIDTH) at exact boundary"
    );
}

#[test]
fn test_resolve_tree_layout_hides_when_terminal_too_narrow() {
    // terminal_cols=90, allocated = 90 - 80 - 1 = 9, which is < MIN_TREE_WIDTH (15)
    let result = resolve_tree_layout(30, 90, true, 5);
    assert_eq!(result, None, "should hide when allocated width < MIN_TREE_WIDTH");
}

#[test]
fn test_tree_status_symbol_modified() {
    let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Modified)];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped = crate::ansi::strip_ansi(&lines[0]);
    assert!(
        stripped.contains("M "),
        "modified entry should contain 'M ': {stripped:?}"
    );
}

#[test]
fn test_tree_status_symbol_added() {
    let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Added)];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped = crate::ansi::strip_ansi(&lines[0]);
    assert!(
        stripped.contains("A "),
        "added entry should contain 'A ': {stripped:?}"
    );
}

#[test]
fn test_tree_status_symbol_deleted() {
    let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Deleted)];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped = crate::ansi::strip_ansi(&lines[0]);
    assert!(
        stripped.contains("D "),
        "deleted entry should contain 'D ': {stripped:?}"
    );
}

#[test]
fn test_tree_status_symbol_renamed() {
    let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Renamed)];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped = crate::ansi::strip_ansi(&lines[0]);
    assert!(
        stripped.contains("R "),
        "renamed entry should contain 'R ': {stripped:?}"
    );
}

#[test]
fn test_tree_status_symbol_untracked() {
    let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Untracked)];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped = crate::ansi::strip_ansi(&lines[0]);
    assert!(
        stripped.contains("? "),
        "untracked entry should contain '? ': {stripped:?}"
    );
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
    assert!(
        !stripped.contains("M "),
        "directory must not show M: {stripped:?}"
    );
    assert!(
        !stripped.contains("A "),
        "directory must not show A: {stripped:?}"
    );
    assert!(
        !stripped.contains("D "),
        "directory must not show D: {stripped:?}"
    );
    assert!(
        !stripped.contains("R "),
        "directory must not show R: {stripped:?}"
    );
    assert!(
        !stripped.contains("? "),
        "directory must not show ?: {stripped:?}"
    );
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
    let files = vec![
        make_diff_file("a.rs"),
        make_diff_file("b.rs"),
        make_diff_file("c.rs"),
    ];
    assert_debug_snapshot!(build_tree_entries(&files));
}

#[test]
fn snapshot_tree_entries_nested() {
    let mut files = vec![
        make_diff_file("src/lib.rs"),
        make_diff_file("src/main.rs"),
        make_diff_file("README.md"),
    ];
    crate::git::sort_files_for_display(&mut files);
    assert_debug_snapshot!(build_tree_entries(&files));
}

#[test]
fn snapshot_tree_entries_single_child_collapse() {
    let mut files = vec![
        make_diff_file("src/lib/foo.rs"),
        make_diff_file("src/lib/bar.rs"),
        make_diff_file("tests/integration.rs"),
    ];
    crate::git::sort_files_for_display(&mut files);
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
    let mut files = vec![
        make_diff_file("src/lib.rs"),
        make_diff_file("src/main.rs"),
        make_diff_file("README.md"),
    ];
    crate::git::sort_files_for_display(&mut files);
    let entries = build_tree_entries(&files);
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped: Vec<String> = lines.iter().map(|l| strip(l)).collect();
    assert_snapshot!(stripped.join("\n"));
}

#[test]
fn test_truncate_label_no_op_when_fits() {
    assert_eq!(truncate_label("foo.rs", 10), "foo.rs");
}

#[test]
fn test_truncate_label_truncates_with_ellipsis() {
    assert_eq!(truncate_label("very_long_filename.rs", 10), "very_lon..");
}

#[test]
fn test_truncate_label_minimum_width() {
    assert_eq!(truncate_label("abcdef", 3), "a..");
}

#[test]
fn test_truncate_label_width_2() {
    assert_eq!(truncate_label("abcdef", 2), "..");
}

#[test]
fn test_truncate_label_width_1() {
    assert_eq!(truncate_label("abcdef", 1), ".");
}

#[test]
fn test_truncate_label_width_0() {
    assert_eq!(truncate_label("abcdef", 0), "");
}

#[test]
fn test_truncate_label_empty_input() {
    assert_eq!(truncate_label("", 5), "");
}

#[test]
fn test_truncate_label_char_boundary() {
    assert_eq!(truncate_label("cafe_resume", 7), "cafe_..");
}

#[test]
fn test_build_tree_lines_truncates_non_cursor_entries() {
    let entries = vec![
        entry("very_long_directory_name", 0, None),
        entry("a.rs", 1, Some(0)),
        entry("b.rs", 1, Some(1)),
        entry("extremely_long_filename_here.rs", 1, Some(2)),
        entry("another_long_filename_here.rs", 1, Some(3)),
    ];
    // Cursor at entry 0 (visible index 0), fisheye radius 2 covers visible 0-2.
    // Entries at visible indices 3 and 4 are beyond radius, should be truncated.
    let (lines, _) = build_tree_lines(&entries, 0, 25);
    let stripped3 = strip(&lines[3]);
    let stripped4 = strip(&lines[4]);
    assert!(
        stripped3.contains(".."),
        "entry beyond fisheye radius should be truncated: {stripped3:?}"
    );
    assert!(
        stripped4.contains(".."),
        "entry beyond fisheye radius should be truncated: {stripped4:?}"
    );
}

#[test]
fn test_build_tree_lines_expands_cursor_entry() {
    // With enough width, cursor entry shows full label
    let entries = vec![
        entry("very_long_directory_name", 0, None),
        entry("a.rs", 1, Some(0)),
        entry("b.rs", 1, Some(1)),
        entry("extremely_long_filename_here.rs", 1, Some(2)),
        entry("another_long_filename_here.rs", 1, Some(3)),
    ];
    let width = compute_tree_width(&entries);
    let (lines, _) = build_tree_lines(&entries, 0, width);
    let stripped0 = strip(&lines[0]);
    assert!(
        !stripped0.contains(".."),
        "cursor entry should NOT be truncated at full width: {stripped0:?}"
    );
}

#[test]
fn test_build_tree_lines_no_overflow() {
    let entries = vec![
        entry("very_long_directory_name", 0, None),
        entry("a.rs", 1, Some(0)),
        entry("b.rs", 1, Some(1)),
        entry("extremely_long_filename_here.rs", 1, Some(2)),
        entry("another_long_filename_here.rs", 1, Some(3)),
    ];
    let width = 25;
    let (lines, _) = build_tree_lines(&entries, 0, width);
    for (i, line) in lines.iter().enumerate() {
        let stripped = strip(line);
        let vis_width = stripped.chars().count();
        assert!(
            vis_width <= width,
            "line {i} overflows: {vis_width} > {width}: {stripped:?}"
        );
    }
}

#[test]
fn test_build_tree_lines_expanded_still_truncated_when_wider_than_panel() {
    // Panel width 20, cursor at entry 2 (fisheye covers 0-4).
    // Even expanded entries must not exceed the panel width.
    let entries = vec![
        entry("a_very_very_very_long_name.rs", 0, Some(0)),
        entry("b_very_very_very_long_name.rs", 0, Some(1)),
        entry("c_very_very_very_long_name.rs", 0, Some(2)),
        entry("d_very_very_very_long_name.rs", 0, Some(3)),
        entry("e_very_very_very_long_name.rs", 0, Some(4)),
    ];
    let width = 20;
    let (lines, _) = build_tree_lines(&entries, 2, width);
    for (i, line) in lines.iter().enumerate() {
        let stripped = strip(line);
        let vis_width = stripped.chars().count();
        assert!(
            vis_width <= width,
            "expanded line {i} overflows: {vis_width} > {width}: {stripped:?}"
        );
    }
}

#[test]
fn test_build_tree_lines_fisheye_gives_more_space_near_cursor() {
    let entries = vec![
        entry("a_very_long_filename_000.rs", 0, Some(0)),
        entry("b_very_long_filename_001.rs", 0, Some(1)),
        entry("c_very_long_filename_002.rs", 0, Some(2)),
        entry("d_very_long_filename_003.rs", 0, Some(3)),
        entry("e_very_long_filename_004.rs", 0, Some(4)),
        entry("f_very_long_filename_005.rs", 0, Some(5)),
    ];
    let width = 30;
    let (lines, _) = build_tree_lines(&entries, 3, width);
    // Entry 0 is far from cursor (distance 3), should be more truncated
    let distant = strip(&lines[0]).trim_end().to_string();
    assert!(
        distant.contains(".."),
        "distant entry should be truncated: {distant:?}"
    );
    // Cursor entry (3) should be expanded (more label budget)
    let cursor = strip(&lines[3]).trim_end().to_string();
    assert!(
        cursor.len() > distant.len(),
        "cursor entry should be wider than distant: cursor={cursor:?} distant={distant:?}"
    );
    // No line overflows the panel
    for (i, line) in lines.iter().enumerate() {
        let stripped = strip(line);
        assert!(
            stripped.chars().count() <= width,
            "line {i} overflows: {stripped:?}"
        );
    }
}

// ---- diff_area_width ----

#[test]
fn diff_area_width_tree_hidden_returns_cols() {
    assert_eq!(diff_area_width(120, 30, false, false), 120);
}

#[test]
fn diff_area_width_tree_visible_subtracts_tree_and_separator() {
    // cols - tree_width - 1 (separator)
    assert_eq!(diff_area_width(120, 30, true, false), 89);
}

#[test]
fn diff_area_width_tree_visible_with_scrollbar() {
    // cols - tree_width - 1 (separator) - 1 (scrollbar)
    assert_eq!(diff_area_width(120, 30, true, true), 88);
}

#[test]
fn diff_area_width_scrollbar_only_no_tree() {
    // cols - 1 (scrollbar)
    assert_eq!(diff_area_width(120, 30, false, true), 119);
}

#[test]
fn diff_area_width_saturates_on_small_cols() {
    // cols=10, tree_width=30: 10 - 30 - 1 saturates to 0
    assert_eq!(diff_area_width(10, 30, true, false), 0);
}

#[test]
fn diff_area_width_zero_cols() {
    assert_eq!(diff_area_width(0, 0, false, false), 0);
    assert_eq!(diff_area_width(0, 0, true, false), 0);
}
