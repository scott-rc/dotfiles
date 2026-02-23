pub mod diff;

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub enum DiffSource {
    WorkingTree,
    Staged,
    Commit(String),
    Range(String, String),
}

impl DiffSource {
    pub fn diff_args(&self) -> Vec<String> {
        let mut args = vec!["diff".into()];
        match self {
            Self::WorkingTree => {}
            Self::Staged => args.push("--staged".into()),
            Self::Commit(r) => {
                args.push(format!("{r}~1"));
                args.push(r.clone());
            }
            Self::Range(l, r) => {
                args.push(l.clone());
                args.push(r.clone());
            }
        }
        args
    }

    pub fn diff_args_full_context(&self) -> Vec<String> {
        let mut args = self.diff_args();
        args.insert(1, "-U999999".into());
        args
    }
}

pub fn resolve_source(staged: bool, source: &[String]) -> DiffSource {
    if staged {
        return DiffSource::Staged;
    }
    match source {
        [] => DiffSource::WorkingTree,
        [arg] => {
            if arg.contains("..") {
                let parts: Vec<&str> = arg.splitn(2, "..").collect();
                DiffSource::Range(parts[0].into(), parts[1].into())
            } else {
                DiffSource::Commit(arg.clone())
            }
        }
        [left, right, ..] => DiffSource::Range(left.clone(), right.clone()),
    }
}

