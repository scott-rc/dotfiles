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

/// Get the repo root from any path inside it.
pub fn repo_root(from: &Path) -> Option<PathBuf> {
    let out = run(from, &["rev-parse", "--show-toplevel"])?;
    Some(PathBuf::from(out.trim()))
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(matches!(resolve_source(false, &[]), DiffSource::WorkingTree));
        assert!(matches!(resolve_source(true, &[]), DiffSource::Staged));
        match resolve_source(false, &["main..HEAD".into()]) {
            DiffSource::Range(l, r) => {
                assert_eq!(l, "main");
                assert_eq!(r, "HEAD");
            }
            other => panic!("expected Range, got {other:?}"),
        }
    }
}
