use std::collections::HashSet;

use super::diff::{DiffFile, DiffHunk, LineKind};

/// Generate a unified diff patch for a single hunk, with only the selected changed lines.
///
/// - Context lines always emit as context (` `)
/// - Selected added lines emit as `+`
/// - Non-selected added lines are dropped entirely
/// - Selected deleted lines emit as `-`
/// - Non-selected deleted lines are converted to context (` `)
pub(crate) fn generate_line_patch(
    file: &DiffFile,
    hunk: &DiffHunk,
    selected: &HashSet<usize>,
) -> String {
    let mut body = String::new();
    let mut old_count: u32 = 0;
    let mut new_count: u32 = 0;

    for (i, line) in hunk.lines.iter().enumerate() {
        match line.kind {
            LineKind::Context => {
                body.push(' ');
                body.push_str(&line.content);
                body.push('\n');
                old_count += 1;
                new_count += 1;
            }
            LineKind::Added => {
                if selected.contains(&i) {
                    body.push('+');
                    body.push_str(&line.content);
                    body.push('\n');
                    new_count += 1;
                }
                // Non-selected added lines are dropped entirely
            }
            LineKind::Deleted => {
                if selected.contains(&i) {
                    body.push('-');
                    body.push_str(&line.content);
                    body.push('\n');
                    old_count += 1;
                } else {
                    // Non-selected deleted lines become context
                    body.push(' ');
                    body.push_str(&line.content);
                    body.push('\n');
                    old_count += 1;
                    new_count += 1;
                }
            }
        }
    }

    let old_header = match &file.old_path {
        None => "--- /dev/null\n".to_string(),
        Some(p) => format!("--- a/{p}\n"),
    };
    let new_header = match &file.new_path {
        None => "+++ /dev/null\n".to_string(),
        Some(p) => format!("+++ b/{p}\n"),
    };

    format!(
        "{old_header}{new_header}@@ -{},{old_count} +{},{new_count} @@\n{body}",
        hunk.old_start, hunk.new_start
    )
}

/// Generate a unified diff patch for an entire hunk (all changes selected).
pub fn generate_hunk_patch(file: &DiffFile, hunk: &DiffHunk) -> String {
    let selected: HashSet<usize> = hunk
        .lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.kind != LineKind::Context)
        .map(|(i, _)| i)
        .collect();
    generate_line_patch(file, hunk, &selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::diff::{DiffFile, DiffHunk, DiffLine, FileStatus, LineKind};

    fn make_line(kind: LineKind, content: &str, old: Option<u32>, new: Option<u32>) -> DiffLine {
        DiffLine {
            kind,
            content: content.to_string(),
            old_lineno: old,
            new_lineno: new,
        }
    }

    #[test]
    fn test_single_added_line_selected() {
        let hunk = DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![make_line(LineKind::Added, "new line", None, Some(1))],
        };
        let file = DiffFile {
            old_path: Some("foo.rs".into()),
            new_path: Some("foo.rs".into()),
            status: FileStatus::Modified,
            hunks: vec![],
        };
        let selected: HashSet<usize> = [0].into();
        let patch = generate_line_patch(&file, &hunk, &selected);
        assert!(patch.contains("+new line\n"), "patch:\n{patch}");
        assert!(patch.contains("@@ -1,0 +1,1 @@"), "patch:\n{patch}");
    }

    #[test]
    fn test_single_deleted_line_not_selected() {
        let hunk = DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![make_line(LineKind::Deleted, "old line", Some(1), None)],
        };
        let file = DiffFile {
            old_path: Some("foo.rs".into()),
            new_path: Some("foo.rs".into()),
            status: FileStatus::Modified,
            hunks: vec![],
        };
        let selected: HashSet<usize> = HashSet::new();
        let patch = generate_line_patch(&file, &hunk, &selected);
        assert!(patch.contains(" old line\n"), "patch:\n{patch}");
        // old_count = 1 (context), new_count = 1 (context)
        assert!(patch.contains("@@ -1,1 +1,1 @@"), "patch:\n{patch}");
    }

    #[test]
    fn test_non_selected_deleted_becomes_context() {
        let hunk = DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![
                make_line(LineKind::Deleted, "removed", Some(1), None),
                make_line(LineKind::Added, "added", None, Some(1)),
            ],
        };
        let file = DiffFile {
            old_path: Some("foo.rs".into()),
            new_path: Some("foo.rs".into()),
            status: FileStatus::Modified,
            hunks: vec![],
        };
        // Select only the added line (index 1), not the deleted (index 0)
        let selected: HashSet<usize> = [1].into();
        let patch = generate_line_patch(&file, &hunk, &selected);
        // Deleted line should appear as context
        assert!(patch.contains(" removed\n"), "patch:\n{patch}");
        // Added line should appear as added
        assert!(patch.contains("+added\n"), "patch:\n{patch}");
    }

    #[test]
    fn test_partial_selection_mixed_hunk() {
        // 2 context + 1 deleted + 2 added; select the deleted and one added
        let hunk = DiffHunk {
            old_start: 5,
            new_start: 5,
            lines: vec![
                make_line(LineKind::Context, "ctx1", Some(5), Some(5)),
                make_line(LineKind::Context, "ctx2", Some(6), Some(6)),
                make_line(LineKind::Deleted, "del", Some(7), None),
                make_line(LineKind::Added, "add1", None, Some(7)),
                make_line(LineKind::Added, "add2", None, Some(8)),
            ],
        };
        let file = DiffFile {
            old_path: Some("bar.rs".into()),
            new_path: Some("bar.rs".into()),
            status: FileStatus::Modified,
            hunks: vec![],
        };
        // Select deleted (idx 2) and first added (idx 3), skip second added (idx 4)
        let selected: HashSet<usize> = [2, 3].into();
        let patch = generate_line_patch(&file, &hunk, &selected);
        // Non-selected added line "add2" should be absent
        assert!(!patch.contains("add2"), "patch:\n{patch}");
        // old_count: 2 ctx + 1 del = 3; new_count: 2 ctx + 1 add = 3
        assert!(patch.contains("@@ -5,3 +5,3 @@"), "patch:\n{patch}");
    }

    #[test]
    fn test_generate_hunk_patch_selects_all_changes() {
        let hunk = DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![
                make_line(LineKind::Context, "ctx", Some(1), Some(1)),
                make_line(LineKind::Added, "new", None, Some(2)),
                make_line(LineKind::Deleted, "old", Some(2), None),
            ],
        };
        let file = DiffFile {
            old_path: Some("f.rs".into()),
            new_path: Some("f.rs".into()),
            status: FileStatus::Modified,
            hunks: vec![],
        };
        let all_changed: HashSet<usize> = [1, 2].into();
        let expected = generate_line_patch(&file, &hunk, &all_changed);
        let actual = generate_hunk_patch(&file, &hunk);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_new_file_patch_header() {
        let hunk = DiffHunk {
            old_start: 0,
            new_start: 1,
            lines: vec![make_line(LineKind::Added, "hello", None, Some(1))],
        };
        let file = DiffFile {
            old_path: None,
            new_path: Some("new.txt".into()),
            status: FileStatus::Added,
            hunks: vec![],
        };
        let selected: HashSet<usize> = [0].into();
        let patch = generate_line_patch(&file, &hunk, &selected);
        assert!(patch.starts_with("--- /dev/null\n"), "patch:\n{patch}");
    }
}