/// Run a git command in the given repo, return stdout or None on failure.
pub fn run(repo: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Run `git diff`; on failure print stderr and exit(1).
/// Returns stdout (which may be empty) on success.
pub fn run_diff(repo: &Path, args: &[&str]) -> String {
    let out = match Command::new("git").args(args).current_dir(repo).output() {
        Ok(out) => out,
        Err(e) => {
            eprintln!("gd: failed to run git: {e}");
            std::process::exit(1);
        }
    };
    if !out.status.success() {
        eprint!("{}", String::from_utf8_lossy(&out.stderr));
        std::process::exit(1);
    }
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Get the repo root from any path inside it.
pub fn repo_root(from: &Path) -> Option<PathBuf> {
    let out = run(from, &["rev-parse", "--show-toplevel"])?;
    Some(PathBuf::from(out.trim()))
}

/// Pure helper: given the current branch name, default branch name, and a list of
/// (branch, commits-ahead-of-merge-base) pairs, return the best base branch.
///
/// Returns `default` when `current` is empty (detached HEAD), equals `default`
/// (already on the default branch), or no valid candidates remain after filtering.
/// Among candidates, the one with the fewest commits ahead wins; ties go to the
/// first entry seen (strict `<` comparison).
pub(crate) fn select_base_branch(
    current: &str,
    default: &str,
    candidates: &[(String, u64)],
) -> String {
    if current.is_empty() || current == default {
        return default.to_string();
    }
    let mut best: Option<(&str, u64)> = None;
    for (branch, count) in candidates {
        if branch == current {
            continue;
        }
        match best {
            None => best = Some((branch, *count)),
            Some((_, best_count)) if *count < best_count => best = Some((branch, *count)),
            _ => {}
        }
    }
    best.map_or_else(|| default.to_string(), |(b, _)| b.to_string())
}

/// Detect the base branch of the current branch by finding the local branch
/// with the fewest commits between its merge-base and HEAD.
pub fn find_base_branch(repo: &Path) -> String {
    let current = run(repo, &["branch", "--show-current"])
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let default = run(repo, &["rev-parse", "--abbrev-ref", "origin/HEAD"])
        .map(|s| s.trim().trim_start_matches("origin/").to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "main".to_string());

    let branches_raw = run(
        repo,
        &["for-each-ref", "--format=%(refname:short)", "refs/heads/"],
    )
    .unwrap_or_default();

    let mut candidates: Vec<(String, u64)> = Vec::new();
    for branch in branches_raw.lines().filter(|l| !l.is_empty()) {
        if branch == current {
            continue;
        }
        let mb = match run(repo, &["merge-base", &current, branch]) {
            Some(s) => s.trim().to_string(),
            None => continue,
        };
        let count_str = match run(repo, &["rev-list", "--count", &format!("{mb}..{current}")]) {
            Some(s) => s.trim().to_string(),
            None => continue,
        };
        if let Ok(count) = count_str.parse::<u64>() {
            candidates.push((branch.to_string(), count));
        }
    }

    select_base_branch(&current, &default, &candidates)
}

/// List untracked files (respecting .gitignore).
pub fn untracked_files(repo: &Path) -> Vec<String> {
    run(repo, &["ls-files", "--others", "--exclude-standard"])
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// Append untracked files as synthetic DiffFile entries when in working tree mode.
/// Skips non-files, files over 256KB, and binary files (contain null bytes).
pub fn append_untracked(
    repo: &Path,
    source: &DiffSource,
    no_untracked: bool,
    files: &mut Vec<diff::DiffFile>,
) {
    if !matches!(source, DiffSource::WorkingTree) || no_untracked {
        return;
    }
    let max_size: u64 = 256 * 1024;
    for path in untracked_files(repo) {
        let full = repo.join(&path);
        let Ok(meta) = full.metadata() else {
            continue;
        };
        if !meta.is_file() || meta.len() > max_size {
            continue;
        }
        let Ok(content) = std::fs::read(&full) else {
            continue;
        };
        if content.contains(&0) {
            continue;
        }
        let text = String::from_utf8_lossy(&content);
        files.push(diff::DiffFile::from_content(&path, &text));
    }
}

pub fn sort_files_for_display(files: &mut [diff::DiffFile]) {
    files.sort_by(|a, b| a.path().cmp(b.path()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn snapshot_diff_args_working_tree() {
        assert_debug_snapshot!(DiffSource::WorkingTree.diff_args());
    }

    #[test]
    fn snapshot_diff_args_staged() {
        assert_debug_snapshot!(DiffSource::Staged.diff_args());
    }

    #[test]
    fn snapshot_diff_args_commit() {
        assert_debug_snapshot!(DiffSource::Commit("abc123".into()).diff_args());
    }

    #[test]
    fn snapshot_diff_args_range() {
        assert_debug_snapshot!(DiffSource::Range("main".into(), "HEAD".into()).diff_args());
    }

    #[test]
    fn snapshot_full_context_args_working_tree() {
        assert_debug_snapshot!(DiffSource::WorkingTree.diff_args_full_context());
    }

    #[test]
    fn snapshot_full_context_args_staged() {
        assert_debug_snapshot!(DiffSource::Staged.diff_args_full_context());
    }

    #[test]
    fn snapshot_full_context_args_commit() {
        assert_debug_snapshot!(DiffSource::Commit("abc123".into()).diff_args_full_context());
    }

    #[test]
    fn snapshot_full_context_args_range() {
        assert_debug_snapshot!(
            DiffSource::Range("main".into(), "HEAD".into()).diff_args_full_context()
        );
    }

    #[test]
    fn snapshot_resolve_source_working_tree() {
        assert_debug_snapshot!(resolve_source(false, &[]));
    }

    #[test]
    fn snapshot_resolve_source_staged() {
        assert_debug_snapshot!(resolve_source(true, &[]));
    }

    #[test]
    fn snapshot_resolve_source_commit() {
        assert_debug_snapshot!(resolve_source(false, &["abc123".into()]));
    }

    #[test]
    fn snapshot_resolve_source_range_dotdot() {
        assert_debug_snapshot!(resolve_source(false, &["main..HEAD".into()]));
    }

    #[test]
    fn snapshot_resolve_source_range_two_args() {
        assert_debug_snapshot!(resolve_source(false, &["main".into(), "HEAD".into()]));
    }

    #[test]
    fn snapshot_resolve_source_staged_overrides() {
        assert_debug_snapshot!(resolve_source(true, &["HEAD".into()]));
    }

    #[test]
    fn test_select_base_branch_prefers_fewest_commits_ahead() {
        let result = select_base_branch(
            "feature",
            "main",
            &[("main".into(), 3u64), ("other".into(), 10u64)],
        );
        assert_eq!(result, "main");
    }

    #[test]
    fn test_select_base_branch_returns_default_when_no_branches() {
        let result = select_base_branch("feature", "main", &[]);
        assert_eq!(result, "main");
    }

    #[test]
    fn test_select_base_branch_skips_current_branch() {
        let result = select_base_branch(
            "feature",
            "main",
            &[("feature".into(), 0u64), ("main".into(), 5u64)],
        );
        assert_eq!(result, "main");
    }

    #[test]
    fn test_select_base_branch_returns_default_when_on_default_branch() {
        let result = select_base_branch("main", "main", &[("other".into(), 2u64)]);
        assert_eq!(result, "main");
    }

    #[test]
    fn test_select_base_branch_empty_current_returns_default() {
        let result = select_base_branch("", "main", &[("other".into(), 1u64)]);
        assert_eq!(result, "main");
    }

    #[test]
    fn test_select_base_branch_ties_pick_first_seen() {
        let result = select_base_branch(
            "feature",
            "main",
            &[("branchA".into(), 4u64), ("branchB".into(), 4u64)],
        );
        assert_eq!(result, "branchA");
    }

    #[test]
    fn test_diff_source_full_context_args() {
        assert_eq!(
            DiffSource::WorkingTree.diff_args_full_context(),
            vec!["diff", "-U999999"],
        );
        assert_eq!(
            DiffSource::Staged.diff_args_full_context(),
            vec!["diff", "-U999999", "--staged"],
        );
    }

    #[test]
    fn test_diff_source_commit_full_context() {
        assert_eq!(
            DiffSource::Commit("abc".into()).diff_args_full_context(),
            vec!["diff", "-U999999", "abc~1", "abc"],
        );
    }

    #[test]
    fn test_resolve_source() {
        assert!(matches!(
            resolve_source(false, &[]),
            DiffSource::WorkingTree
        ));
        assert!(matches!(resolve_source(true, &[]), DiffSource::Staged));
        match resolve_source(false, &["main..HEAD".into()]) {
            DiffSource::Range(l, r) => {
                assert_eq!(l, "main");
                assert_eq!(r, "HEAD");
            }
            other => panic!("expected Range, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_source_two_args_range() {
        match resolve_source(false, &["main".into(), "HEAD".into()]) {
            DiffSource::Range(l, r) => {
                assert_eq!(l, "main");
                assert_eq!(r, "HEAD");
            }
            other => panic!("expected Range, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_source_staged_overrides_args() {
        assert!(matches!(
            resolve_source(true, &["HEAD".into()]),
            DiffSource::Staged
        ));
    }

    #[test]
    fn test_resolve_source_dotdot_range_with_ref() {
        match resolve_source(false, &["HEAD~3..HEAD".into()]) {
            DiffSource::Range(l, r) => {
                assert_eq!(l, "HEAD~3");
                assert_eq!(r, "HEAD");
            }
            other => panic!("expected Range, got {other:?}"),
        }
    }

    #[test]
    fn test_sort_files_for_display_orders_by_path() {
        let mut files = vec![
            diff::DiffFile::from_content("z-last.txt", "z"),
            diff::DiffFile::from_content("a-first.txt", "a"),
            diff::DiffFile::from_content("m-middle.txt", "m"),
        ];
        sort_files_for_display(&mut files);
        let paths: Vec<&str> = files.iter().map(diff::DiffFile::path).collect();
        assert_eq!(paths, vec!["a-first.txt", "m-middle.txt", "z-last.txt"]);
    }

    #[test]
    fn test_run_diff_returns_empty_on_success_empty_output() {
        let dir_name = format!(
            "gd-run-diff-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let repo = std::env::temp_dir().join(dir_name);
        std::fs::create_dir_all(&repo).expect("create temp repo");

        let init = Command::new("git")
            .arg("init")
            .current_dir(&repo)
            .output()
            .expect("run git init");
        assert!(init.status.success(), "git init should succeed");

        let diff = run_diff(&repo, &["diff"]);
        assert_eq!(diff, "");

        std::fs::remove_dir_all(&repo).expect("cleanup temp repo");
    }
}
