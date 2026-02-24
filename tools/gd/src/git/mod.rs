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
            Self::Commit(_) => unreachable!("Commit is resolved to Range in main"),
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
            if arg.contains("...") {
                let parts: Vec<&str> = arg.splitn(2, "...").collect();
                DiffSource::Range(parts[0].into(), format!("...{}", parts[1]))
            } else if arg.contains("..") {
                let parts: Vec<&str> = arg.splitn(2, "..").collect();
                DiffSource::Range(parts[0].into(), parts[1].into())
            } else {
                DiffSource::Commit(arg.clone())
            }
        }
        [left, right, ..] => DiffSource::Range(left.clone(), right.clone()),
    }
}

/// Resolve the parent of a commit. Falls back to the empty tree SHA for root commits.
pub fn resolve_commit_parent(repo: &Path, commit: &str) -> String {
    let parent_ref = format!("{commit}~1");
    match run(repo, &["rev-parse", "--verify", "--quiet", &parent_ref]) {
        Some(s) => s.trim().to_string(),
        None => "4b825dc642cb6eb9a060e54bf899d15d4a9a7882".to_string(),
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

/// Pure helper: parse `git log --first-parent --format=%D` output to find the
/// closest ancestor branch. `skip` contains ref names to ignore (e.g. current
/// branch, its HEAD pointer, and remote tracking ref). Returns the first branch
/// name found, with `origin/` prefix stripped.
pub(crate) fn parse_first_parent_base(log_output: &str, skip: &[&str]) -> Option<String> {
    for line in log_output.lines() {
        if line.is_empty() {
            continue;
        }
        for r in line.split(", ") {
            let name = r.strip_prefix("HEAD -> ").unwrap_or(r);
            if skip.iter().any(|s| *s == name) {
                continue;
            }
            return Some(name.strip_prefix("origin/").unwrap_or(name).to_string());
        }
    }
    None
}

/// Detect the base branch by walking first-parent history and finding the first
/// ancestor commit decorated with another branch. Mirrors the `gbb` fish function.
pub fn find_base_branch(repo: &Path) -> String {
    let current = run(repo, &["branch", "--show-current"])
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let default = run(repo, &["rev-parse", "--abbrev-ref", "origin/HEAD"])
        .map(|s| s.trim().trim_start_matches("origin/").to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "main".to_string());

    if current.is_empty() || current == default {
        return default;
    }

    let skip_head = format!("HEAD -> {current}");
    let skip_origin = format!("origin/{current}");
    let skip = [skip_head.as_str(), current.as_str(), skip_origin.as_str()];

    if let Some(log_output) = run(
        repo,
        &[
            "log",
            "--first-parent",
            "--format=%D",
            "--decorate-refs=refs/heads/",
            "--decorate-refs=refs/remotes/origin/",
            &current,
        ],
    ) {
        if let Some(base) = parse_first_parent_base(&log_output, &skip) {
            return base;
        }
    }

    default
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
    fn test_parse_first_parent_base_finds_ancestor_branch() {
        let log = "\n\nHEAD -> feature, origin/feature\n\nmain, origin/main\n";
        let skip = ["HEAD -> feature", "feature", "origin/feature"];
        assert_eq!(
            parse_first_parent_base(log, &skip),
            Some("main".to_string())
        );
    }

    #[test]
    fn test_parse_first_parent_base_strips_origin_prefix() {
        let log = "\n\norigin/develop\n";
        let skip = ["HEAD -> feature", "feature", "origin/feature"];
        assert_eq!(
            parse_first_parent_base(log, &skip),
            Some("develop".to_string())
        );
    }

    #[test]
    fn test_parse_first_parent_base_skips_current_branch() {
        let log = "HEAD -> feature\norigin/feature\nmain\n";
        let skip = ["HEAD -> feature", "feature", "origin/feature"];
        assert_eq!(
            parse_first_parent_base(log, &skip),
            Some("main".to_string())
        );
    }

    #[test]
    fn test_parse_first_parent_base_returns_none_when_no_match() {
        let log = "\n\n\n";
        let skip = ["HEAD -> feature", "feature", "origin/feature"];
        assert_eq!(parse_first_parent_base(log, &skip), None);
    }

    #[test]
    fn test_parse_first_parent_base_picks_first_on_multi_decorated_commit() {
        let log = "develop, staging\n";
        let skip = ["HEAD -> feature", "feature", "origin/feature"];
        assert_eq!(
            parse_first_parent_base(log, &skip),
            Some("develop".to_string())
        );
    }

    #[test]
    fn test_parse_first_parent_base_empty_input() {
        assert_eq!(parse_first_parent_base("", &["feature"]), None);
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
    fn test_resolve_source_triple_dot_range() {
        match resolve_source(false, &["a...b".into()]) {
            DiffSource::Range(l, r) => {
                assert_eq!(l, "a");
                assert_eq!(r, "...b");
            }
            other => panic!("expected Range, got {other:?}"),
        }
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
    fn test_resolve_commit_parent_root() {
        let dir_name = format!(
            "gd-root-commit-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let repo = std::env::temp_dir().join(dir_name);
        std::fs::create_dir_all(&repo).expect("create temp repo");

        let init = Command::new("git")
            .args(["init"])
            .current_dir(&repo)
            .output()
            .expect("git init");
        assert!(init.status.success());

        // Configure git user for the commit
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&repo)
            .output()
            .expect("git config email");
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&repo)
            .output()
            .expect("git config name");

        // Create a file and commit it
        std::fs::write(repo.join("file.txt"), "hello").expect("write file");
        Command::new("git")
            .args(["add", "."])
            .current_dir(&repo)
            .output()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&repo)
            .output()
            .expect("git commit");

        let sha = run(&repo, &["rev-parse", "HEAD"])
            .expect("get HEAD sha")
            .trim()
            .to_string();

        let parent = resolve_commit_parent(&repo, &sha);
        assert_eq!(parent, "4b825dc642cb6eb9a060e54bf899d15d4a9a7882");

        std::fs::remove_dir_all(&repo).expect("cleanup temp repo");
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
